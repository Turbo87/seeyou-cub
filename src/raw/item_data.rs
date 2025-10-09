use crate::error::Result;
use crate::raw::io::{read_bytes, read_i16, read_u8, read_u32};
use crate::{CubDataId, Error, Header, PointOp, RawItemData};
use std::io::Read;

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
/// The parsed `RawItemData` or an error if reading fails
pub fn read_item_data<R: Read>(reader: &mut R, header: &Header) -> Result<RawItemData> {
    let byte_order = header.byte_order();

    let mut item_data = RawItemData {
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
            Err(Error::IoError(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(item_data);
            }
            Err(e) => return Err(e),
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

/// Parse attribute section starting with given flag
fn parse_attributes<R: Read>(
    reader: &mut R,
    header: &Header,
    first_flag: u8,
    mut item_data: RawItemData,
) -> Result<RawItemData> {
    let byte_order = header.byte_order();

    // First attribute: name
    if (first_flag & 0x40) != 0 {
        // Skip remaining bytes of point structure
        let skip_count = (header.size_of_point - 1) as usize;
        let mut discard = vec![0u8; skip_count];
        reader.read_exact(&mut discard)?;

        let name_len = (first_flag & 0x3F) as usize;
        if name_len > 0 {
            item_data.name = Some(read_bytes(reader, name_len)?);
        }
    }

    // Parse all optional attributes (frequency and 0xA0 records)
    loop {
        let flag = match read_u8(reader) {
            Ok(flag) => flag,
            Err(Error::IoError(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(item_data);
            }
            Err(e) => return Err(e),
        };

        match flag {
            flag if (flag & 0xC0) == 0xC0 => {
                // Frequency attribute
                let freq_name_len = (flag & 0x3F) as usize;
                item_data.frequency = Some(read_u32(reader, byte_order)?);

                if freq_name_len > 0 {
                    item_data.frequency_name = Some(read_bytes(reader, freq_name_len)?);
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
fn parse_optional_data_record<R: Read>(reader: &mut R, item_data: &mut RawItemData) -> Result<()> {
    let data_id = read_u8(reader)?;
    let b1 = read_u8(reader)?;
    let b2 = read_u8(reader)?;
    let b3 = read_u8(reader)?;

    match CubDataId::from_byte(data_id) {
        Some(CubDataId::IcaoCode) => {
            let len = b3 as usize;
            item_data.icao_code = Some(read_bytes(reader, len)?);
        }

        Some(CubDataId::SecondaryFrequency) => {
            let value = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);
            item_data.secondary_frequency = Some(value);
        }

        Some(CubDataId::ExceptionRules) => {
            let len = (((b2 as u16) << 8) | (b3 as u16)) as usize;
            item_data.exception_rules = Some(read_bytes(reader, len)?);
        }

        Some(CubDataId::NotamRemarks) => {
            let len = (((b2 as u16) << 8) | (b3 as u16)) as usize;
            item_data.notam_remarks = Some(read_bytes(reader, len)?);
        }

        Some(CubDataId::NotamId) => {
            let len = b3 as usize;
            item_data.notam_id = Some(read_bytes(reader, len)?);
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
    use crate::raw::{read_header, read_item};
    use std::fs::File;
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn read_item_data_from_fixture() {
        let mut file =
            File::open("tests/fixtures/france_2024.07.02.cub").expect("Failed to open fixture");

        // Read header and first item
        let header = read_header(&mut file).unwrap();

        file.seek(SeekFrom::Start(header.header_offset as u64))
            .unwrap();
        let item = read_item(&mut file, &header).unwrap();

        // Seek to item data
        let data_offset = header.data_offset + item.points_offset;
        file.seek(SeekFrom::Start(data_offset as u64)).unwrap();

        // Read item data
        let item_data = read_item_data(&mut file, &header).expect("Failed to read item data");
        insta::assert_debug_snapshot!(item_data);

        // Verify name field is raw bytes and can be decoded
        assert!(item_data.name.is_some());
        let name_bytes = item_data.name.as_ref().unwrap();
        let name_str = String::from_utf8_lossy(name_bytes);
        assert_eq!(name_str, "R265 LA GREMUSE");
    }

    #[test]
    fn read_item_data_with_all_optional_fields() {
        // Create minimal header with LE byte order and size_of_point = 5
        let header = Header {
            title: String::from("Test"),
            allowed_serials: [0; 8],
            pc_byte_order: 1, // LE
            is_secured: 0,
            crc32: 0,
            key: [0; 16],
            size_of_item: 42,
            size_of_point: 5,
            hdr_items: 1,
            max_pts: 100,
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            max_width: 0.0,
            max_height: 0.0,
            lo_la_scale: 0.0001,
            header_offset: 0,
            data_offset: 0,
            alignment: 0,
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
        let item_data = read_item_data(&mut cursor, &header).expect("Failed to read item data");

        insta::assert_debug_snapshot!(item_data);
    }
}
