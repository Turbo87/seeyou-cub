use crate::error::Result;
use crate::utils::io::{read_f32_le, write_f32_le};
use std::io::{Read, Write};

/// Bounding box for geographic areas
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub left: f32,   // west longitude (radians)
    pub top: f32,    // north latitude (radians)
    pub right: f32,  // east longitude (radians)
    pub bottom: f32, // south latitude (radians)
}

impl BoundingBox {
    /// Read bounding box from reader
    ///
    /// Reads 4 f32 values (16 bytes total) in little-endian format.
    ///
    /// # Returns
    ///
    /// The parsed `BoundingBox` or an error if reading fails
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let left = read_f32_le(reader)?;
        let top = read_f32_le(reader)?;
        let right = read_f32_le(reader)?;
        let bottom = read_f32_le(reader)?;

        Ok(Self {
            left,
            top,
            right,
            bottom,
        })
    }

    /// Write bounding box to writer
    ///
    /// Writes 4 f32 values (16 bytes total) in little-endian format.
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        write_f32_le(writer, self.left)?;
        write_f32_le(writer, self.top)?;
        write_f32_le(writer, self.right)?;
        write_f32_le(writer, self.bottom)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_basic_construction() {
        let bbox = BoundingBox {
            left: -0.1,
            top: 0.9,
            right: 0.1,
            bottom: 0.8,
        };

        assert_eq!(bbox.left, -0.1);
        assert_eq!(bbox.top, 0.9);
        assert_eq!(bbox.right, 0.1);
        assert_eq!(bbox.bottom, 0.8);
    }

    #[test]
    fn test_read() {
        // Create binary data for bounding box (4 × f32 LE = 16 bytes)
        let data = [
            0x00, 0x00, 0x80, 0xBF, // -1.0 (left)
            0x00, 0x00, 0x80, 0x3F, // 1.0 (top)
            0x00, 0x00, 0x00, 0x40, // 2.0 (right)
            0x00, 0x00, 0x40, 0xC0, // -3.0 (bottom)
        ];

        let mut cursor = Cursor::new(&data);
        let bbox = BoundingBox::read(&mut cursor).expect("Failed to read");

        assert_eq!(bbox.left, -1.0);
        assert_eq!(bbox.top, 1.0);
        assert_eq!(bbox.right, 2.0);
        assert_eq!(bbox.bottom, -3.0);
    }

    #[test]
    fn test_write() {
        let bbox = BoundingBox {
            left: -1.0,
            top: 1.0,
            right: 2.0,
            bottom: -3.0,
        };

        let mut buf = Vec::new();
        bbox.write(&mut buf).expect("Failed to write");

        assert_eq!(buf.len(), 16);
        assert_eq!(
            buf,
            vec![
                0x00, 0x00, 0x80, 0xBF, // -1.0 (left)
                0x00, 0x00, 0x80, 0x3F, // 1.0 (top)
                0x00, 0x00, 0x00, 0x40, // 2.0 (right)
                0x00, 0x00, 0x40, 0xC0, // -3.0 (bottom)
            ]
        );
    }

    #[test]
    fn test_write_read_round_trip() {
        let original = BoundingBox {
            left: -0.5,
            top: 0.9,
            right: 0.3,
            bottom: 0.1,
        };

        // Write
        let mut buf = Vec::new();
        original.write(&mut buf).expect("Failed to write");

        // Read back
        let mut cursor = Cursor::new(&buf);
        let read_back = BoundingBox::read(&mut cursor).expect("Failed to read");

        assert_eq!(read_back, original);
    }
}
