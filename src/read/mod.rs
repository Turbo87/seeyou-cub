use crate::error::Result;
use crate::read::header::parse_header;
use crate::read::item::ItemIterator;
use crate::read::item_data::read_item_data;
use crate::types::{Header, Item, ItemData};
use std::io::{Read, Seek};

mod header;
pub(crate) mod io;
mod item;
mod item_data;

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

    /// Consume the reader and return the underlying reader
    pub fn into_inner(self) -> R {
        self.inner
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
        parse_header(&mut self.inner, warnings)
    }

    /// Parse airspace items
    ///
    /// Returns an iterator that lazily parses items from the file.
    /// Requires the header to determine byte order and item count.
    /// Warnings are pushed to the provided vector during iteration.
    ///
    /// The iterator seeks to the items section on first iteration. If the seek
    /// fails, the first item will be an `Err`.
    pub fn read_items<'a>(
        &'a mut self,
        header: &Header,
        warnings: &'a mut Vec<crate::error::Warning>,
    ) -> ItemIterator<'a, R> {
        ItemIterator::new(&mut self.inner, header, warnings)
    }

    /// Parse complete item data (geometry + metadata)
    ///
    /// Parses the point stream for the given item, returning both the boundary
    /// geometry and any associated metadata (name, frequency, ICAO code, etc.).
    /// Requires the header to determine byte order and offsets.
    /// Warnings are pushed to the provided vector during parsing.
    pub fn read_item_data(
        &mut self,
        header: &Header,
        item: &Item,
        warnings: &mut Vec<crate::error::Warning>,
    ) -> Result<ItemData> {
        read_item_data(&mut self.inner, header, item, warnings)
    }
}
