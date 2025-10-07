//! SeeYou CUB file format parser
//!
//! This crate provides a low-level parser for the SeeYou CUB binary file format,
//! which stores airspace data for flight navigation software.
//!
//! # Examples
//!
//! ```no_run
//! use seeyou_cub::CubReader;
//!
//! let mut reader = CubReader::from_path("airspace.cub")?;
//! let mut warnings = Vec::new();
//!
//! let header = reader.read_header(&mut warnings)?;
//! let items: Vec<_> = reader
//!     .read_items(&header, &mut warnings)
//!     .collect::<Result<Vec<_>, _>>()?;
//!
//! println!("Airspaces: {}", items.len());
//!
//! for item in &items {
//!     println!("{:?}: {} - {} meters",
//!         item.style(),
//!         item.min_alt,
//!         item.max_alt
//!     );
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Features
//!
//! - `datetime`: Enable `jiff` integration for date/time decoding

// Re-export public API
pub use error::{Error, Warning};
pub use types::*;

mod error;
mod read;
mod types;
