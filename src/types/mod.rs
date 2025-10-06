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
