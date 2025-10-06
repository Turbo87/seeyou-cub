use std::io::{Read, Seek, SeekFrom};
use crate::error::{Result, Warning};
use crate::types::{Header, Item, ParsedPoint, OptionalData, CubDataId};
use crate::read::io::*;

/// Iterator that lazily parses CubPoint sequences for an item
pub struct PointIterator<'a, R> {
    reader: &'a mut R,
    header: &'a Header,
    item: Item,
    origin_x: f32,
    origin_y: f32,
    done: bool,
    warnings: Vec<Warning>,
    // Attributes parsed from current point sequence
    current_name: Option<String>,
    current_frequency: Option<u32>,
    current_frequency_name: Option<String>,
    current_optional: Vec<OptionalData>,
}

impl<'a, R: Read + Seek> PointIterator<'a, R> {
    /// Create new point iterator for an item
    pub(crate) fn new(
        reader: &'a mut R,
        header: &'a Header,
        item: &Item,
    ) -> Result<Self> {
        // Seek to first point for this item
        let offset = header.data_offset as u64 + item.points_offset as u64;
        reader.seek(SeekFrom::Start(offset))?;

        // Initialize origin to item's bottom-left
        let origin_x = item.left;
        let origin_y = item.bottom;

        Ok(Self {
            reader,
            header,
            item: item.clone(),
            origin_x,
            origin_y,
            done: false,
            warnings: Vec::new(),
            current_name: None,
            current_frequency: None,
            current_frequency_name: None,
            current_optional: Vec::new(),
        })
    }

    /// Get warnings collected during parsing
    pub fn warnings(&self) -> &[Warning] {
        &self.warnings
    }

    /// Parse next CubPoint
    fn parse_next_point(&mut self) -> Result<Option<ParsedPoint>> {
        if self.done {
            return Ok(None);
        }

        let byte_order = self.header.byte_order();

        loop {
            let flag = match read_u8(self.reader) {
                Ok(flag) => flag,
                Err(crate::error::Error::IoError(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Hit EOF while trying to read next flag - treat as end of points
                    self.done = true;
                    return Ok(None);
                },
                Err(e) => return Err(e),
            };

            match flag {
                0x81 => {
                    // Set origin offset
                    let delta_x = read_i16(self.reader, byte_order)? as f32 * self.header.lo_la_scale;
                    let delta_y = read_i16(self.reader, byte_order)? as f32 * self.header.lo_la_scale;
                    self.origin_x += delta_x;
                    self.origin_y += delta_y;

                    // Continue reading
                }

                0x01 => {
                    // New point
                    let x = read_i16(self.reader, byte_order)? as f32 * self.header.lo_la_scale;
                    let y = read_i16(self.reader, byte_order)? as f32 * self.header.lo_la_scale;

                    let lon = self.origin_x + x;
                    let lat = self.origin_y + y;

                    // Build point with current attributes
                    let point = ParsedPoint {
                        lon,
                        lat,
                        name: self.current_name.take(),
                        frequency: self.current_frequency.take(),
                        frequency_name: self.current_frequency_name.take(),
                        optional_data: std::mem::take(&mut self.current_optional),
                    };

                    return Ok(Some(point));
                }

                flag if (flag & 0x40) != 0 => {
                    // Attribute block
                    self.parse_attributes(flag)?;
                }

                0x00 => {
                    // End of points
                    self.done = true;
                    return Ok(None);
                }

                _ => {
                    // Unknown flag - collect warning and skip
                    self.warnings.push(Warning::UnknownPointFlag(flag));
                    // Try to skip this point (size_of_point - 1 for flag already read)
                    skip_bytes(self.reader, (self.header.size_of_point - 1) as usize)?;
                }
            }
        }
    }

    /// Parse attribute records starting with given flag
    fn parse_attributes(&mut self, first_flag: u8) -> Result<()> {
        let byte_order = self.header.byte_order();

        // First attribute: name
        if (first_flag & 0x40) != 0 {
            let name_len = (first_flag & 0x3F) as usize;
            if name_len > 0 {
                let name = read_string(self.reader, name_len)?;
                self.current_name = Some(name.trim_end_matches('\0').to_string());
            }
        }

        // Check for frequency attribute
        let next_flag = read_u8(self.reader)?;
        if (next_flag & 0xC0) == 0xC0 {
            let freq_name_len = (next_flag & 0x3F) as usize;
            let frequency = read_u32(self.reader, byte_order)?;
            self.current_frequency = Some(frequency);

            if freq_name_len > 0 {
                let freq_name = read_string(self.reader, freq_name_len)?;
                self.current_frequency_name = Some(freq_name.trim_end_matches('\0').to_string());
            }

            // Read next flag for optional data
            self.parse_optional_data()?;
        } else if next_flag == 0xA0 {
            // Optional data without frequency
            self.parse_optional_data_with_flag(next_flag)?;
        } else {
            // No more attributes, seek back one byte
            use std::io::SeekFrom;
            self.reader.seek(SeekFrom::Current(-1))?;
        }

        Ok(())
    }

    /// Parse optional data records (0xA0 flags)
    fn parse_optional_data(&mut self) -> Result<()> {
        loop {
            let flag = read_u8(self.reader)?;
            if flag != 0xA0 {
                // Not optional data, seek back
                use std::io::SeekFrom;
                self.reader.seek(SeekFrom::Current(-1))?;
                break;
            }

            self.parse_optional_data_record()?;
        }

        Ok(())
    }

    /// Parse optional data with flag already read
    fn parse_optional_data_with_flag(&mut self, flag: u8) -> Result<()> {
        if flag == 0xA0 {
            self.parse_optional_data_record()?;
            self.parse_optional_data()?;
        }
        Ok(())
    }

    /// Parse single optional data record
    fn parse_optional_data_record(&mut self) -> Result<()> {
        let data_id = read_u8(self.reader)?;
        let b1 = read_u8(self.reader)?;
        let b2 = read_u8(self.reader)?;
        let b3 = read_u8(self.reader)?;

        let data_id_enum = CubDataId::from_byte(data_id);

        match data_id_enum {
            Some(CubDataId::IcaoCode) => {
                let len = b3 as usize;
                let icao = read_string(self.reader, len)?;
                self.current_optional.push(OptionalData::IcaoCode(icao));
            }

            Some(CubDataId::SecondaryFrequency) => {
                let value = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);
                self.current_optional.push(OptionalData::SecondaryFrequency(value));
            }

            Some(CubDataId::ExceptionRules) => {
                let len = (((b2 as u16) << 8) | (b3 as u16)) as usize;
                let rules = read_string(self.reader, len)?;
                self.current_optional.push(OptionalData::ExceptionRules(rules));
            }

            Some(CubDataId::NotamRemarks) => {
                let len = (((b2 as u16) << 8) | (b3 as u16)) as usize;
                let remarks = read_string(self.reader, len)?;
                self.current_optional.push(OptionalData::NotamRemarks(remarks));
            }

            Some(CubDataId::NotamId) => {
                let len = b3 as usize;
                let id = read_string(self.reader, len)?;
                self.current_optional.push(OptionalData::NotamId(id));
            }

            Some(CubDataId::NotamInsertTime) => {
                let b4 = read_u8(self.reader)?;
                let value = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);
                let time = (value << 8) | (b4 as u32);
                self.current_optional.push(OptionalData::NotamInsertTime(time));
            }

            None => {
                self.warnings.push(Warning::UnknownPointFlag(data_id));
            }
        }

        Ok(())
    }
}

impl<'a, R: Read + Seek> Iterator for PointIterator<'a, R> {
    type Item = Result<ParsedPoint>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next_point().transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn minimal_header() -> Header {
        Header {
            title: String::new(),
            allowed_serials: [0; 8],
            pc_byte_order: 1,
            is_secured: 0,
            crc32: 0,
            key: [0; 16],
            size_of_item: 42,
            size_of_point: 5,
            hdr_items: 1,
            max_pts: 100,
            left: 0.0,
            top: 1.0,
            right: 1.0,
            bottom: 0.0,
            max_width: 1.0,
            max_height: 1.0,
            lo_la_scale: 0.0001,
            header_offset: 0,
            data_offset: 0,  // Points start at beginning for this test
            alignment: 0,
        }
    }

    fn minimal_item() -> Item {
        Item {
            left: 0.0,
            top: 1.0,
            right: 1.0,
            bottom: 0.0,
            type_byte: 0x04,
            alt_style_byte: 0,
            min_alt: 0,
            max_alt: 1000,
            points_offset: 0,
            time_out: 0,
            extra_data: 0,
            active_time: 0,
            extended_type_byte: 0,
        }
    }

    #[test]
    fn parse_simple_point() {
        let mut bytes = Vec::new();

        // Point: 0x01 flag + coords
        bytes.push(0x01);
        bytes.extend_from_slice(&100i16.to_le_bytes());  // x offset
        bytes.extend_from_slice(&200i16.to_le_bytes());  // y offset

        // End marker
        bytes.push(0x00);

        let mut cursor = Cursor::new(bytes);
        let header = minimal_header();
        let item = minimal_item();

        let mut iter = PointIterator::new(&mut cursor, &header, &item).unwrap();

        let point = iter.next().unwrap().unwrap();
        assert!((point.lon - (0.0 + 100.0 * 0.0001)).abs() < 0.00001);
        assert!((point.lat - (0.0 + 200.0 * 0.0001)).abs() < 0.00001);
        assert!(point.name.is_none());

        assert!(iter.next().is_none());
    }
}