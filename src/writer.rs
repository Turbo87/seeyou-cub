//! High-level CUB file writer with builder API
//!
//! Provides a convenient API for creating CUB files from airspace data. The writer
//! follows a collect-then-write design: all airspaces are stored in memory, then
//! coordinate scales, bounding boxes, and byte offsets are calculated automatically
//! before writing the complete file.
//!
//! # Design
//!
//! The writer uses a mutable builder pattern with `&mut self` methods for chaining:
//!
//! ```no_run
//! use seeyou_cub::writer::CubWriter;
//! use seeyou_cub::{Airspace, Point, CubStyle, CubClass, AltStyle, DaysActive};
//!
//! let airspace = Airspace {
//!     style: CubStyle::DangerArea,
//!     class: CubClass::ClassD,
//!     min_alt: 0,
//!     max_alt: 5000,
//!     min_alt_style: AltStyle::MeanSeaLevel,
//!     max_alt_style: AltStyle::MeanSeaLevel,
//!     days_active: DaysActive::all(),
//!     points: vec![
//!         Point::lat_lon(0.8, 0.4),
//!         Point::lat_lon(0.81, 0.41),
//!         Point::lat_lon(0.82, 0.42),
//!     ],
//!     name: Some("My Airspace".to_string()),
//!     ..Default::default()
//! };
//!
//! CubWriter::new("My Airspace Data")
//!     .add_airspace(airspace)
//!     .write_to_path("output.cub")?;
//! # Ok::<(), seeyou_cub::Error>(())
//! ```
//!
//! # Automatic Calculations
//!
//! The writer handles several complex tasks automatically:
//!
//! - **Bounding boxes**: Calculated from points if not provided in `Airspace::bounding_box`
//! - **Coordinate conversion**: Points converted to i16 offsets with automatic `MoveOrigin` insertion
//! - **Byte offsets**: All file offsets calculated during the write process
//! - **Global metadata**: Header fields like `max_pts` and global bounding box computed from all airspaces
//!
//! # Configuration
//!
//! ## Defaults
//!
//! - **Byte order**: Little-endian
//! - **Coordinate precision**: ~1 meter at the equator (`lo_la_scale = 1.5723687e-7` radians)
//!
//! ## Overrides
//!
//! Both settings can be customized:
//!
//! ```no_run
//! use seeyou_cub::writer::CubWriter;
//! use seeyou_cub::ByteOrder;
//!
//! CubWriter::new("Custom Settings")
//!     .with_byte_order(ByteOrder::BE)
//!     .with_lo_la_scale(0.0001)
//!     .write_to_path("output.cub")?;
//! # Ok::<(), seeyou_cub::Error>(())
//! ```
//!
//! # Limitations
//!
//! ## Anti-meridian Handling
//!
//! The writer does **not** correctly handle airspaces that cross the ±180° longitude line
//! (the anti-meridian). Bounding box calculation uses simple min/max logic, which produces
//! incorrect results for such airspaces. If your data includes airspaces crossing the
//! anti-meridian, you must calculate and provide the correct bounding box manually via
//! `Airspace::bounding_box`.

use crate::error::Result;
use crate::raw::{Header, Item, ItemData, PointOp};
use crate::utils::ByteString;
use crate::{Airspace, AltStyle, BoundingBox, ByteOrder, CubClass, CubStyle, DaysActive, Point};
use std::io::Cursor;

/// Default coordinate scale factor
///
/// This value (`1.5723687e-7` radians) provides approximately 1 meter precision
/// at the equator and allows coordinates within ~33km from the origin
/// before requiring a `MoveOrigin` operation.
const DEFAULT_LO_LA_SCALE: f32 = 1.5723687e-7;

/// High-level CUB file writer with builder API
///
/// Provides a convenient API for creating CUB files from airspace data.
/// All low-level complexity (coordinate conversion, bounding box calculation,
/// offset tracking) is handled automatically.
///
/// # Example
///
/// ```
/// use seeyou_cub::writer::CubWriter;
///
/// CubWriter::new("My Airspace Data");
/// ```
pub struct CubWriter {
    title: String,
    airspaces: Vec<Airspace>,
    byte_order: ByteOrder,
    lo_la_scale: f32,
}

impl CubWriter {
    /// Create a new writer with the given title
    ///
    /// The title will be stored in the CUB file header.
    /// Default byte order is little-endian, and coordinate precision is 1 meter
    /// at the equator.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            airspaces: Vec::new(),
            byte_order: ByteOrder::LE,
            lo_la_scale: DEFAULT_LO_LA_SCALE,
        }
    }

    /// Write CUB file to a writer
    ///
    /// Processes all airspaces in memory, calculates bounding boxes and offsets,
    /// then writes the complete file.
    ///
    /// # Returns
    ///
    /// Ok(()) on success or an error if writing fails
    pub fn write<W: std::io::Write + std::io::Seek>(&mut self, mut writer: W) -> Result<()> {
        // Create header with known values (will update counts and offsets later)
        let mut header = Header {
            title: ByteString::from(self.title.as_bytes().to_vec()),
            allowed_serials: [0; 8],
            pc_byte_order: self.byte_order.as_pc_byte_order(),
            key: [0; 16],
            size_of_item: 43,
            size_of_point: 5,
            hdr_items: 0, // Will be updated later
            max_pts: 0,   // Will be updated later
            bounding_box: BoundingBox {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            }, // Will be updated later
            max_width: 0.0, // Will be updated later
            max_height: 0.0, // Will be updated later
            lo_la_scale: self.lo_la_scale,
            data_offset: 0, // Will be updated later
        };

        let mut global_bbox: Option<BoundingBox> = None;
        let mut items_buffer = Cursor::new(Vec::new());
        let mut item_data_buffer = Cursor::new(Vec::new());

        for airspace in &self.airspaces {
            // Calculate bbox if missing
            let bbox = airspace
                .bounding_box
                .or_else(|| BoundingBox::from_points(&airspace.points))
                .unwrap_or_else(|| BoundingBox::from(Point::lat_lon(0., 0.)));

            // Accumulate into global bbox
            match global_bbox {
                None => global_bbox = Some(bbox),
                Some(ref mut global) => global.merge(bbox),
            }

            // Convert points to PointOps
            let point_ops =
                PointOp::from_points(&airspace.points, self.lo_la_scale, bbox.left, bbox.bottom)?;

            // Record current data offset (for `Item::points_offset` field)
            let data_offset = item_data_buffer.position() as i32;

            // Create `ItemData` and write to data buffer
            let item_data = ItemData {
                point_ops,
                name: airspace
                    .name
                    .as_ref()
                    .map(|s| ByteString::from(s.as_bytes().to_vec())),
                frequency: airspace.frequency.map(|f| (f * 1000.) as u32),
                frequency_name: airspace
                    .frequency_name
                    .as_ref()
                    .map(|s| ByteString::from(s.as_bytes().to_vec())),
                icao_code: airspace
                    .icao_code
                    .as_ref()
                    .map(|s| ByteString::from(s.as_bytes().to_vec())),
                secondary_frequency: airspace.secondary_frequency.map(|f| (f * 1000.) as u32),
                exception_rules: airspace
                    .exception_rules
                    .as_ref()
                    .map(|s| ByteString::from(s.as_bytes().to_vec())),
                notam_remarks: airspace
                    .notam_remarks
                    .as_ref()
                    .map(|s| ByteString::from(s.as_bytes().to_vec())),
                notam_id: airspace
                    .notam_id
                    .as_ref()
                    .map(|s| ByteString::from(s.as_bytes().to_vec())),
                notam_insert_time: airspace.notam_insert_time,
            };
            item_data.write(&mut item_data_buffer, &header)?;

            // Create and write Item
            let item = Item {
                bounding_box: bbox,
                type_byte: encode_type_byte(airspace.style, airspace.class),
                alt_style_byte: encode_alt_style_byte(
                    airspace.min_alt_style,
                    airspace.max_alt_style,
                ),
                min_alt: airspace.min_alt,
                max_alt: airspace.max_alt,
                points_offset: data_offset,
                extra_data: airspace.extra_data,
                active_time: encode_active_time(
                    airspace.start_date.as_ref(),
                    airspace.end_date.as_ref(),
                    &airspace.days_active,
                ),
                extended_type_byte: airspace.extended_type.map(|t| t.as_byte()).unwrap_or(0),
            };
            item.write(&mut items_buffer, &header)?;
        }

        // Update header with calculated values
        let items_size = items_buffer.position() as i32;
        header.data_offset = crate::raw::HEADER_SIZE as i32 + items_size;
        header.hdr_items = self.airspaces.len() as i32;

        let max_pts = self.airspaces.iter().map(|a| a.points.len()).max();
        header.max_pts = max_pts.unwrap_or(0) as i32;

        if let Some(bbox) = global_bbox {
            header.bounding_box = bbox;
        }
        header.max_width = header.bounding_box.right - header.bounding_box.left;
        header.max_height = header.bounding_box.top - header.bounding_box.bottom;

        header.write(&mut writer)?;
        writer.write_all(&items_buffer.into_inner())?;
        writer.write_all(&item_data_buffer.into_inner())?;

        Ok(())
    }

    /// Add a single airspace to the writer
    ///
    /// Returns `&mut self` to allow method chaining.
    pub fn add_airspace(&mut self, airspace: Airspace) -> &mut Self {
        self.airspaces.push(airspace);
        self
    }

    /// Add multiple airspaces from an iterator
    ///
    /// Returns `&mut self` to allow method chaining.
    pub fn add_airspaces<I: IntoIterator<Item = Airspace>>(&mut self, airspaces: I) -> &mut Self {
        self.airspaces.extend(airspaces);
        self
    }

    /// Configure byte order for the output file
    ///
    /// Default is little-endian. Returns `&mut self` to allow method chaining.
    pub fn with_byte_order(&mut self, byte_order: ByteOrder) -> &mut Self {
        self.byte_order = byte_order;
        self
    }

    /// Override the coordinate precision scale
    ///
    /// The scale determines the precision of stored coordinates.
    /// Default is 1 meter precision at the equator.
    /// Returns `&mut self` to allow method chaining.
    pub fn with_lo_la_scale(&mut self, scale: f32) -> &mut Self {
        self.lo_la_scale = scale;
        self
    }

    /// Write CUB file to a file path
    ///
    /// Convenience wrapper around `write()` that creates a file at the given path.
    ///
    /// # Returns
    ///
    /// Ok(()) on success or an error if file creation or writing fails
    pub fn write_to_path<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let file = std::fs::File::create(path)?;
        self.write(file)
    }
}

// Helper functions for encoding bit-packed fields
fn encode_type_byte(style: CubStyle, class: CubClass) -> u8 {
    let style_nibble = style.as_nibble();
    let class_nibble = class.as_nibble();
    (style_nibble << 4) | class_nibble
}

fn encode_alt_style_byte(min: AltStyle, max: AltStyle) -> u8 {
    let min_nibble = min.as_nibble();
    let max_nibble = max.as_nibble();
    (min_nibble << 4) | max_nibble
}

fn encode_notam_time(dt: &crate::DateTime) -> u32 {
    let year = dt.year - 2000;
    let month = (dt.month - 1) as u32;
    let day = (dt.day - 1) as u32;
    let hour = dt.hour as u32;
    let minute = dt.minute as u32;

    minute + 60 * (hour + 24 * (day + 31 * (month + 12 * year)))
}

fn encode_active_time(
    start_date: Option<&crate::DateTime>,
    end_date: Option<&crate::DateTime>,
    days: &DaysActive,
) -> u64 {
    let days_bits = (days.as_bits() & 0xFFF) << 52;

    let start_bits = if let Some(dt) = start_date {
        (encode_notam_time(dt) as u64) << 26
    } else {
        0
    };

    let end_bits = if let Some(dt) = end_date {
        encode_notam_time(dt) as u64
    } else {
        0x3FFFFFF // Max value indicates no end date
    };

    days_bits | start_bits | end_bits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AltStyle, CubClass, CubReader, CubStyle, DaysActive, Point};
    use insta::assert_debug_snapshot;
    use std::io::Cursor;

    #[test]
    fn new_accepts_string_types() {
        let _writer1 = CubWriter::new("string slice");
        let _writer2 = CubWriter::new(String::from("owned string"));
    }

    #[test]
    fn default_lo_la_scale_provides_meter_precision() {
        // Convert to degrees for easier understanding
        let precision_degrees = DEFAULT_LO_LA_SCALE.to_degrees();

        // At equator, 1 degree latitude ≈ 111km
        let precision_meters = precision_degrees * 111_000.0;

        assert_debug_snapshot!(precision_meters, @"1.0");
    }

    #[test]
    fn write_empty_cub_file() {
        let mut writer = CubWriter::new("Empty Test");
        let mut buf = Cursor::new(Vec::new());

        writer.write(&mut buf).expect("Failed to write");

        // Verify file is exactly 210 bytes (header only)
        let bytes = buf.into_inner();
        assert_eq!(bytes.len(), 210);

        // Read back and verify
        let mut cursor = Cursor::new(&bytes[..]);
        let mut reader = CubReader::new(&mut cursor).expect("Failed to read");

        // Should have no airspaces
        let count = reader.read_airspaces().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn write_single_airspace_with_points() {
        let mut writer = CubWriter::new("Single Airspace Test");

        // Create simple airspace with a few points
        let airspace = Airspace {
            class: CubClass::ClassE,
            min_alt: 0,
            max_alt: 5000,
            min_alt_style: AltStyle::MeanSeaLevel,
            max_alt_style: AltStyle::MeanSeaLevel,
            days_active: DaysActive::all(),
            points: vec![
                Point::lat_lon(0.8, 0.4),
                Point::lat_lon(0.81, 0.41),
                Point::lat_lon(0.82, 0.42),
            ],
            name: Some("Test Airspace".to_string()),
            ..Default::default()
        };

        writer.add_airspace(airspace);

        let mut buf = Cursor::new(Vec::new());
        writer.write(&mut buf).expect("Failed to write");

        // Read back and verify
        let bytes = buf.into_inner();
        let mut cursor = Cursor::new(&bytes[..]);
        let mut reader = CubReader::new(&mut cursor).expect("Failed to read");

        let airspaces: Vec<_> = reader
            .read_airspaces()
            .collect::<Result<_>>()
            .expect("Failed to read airspaces");

        assert_eq!(airspaces.len(), 1);
        let airspace = &airspaces[0];
        assert_eq!(airspace.name, Some("Test Airspace".to_string()));
        assert_eq!(airspace.points.len(), 3);

        // Verify bounding box was calculated
        assert!(airspace.bounding_box.is_some());
    }

    #[test]
    fn write_multiple_airspaces() {
        let mut writer = CubWriter::new("Multiple Airspaces Test");

        // Create first airspace
        let airspace1 = Airspace {
            style: CubStyle::DangerArea,
            class: CubClass::ClassD,
            min_alt: 0,
            max_alt: 3000,
            min_alt_style: AltStyle::MeanSeaLevel,
            max_alt_style: AltStyle::MeanSeaLevel,
            days_active: DaysActive::all(),
            points: vec![
                Point::lat_lon(0.5, 0.2),
                Point::lat_lon(0.51, 0.21),
                Point::lat_lon(0.52, 0.22),
                Point::lat_lon(0.53, 0.23),
            ],
            name: Some("Danger Area 1".to_string()),
            icao_code: Some("DA1".to_string()),
            frequency: Some(123.450),
            ..Default::default()
        };

        // Create second airspace
        let airspace2 = Airspace {
            style: CubStyle::RestrictedArea,
            class: CubClass::ClassC,
            min_alt: 1000,
            max_alt: 5000,
            min_alt_style: AltStyle::AboveGroundLevel,
            max_alt_style: AltStyle::FlightLevel,
            days_active: DaysActive::all(),
            points: vec![Point::lat_lon(0.6, 0.3), Point::lat_lon(0.61, 0.31)],
            name: Some("Restricted 2".to_string()),
            ..Default::default()
        };

        writer.add_airspace(airspace1);
        writer.add_airspace(airspace2);

        let mut buf = Cursor::new(Vec::new());
        writer.write(&mut buf).expect("Failed to write");

        // Read back and verify
        let bytes = buf.into_inner();
        let mut cursor = Cursor::new(&bytes[..]);
        let mut reader = CubReader::new(&mut cursor).expect("Failed to read");

        let airspaces: Vec<_> = reader
            .read_airspaces()
            .collect::<Result<_>>()
            .expect("Failed to read airspaces");

        assert_eq!(airspaces.len(), 2);

        // Verify first airspace
        let a1 = &airspaces[0];
        assert_eq!(a1.name, Some("Danger Area 1".to_string()));
        assert_eq!(a1.points.len(), 4);
        assert_eq!(a1.icao_code, Some("DA1".to_string()));
        assert_eq!(a1.frequency, Some(123.450));

        // Verify second airspace
        let a2 = &airspaces[1];
        assert_eq!(a2.name, Some("Restricted 2".to_string()));
        assert_eq!(a2.points.len(), 2);
    }

    #[test]
    fn write_to_path() {
        let airspace = Airspace {
            class: CubClass::ClassE,
            min_alt: 0,
            max_alt: 5000,
            min_alt_style: AltStyle::MeanSeaLevel,
            max_alt_style: AltStyle::MeanSeaLevel,
            days_active: DaysActive::all(),
            points: vec![Point::lat_lon(0.5, 0.5), Point::lat_lon(0.51, 0.51)],
            name: Some("Path Test Airspace".to_string()),
            ..Default::default()
        };

        // Write to temporary file
        let temp_path = std::env::temp_dir().join("test_write_to_path.cub");

        CubWriter::new("Path Test")
            .add_airspace(airspace)
            .write_to_path(&temp_path)
            .expect("Failed to write to path");

        // Read back and verify
        let airspaces: Vec<_> = CubReader::from_path(&temp_path)
            .expect("Failed to read from path")
            .read_airspaces()
            .collect::<Result<_>>()
            .expect("Failed to read airspaces");

        assert_eq!(airspaces.len(), 1);
        assert_eq!(airspaces[0].name, Some("Path Test Airspace".to_string()));
    }

    #[test]
    fn round_trip_france_fixture() {
        // Read France fixture
        let original_airspaces: Vec<_> =
            CubReader::from_path("tests/fixtures/france_2024.07.02.cub")
                .expect("Failed to open")
                .read_airspaces()
                .collect::<Result<_>>()
                .expect("Failed to read airspaces");

        // Write out again
        let mut cursor = Cursor::new(Vec::new());
        CubWriter::new("Round-trip Test")
            .add_airspaces(original_airspaces.clone())
            .write(&mut cursor)
            .expect("Failed to write temp file");

        // Read back
        cursor.set_position(0);
        let read_back_airspaces: Vec<_> = CubReader::new(&mut cursor)
            .expect("Failed to read temp file")
            .read_airspaces()
            .collect::<Result<_>>()
            .expect("Failed to read airspaces from temp file");

        // Compare counts
        assert_eq!(
            read_back_airspaces.len(),
            original_airspaces.len(),
            "Airspace count mismatch"
        );

        // Compare each airspace
        for (i, (original, read_back)) in original_airspaces
            .iter()
            .zip(read_back_airspaces.iter())
            .enumerate()
        {
            assert_eq!(original.name, read_back.name, "Airspace {i} name mismatch");
            assert_eq!(
                original.points.len(),
                read_back.points.len(),
                "Airspace {i} point count mismatch",
            );

            // Compare points with small floating-point tolerance
            for (j, (orig_point, read_point)) in original
                .points
                .iter()
                .zip(read_back.points.iter())
                .enumerate()
            {
                let lat_diff = (orig_point.lat - read_point.lat).abs();
                let lon_diff = (orig_point.lon - read_point.lon).abs();

                assert!(
                    lat_diff < 0.00001,
                    "Airspace {i} point {j} latitude differs by {lat_diff}",
                );
                assert!(
                    lon_diff < 0.00001,
                    "Airspace {i} point {j} longitude differs by {lon_diff}",
                );
            }
        }
    }
}
