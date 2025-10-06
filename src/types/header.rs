use crate::types::ByteOrder;

/// CUB file header (first 210 bytes)
///
/// Contains metadata about the airspace file including bounding box,
/// item counts, and structural information needed for parsing.
#[derive(Debug, Clone)]
pub struct Header {
    // Raw fields (public)
    pub title: String,
    pub allowed_serials: [u16; 8],
    pub pc_byte_order: u8,
    pub is_secured: u8,
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
    /// Get bounding box as (west, south, east, north) in radians
    pub fn bounding_box(&self) -> (f32, f32, f32, f32) {
        (self.left, self.bottom, self.right, self.top)
    }

    /// Check if file is encrypted
    pub fn is_encrypted(&self) -> bool {
        self.is_secured != 0
    }

    /// Get byte order for integers
    pub fn byte_order(&self) -> ByteOrder {
        ByteOrder::from_pc_byte_order(self.pc_byte_order)
    }
}
