mod enums;
mod header;
mod item;
mod point;

pub use enums::*;
pub use header::*;
pub use item::*;
pub use point::*;

use crate::error::Result;
use std::io::{Read, Seek};

/// Low-level CUB file reader
///
/// Provides methods for parsing CUB file components (header, items, points)
/// without storing the parsed data. Users own the parsed data independently.
pub struct CubReader<R> {
    inner: R,
}

impl<R> CubReader<R> {
    /// Create new reader from any Read + Seek source
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
}

impl CubReader<std::fs::File> {
    /// Open a CUB file from a path
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        Ok(Self::new(std::fs::File::open(path)?))
    }
}

impl<R: Read + Seek> CubReader<R> {
    /// Parse CUB file header
    ///
    /// Reads and validates the 210-byte header at the start of the file.
    /// Warnings are pushed to the provided vector for spec violations that
    /// were recovered from.
    pub fn read_header(&mut self, warnings: &mut Vec<crate::error::Warning>) -> Result<Header> {
        crate::read::parse_header(&mut self.inner, warnings)
    }

    /// Parse airspace items
    ///
    /// Returns an iterator that lazily parses items from the file.
    /// Requires the header to determine byte order and item count.
    /// Warnings are pushed to the provided vector during iteration.
    pub fn read_items<'a>(
        &'a mut self,
        header: &Header,
        warnings: &'a mut Vec<crate::error::Warning>,
    ) -> Result<crate::read::ItemIterator<'a, R>> {
        crate::read::ItemIterator::new(&mut self.inner, header, warnings)
    }

    /// Parse points for a specific item
    ///
    /// Returns an iterator that lazily parses points for the given item.
    /// Requires the header to determine byte order and point offsets.
    /// Warnings are pushed to the provided vector during iteration.
    pub fn read_points<'a>(
        &'a mut self,
        header: &'a Header,
        item: &Item,
        warnings: &'a mut Vec<crate::error::Warning>,
    ) -> Result<crate::read::PointIterator<'a, R>> {
        crate::read::PointIterator::new(&mut self.inner, header, item, warnings)
    }
}
