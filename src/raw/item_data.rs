use crate::error::Result;
use crate::raw::{Header, PointOp};
use crate::utils::ByteString;
use crate::utils::io::{read_i16, read_u8, read_u32, write_i16, write_u8, write_u32};
use crate::{CubDataId, Error};
use std::io::{Read, Write};

/// Low-level item data with raw point operations and unprocessed attributes
///
/// This struct represents data as close to the file format as possible:
/// - Point operations are raw i16 offsets (not yet converted to lat/lon)
/// - Strings are raw bytes (not yet decoded from UTF-8/Extended ASCII)
/// - Optional attributes remain as raw bytes for maximum flexibility
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemData {
    /// Raw point operations (origin moves and new points with i16 x/y offsets)
    pub point_ops: Vec<PointOp>,

    /// Airspace name (raw bytes, not decoded)
    pub name: Option<ByteString>,
    /// Primary frequency in Hz
    pub frequency: Option<u32>,
    /// Primary frequency name/label (raw bytes, not decoded)
    pub frequency_name: Option<ByteString>,
    /// ICAO code (raw bytes, not decoded)
    pub icao_code: Option<ByteString>,
    /// Secondary frequency in Hz
    pub secondary_frequency: Option<u32>,
    /// Class exception rules (raw bytes, not decoded)
    pub exception_rules: Option<ByteString>,
    /// NOTAM remarks (raw bytes, not decoded)
    pub notam_remarks: Option<ByteString>,
    /// NOTAM identifier (raw bytes, not decoded)
    pub notam_id: Option<ByteString>,
    /// NOTAM insert time (raw encoded value)
    pub notam_insert_time: Option<u32>,
}

impl ItemData {
    /// Read raw item data from the current position
    ///
    /// Reads point operations and attributes without decoding strings or converting coordinates.
    ///
    /// # Arguments
    ///
    /// * `reader` - Must be positioned at the item's data section
    /// * `header` - Header containing byte order and scaling factor
    ///
    /// # Returns
    ///
    /// The parsed `ItemData` or an error if reading fails
    pub fn read<R: Read>(reader: &mut R, header: &Header) -> Result<Self> {
        let byte_order = header.byte_order();

        let mut item_data = Self {
            point_ops: Vec::with_capacity(4),
            name: None,
            frequency: None,
            frequency_name: None,
            icao_code: None,
            secondary_frequency: None,
            exception_rules: None,
            notam_remarks: None,
            notam_id: None,
            notam_insert_time: None,
        };

        loop {
            let flag = match read_u8(reader) {
                Ok(flag) => flag,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return Ok(item_data);
                }
                Err(e) => return Err(e.into()),
            };

            match flag {
                0x81 => {
                    // Origin update
                    let x = read_i16(reader, byte_order)?;
                    let y = read_i16(reader, byte_order)?;
                    item_data.point_ops.push(PointOp::MoveOrigin { x, y });
                }

                0x01 => {
                    // Geometry point
                    let x = read_i16(reader, byte_order)?;
                    let y = read_i16(reader, byte_order)?;
                    item_data.point_ops.push(PointOp::NewPoint { x, y });
                }

                flag if (flag & 0x40) != 0 => {
                    // Attributes section - parse and return
                    return parse_attributes(reader, header, flag, item_data);
                }

                _ => {
                    return Err(Error::UnexpectedPointFlag(flag));
                }
            }
        }
    }

    /// Write item data to writer
    ///
    /// Writes point operations and all optional attributes.
    ///
    /// # Arguments
    ///
    /// * `writer` - Writer to write to
    /// * `header` - Header containing byte order and size_of_point
    ///
    /// # Returns
    ///
    /// Number of bytes written or an error
    pub fn write<W: Write>(&self, writer: &mut W, header: &Header) -> Result<usize> {
        let byte_order = header.byte_order();
        let mut bytes_written = 0;

        // Write point operations
        for point_op in &self.point_ops {
            match point_op {
                PointOp::MoveOrigin { x, y } => {
                    write_u8(writer, 0x81)?;
                    write_i16(writer, *x, byte_order)?;
                    write_i16(writer, *y, byte_order)?;
                    bytes_written += 5;
                }
                PointOp::NewPoint { x, y } => {
                    write_u8(writer, 0x01)?;
                    write_i16(writer, *x, byte_order)?;
                    write_i16(writer, *y, byte_order)?;
                    bytes_written += 5;
                }
            }
        }

        // Write name attribute if present
        if let Some(ref name) = self.name {
            let name_len = name.as_bytes().len().min(63); // Max 6 bits
            let flag = 0x40 | (name_len as u8);
            write_u8(writer, flag)?;
            bytes_written += 1;

            // Write remaining bytes of point structure (size_of_point - 1)
            let padding_len = (header.size_of_point - 1) as usize;
            let padding = vec![0u8; padding_len];
            writer.write_all(&padding)?;
            bytes_written += padding_len;

            // Write name bytes
            if name_len > 0 {
                writer.write_all(&name.as_bytes()[..name_len])?;
                bytes_written += name_len;
            }
        }

        // Write frequency attribute if present
        if let Some(freq) = self.frequency {
            let freq_name_len = self
                .frequency_name
                .as_ref()
                .map(|n| n.as_bytes().len().min(63))
                .unwrap_or(0);
            let flag = 0xC0 | (freq_name_len as u8);
            write_u8(writer, flag)?;
            bytes_written += 1;

            write_u32(writer, freq, byte_order)?;
            bytes_written += 4;

            if let Some(ref freq_name) = self.frequency_name
                && freq_name_len > 0
            {
                writer.write_all(&freq_name.as_bytes()[..freq_name_len])?;
                bytes_written += freq_name_len;
            }
        }

        // Write optional data records (all start with 0xA0 flag)

        // ICAO Code
        if let Some(ref icao_code) = self.icao_code {
            let len = icao_code.as_bytes().len().min(255) as u8;
            write_u8(writer, 0xA0)?;
            write_u8(writer, CubDataId::IcaoCode.as_byte())?;
            write_u8(writer, 0)?; // b1 (unused)
            write_u8(writer, 0)?; // b2 (unused)
            write_u8(writer, len)?; // b3 = length
            writer.write_all(icao_code.as_bytes())?;
            bytes_written += 5 + len as usize;
        }

        // Secondary Frequency
        if let Some(freq) = self.secondary_frequency {
            write_u8(writer, 0xA0)?;
            write_u8(writer, CubDataId::SecondaryFrequency.as_byte())?;
            write_u8(writer, ((freq >> 16) & 0xFF) as u8)?; // b1
            write_u8(writer, ((freq >> 8) & 0xFF) as u8)?; // b2
            write_u8(writer, (freq & 0xFF) as u8)?; // b3
            bytes_written += 5;
        }

        // Exception Rules
        if let Some(ref rules) = self.exception_rules {
            let len = rules.as_bytes().len().min(65535) as u16;
            write_u8(writer, 0xA0)?;
            write_u8(writer, CubDataId::ExceptionRules.as_byte())?;
            write_u8(writer, 0)?; // b1 (unused)
            write_u8(writer, ((len >> 8) & 0xFF) as u8)?; // b2
            write_u8(writer, (len & 0xFF) as u8)?; // b3
            writer.write_all(rules.as_bytes())?;
            bytes_written += 5 + len as usize;
        }

        // NOTAM Remarks
        if let Some(ref remarks) = self.notam_remarks {
            let len = remarks.as_bytes().len().min(65535) as u16;
            write_u8(writer, 0xA0)?;
            write_u8(writer, CubDataId::NotamRemarks.as_byte())?;
            write_u8(writer, 0)?; // b1 (unused)
            write_u8(writer, ((len >> 8) & 0xFF) as u8)?; // b2
            write_u8(writer, (len & 0xFF) as u8)?; // b3
            writer.write_all(remarks.as_bytes())?;
            bytes_written += 5 + len as usize;
        }

        // NOTAM ID
        if let Some(ref notam_id) = self.notam_id {
            let len = notam_id.as_bytes().len().min(255) as u8;
            write_u8(writer, 0xA0)?;
            write_u8(writer, CubDataId::NotamId.as_byte())?;
            write_u8(writer, 0)?; // b1 (unused)
            write_u8(writer, 0)?; // b2 (unused)
            write_u8(writer, len)?; // b3 = length
            writer.write_all(notam_id.as_bytes())?;
            bytes_written += 5 + len as usize;
        }

        // NOTAM Insert Time
        if let Some(time) = self.notam_insert_time {
            write_u8(writer, 0xA0)?;
            write_u8(writer, CubDataId::NotamInsertTime.as_byte())?;
            write_u8(writer, ((time >> 24) & 0xFF) as u8)?; // b1
            write_u8(writer, ((time >> 16) & 0xFF) as u8)?; // b2
            write_u8(writer, ((time >> 8) & 0xFF) as u8)?; // b3
            write_u8(writer, (time & 0xFF) as u8)?; // b4
            bytes_written += 6;
        }

        Ok(bytes_written)
    }
}

/// Parse attribute section starting with given flag
fn parse_attributes<R: Read>(
    reader: &mut R,
    header: &Header,
    first_flag: u8,
    mut item_data: ItemData,
) -> Result<ItemData> {
    let byte_order = header.byte_order();

    // First attribute: name
    if (first_flag & 0x40) != 0 {
        // Skip remaining bytes of point structure
        let skip_count = (header.size_of_point - 1) as usize;
        let mut discard = vec![0u8; skip_count];
        reader.read_exact(&mut discard)?;

        let name_len = (first_flag & 0x3F) as usize;
        if name_len > 0 {
            item_data.name = Some(ByteString::read(reader, name_len)?);
        }
    }

    // Parse all optional attributes (frequency and 0xA0 records)
    loop {
        let flag = match read_u8(reader) {
            Ok(flag) => flag,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(item_data);
            }
            Err(e) => return Err(e.into()),
        };

        match flag {
            flag if (flag & 0xC0) == 0xC0 => {
                // Frequency attribute
                let freq_name_len = (flag & 0x3F) as usize;
                item_data.frequency = Some(read_u32(reader, byte_order)?);

                if freq_name_len > 0 {
                    item_data.frequency_name = Some(ByteString::read(reader, freq_name_len)?);
                }
            }

            0xA0 => {
                parse_optional_data_record(reader, &mut item_data)?;
            }

            _ => {
                // Unknown flag, stop parsing
                return Ok(item_data);
            }
        }
    }
}

/// Parse single optional data record
fn parse_optional_data_record<R: Read>(reader: &mut R, item_data: &mut ItemData) -> Result<()> {
    let data_id = read_u8(reader)?;
    let b1 = read_u8(reader)?;
    let b2 = read_u8(reader)?;
    let b3 = read_u8(reader)?;

    match CubDataId::from_byte(data_id) {
        Some(CubDataId::IcaoCode) => {
            let len = b3 as usize;
            item_data.icao_code = Some(ByteString::read(reader, len)?);
        }

        Some(CubDataId::SecondaryFrequency) => {
            let value = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);
            item_data.secondary_frequency = Some(value);
        }

        Some(CubDataId::ExceptionRules) => {
            let len = (((b2 as u16) << 8) | (b3 as u16)) as usize;
            item_data.exception_rules = Some(ByteString::read(reader, len)?);
        }

        Some(CubDataId::NotamRemarks) => {
            let len = (((b2 as u16) << 8) | (b3 as u16)) as usize;
            item_data.notam_remarks = Some(ByteString::read(reader, len)?);
        }

        Some(CubDataId::NotamId) => {
            let len = b3 as usize;
            item_data.notam_id = Some(ByteString::read(reader, len)?);
        }

        Some(CubDataId::NotamInsertTime) => {
            let b4 = read_u8(reader)?;
            let value =
                ((b1 as u32) << 24) | ((b2 as u32) << 16) | ((b3 as u32) << 8) | (b4 as u32);
            item_data.notam_insert_time = Some(value);
        }

        // Unknown data ID
        None => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::Item;
    use std::fs::File;
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn read_item_data_from_fixture() {
        let mut file =
            File::open("tests/fixtures/france_2024.07.02.cub").expect("Failed to open fixture");

        // Read header and first item
        let header = Header::read(&mut file).unwrap();

        file.seek(SeekFrom::Start(header.header_offset as u64))
            .unwrap();
        let item = Item::read(&mut file, &header).unwrap();

        // Seek to item data
        let data_offset = header.data_offset + item.points_offset;
        file.seek(SeekFrom::Start(data_offset as u64)).unwrap();

        // Read item data
        let item_data = ItemData::read(&mut file, &header).expect("Failed to read item data");
        insta::assert_debug_snapshot!(item_data);

        // Verify name field is raw bytes and can be decoded
        assert!(item_data.name.is_some());
        let name_bytes = item_data.name.as_ref().unwrap().as_bytes();
        let name_str = String::from_utf8_lossy(name_bytes);
        assert_eq!(name_str, "R265 LA GREMUSE");
    }

    #[test]
    fn read_item_data_with_all_optional_fields() {
        // Create minimal header with LE byte order and size_of_point = 5
        let header = Header {
            title: ByteString::from(b"Test".to_vec()),
            allowed_serials: [0; 8],
            pc_byte_order: 1, // LE
            key: [0; 16],
            size_of_item: 42,
            size_of_point: 5,
            hdr_items: 1,
            max_pts: 100,
            bounding_box: crate::BoundingBox {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
            max_width: 0.0,
            max_height: 0.0,
            lo_la_scale: 0.0001,
            header_offset: 0,
            data_offset: 0,
        };

        // Build byte stream with all optional fields
        let mut data = Vec::new();

        // Add point operations (each is 5 bytes: flag + x + y)
        data.push(0x81); // Set origin
        data.extend_from_slice(&100i16.to_le_bytes()); // x
        data.extend_from_slice(&200i16.to_le_bytes()); // y

        data.push(0x01); // New point
        data.extend_from_slice(&10i16.to_le_bytes()); // x
        data.extend_from_slice(&20i16.to_le_bytes()); // y

        data.push(0x01); // Another point
        data.extend_from_slice(&30i16.to_le_bytes()); // x
        data.extend_from_slice(&40i16.to_le_bytes()); // y

        // Name attribute (flag & 0x40 != 0)
        let name = b"Test Airspace";
        data.push(0x40 | (name.len() as u8)); // Name flag + length
        data.extend_from_slice(&[0u8; 4]); // Remaining bytes of point structure
        data.extend_from_slice(name);

        // Frequency attribute (flag & 0xC0 == 0xC0)
        let freq_name = b"Tower";
        data.push(0xC0 | (freq_name.len() as u8)); // Frequency flag + freq name length
        data.extend_from_slice(&123450u32.to_le_bytes()); // Frequency in Hz
        data.extend_from_slice(freq_name);

        // Optional data records (all with flag 0xA0)

        // ICAO Code (CubDataId = 0)
        data.push(0xA0);
        data.push(0);
        data.push(0); // b1 (unused)
        data.push(0); // b2 (unused)
        let icao = b"LFPG";
        data.push(icao.len() as u8); // b3 = length
        data.extend_from_slice(icao);

        // Secondary Frequency (CubDataId = 1)
        data.push(0xA0);
        data.push(1);
        let sec_freq = 128500u32;
        data.push(((sec_freq >> 16) & 0xFF) as u8);
        data.push(((sec_freq >> 8) & 0xFF) as u8);
        data.push((sec_freq & 0xFF) as u8);

        // Exception Rules (CubDataId = 2)
        data.push(0xA0);
        data.push(2);
        let exc_rules = b"Class D when tower active";
        let exc_len = exc_rules.len() as u16;
        data.push(0); // b1 (unused)
        data.push(((exc_len >> 8) & 0xFF) as u8); // b2
        data.push((exc_len & 0xFF) as u8); // b3
        data.extend_from_slice(exc_rules);

        // NOTAM Remarks (CubDataId = 3)
        data.push(0xA0);
        data.push(3);
        let notam_remarks = b"Active during airshow";
        let notam_len = notam_remarks.len() as u16;
        data.push(0); // b1 (unused)
        data.push(((notam_len >> 8) & 0xFF) as u8); // b2
        data.push((notam_len & 0xFF) as u8); // b3
        data.extend_from_slice(notam_remarks);

        // NOTAM ID (CubDataId = 4)
        data.push(0xA0);
        data.push(4);
        data.push(0); // b1 (unused)
        data.push(0); // b2 (unused)
        let notam_id = b"A1234/25";
        data.push(notam_id.len() as u8); // b3 = length
        data.extend_from_slice(notam_id);

        // NOTAM Insert Time (CubDataId = 5)
        data.push(0xA0);
        data.push(5);
        let insert_time = 0x12345678u32;
        data.push(((insert_time >> 24) & 0xFF) as u8); // b1
        data.push(((insert_time >> 16) & 0xFF) as u8); // b2
        data.push(((insert_time >> 8) & 0xFF) as u8); // b3
        data.push((insert_time & 0xFF) as u8); // b4

        let mut cursor = Cursor::new(data);
        let item_data = ItemData::read(&mut cursor, &header).expect("Failed to read item data");

        insta::assert_debug_snapshot!(item_data);
    }

    #[test]
    fn write_item_data_round_trip() {
        // Create a minimal header
        let header = Header {
            title: ByteString::from(vec![]),
            allowed_serials: [0; 8],
            pc_byte_order: 0, // LE
            key: [0; 16],
            size_of_item: 43,
            size_of_point: 5,
            hdr_items: 1,
            max_pts: 10,
            bounding_box: crate::BoundingBox {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
            max_width: 0.0,
            max_height: 0.0,
            lo_la_scale: 1000.0,
            header_offset: 210,
            data_offset: 253,
        };

        // Create item data with all fields populated
        let original = ItemData {
            point_ops: vec![
                PointOp::MoveOrigin { x: 100, y: 200 },
                PointOp::NewPoint { x: 10, y: 20 },
                PointOp::NewPoint { x: 30, y: 40 },
            ],
            name: Some(ByteString::from(b"Test Airspace".to_vec())),
            frequency: Some(123450),
            frequency_name: Some(ByteString::from(b"Tower".to_vec())),
            icao_code: Some(ByteString::from(b"LFPG".to_vec())),
            secondary_frequency: Some(128500),
            exception_rules: Some(ByteString::from(b"Class D when tower active".to_vec())),
            notam_remarks: Some(ByteString::from(b"Active during airshow".to_vec())),
            notam_id: Some(ByteString::from(b"A1234/25".to_vec())),
            notam_insert_time: Some(0x12345678),
        };

        // Write to buffer
        let mut buf = Vec::new();
        let written = original
            .write(&mut buf, &header)
            .expect("Failed to write item data");
        assert!(written > 0);

        // Read back
        let mut cursor = Cursor::new(buf);
        let read_back = ItemData::read(&mut cursor, &header).expect("Failed to read item data");

        // Verify all fields match
        assert_eq!(read_back, original);
    }

    #[test]
    fn write_item_data_with_be_byte_order() {
        // Create header with BE byte order
        let header = Header {
            title: ByteString::from(vec![]),
            allowed_serials: [0; 8],
            pc_byte_order: 1, // BE
            key: [0; 16],
            size_of_item: 43,
            size_of_point: 5,
            hdr_items: 1,
            max_pts: 10,
            bounding_box: crate::BoundingBox {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
            max_width: 0.0,
            max_height: 0.0,
            lo_la_scale: 1000.0,
            header_offset: 210,
            data_offset: 253,
        };

        let original = ItemData {
            point_ops: vec![
                PointOp::MoveOrigin { x: -500, y: 500 },
                PointOp::NewPoint { x: 100, y: -100 },
            ],
            name: Some(ByteString::from(b"BE Test".to_vec())),
            frequency: Some(118500),
            frequency_name: Some(ByteString::from(b"ATIS".to_vec())),
            icao_code: None,
            secondary_frequency: None,
            exception_rules: None,
            notam_remarks: None,
            notam_id: None,
            notam_insert_time: None,
        };

        // Write and read back
        let mut buf = Vec::new();
        original.write(&mut buf, &header).expect("Failed to write");
        let mut cursor = Cursor::new(buf);
        let read_back = ItemData::read(&mut cursor, &header).expect("Failed to read");

        assert_eq!(read_back, original);
    }

    #[test]
    fn write_item_data_with_no_optional_fields() {
        let header = Header {
            title: ByteString::from(vec![]),
            allowed_serials: [0; 8],
            pc_byte_order: 0,
            key: [0; 16],
            size_of_item: 43,
            size_of_point: 5,
            hdr_items: 1,
            max_pts: 10,
            bounding_box: crate::BoundingBox {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
            max_width: 0.0,
            max_height: 0.0,
            lo_la_scale: 1000.0,
            header_offset: 210,
            data_offset: 253,
        };

        // Only point operations, no optional fields
        let original = ItemData {
            point_ops: vec![PointOp::NewPoint { x: 10, y: 20 }],
            name: None,
            frequency: None,
            frequency_name: None,
            icao_code: None,
            secondary_frequency: None,
            exception_rules: None,
            notam_remarks: None,
            notam_id: None,
            notam_insert_time: None,
        };

        // Write and read back
        let mut buf = Vec::new();
        original.write(&mut buf, &header).expect("Failed to write");
        let mut cursor = Cursor::new(buf);
        let read_back = ItemData::read(&mut cursor, &header).expect("Failed to read");

        assert_eq!(read_back, original);
    }

    #[test]
    fn write_item_data_with_max_string_lengths() {
        let header = Header {
            title: ByteString::from(vec![]),
            allowed_serials: [0; 8],
            pc_byte_order: 0,
            key: [0; 16],
            size_of_item: 43,
            size_of_point: 5,
            hdr_items: 1,
            max_pts: 10,
            bounding_box: crate::BoundingBox {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
            max_width: 0.0,
            max_height: 0.0,
            lo_la_scale: 1000.0,
            header_offset: 210,
            data_offset: 253,
        };

        // Create strings at maximum lengths
        let max_63_bytes = vec![b'A'; 63];
        let max_255_bytes = vec![b'B'; 255];
        let max_65535_bytes = vec![b'C'; 65535];

        let original = ItemData {
            point_ops: vec![PointOp::NewPoint { x: 0, y: 0 }],
            name: Some(ByteString::from(max_63_bytes.clone())),
            frequency: Some(123450),
            frequency_name: Some(ByteString::from(max_63_bytes.clone())),
            icao_code: Some(ByteString::from(max_255_bytes.clone())),
            secondary_frequency: Some(0xFFFFFF),
            exception_rules: Some(ByteString::from(max_65535_bytes.clone())),
            notam_remarks: Some(ByteString::from(b"Max remarks".to_vec())),
            notam_id: Some(ByteString::from(max_255_bytes.clone())),
            notam_insert_time: Some(0xFFFFFFFF),
        };

        // Write and read back
        let mut buf = Vec::new();
        original.write(&mut buf, &header).expect("Failed to write");
        let mut cursor = Cursor::new(buf);
        let read_back = ItemData::read(&mut cursor, &header).expect("Failed to read");

        assert_eq!(read_back, original);
    }

    #[test]
    fn write_item_data_with_many_point_operations() {
        let header = Header {
            title: ByteString::from(vec![]),
            allowed_serials: [0; 8],
            pc_byte_order: 0,
            key: [0; 16],
            size_of_item: 43,
            size_of_point: 5,
            hdr_items: 1,
            max_pts: 100,
            bounding_box: crate::BoundingBox {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
            max_width: 0.0,
            max_height: 0.0,
            lo_la_scale: 1000.0,
            header_offset: 210,
            data_offset: 253,
        };

        // Create various point operation patterns
        let mut point_ops = vec![
            PointOp::MoveOrigin { x: 1000, y: 2000 },
            PointOp::NewPoint { x: 10, y: 20 },
            PointOp::NewPoint { x: 30, y: 40 },
            PointOp::MoveOrigin { x: -500, y: -500 },
            PointOp::NewPoint { x: 5, y: 5 },
        ];

        // Add many more points
        for i in 0..50 {
            point_ops.push(PointOp::NewPoint {
                x: i * 10,
                y: i * 20,
            });
        }

        let original = ItemData {
            point_ops,
            name: Some(ByteString::from(b"Complex polygon".to_vec())),
            frequency: None,
            frequency_name: None,
            icao_code: None,
            secondary_frequency: None,
            exception_rules: None,
            notam_remarks: None,
            notam_id: None,
            notam_insert_time: None,
        };

        // Write and read back
        let mut buf = Vec::new();
        original.write(&mut buf, &header).expect("Failed to write");
        let mut cursor = Cursor::new(buf);
        let read_back = ItemData::read(&mut cursor, &header).expect("Failed to read");

        assert_eq!(read_back.point_ops.len(), original.point_ops.len());
        assert_eq!(read_back, original);
    }
}
