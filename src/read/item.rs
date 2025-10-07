use crate::error::{Result, Warning};
use crate::read::io::*;
use crate::types::{Header, Item};
use std::io::{Read, Seek, SeekFrom};

/// Parse all items from CUB file
pub fn parse_items<R: Read + Seek>(
    reader: &mut R,
    header: &Header,
) -> Result<(Vec<Item>, Vec<Warning>)> {
    let warnings = Vec::new();
    let byte_order = header.byte_order();

    // Seek to items section
    reader.seek(SeekFrom::Start(header.header_offset as u64))?;

    let mut items = Vec::with_capacity(header.hdr_items as usize);

    for _ in 0..header.hdr_items {
        // Create 43-byte zero-filled buffer and read SizeOfItem bytes into it
        // Per spec: remaining bytes should be set to 0 if SizeOfItem < 43
        let mut item_buffer = [0u8; 43];
        let bytes_to_read = std::cmp::min(header.size_of_item as usize, 43);
        reader.read_exact(&mut item_buffer[..bytes_to_read])?;

        // If SizeOfItem > 43, skip the extra bytes
        if header.size_of_item > 43 {
            skip_bytes(reader, (header.size_of_item - 43) as usize)?;
        }

        // Parse from the buffer using a cursor (full 43-byte buffer, zero-padded if needed)
        let mut cursor = std::io::Cursor::new(&item_buffer);

        let left = read_f32_le(&mut cursor)?;
        let top = read_f32_le(&mut cursor)?;
        let right = read_f32_le(&mut cursor)?;
        let bottom = read_f32_le(&mut cursor)?;

        let type_byte = read_u8(&mut cursor)?;
        let alt_style_byte = read_u8(&mut cursor)?;
        let min_alt = read_i16(&mut cursor, byte_order)?;
        let max_alt = read_i16(&mut cursor, byte_order)?;
        let points_offset = read_i32(&mut cursor, byte_order)?;
        let time_out = read_i32(&mut cursor, byte_order)?;
        let extra_data = read_u32(&mut cursor, byte_order)?;
        let active_time = read_u64(&mut cursor, byte_order)?;
        let extended_type_byte = read_u8(&mut cursor)?;

        items.push(Item {
            left,
            top,
            right,
            bottom,
            type_byte,
            alt_style_byte,
            min_alt,
            max_alt,
            points_offset,
            time_out,
            extra_data,
            active_time,
            extended_type_byte,
        });
    }

    Ok((items, warnings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ByteOrder;
    use std::io::Cursor;

    fn minimal_header() -> Header {
        Header {
            title: String::new(),
            allowed_serials: [0; 8],
            pc_byte_order: 1,
            is_secured: 0,
            crc32: 0,
            key: [0; 16],
            size_of_item: 43,
            size_of_point: 5,
            hdr_items: 2,
            max_pts: 100,
            left: 0.0,
            top: 1.0,
            right: 1.0,
            bottom: 0.0,
            max_width: 1.0,
            max_height: 1.0,
            lo_la_scale: 1.0,
            header_offset: 0,
            data_offset: 86, // 2 items × 43 bytes
            alignment: 0,
        }
    }

    fn build_item_bytes(byte_order: ByteOrder) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Item 1
        bytes.extend_from_slice(&0.1f32.to_le_bytes()); // left
        bytes.extend_from_slice(&0.5f32.to_le_bytes()); // top
        bytes.extend_from_slice(&0.4f32.to_le_bytes()); // right
        bytes.extend_from_slice(&0.2f32.to_le_bytes()); // bottom

        bytes.push(0x04); // type (DA)
        bytes.push(0x23); // alt_style (max=FL, min=MSL)

        let write_i16 = |bytes: &mut Vec<u8>, val: i16| {
            bytes.extend_from_slice(&match byte_order {
                ByteOrder::LE => val.to_le_bytes(),
                ByteOrder::BE => val.to_be_bytes(),
            });
        };
        let write_i32 = |bytes: &mut Vec<u8>, val: i32| {
            bytes.extend_from_slice(&match byte_order {
                ByteOrder::LE => val.to_le_bytes(),
                ByteOrder::BE => val.to_be_bytes(),
            });
        };
        let write_u32 = |bytes: &mut Vec<u8>, val: u32| {
            bytes.extend_from_slice(&match byte_order {
                ByteOrder::LE => val.to_le_bytes(),
                ByteOrder::BE => val.to_be_bytes(),
            });
        };
        let write_u64 = |bytes: &mut Vec<u8>, val: u64| {
            bytes.extend_from_slice(&match byte_order {
                ByteOrder::LE => val.to_le_bytes(),
                ByteOrder::BE => val.to_be_bytes(),
            });
        };

        write_i16(&mut bytes, 100); // min_alt
        write_i16(&mut bytes, 5000); // max_alt
        write_i32(&mut bytes, 0); // points_offset
        write_i32(&mut bytes, 0); // time_out
        write_u32(&mut bytes, 0); // extra_data
        write_u64(&mut bytes, 0x3FFFFFF); // active_time (default)
        bytes.push(0); // extended_type

        // Item 2 (copy of item 1 for simplicity)
        let item1 = bytes.clone();
        bytes.extend_from_slice(&item1);

        bytes
    }

    #[test]
    fn parse_items_le() {
        let header = minimal_header();
        let bytes = build_item_bytes(ByteOrder::LE);
        let mut cursor = Cursor::new(bytes);

        let (items, warnings) = parse_items(&mut cursor, &header).unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].left, 0.1);
        assert_eq!(items[0].min_alt, 100);
        assert_eq!(items[0].max_alt, 5000);
        assert!(warnings.is_empty());
    }
}
