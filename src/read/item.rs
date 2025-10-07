use crate::error::{Result, Warning};
use crate::read::io::*;
use crate::types::{ByteOrder, Header, Item};
use std::io::{Read, Seek, SeekFrom};

/// Iterator that parses items from a CUB file
pub struct ItemIterator<'a, R> {
    reader: &'a mut R,
    byte_order: ByteOrder,
    size_of_item: i32,
    remaining: i32,
    header_offset: i32,
    started: bool,
}

impl<'a, R: Read + Seek> ItemIterator<'a, R> {
    /// Create new item iterator
    pub fn new(reader: &'a mut R, header: &Header, warnings: &mut Vec<Warning>) -> Self {
        let _ = warnings; // Unused for now, but part of API for future warnings

        Self {
            reader,
            byte_order: header.byte_order(),
            size_of_item: header.size_of_item,
            remaining: header.hdr_items,
            header_offset: header.header_offset,
            started: false,
        }
    }
}

impl<'a, R: Read + Seek> Iterator for ItemIterator<'a, R> {
    type Item = Result<Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        // Seek to items section on first call
        if !self.started {
            self.started = true;
            if let Err(e) = self.reader.seek(SeekFrom::Start(self.header_offset as u64)) {
                return Some(Err(e.into()));
            }
        }

        self.remaining -= 1;

        // Create 43-byte zero-filled buffer and read SizeOfItem bytes into it
        // Per spec: remaining bytes should be set to 0 if SizeOfItem < 43
        let mut item_buffer = [0u8; 43];
        let bytes_to_read = std::cmp::min(self.size_of_item as usize, 43);

        if let Err(e) = self.reader.read_exact(&mut item_buffer[..bytes_to_read]) {
            return Some(Err(e.into()));
        }

        // If SizeOfItem > 43, skip the extra bytes
        if self.size_of_item > 43 {
            if let Err(e) = skip_bytes(self.reader, (self.size_of_item - 43) as usize) {
                return Some(Err(e));
            }
        }

        // Parse from the buffer using a cursor (full 43-byte buffer, zero-padded if needed)
        let mut cursor = std::io::Cursor::new(&item_buffer);

        macro_rules! try_parse {
            ($expr:expr) => {
                match $expr {
                    Ok(val) => val,
                    Err(e) => return Some(Err(e)),
                }
            };
        }

        let left = try_parse!(read_f32_le(&mut cursor));
        let top = try_parse!(read_f32_le(&mut cursor));
        let right = try_parse!(read_f32_le(&mut cursor));
        let bottom = try_parse!(read_f32_le(&mut cursor));

        let type_byte = try_parse!(read_u8(&mut cursor));
        let alt_style_byte = try_parse!(read_u8(&mut cursor));
        let min_alt = try_parse!(read_i16(&mut cursor, self.byte_order));
        let max_alt = try_parse!(read_i16(&mut cursor, self.byte_order));
        let points_offset = try_parse!(read_i32(&mut cursor, self.byte_order));
        let time_out = try_parse!(read_i32(&mut cursor, self.byte_order));
        let extra_data = try_parse!(read_u32(&mut cursor, self.byte_order));
        let active_time = try_parse!(read_u64(&mut cursor, self.byte_order));
        let extended_type_byte = try_parse!(read_u8(&mut cursor));

        Some(Ok(Item {
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
        }))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining as usize;
        (remaining, Some(remaining))
    }
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
        let mut warnings = Vec::new();

        let items: Vec<_> = ItemIterator::new(&mut cursor, &header, &mut warnings)
            .collect::<Result<Vec<_>>>()
            .unwrap();

        assert_eq!(items.len(), 2);
        insta::assert_debug_snapshot!(items);
        assert!(warnings.is_empty());
    }
}
