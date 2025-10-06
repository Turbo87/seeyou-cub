mod header;
mod io;
mod item;
mod point;

pub use header::parse_header;
pub use item::parse_items;
pub use point::PointIterator;

use crate::error::{Result, Warning};
use crate::types::CubFile;
use std::io::{Read, Seek};

/// Parse a CUB file from a reader
///
/// This function performs lenient parsing: it will attempt to parse as much
/// as possible even when encountering spec violations. Issues that don't
/// prevent parsing are collected as warnings.
///
/// # Arguments
///
/// * `reader` - Any type implementing `Read + Seek`, typically a `File`
///
/// # Returns
///
/// Returns a tuple of:
/// - `CubFile<R>`: Parsed file with header, items, and reader for lazy point parsing
/// - `Vec<Warning>`: Non-fatal issues encountered during parsing
///
/// # Errors
///
/// Returns `Error` for unrecoverable failures:
/// - `InvalidMagicBytes`: File is not a valid CUB file
/// - `EncryptedFile`: File is encrypted (not yet supported)
/// - `IoError`: I/O failure while reading
/// - `UnexpectedEof`: File truncated or invalid offsets
///
/// # Examples
///
/// ```no_run
/// use seeyou_cub::parse;
/// use std::fs::File;
///
/// let file = File::open("airspace.cub")?;
/// let (mut cub, warnings) = parse(file)?;
///
/// // Access header
/// println!("Bounding box: {:?}", cub.header().bounding_box());
///
/// // Iterate items
/// for i in 0..cub.items().len() {
///     let item = cub.items()[i].clone();
///     println!("Airspace: {:?}", item.style());
///
///     // Parse geometry for this item
///     for point in cub.read_points(&item)? {
///         let pt = point?;
///         println!("  Point: {} {}", pt.lon, pt.lat);
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
