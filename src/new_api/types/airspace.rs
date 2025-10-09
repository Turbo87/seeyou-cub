use crate::new_api::types::Point;
use crate::types::{AltStyle, CubClass, CubStyle, DaysActive, ExtendedType};

#[cfg(feature = "datetime")]
use jiff::civil::DateTime;

/// High-level airspace representation with fully decoded data
///
/// Combines metadata from `Item` and geometry/attributes from `ItemData`.
/// All bit-packed fields are decoded into enums, strings are decoded from bytes,
/// and coordinates are converted from raw i16 offsets to f64 lat/lon.
#[derive(Debug, Clone)]
pub struct Airspace {
    // Bounding box in radians (converted to f64 for consistency)
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,

    // Decoded airspace classification
    pub style: CubStyle,
    pub class: CubClass,
    pub extended_type: Option<ExtendedType>,

    // Altitude data
    pub min_alt: i16,
    pub max_alt: i16,
    pub min_alt_style: AltStyle,
    pub max_alt_style: AltStyle,

    // Time-related fields
    pub time_out: i32,
    pub active_time: u64,
    pub extra_data: u32,

    // Decoded temporal data
    pub days_active: DaysActive,

    // Geometry (converted from raw i16 to f64 lat/lon radians)
    pub points: Vec<Point>,

    // Decoded string attributes
    pub name: Option<String>,
    pub frequency_name: Option<String>,
    pub icao_code: Option<String>,
    pub exception_rules: Option<String>,
    pub notam_remarks: Option<String>,
    pub notam_id: Option<String>,

    // Numeric attributes
    pub frequency: Option<u32>,
    pub secondary_frequency: Option<u32>,
    pub notam_insert_time: Option<u32>,
}

impl Airspace {
    /// Get raw start date (encoded minutes)
    pub fn start_date_raw(&self) -> Option<u32> {
        let value = ((self.active_time >> 26) & 0x3FFFFFF) as u32;
        if value == 0 { None } else { Some(value) }
    }

    /// Get raw end date (encoded minutes)
    pub fn end_date_raw(&self) -> Option<u32> {
        let value = (self.active_time & 0x3FFFFFF) as u32;
        if value == 0x3FFFFFF {
            None
        } else {
            Some(value)
        }
    }

    /// Check if ExtraData contains NOTAM data
    pub fn has_notam_data(&self) -> bool {
        (self.extra_data >> 30) == 0 && self.extra_data != 0
    }

    /// Get bounding box as (west, south, east, north) in radians
    pub fn bounding_box(&self) -> (f64, f64, f64, f64) {
        (self.left, self.bottom, self.right, self.top)
    }
}

#[cfg(feature = "datetime")]
impl Airspace {
    /// Get start date as DateTime (requires "datetime" feature)
    pub fn start_date(&self) -> Option<DateTime> {
        self.start_date_raw().and_then(|raw| {
            let (year, month, day, hour, minute) = crate::types::decode_notam_time(raw);
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
        })
    }

    /// Get end date as DateTime (requires "datetime" feature)
    pub fn end_date(&self) -> Option<DateTime> {
        self.end_date_raw().and_then(|raw| {
            let (year, month, day, hour, minute) = crate::types::decode_notam_time(raw);
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
        })
    }
}
