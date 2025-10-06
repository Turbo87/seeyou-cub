//! SeeYou CUB file format parser
//!
//! This crate provides a parser for the SeeYou CUB binary file format,
//! which stores airspace data for flight navigation software.
//!
//! # Examples
//!
//! ```no_run
//! use seeyou_cub::parse;
//! use std::fs::File;
//!
//! let file = File::open("airspace.cub")?;
//! let (cub, warnings) = parse(file)?;
//!
//! println!("Airspaces: {}", cub.items().len());
//!
//! for item in cub.items() {
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
pub use read::parse;
pub use types::*;

mod error;
mod read;
mod types;
