use crate::error::Result;
use crate::raw::io::{read_f32_le, read_i16, read_i32, read_u8, read_u32, read_u64};
use crate::types::{Header, Item};
use std::io::{Cursor, Read};

impl Item {
    /// Read a single airspace item from current position
    ///
    /// Reads exactly `header.size_of_item` bytes and parses them into an `Item`.
    ///
    /// # Arguments
    ///
    /// * `reader` - Must be positioned at the start of an item
    /// * `header` - Header containing byte order and item size
    ///
    /// # Returns
    ///
    /// The parsed `Item` or an error if reading/parsing fails
    pub fn read<R: Read>(reader: &mut R, header: &Header) -> Result<Self> {
        // Create 43-byte zero-filled buffer and read `size_of_item` bytes into it
        // Per spec: remaining bytes should be set to `0` if `size_of_item < 43`
        let mut item_buffer = [0u8; 43];
        let bytes_to_read = std::cmp::min(header.size_of_item as usize, 43);

        reader.read_exact(&mut item_buffer[..bytes_to_read])?;

        // If size_of_item > 43, read and discard the extra bytes
        if header.size_of_item > 43 {
            let extra_bytes = (header.size_of_item - 43) as usize;
            let mut discard = vec![0u8; extra_bytes];
            reader.read_exact(&mut discard)?;
        }

        // Parse from the buffer using a cursor (full 43-byte zero-padded buffer)
        let mut cursor = Cursor::new(&item_buffer);

        // Read bounding box
        let left = read_f32_le(&mut cursor)?;
        let top = read_f32_le(&mut cursor)?;
        let right = read_f32_le(&mut cursor)?;
        let bottom = read_f32_le(&mut cursor)?;

        // Read bit-packed fields
        let byte_order = header.byte_order();
        let type_byte = read_u8(&mut cursor)?;
        let alt_style_byte = read_u8(&mut cursor)?;
        let min_alt = read_i16(&mut cursor, byte_order)?;
        let max_alt = read_i16(&mut cursor, byte_order)?;
        let points_offset = read_i32(&mut cursor, byte_order)?;
        let time_out = read_i32(&mut cursor, byte_order)?;
        let extra_data = read_u32(&mut cursor, byte_order)?;

        let mut active_time = read_u64(&mut cursor, byte_order)?;
        if active_time == 0 {
            active_time = 0x3FFFFFF;
        }

        let extended_type_byte = read_u8(&mut cursor)?;

        Ok(Self {
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn read_item_from_fixture() {
        let mut file =
            File::open("tests/fixtures/france_2024.07.02.cub").expect("Failed to open fixture");

        // Read header first to get item offset
        let header = Header::read(&mut file).unwrap();

        // Seek to first item
        file.seek(SeekFrom::Start(header.header_offset as u64))
            .unwrap();

        // Read item
        let item = Item::read(&mut file, &header).expect("Failed to read item");
        insta::assert_debug_snapshot!(item);
    }
}
