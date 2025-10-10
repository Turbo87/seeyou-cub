use crate::error::Result;
use crate::types::Point;
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

    /// Extend bounding box to include a point
    ///
    /// Grows the bounding box if necessary to encompass the given point.
    /// If the point is already inside the bbox, no change is made.
    pub fn extend(&mut self, point: Point) {
        self.left = self.left.min(point.lon);
        self.right = self.right.max(point.lon);
        self.top = self.top.max(point.lat);
        self.bottom = self.bottom.min(point.lat);
    }
}

impl From<Point> for BoundingBox {
    fn from(point: Point) -> Self {
        Self {
            left: point.lon,
            top: point.lat,
            right: point.lon,
            bottom: point.lat,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Point;
    use std::io::Cursor;

    #[test]
    fn test_from_point() {
        // Create a point (Paris: 48.8566°N, 2.3522°E in radians)
        let point = Point::new(0.852_941_4, 0.041_037_06);

        // Create bounding box from single point
        let bbox = BoundingBox::from(point);

        // All bounds should equal the point's coordinates
        assert_eq!(bbox.left, 0.041_037_06); // lon
        assert_eq!(bbox.top, 0.852_941_4); // lat
        assert_eq!(bbox.right, 0.041_037_06); // lon
        assert_eq!(bbox.bottom, 0.852_941_4); // lat
    }

    #[test]
    fn test_extend_north() {
        // Start with a bbox from single point
        let mut bbox = BoundingBox::from(Point::new(0.5, 0.5));

        // Extend north (higher latitude)
        let north_point = Point::new(0.8, 0.5);
        bbox.extend(north_point);

        assert_eq!(bbox.left, 0.5);
        assert_eq!(bbox.top, 0.8); // Extended north
        assert_eq!(bbox.right, 0.5);
        assert_eq!(bbox.bottom, 0.5);
    }

    #[test]
    fn test_extend_south() {
        // Start with a bbox from single point
        let mut bbox = BoundingBox::from(Point::new(0.5, 0.5));

        // Extend south (lower latitude)
        let south_point = Point::new(0.2, 0.5);
        bbox.extend(south_point);

        assert_eq!(bbox.left, 0.5);
        assert_eq!(bbox.top, 0.5);
        assert_eq!(bbox.right, 0.5);
        assert_eq!(bbox.bottom, 0.2); // Extended south
    }

    #[test]
    fn test_extend_east() {
        // Start with a bbox from single point
        let mut bbox = BoundingBox::from(Point::new(0.5, 0.5));

        // Extend east (higher longitude)
        let east_point = Point::new(0.5, 0.8);
        bbox.extend(east_point);

        assert_eq!(bbox.left, 0.5);
        assert_eq!(bbox.top, 0.5);
        assert_eq!(bbox.right, 0.8); // Extended east
        assert_eq!(bbox.bottom, 0.5);
    }

    #[test]
    fn test_extend_west() {
        // Start with a bbox from single point
        let mut bbox = BoundingBox::from(Point::new(0.5, 0.5));

        // Extend west (lower longitude)
        let west_point = Point::new(0.5, 0.2);
        bbox.extend(west_point);

        assert_eq!(bbox.left, 0.2); // Extended west
        assert_eq!(bbox.top, 0.5);
        assert_eq!(bbox.right, 0.5);
        assert_eq!(bbox.bottom, 0.5);
    }

    #[test]
    fn test_extend_multiple_directions() {
        // Start with a bbox from single point
        let mut bbox = BoundingBox::from(Point::new(0.5, 0.5));

        // Extend in multiple directions
        bbox.extend(Point::new(0.8, 0.8)); // NE
        bbox.extend(Point::new(0.2, 0.2)); // SW
        bbox.extend(Point::new(0.9, 0.1)); // SE

        assert_eq!(bbox.left, 0.1); // Westmost
        assert_eq!(bbox.top, 0.9); // Northmost
        assert_eq!(bbox.right, 0.8); // Eastmost
        assert_eq!(bbox.bottom, 0.2); // Southmost
    }

    #[test]
    fn test_extend_with_point_inside_bbox() {
        // Start with a bbox
        let mut bbox = BoundingBox {
            left: 0.0,
            top: 1.0,
            right: 1.0,
            bottom: 0.0,
        };

        // Extend with point inside - should not change bounds
        bbox.extend(Point::new(0.5, 0.5));

        assert_eq!(bbox.left, 0.0);
        assert_eq!(bbox.top, 1.0);
        assert_eq!(bbox.right, 1.0);
        assert_eq!(bbox.bottom, 0.0);
    }

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
