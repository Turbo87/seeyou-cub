use crate::types::CubDataId;

/// A parsed geometric point with optional attributes
///
/// Represents a single point in an airspace boundary, with coordinates
/// in radians and optional metadata like name and frequency information.
#[derive(Debug, Clone)]
pub struct ParsedPoint {
    /// Longitude in radians
    pub lon: f32,
    /// Latitude in radians
    pub lat: f32,
    /// Optional airspace name (present on first point of sequence)
    pub name: Option<String>,
    /// Optional frequency in Hz
    pub frequency: Option<u32>,
    /// Optional frequency name/label
    pub frequency_name: Option<String>,
    /// Optional additional data
    pub optional_data: Vec<OptionalData>,
}

/// Optional data records found in point sequences
///
/// Additional metadata that can be attached to airspace points,
/// such as ICAO codes, NOTAM information, and exception rules.
#[derive(Debug, Clone)]
pub enum OptionalData {
    IcaoCode(String),
    SecondaryFrequency(u32),
    ExceptionRules(String),
    NotamRemarks(String),
    NotamId(String),
    NotamInsertTime(u32), // Raw encoded minutes
}

impl OptionalData {
    pub fn data_id(&self) -> CubDataId {
        match self {
            OptionalData::IcaoCode(_) => CubDataId::IcaoCode,
            OptionalData::SecondaryFrequency(_) => CubDataId::SecondaryFrequency,
            OptionalData::ExceptionRules(_) => CubDataId::ExceptionRules,
            OptionalData::NotamRemarks(_) => CubDataId::NotamRemarks,
            OptionalData::NotamId(_) => CubDataId::NotamId,
            OptionalData::NotamInsertTime(_) => CubDataId::NotamInsertTime,
        }
    }
}

#[cfg(feature = "datetime")]
use jiff::civil::DateTime;

#[cfg(feature = "datetime")]
impl OptionalData {
    /// Get NOTAM insert time as DateTime (requires "datetime" feature)
    pub fn notam_insert_datetime(&self) -> Option<DateTime> {
        match self {
            OptionalData::NotamInsertTime(raw) => {
                let (year, month, day, hour, minute) = crate::types::item::decode_notam_time(*raw);
                DateTime::new(
                    year as i16,
                    month as i8,
                    day as i8,
                    hour as i8,
                    minute as i8,
                    0,
                    0,
                )
                .ok()
            }
            _ => None,
        }
    }
}
