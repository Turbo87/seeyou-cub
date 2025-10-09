//! Low-level CUB file parsing functions
//!
//! This module provides direct access to CUB file components with minimal
//! transformation. All functions read from the current cursor position
//! without seeking. Users must manage file positioning themselves.

mod header;
mod io;
mod item;
mod item_data;

pub use self::header::read_header;
pub use self::item::read_item;
pub use self::item_data::read_item_data;
