mod io;
mod header;
mod item;
mod point;

pub use header::parse_header;
pub use item::parse_items;
pub use point::PointIterator;

use std::io::{Read, Seek};
use crate::error::{Result, Warning};
use crate::types::CubFile;

/// Parse a CUB file from a reader
pub fn parse<R: Read + Seek>(mut reader: R) -> Result<(CubFile<R>, Vec<Warning>)> {
    let mut all_warnings = Vec::new();

    // Parse header
    let (header, header_warnings) = parse_header(&mut reader)?;
    all_warnings.extend(header_warnings);

    // Parse items
    let (items, item_warnings) = parse_items(&mut reader, &header)?;
    all_warnings.extend(item_warnings);

    // Create CubFile
    let cub_file = CubFile::new(header, items, reader);

    Ok((cub_file, all_warnings))
}
