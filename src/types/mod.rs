mod enums;
mod header;
mod item;
mod point;

pub use enums::*;
pub use header::*;
pub use item::*;
pub use point::*;

/// Parsed CUB file with header, items, and reader for lazy point parsing
pub struct CubFile<R> {
    header: Header,
    items: Vec<Item>,
    reader: R,
}

impl<R> CubFile<R> {
    /// Create new CubFile (used internally by parser)
    pub(crate) fn new(header: Header, items: Vec<Item>, reader: R) -> Self {
        Self {
            header,
            items,
            reader,
        }
    }

    /// Get file header
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Get all airspace items
    pub fn items(&self) -> &[Item] {
        &self.items
    }
}

use crate::error::Result;
use std::io::{Read, Seek};

impl<R: Read + Seek> CubFile<R> {
    /// Parse points for a specific item
    /// Returns iterator that lazily parses CubPoint sequences
    pub fn read_points(&mut self, item: &Item) -> Result<crate::read::PointIterator<'_, R>> {
        crate::read::PointIterator::new(
            &mut self.reader,
            &self.header,
            item,
        )
    }
}
