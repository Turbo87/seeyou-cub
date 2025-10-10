//! High-level CUB file writer with builder API

use crate::{Airspace, ByteOrder};

/// Default coordinate scale factor
///
/// This value (`1.5723687e-7` radians) provides approximately 1 meter precision
/// at the equator and allows coordinates within ~33km from the origin
/// before requiring a `MoveOrigin` operation.
const DEFAULT_LO_LA_SCALE: f32 = 1.5723687e-7;

/// High-level CUB file writer with builder API
///
/// Provides a convenient API for creating CUB files from airspace data.
/// All low-level complexity (coordinate conversion, bounding box calculation,
/// offset tracking) is handled automatically.
///
/// # Example
///
/// ```
/// use seeyou_cub::writer::CubWriter;
///
/// CubWriter::new("My Airspace Data");
/// ```
pub struct CubWriter {
    title: String,
    airspaces: Vec<Airspace>,
    byte_order: ByteOrder,
    lo_la_scale: f32,
}

impl CubWriter {
    /// Create a new writer with the given title
    ///
    /// The title will be stored in the CUB file header.
    /// Default byte order is little-endian, and coordinate precision is 1 meter
    /// at the equator.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            airspaces: Vec::new(),
            byte_order: ByteOrder::LE,
            lo_la_scale: DEFAULT_LO_LA_SCALE,
        }
    }

    /// Add a single airspace to the writer
    ///
    /// Returns `&mut self` to allow method chaining.
    pub fn add_airspace(&mut self, airspace: Airspace) -> &mut Self {
        self.airspaces.push(airspace);
        self
    }

    /// Add multiple airspaces from an iterator
    ///
    /// Returns `&mut self` to allow method chaining.
    pub fn add_airspaces<I: IntoIterator<Item = Airspace>>(&mut self, airspaces: I) -> &mut Self {
        self.airspaces.extend(airspaces);
        self
    }

    /// Configure byte order for the output file
    ///
    /// Default is little-endian. Returns `&mut self` to allow method chaining.
    pub fn with_byte_order(&mut self, byte_order: ByteOrder) -> &mut Self {
        self.byte_order = byte_order;
        self
    }

    /// Override the coordinate precision scale
    ///
    /// The scale determines the precision of stored coordinates.
    /// Default is 1 meter precision at the equator.
    /// Returns `&mut self` to allow method chaining.
    pub fn with_lo_la_scale(&mut self, scale: f32) -> &mut Self {
        self.lo_la_scale = scale;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn new_accepts_string_types() {
        let _writer1 = CubWriter::new("string slice");
        let _writer2 = CubWriter::new(String::from("owned string"));
    }

    #[test]
    fn default_lo_la_scale_provides_meter_precision() {
        // Convert to degrees for easier understanding
        let precision_degrees = DEFAULT_LO_LA_SCALE.to_degrees();

        // At equator, 1 degree latitude ≈ 111km
        let precision_meters = precision_degrees * 111_000.0;

        assert_debug_snapshot!(precision_meters, @"1.0");
    }
}
