//! High-level CUB file reader with iterator-based API

use crate::error::Result;
use crate::raw::{Header, Item, ItemData, PointOp};
use crate::{Airspace, BoundingBox};
use std::borrow::Cow;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// High-level CUB file reader with iterator-based API
///
/// Provides convenient access to CUB file contents with automatic decoding of:
/// - Strings (UTF-8 with Extended ASCII fallback)
/// - Coordinates (raw i16 offsets → f32 lat/lon radians)
/// - Bit-packed fields (enums and flags)
///
/// # Example
///
/// ```no_run
/// use seeyou_cub::CubReader;
///
/// let mut reader = CubReader::from_path("airspace.cub")?;
/// for result in reader.read_airspaces() {
///     let airspace = result?;
///     println!("{:?}: {} points", airspace.name, airspace.points.len());
/// }
/// # Ok::<(), seeyou_cub::Error>(())
/// ```
pub struct CubReader<R: Read + Seek> {
    reader: BufReader<R>,
    header: Header,
}

impl CubReader<File> {
    /// Create a reader from a file path
    ///
    /// Opens the file and reads the header immediately to validate format.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        Self::new(file)
    }
}

impl<R: Read + Seek> CubReader<R> {
    /// Create a reader from any `Read + Seek` source
    ///
    /// Reads the header immediately to validate the format and store metadata.
    pub fn new(mut reader: R) -> Result<Self> {
        let header = Header::read(&mut reader)?;
        Ok(Self {
            reader: BufReader::new(reader),
            header,
        })
    }

    /// Get reference to a parsed header
    pub fn raw_header(&self) -> &Header {
        &self.header
    }

    /// Get the CUB file title
    pub fn title(&self) -> Cow<'_, str> {
        self.header.title.decode()
    }

    /// Get bounding box covering all airspaces
    ///
    /// Returns `(west, south, east, north)` in radians.
    ///
    /// This value is read from the file header and represents the pre-calculated
    /// bounding box for all airspaces in the file.
    pub fn bounding_box(&self) -> &BoundingBox {
        self.header.bounding_box()
    }

    /// Create iterator over all airspaces in the file
    ///
    /// Returns an iterator that yields `Result<Airspace>` for each airspace.
    ///
    /// The iterator performs lazy parsing - airspaces are only decoded when `.next()` is called.
    pub fn read_airspaces(&mut self) -> AirspaceIterator<'_, R> {
        AirspaceIterator {
            reader: &mut self.reader,
            header: &self.header,
            current_index: 0,
        }
    }
}

/// Iterator over airspaces in a CUB file
///
/// Yields `Result<Airspace>` for each airspace.
/// Created by calling `CubReader::read_airspaces()`.
pub struct AirspaceIterator<'a, R: Read + Seek> {
    reader: &'a mut BufReader<R>,
    header: &'a Header,
    current_index: i32,
}

impl<R: Read + Seek> AirspaceIterator<'_, R> {
    fn read_airspace(&mut self, index: usize) -> Result<Airspace> {
        let item_offset =
            crate::raw::HEADER_SIZE as u64 + (index as u64 * self.header.size_of_item as u64);

        self.reader.seek(SeekFrom::Start(item_offset))?;
        let item = Item::read(self.reader, self.header)?;

        let data_offset = self.header.data_offset as u64 + item.points_offset as u64;
        self.reader.seek(SeekFrom::Start(data_offset))?;

        let raw_data = ItemData::read(self.reader, self.header)?;

        // Convert to high-level Airspace
        convert_to_airspace(self.header, &item, raw_data)
    }
}

impl<'a, R: Read + Seek> Iterator for AirspaceIterator<'a, R> {
    type Item = Result<Airspace>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.header.hdr_items {
            return None;
        }

        let airspace = match self.read_airspace(self.current_index as usize) {
            Ok(airspace) => airspace,
            Err(err) => return Some(Err(err)),
        };
        self.current_index += 1;
        Some(Ok(airspace))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.header.hdr_items - self.current_index) as usize;
        (remaining, Some(remaining))
    }
}

impl<'a, R: Read + Seek> ExactSizeIterator for AirspaceIterator<'a, R> {}

/// Convert raw item + item data to high-level Airspace
fn convert_to_airspace(header: &Header, item: &Item, item_data: ItemData) -> Result<Airspace> {
    // Convert coordinates from raw i16 offsets to f32 lat/lon radians
    let points = PointOp::resolve(
        &item_data.point_ops,
        header.lo_la_scale,
        item.bounding_box.left,
        item.bounding_box.bottom,
    )?;

    // Decode strings from raw bytes
    let name = item_data.name.as_ref().map(|bs| bs.decode().into_owned());
    let frequency_name = item_data
        .frequency_name
        .as_ref()
        .map(|bs| bs.decode().into_owned());
    let icao_code = item_data
        .icao_code
        .as_ref()
        .map(|bs| bs.decode().into_owned());
    let exception_rules = item_data
        .exception_rules
        .as_ref()
        .map(|bs| bs.decode().into_owned());
    let notam_remarks = item_data
        .notam_remarks
        .as_ref()
        .map(|bs| bs.decode().into_owned());
    let notam_id = item_data
        .notam_id
        .as_ref()
        .map(|bs| bs.decode().into_owned());

    Ok(Airspace {
        // Bounding box (always populated by reader)
        bounding_box: Some(item.bounding_box),

        // Decoded airspace classification
        style: item.style(),
        class: item.class(),
        extended_type: item.extended_type(),

        // Altitude data
        min_alt: item.min_alt,
        max_alt: item.max_alt,
        min_alt_style: item.min_alt_style(),
        max_alt_style: item.max_alt_style(),

        // Time-related fields
        time_out: item.time_out,
        start_date: item.start_date(),
        end_date: item.end_date(),
        extra_data: item.extra_data,

        // Decoded temporal data
        days_active: item.days_active(),

        // Geometry (converted from raw i16 to f32 lat/lon radians)
        points,

        // Decoded string attributes
        name,
        frequency_name,
        icao_code,
        exception_rules,
        notam_remarks,
        notam_id,

        // Numeric attributes
        frequency: item_data.frequency,
        secondary_frequency: item_data.secondary_frequency,
        notam_insert_time: item_data.notam_insert_time,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn read_header_from_fixture() {
        let reader = CubReader::from_path("tests/fixtures/france_2024.07.02.cub")
            .expect("Failed to open fixture");

        let header = reader.raw_header();
        assert_debug_snapshot!(header);
    }

    #[test]
    fn read_first_airspace() {
        let mut reader = CubReader::from_path("tests/fixtures/france_2024.07.02.cub")
            .expect("Failed to open fixture");

        let airspace = reader
            .read_airspaces()
            .next()
            .expect("Expected at least one airspace")
            .expect("Failed to read first airspace");

        assert_debug_snapshot!(airspace);
    }

    #[test]
    fn read_all_airspaces_count() {
        let mut reader = CubReader::from_path("tests/fixtures/france_2024.07.02.cub")
            .expect("Failed to open fixture");

        let count = reader.read_airspaces().count();
        assert_eq!(count, 1368);
    }

    #[test]
    fn iterator_size_hint() {
        let mut reader = CubReader::from_path("tests/fixtures/france_2024.07.02.cub")
            .expect("Failed to open fixture");

        let mut iter = reader.read_airspaces();
        assert_eq!(iter.size_hint(), (1368, Some(1368)));

        // Consume one item
        let _ = iter.next();
        assert_eq!(iter.size_hint(), (1367, Some(1367)));
    }

    #[test]
    fn verify_string_decoding() {
        let mut reader = CubReader::from_path("tests/fixtures/france_2024.07.02.cub")
            .expect("Failed to open fixture");

        let mut names: Vec<_> = reader
            .read_airspaces()
            .filter_map(|r| r.ok())
            .map(|a| a.name.unwrap_or_default())
            .collect();

        names.sort();
        assert_debug_snapshot!(names);
    }
}
