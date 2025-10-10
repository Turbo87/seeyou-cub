use crate::ByteOrder;
use crate::error::{Error, Result};
use crate::utils::ByteString;
use crate::utils::io::{read_f32_le, read_i32, read_u8, read_u16, read_u32};
use std::io::Read;

/// Minimum accepted `size_of_item`. Anything below that would not include the
/// `points_offset` field, which is a hard requirement.
const MIN_SIZE_OF_ITEM: i32 = 26;

/// Minimum accepted `size_of_point` (defined by the spec).
const MIN_SIZE_OF_POINT: i32 = 5;

/// CUB file header (first 210 bytes)
///
/// Contains metadata about the airspace file including bounding box,
/// item counts, and structural information needed for parsing.
#[derive(Debug, Clone)]
pub struct Header {
    // Raw fields (public)
    pub title: ByteString,
    pub allowed_serials: [u16; 8],
    pub pc_byte_order: u8,
    pub crc32: u32,
    pub key: [u8; 16],
    pub size_of_item: i32,
    pub size_of_point: i32,
    pub hdr_items: i32,
    pub max_pts: i32,
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub max_width: f32,
    pub max_height: f32,
    pub lo_la_scale: f32,
    pub header_offset: i32,
    pub data_offset: i32,
    pub alignment: i32,
}

impl Header {
    /// Read CUB file header from current position
    ///
    /// Reads exactly 210 bytes and parses them into a `Header` struct.
    ///
    /// # Arguments
    ///
    /// * `reader` - Must be positioned at byte 0 (start of file)
    ///
    /// # Returns
    ///
    /// The parsed `Header` or an error if reading/parsing fails
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        // Read and validate magic bytes (offset 0-3, always LE)
        let ident = {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            u32::from_le_bytes(buf)
        };

        if ident != 0x425543C2 {
            return Err(Error::InvalidMagicBytes);
        }

        // Read title (offset 4-115, 112 bytes)
        let mut title_buf = [0u8; 112];
        reader.read_exact(&mut title_buf)?;
        // Trim null padding and convert to Vec only for non-null bytes
        let title = if let Some(pos) = title_buf.iter().rposition(|&b| b != 0) {
            ByteString::from(title_buf[..=pos].to_vec())
        } else {
            ByteString::from(vec![])
        };

        // Read allowed serials (offset 116-131, 8 × u16, always LE)
        let mut allowed_serials = [0u16; 8];
        for serial in &mut allowed_serials {
            *serial = read_u16(reader, ByteOrder::LE)?;
        }

        // Read PcByteOrder (offset 132)
        let pc_byte_order = read_u8(reader)?;
        let byte_order = ByteOrder::from_pc_byte_order(pc_byte_order);

        // Read IsSecured (offset 133)
        let is_secured = read_u8(reader)?;

        // Check encryption
        if is_secured != 0 {
            return Err(Error::EncryptedFile);
        }

        // Read Crc32 (offset 134-137)
        let crc32 = read_u32(reader, byte_order)?;

        // Read Key (offset 138-153, 16 bytes)
        let key = {
            let mut buf = [0u8; 16];
            reader.read_exact(&mut buf)?;
            buf
        };

        // Read remaining header fields
        let size_of_item = read_i32(reader, byte_order)?;
        let size_of_point = read_i32(reader, byte_order)?;
        let hdr_items = read_i32(reader, byte_order)?;
        let max_pts = read_i32(reader, byte_order)?;

        let left = read_f32_le(reader)?;
        let top = read_f32_le(reader)?;
        let right = read_f32_le(reader)?;
        let bottom = read_f32_le(reader)?;
        let max_width = read_f32_le(reader)?;
        let max_height = read_f32_le(reader)?;
        let lo_la_scale = read_f32_le(reader)?;

        let header_offset = read_i32(reader, byte_order)?;
        let data_offset = read_i32(reader, byte_order)?;
        let alignment = read_i32(reader, byte_order)?;

        if size_of_item < MIN_SIZE_OF_ITEM {
            return Err(Error::UndersizedItems { size_of_item });
        }

        if size_of_point < MIN_SIZE_OF_POINT {
            return Err(Error::UndersizedPoints { size_of_point });
        }

        let header = Self {
            title,
            allowed_serials,
            pc_byte_order,
            crc32,
            key,
            size_of_item,
            size_of_point,
            hdr_items,
            max_pts,
            left,
            top,
            right,
            bottom,
            max_width,
            max_height,
            lo_la_scale,
            header_offset,
            data_offset,
            alignment,
        };

        Ok(header)
    }

    /// Get bounding box as (west, south, east, north) in radians
    pub fn bounding_box(&self) -> (f32, f32, f32, f32) {
        (self.left, self.bottom, self.right, self.top)
    }

    /// Get byte order for integers
    pub fn byte_order(&self) -> ByteOrder {
        ByteOrder::from_pc_byte_order(self.pc_byte_order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn read_header_from_fixture() {
        let mut file =
            File::open("tests/fixtures/france_2024.07.02.cub").expect("Failed to open fixture");

        let header = Header::read(&mut file).expect("Failed to read header");
        insta::assert_debug_snapshot!(header);
    }
}
