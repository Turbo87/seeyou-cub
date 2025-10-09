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
//!
//! for result in reader.read_airspaces() {
//!     let airspace = result?;
//!
//!     if let Some(name) = &airspace.name {
//!         println!("{}: {:?} {:?}", name, airspace.style, airspace.class);
//!         println!("  Altitude: {} - {} meters", airspace.min_alt, airspace.max_alt);
//!         println!("  Points: {}", airspace.points.len());
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Features
//!
//! - `datetime`: Enable `jiff` integration for date/time decoding

pub use crate::error::Error;
pub use crate::reader::CubReader;
pub use crate::types::*;

mod convert;
mod decode;
mod error;
pub mod raw;
mod reader;
mod types;
