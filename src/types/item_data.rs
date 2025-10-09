use crate::PointOp;

/// Low-level item data with raw point operations and unprocessed attributes
///
/// This struct represents data as close to the file format as possible:
/// - Point operations are raw i16 offsets (not yet converted to lat/lon)
/// - Strings are raw bytes (not yet decoded from UTF-8/Extended ASCII)
/// - Optional attributes remain as raw bytes for maximum flexibility
#[derive(Debug, Clone, PartialEq)]
pub struct ItemData {
    /// Raw point operations (origin moves and new points with i16 x/y offsets)
    pub point_ops: Vec<PointOp>,

    /// Airspace name (raw bytes, not decoded)
    pub name: Option<Vec<u8>>,
    /// Primary frequency in Hz
    pub frequency: Option<u32>,
    /// Primary frequency name/label (raw bytes, not decoded)
    pub frequency_name: Option<Vec<u8>>,
    /// ICAO code (raw bytes, not decoded)
    pub icao_code: Option<Vec<u8>>,
    /// Secondary frequency in Hz
    pub secondary_frequency: Option<u32>,
    /// Class exception rules (raw bytes, not decoded)
    pub exception_rules: Option<Vec<u8>>,
    /// NOTAM remarks (raw bytes, not decoded)
    pub notam_remarks: Option<Vec<u8>>,
    /// NOTAM identifier (raw bytes, not decoded)
    pub notam_id: Option<Vec<u8>>,
    /// NOTAM insert time (raw encoded value)
    pub notam_insert_time: Option<u32>,
}
