/// A single geometric point in an airspace boundary
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    /// Longitude in degrees
    pub lon: f32,
    /// Latitude in degrees
    pub lat: f32,
}

/// Complete data for an airspace item, including geometry and metadata
///
/// Contains both the boundary geometry (as a sequence of points) and
/// optional metadata attributes parsed from the point stream.
#[derive(Debug, Clone)]
pub struct ItemData {
    /// Boundary geometry points
    pub points: Vec<Point>,

    /// Airspace name
    pub name: Option<String>,
    /// Primary frequency in Hz
    pub frequency: Option<u32>,
    /// Primary frequency name/label
    pub frequency_name: Option<String>,
    /// ICAO code
    pub icao_code: Option<String>,
    /// Secondary frequency in Hz
    pub secondary_frequency: Option<u32>,
    /// Class exception rules
    pub exception_rules: Option<String>,
    /// NOTAM remarks
    pub notam_remarks: Option<String>,
    /// NOTAM identifier
    pub notam_id: Option<String>,
    /// NOTAM insert time (raw encoded value)
    pub notam_insert_time: Option<u32>,
}

#[cfg(feature = "datetime")]
use jiff::civil::DateTime;

#[cfg(feature = "datetime")]
impl ItemData {
    /// Get NOTAM insert time as DateTime (requires "datetime" feature)
    pub fn notam_insert_datetime(&self) -> Option<DateTime> {
        self.notam_insert_time.and_then(|raw| {
            let (year, month, day, hour, minute) = crate::types::item::decode_notam_time(raw);
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
