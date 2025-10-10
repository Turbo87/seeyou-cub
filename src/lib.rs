#![doc = include_str!("../README.md")]

pub use crate::error::Error;
pub use crate::reader::CubReader;
pub use crate::types::*;
pub use crate::writer::CubWriter;

mod error;
pub mod raw;
mod reader;
mod types;
pub mod utils;
pub mod writer;
