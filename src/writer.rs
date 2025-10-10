//! High-level CUB file writer with builder API

use crate::{Airspace, ByteOrder};

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
}

impl CubWriter {
    /// Create a new writer with the given title
    ///
    /// The title will be stored in the CUB file header.
    /// Default byte order is little-endian.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            airspaces: Vec::new(),
            byte_order: ByteOrder::LE,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_string_types() {
        let _writer1 = CubWriter::new("string slice");
        let _writer2 = CubWriter::new(String::from("owned string"));
    }
}
