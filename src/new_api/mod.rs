//! New API implementation (work in progress)
//!
//! This module contains the redesigned two-tier API that will eventually
//! replace the current API. Development happens here in parallel to keep
//! the existing API working.

pub mod convert;
pub mod decode;
pub mod raw;
pub mod reader;
pub mod types;

pub use reader::CubReader;
