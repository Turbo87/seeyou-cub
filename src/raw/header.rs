use crate::error::{Error, Result};
use crate::utils::ByteString;
use crate::utils::io::{read_f32_le, read_i32, read_u8, read_u16, read_u32};
use crate::utils::io::{write_f32_le, write_i32, write_u8, write_u16, write_u32};
use crate::{BoundingBox, ByteOrder};
use std::io::{Read, Write};

/// CUB file magic bytes identifier.
pub const FILE_IDENTIFIER: u32 = 0x425543C2;

/// CUB file header size in bytes (always 210 bytes as defined by the spec).
pub const HEADER_SIZE: usize = 210;

/// Minimum accepted `size_of_item`. Anything below that would not include the
/// `points_offset` field, which is a hard requirement.
const MIN_SIZE_OF_ITEM: i32 = 26;

/// Minimum accepted `size_of_point` (defined by the spec).
const MIN_SIZE_OF_POINT: i32 = 5;

/// CUB file header (first 210 bytes)
///
/// Contains metadata about the airspace file including bounding box,
/// item counts, and structural information needed for parsing.
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    // Raw fields (public)
    pub title: ByteString,
    pub allowed_serials: [u16; 8],
    pub pc_byte_order: u8,
    pub key: [u8; 16],
    pub size_of_item: i32,
    pub size_of_point: i32,
    pub hdr_items: i32,
    pub max_pts: i32,
    pub bounding_box: BoundingBox,
    pub max_width: f32,
    pub max_height: f32,
    pub lo_la_scale: f32,
    pub data_offset: i32,
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

        if ident != FILE_IDENTIFIER {
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
        let _crc32 = read_u32(reader, byte_order)?;

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

        let bounding_box = BoundingBox::read(reader)?;
        let max_width = read_f32_le(reader)?;
        let max_height = read_f32_le(reader)?;
        let lo_la_scale = read_f32_le(reader)?;

        let header_offset = read_i32(reader, byte_order)?;
        if header_offset != HEADER_SIZE as i32 {
            return Err(Error::InvalidHeaderOffset {
                found: header_offset,
            });
        }

        let data_offset = read_i32(reader, byte_order)?;
        let _alignment = read_i32(reader, byte_order)?;

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
            key,
            size_of_item,
            size_of_point,
            hdr_items,
            max_pts,
            bounding_box,
            max_width,
            max_height,
            lo_la_scale,
            data_offset,
        };

        Ok(header)
    }

    /// Get bounding box
    pub fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }

    /// Get byte order for integers
    pub fn byte_order(&self) -> ByteOrder {
        ByteOrder::from_pc_byte_order(self.pc_byte_order)
    }

    /// Write CUB file header to writer
    ///
    /// Writes exactly 210 bytes to the writer.
    ///
    /// # Returns
    ///
    /// Number of bytes written (always 210) or an error
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<usize> {
        let byte_order = self.byte_order();

        // Write magic bytes (offset 0-3, always LE)
        writer.write_all(&FILE_IDENTIFIER.to_le_bytes())?;

        // Write title (offset 4-115, 112 bytes, null-padded)
        let mut title_buf = [0u8; 112];
        let title_bytes = self.title.as_bytes();
        let copy_len = title_bytes.len().min(112);
        title_buf[..copy_len].copy_from_slice(&title_bytes[..copy_len]);
        writer.write_all(&title_buf)?;

        // Write allowed serials (offset 116-131, 8 × u16, always LE)
        for &serial in &self.allowed_serials {
            write_u16(writer, serial, ByteOrder::LE)?;
        }

        // Write PcByteOrder (offset 132)
        write_u8(writer, self.pc_byte_order)?;

        // Write IsSecured (offset 133)
        write_u8(writer, 0)?; // Always 0 (not encrypted)

        // Write Crc32 (offset 134-137)
        write_u32(writer, 0, byte_order)?; // CRC not implemented

        // Write Key (offset 138-153, 16 bytes)
        writer.write_all(&self.key)?;

        // Write remaining header fields (offset 154-209)
        write_i32(writer, self.size_of_item, byte_order)?;
        write_i32(writer, self.size_of_point, byte_order)?;
        write_i32(writer, self.hdr_items, byte_order)?;
        write_i32(writer, self.max_pts, byte_order)?;

        // Floats are always LE
        self.bounding_box.write(writer)?;
        write_f32_le(writer, self.max_width)?;
        write_f32_le(writer, self.max_height)?;
        write_f32_le(writer, self.lo_la_scale)?;

        write_i32(writer, HEADER_SIZE as i32, byte_order)?;
        write_i32(writer, self.data_offset, byte_order)?;

        // Write alignment (offset 206-209)
        write_i32(writer, 0, byte_order)?; // Alignment field (ignored)

        Ok(HEADER_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Cursor;

    #[test]
    fn read_header_from_fixture() {
        let mut file =
            File::open("tests/fixtures/france_2024.07.02.cub").expect("Failed to open fixture");

        let header = Header::read(&mut file).expect("Failed to read header");
        insta::assert_debug_snapshot!(header);
    }

    #[test]
    fn write_header_round_trip() {
        // Create a header with known values
        let original = Header {
            title: ByteString::from(b"Test Airspace".to_vec()),
            allowed_serials: [1, 2, 3, 4, 5, 6, 7, 8],
            pc_byte_order: 0, // LE
            key: [0; 16],
            size_of_item: 42,
            size_of_point: 5,
            hdr_items: 10,
            max_pts: 100,
            bounding_box: BoundingBox {
                left: -1.0,
                top: 1.0,
                right: 1.0,
                bottom: -1.0,
            },
            max_width: 2.0,
            max_height: 2.0,
            lo_la_scale: 1000.0,
            data_offset: 630,
        };

        // Write to buffer
        let mut buf = Vec::new();
        let written = original.write(&mut buf).expect("Failed to write header");
        assert_eq!(written, 210, "Header should be exactly 210 bytes");

        // Read back
        let mut cursor = Cursor::new(buf);
        let read_back = Header::read(&mut cursor).expect("Failed to read header");

        assert_eq!(read_back, original);
    }

    #[test]
    fn write_header_with_be_byte_order() {
        // Create header with BE byte order
        let original = Header {
            title: ByteString::from(b"BE Test".to_vec()),
            allowed_serials: [1, 2, 3, 4, 5, 6, 7, 8],
            pc_byte_order: 1, // BE
            key: [0xFF; 16],
            size_of_item: 43,
            size_of_point: 5,
            hdr_items: 100,
            max_pts: 1000,
            bounding_box: BoundingBox {
                left: -2.5,
                top: 2.5,
                right: 2.5,
                bottom: -2.5,
            },
            max_width: 5.0,
            max_height: 5.0,
            lo_la_scale: 2000.0,
            data_offset: 4510,
        };

        // Write and read back
        let mut buf = Vec::new();
        original.write(&mut buf).expect("Failed to write");
        let mut cursor = Cursor::new(buf);
        let read_back = Header::read(&mut cursor).expect("Failed to read");

        assert_eq!(read_back, original);
    }

    #[test]
    fn write_header_with_empty_title() {
        // Create header with empty title
        let original = Header {
            title: ByteString::from(vec![]),
            allowed_serials: [0; 8],
            pc_byte_order: 0,
            key: [0; 16],
            size_of_item: 26,
            size_of_point: 5,
            hdr_items: 0,
            max_pts: 0,
            bounding_box: BoundingBox {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
            max_width: 0.0,
            max_height: 0.0,
            lo_la_scale: 1.0,
            data_offset: 210,
        };

        // Write and read back
        let mut buf = Vec::new();
        original.write(&mut buf).expect("Failed to write");
        let mut cursor = Cursor::new(buf);
        let read_back = Header::read(&mut cursor).expect("Failed to read");

        assert_eq!(read_back, original);
    }

    #[test]
    fn write_header_with_max_title_length() {
        // Create header with maximum title length (112 bytes)
        let long_title = vec![b'X'; 112];
        let original = Header {
            title: ByteString::from(long_title.clone()),
            allowed_serials: [0; 8],
            pc_byte_order: 0,
            key: [0; 16],
            size_of_item: 43,
            size_of_point: 5,
            hdr_items: 1,
            max_pts: 10,
            bounding_box: BoundingBox {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
            max_width: 0.0,
            max_height: 0.0,
            lo_la_scale: 1.0,
            data_offset: 253,
        };

        // Write and read back
        let mut buf = Vec::new();
        original.write(&mut buf).expect("Failed to write");
        let mut cursor = Cursor::new(buf);
        let read_back = Header::read(&mut cursor).expect("Failed to read");

        assert_eq!(read_back.title.as_bytes(), &long_title[..]);
    }
}
