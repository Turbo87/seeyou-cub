use crate::{AltStyle, BoundingBox, CubClass, CubStyle, DateTime, DaysActive, ExtendedType, Point};

/// High-level airspace representation with fully decoded data
///
/// Combines metadata from `Item` and geometry/attributes from `ItemData`.
/// All bit-packed fields are decoded into enums, strings are decoded from bytes,
/// and coordinates are converted from raw i16 offsets to f32 lat/lon.
#[derive(Debug, Clone)]
pub struct Airspace {
    pub name: Option<String>,
    pub icao_code: Option<String>,

    // Geometry (converted from raw i16 to f32 lat/lon radians)
    pub points: Vec<Point>,

    // Bounding box in radians (None if not yet calculated)
    pub bounding_box: Option<BoundingBox>,

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
    pub start_date: Option<DateTime>,
    pub end_date: Option<DateTime>,
    pub days_active: DaysActive,

    // Optional data attributes
    pub frequency: Option<f32>,
    pub frequency_name: Option<String>,
    pub secondary_frequency: Option<f32>,
    pub exception_rules: Option<String>,
    pub notam_id: Option<String>,
    pub notam_remarks: Option<String>,
    pub notam_insert_time: Option<u32>,

    pub extra_data: u32,
}

impl Airspace {
    /// Check if ExtraData contains NOTAM data
    pub fn has_notam_data(&self) -> bool {
        (self.extra_data >> 30) == 0 && self.extra_data != 0
    }

    /// Get bounding box
    pub fn bounding_box(&self) -> Option<&BoundingBox> {
        self.bounding_box.as_ref()
    }
}

impl Default for Airspace {
    fn default() -> Self {
        Self {
            bounding_box: None,
            style: CubStyle::Unknown,
            class: CubClass::Unknown,
            extended_type: None,
            min_alt: 0,
            max_alt: 0,
            min_alt_style: AltStyle::Unknown,
            max_alt_style: AltStyle::Unknown,
            start_date: None,
            end_date: None,
            extra_data: 0,
            days_active: DaysActive::from_bits(0),
            points: vec![],
            name: None,
            frequency_name: None,
            icao_code: None,
            exception_rules: None,
            notam_remarks: None,
            notam_id: None,
            frequency: None,
            secondary_frequency: None,
            notam_insert_time: None,
        }
    }
}
