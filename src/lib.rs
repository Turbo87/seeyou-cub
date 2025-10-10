#![doc = include_str!("../README.md")]

pub use crate::error::Error;
pub use crate::reader::CubReader;
pub use crate::types::*;

mod error;
pub mod raw;
mod reader;
mod types;
pub mod utils;
