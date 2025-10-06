# Implementation Plan: Low-Level CUB Reader

## Overview

Implement a low-level parser for the SeeYou CUB binary file format. This parser will read airspace data files used by flight navigation software.

**Key characteristics:**
- Binary format with three sections: header (210 bytes), items array (fixed-size), points (variable-length)
- Little-endian by default, but supports big-endian integers via flag
- Lenient parsing: collect warnings for spec violations but continue when possible
- Lazy geometry parsing: load header + items eagerly, parse points on-demand

**Reference:** `docs/CUB_file_format.md` contains complete format specification.

## Project Structure

```
src/
  lib.rs              # Public API re-exports
  error.rs            # Error and Warning types
  types/
    mod.rs            # Re-exports all types
    enums.rs          # All enum types
    header.rs         # Header struct
    item.rs           # Item struct
    point.rs          # Point-related types
  read/
    mod.rs            # Public parse() function
    io.rs             # Byte-order aware I/O helpers
    header.rs         # Header parsing
    item.rs           # Item parsing
    point.rs          # Point iterator/parser
tests/
  fixtures/
    france_2024.07.02.cub  # Real test data
  reader_test.rs      # Integration tests
```

## Dependencies to Add

Add to `Cargo.toml`:

```toml
[dependencies]
encoding_rs = "0.8"
jiff = { version = "0.1", optional = true }
thiserror = "2"

[features]
default = []
datetime = ["jiff"]

[dev-dependencies]
```

## Task Breakdown

### Phase 1: Foundation (Error handling, I/O utilities)

#### Task 1.1: Error and Warning types

**Files to create:** `src/error.rs`

**Objective:** Define error and warning types for lenient parsing.

**Implementation:**

```rust
use std::io;

/// Unrecoverable parsing errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("Invalid magic bytes in header (expected 0x425543C2)")]
    InvalidMagicBytes,

    #[error("Encrypted CUB files not supported (encryption format undocumented)")]
    EncryptedFile,

    #[error("Unexpected end of file: {0}")]
    UnexpectedEof(String),

    #[error("Invalid point flag: 0x{0:02X}")]
    InvalidPointFlag(u8),
}

/// Non-fatal issues encountered during lenient parsing
#[derive(Debug, Clone, PartialEq)]
pub enum Warning {
    /// Unknown enum value, used default instead
    InvalidEnumValue {
        field: String,
        value: u8,
        used_default: String,
    },

    /// SizeOfItem/SizeOfPoint smaller than expected structure size
    OversizedItem {
        expected: i32,
        actual: i32,
    },

    /// Unrecognized optional point flag, skipped
    UnknownPointFlag(u8),

    /// Data appears truncated but parsing continued
    TruncatedData {
        context: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Testing approach:**
- No unit tests needed yet (just type definitions)
- Will be tested indirectly through parser tests

**Commit message:** `Add Error and Warning types`

---

#### Task 1.2: ByteOrder enum and basic enums

**Files to create:** `src/types/mod.rs`, `src/types/enums.rs`

**Objective:** Define core enums including byte order and airspace types.

**Implementation in `src/types/enums.rs`:**

```rust
/// Byte ordering for integer fields
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    LE,  // Little Endian
    BE,  // Big Endian
}

impl ByteOrder {
    /// Parse from PcByteOrder byte (offset 130 in header)
    /// 0 = BE, anything else = LE
    pub fn from_pc_byte_order(byte: u8) -> Self {
        if byte == 0 {
            ByteOrder::BE
        } else {
            ByteOrder::LE
        }
    }
}

/// Airspace style/type (extracted from Item.Type field)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubStyle {
    Unknown,
    ControlZone,
    RestrictedArea,
    ProhibitedArea,
    DangerArea,
    TemporaryReservedArea,
    TerminalControlArea,
    TrafficInformationZone,
    Airway,
    ControlArea,
    GliderSector,
    TransponderMandatoryZone,
    MilitaryAerodromeTrafficZone,
    RadioMandatoryZone,
    Notam,
    Advisory,
    AirDefenceIdentificationZone,
    FlightInformationRegion,
    DelegatedFir,
    TrafficInformationArea,
    SpecialRulesZone,
    TemporaryFlightRestriction,
    AerodromeTrafficZone,
    FlightInformationServiceArea,
    LegacyRmz,
    AerialSportingAndRecreationArea,
    TransponderRecommendedZone,
    VfrRoute,
    Alert,
    TemporarySegregatedArea,
    Warning,
}

impl CubStyle {
    /// Parse from combined Type byte value (lowest 4 bits + highest bit)
    pub fn from_type_byte(byte: u8) -> Self {
        let value = (byte & 0x0F) | (byte & 0x80);
        match value {
            0x00 => CubStyle::Unknown,
            0x01 => CubStyle::ControlZone,
            0x02 => CubStyle::RestrictedArea,
            0x03 => CubStyle::ProhibitedArea,
            0x04 => CubStyle::DangerArea,
            0x05 => CubStyle::TemporaryReservedArea,
            0x06 => CubStyle::TerminalControlArea,
            0x07 => CubStyle::TrafficInformationZone,
            0x08 => CubStyle::Airway,
            0x09 => CubStyle::ControlArea,
            0x0a => CubStyle::GliderSector,
            0x0b => CubStyle::TransponderMandatoryZone,
            0x0c => CubStyle::MilitaryAerodromeTrafficZone,
            0x0d => CubStyle::RadioMandatoryZone,
            0x0f => CubStyle::Notam,
            0x80 => CubStyle::Advisory,
            0x81 => CubStyle::AirDefenceIdentificationZone,
            0x82 => CubStyle::FlightInformationRegion,
            0x83 => CubStyle::DelegatedFir,
            0x84 => CubStyle::TrafficInformationArea,
            0x85 => CubStyle::SpecialRulesZone,
            0x86 => CubStyle::TemporaryFlightRestriction,
            0x87 => CubStyle::AerodromeTrafficZone,
            0x88 => CubStyle::FlightInformationServiceArea,
            0x89 => CubStyle::LegacyRmz,
            0x8a => CubStyle::AerialSportingAndRecreationArea,
            0x8b => CubStyle::TransponderRecommendedZone,
            0x8c => CubStyle::VfrRoute,
            0x8d => CubStyle::Alert,
            0x8e => CubStyle::TemporarySegregatedArea,
            0x8f => CubStyle::Warning,
            _ => CubStyle::Unknown,
        }
    }
}

/// Airspace class (extracted from Item.Type field, bits 5-7)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubClass {
    Unknown,
    ClassA,
    ClassB,
    ClassC,
    ClassD,
    ClassE,
    ClassF,
    ClassG,
}

impl CubClass {
    /// Extract from Type byte (bits 5-7)
    pub fn from_type_byte(byte: u8) -> Self {
        let value = (byte >> 4) & 0x07;
        match value {
            0 => CubClass::Unknown,
            1 => CubClass::ClassA,
            2 => CubClass::ClassB,
            3 => CubClass::ClassC,
            4 => CubClass::ClassD,
            5 => CubClass::ClassE,
            6 => CubClass::ClassF,
            7 => CubClass::ClassG,
            _ => CubClass::Unknown,
        }
    }
}

/// Altitude reference style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AltStyle {
    Unknown,
    AboveGroundLevel,
    MeanSeaLevel,
    FlightLevel,
    Unlimited,
    Notam,
}

impl AltStyle {
    /// Parse from 4-bit value
    pub fn from_nibble(value: u8) -> Self {
        match value & 0x0F {
            0 => AltStyle::Unknown,
            1 => AltStyle::AboveGroundLevel,
            2 => AltStyle::MeanSeaLevel,
            3 => AltStyle::FlightLevel,
            4 => AltStyle::Unlimited,
            5 => AltStyle::Notam,
            _ => AltStyle::Unknown,
        }
    }
}

/// Extended airspace type (from ExtendedType field if non-zero)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedType {
    UpperInfoRegion,
    MilitaryTrainingRoute,
    HelicopterTrafficZone,
    AreaControlCenterSector,
    LowerTrafficArea,
    UpperTrafficArea,
    MilitaryTrainingArea,
    OverflightRestriction,
    TraTsaFeedingRoute,
    VfrSector,
}

impl ExtendedType {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(ExtendedType::UpperInfoRegion),
            0x02 => Some(ExtendedType::MilitaryTrainingRoute),
            0x03 => Some(ExtendedType::HelicopterTrafficZone),
            0x04 => Some(ExtendedType::AreaControlCenterSector),
            0x05 => Some(ExtendedType::LowerTrafficArea),
            0x06 => Some(ExtendedType::UpperTrafficArea),
            0x07 => Some(ExtendedType::MilitaryTrainingArea),
            0x08 => Some(ExtendedType::OverflightRestriction),
            0x00 => Some(ExtendedType::TraTsaFeedingRoute),
            0x0a => Some(ExtendedType::VfrSector),
            _ => None,
        }
    }
}
```

**Implementation in `src/types/mod.rs`:**

```rust
mod enums;
mod header;
mod item;
mod point;

pub use enums::*;
pub use header::*;
pub use item::*;
pub use point::*;
```

**Testing approach:**

Create `src/types/enums.rs` with inline tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_order_from_pc_byte_order() {
        assert_eq!(ByteOrder::from_pc_byte_order(0), ByteOrder::BE);
        assert_eq!(ByteOrder::from_pc_byte_order(1), ByteOrder::LE);
        assert_eq!(ByteOrder::from_pc_byte_order(255), ByteOrder::LE);
    }

    #[test]
    fn cub_style_from_type_byte() {
        assert_eq!(CubStyle::from_type_byte(0x00), CubStyle::Unknown);
        assert_eq!(CubStyle::from_type_byte(0x01), CubStyle::ControlZone);
        assert_eq!(CubStyle::from_type_byte(0x04), CubStyle::DangerArea);
        assert_eq!(CubStyle::from_type_byte(0x80), CubStyle::Advisory);
        assert_eq!(CubStyle::from_type_byte(0x8f), CubStyle::Warning);
    }

    #[test]
    fn cub_class_from_type_byte() {
        assert_eq!(CubClass::from_type_byte(0b00000000), CubClass::Unknown);
        assert_eq!(CubClass::from_type_byte(0b00010000), CubClass::ClassA);
        assert_eq!(CubClass::from_type_byte(0b01000000), CubClass::ClassD);
        assert_eq!(CubClass::from_type_byte(0b01110000), CubClass::ClassG);
    }

    #[test]
    fn alt_style_from_nibble() {
        assert_eq!(AltStyle::from_nibble(0), AltStyle::Unknown);
        assert_eq!(AltStyle::from_nibble(1), AltStyle::AboveGroundLevel);
        assert_eq!(AltStyle::from_nibble(3), AltStyle::FlightLevel);
        assert_eq!(AltStyle::from_nibble(15), AltStyle::Unknown);
    }
}
```

**Test command:** `cargo test types::enums`

**Commit message:** `Add ByteOrder and airspace type enums`

---

#### Task 1.3: Add remaining enums (NOTAM, DaysActive, CubDataId)

**Files to modify:** `src/types/enums.rs`

**Objective:** Complete enum definitions for NOTAM data and point attributes.

**Add to `src/types/enums.rs`:**

```rust
/// NOTAM type (from ExtraData bits 28-29)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotamType {
    None,
    Cancel,
    New,
    Replace,
}

impl NotamType {
    pub fn from_bits(bits: u32) -> Self {
        match (bits >> 28) & 0x03 {
            0 => NotamType::None,
            1 => NotamType::Cancel,
            2 => NotamType::New,
            3 => NotamType::Replace,
            _ => NotamType::None,
        }
    }
}

/// NOTAM traffic type (from ExtraData bits 4-6)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotamTraffic {
    Miscellaneous,
    Ifr,
    Vfr,
    IfrAndVfr,
    Checklist,
}

impl NotamTraffic {
    pub fn from_bits(bits: u32) -> Self {
        match (bits >> 4) & 0x07 {
            0 => NotamTraffic::Miscellaneous,
            1 => NotamTraffic::Ifr,
            2 => NotamTraffic::Vfr,
            3 => NotamTraffic::IfrAndVfr,
            4 => NotamTraffic::Checklist,
            _ => NotamTraffic::Miscellaneous,
        }
    }
}

/// NOTAM scope (from ExtraData bits 0-3)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotamScope {
    Unknown,
    Aerodrome,
    EnRoute,
    AerodromeAndEnRoute,
    NavWarning,
    AerodromeAndNavWarning,
    Checklist,
}

impl NotamScope {
    pub fn from_bits(bits: u32) -> Self {
        match bits & 0x0F {
            0 => NotamScope::Unknown,
            1 => NotamScope::Aerodrome,
            2 => NotamScope::EnRoute,
            3 => NotamScope::AerodromeAndEnRoute,
            4 => NotamScope::NavWarning,
            5 => NotamScope::AerodromeAndNavWarning,
            8 => NotamScope::Checklist,
            _ => NotamScope::Unknown,
        }
    }
}

/// Days active flags (bits 52-63 of ActiveTime)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaysActive {
    bits: u16,
}

impl DaysActive {
    pub fn from_bits(bits: u16) -> Self {
        Self { bits }
    }

    pub fn sunday(&self) -> bool { self.bits & 0x001 != 0 }
    pub fn monday(&self) -> bool { self.bits & 0x002 != 0 }
    pub fn tuesday(&self) -> bool { self.bits & 0x004 != 0 }
    pub fn wednesday(&self) -> bool { self.bits & 0x008 != 0 }
    pub fn thursday(&self) -> bool { self.bits & 0x010 != 0 }
    pub fn friday(&self) -> bool { self.bits & 0x020 != 0 }
    pub fn saturday(&self) -> bool { self.bits & 0x040 != 0 }
    pub fn holidays(&self) -> bool { self.bits & 0x080 != 0 }
    pub fn aup(&self) -> bool { self.bits & 0x100 != 0 }
    pub fn irregular(&self) -> bool { self.bits & 0x200 != 0 }
    pub fn by_notam(&self) -> bool { self.bits & 0x400 != 0 }
    pub fn is_unknown(&self) -> bool { self.bits == 0 }
}

/// Optional data type identifier in CubPoint sequences
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubDataId {
    IcaoCode,
    SecondaryFrequency,
    ExceptionRules,
    NotamRemarks,
    NotamId,
    NotamInsertTime,
}

impl CubDataId {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(CubDataId::IcaoCode),
            1 => Some(CubDataId::SecondaryFrequency),
            2 => Some(CubDataId::ExceptionRules),
            3 => Some(CubDataId::NotamRemarks),
            4 => Some(CubDataId::NotamId),
            5 => Some(CubDataId::NotamInsertTime),
            _ => None,
        }
    }
}

/// NOTAM subject and action codes (decoded from ExtraData)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotamCodes {
    pub subject: (char, char),  // First and last letter
    pub action: (char, char),   // First and last letter
}

impl NotamCodes {
    /// Decode from ExtraData field (bits 8-27 encode letters as 1-26)
    pub fn from_extra_data(extra_data: u32) -> Option<Self> {
        // Check if this is NOTAM data (bits 30-31 == 0)
        if (extra_data >> 30) != 0 {
            return None;
        }

        let decode_letter = |bits: u32| -> Option<char> {
            match bits {
                1..=26 => Some((b'A' + (bits - 1) as u8) as char),
                _ => None,
            }
        };

        let subject_first = decode_letter((extra_data >> 23) & 0x1F)?;
        let subject_last = decode_letter((extra_data >> 18) & 0x1F)?;
        let action_first = decode_letter((extra_data >> 13) & 0x1F)?;
        let action_last = decode_letter((extra_data >> 8) & 0x1F)?;

        Some(NotamCodes {
            subject: (subject_first, subject_last),
            action: (action_first, action_last),
        })
    }
}
```

**Add tests:**

```rust
#[cfg(test)]
mod tests {
    // ... existing tests ...

    #[test]
    fn notam_type_from_bits() {
        assert_eq!(NotamType::from_bits(0b00 << 28), NotamType::None);
        assert_eq!(NotamType::from_bits(0b01 << 28), NotamType::Cancel);
        assert_eq!(NotamType::from_bits(0b10 << 28), NotamType::New);
        assert_eq!(NotamType::from_bits(0b11 << 28), NotamType::Replace);
    }

    #[test]
    fn days_active() {
        let days = DaysActive::from_bits(0x001 | 0x004 | 0x040);
        assert!(days.sunday());
        assert!(!days.monday());
        assert!(days.tuesday());
        assert!(days.saturday());
        assert!(!days.holidays());
    }

    #[test]
    fn notam_codes_decode() {
        // Example: subject "AA", action "BB"
        let extra_data =
            (1 << 23) |  // subject first: A
            (1 << 18) |  // subject last: A
            (2 << 13) |  // action first: B
            (2 << 8);    // action last: B

        let codes = NotamCodes::from_extra_data(extra_data).unwrap();
        assert_eq!(codes.subject, ('A', 'A'));
        assert_eq!(codes.action, ('B', 'B'));
    }
}
```

**Test command:** `cargo test types::enums`

**Commit message:** `Add NOTAM and DaysActive enums`

---

#### Task 1.4: I/O helper functions

**Files to create:** `src/read/mod.rs`, `src/read/io.rs`

**Objective:** Create byte-order aware I/O helpers for reading primitive types.

**Implementation in `src/read/io.rs`:**

```rust
use std::io::{Read, Seek};
use crate::error::{Error, Result};
use crate::types::ByteOrder;

/// Read i16 with specified byte order
pub fn read_i16<R: Read>(reader: &mut R, order: ByteOrder) -> Result<i16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(match order {
        ByteOrder::LE => i16::from_le_bytes(buf),
        ByteOrder::BE => i16::from_be_bytes(buf),
    })
}

/// Read u16 with specified byte order
pub fn read_u16<R: Read>(reader: &mut R, order: ByteOrder) -> Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(match order {
        ByteOrder::LE => u16::from_le_bytes(buf),
        ByteOrder::BE => u16::from_be_bytes(buf),
    })
}

/// Read i32 with specified byte order
pub fn read_i32<R: Read>(reader: &mut R, order: ByteOrder) -> Result<i32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(match order {
        ByteOrder::LE => i32::from_le_bytes(buf),
        ByteOrder::BE => i32::from_be_bytes(buf),
    })
}

/// Read u32 with specified byte order
pub fn read_u32<R: Read>(reader: &mut R, order: ByteOrder) -> Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(match order {
        ByteOrder::LE => u32::from_le_bytes(buf),
        ByteOrder::BE => u32::from_be_bytes(buf),
    })
}

/// Read u64 with specified byte order
pub fn read_u64<R: Read>(reader: &mut R, order: ByteOrder) -> Result<u64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(match order {
        ByteOrder::LE => u64::from_le_bytes(buf),
        ByteOrder::BE => u64::from_be_bytes(buf),
    })
}

/// Read f32 (always little-endian per spec)
pub fn read_f32_le<R: Read>(reader: &mut R) -> Result<f32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

/// Read u8
pub fn read_u8<R: Read>(reader: &mut R) -> Result<u8> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

/// Read fixed-length byte array
pub fn read_bytes<R: Read>(reader: &mut R, len: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

/// Read string from bytes, attempting UTF-8 first, falling back to CP1252
pub fn read_string<R: Read>(reader: &mut R, len: usize) -> Result<String> {
    let bytes = read_bytes(reader, len)?;

    // Try UTF-8 first
    match String::from_utf8(bytes.clone()) {
        Ok(s) => Ok(s),
        Err(_) => {
            // Fallback to CP1252 (Windows-1252)
            let (decoded, _encoding, _had_errors) =
                encoding_rs::WINDOWS_1252.decode(&bytes);
            Ok(decoded.into_owned())
        }
    }
}

/// Skip padding bytes
pub fn skip_bytes<R: Read + Seek>(reader: &mut R, count: usize) -> Result<()> {
    use std::io::SeekFrom;
    reader.seek(SeekFrom::Current(count as i64))?;
    Ok(())
}
```

**Implementation in `src/read/mod.rs`:**

```rust
mod io;
mod header;
mod item;
mod point;

pub use header::parse_header;
pub use item::parse_items;
pub use point::{PointIterator, ParsedPoint};

use std::io::{Read, Seek};
use crate::error::{Error, Result, Warning};
use crate::types::CubFile;

/// Parse a CUB file from a reader
pub fn parse<R: Read + Seek>(mut reader: R) -> Result<(CubFile<R>, Vec<Warning>)> {
    todo!("Will implement in Phase 2")
}
```

**Testing approach:**

Create `src/read/io.rs` with inline tests using in-memory cursor:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_i16_le() {
        let data = vec![0x34, 0x12];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_i16(&mut cursor, ByteOrder::LE).unwrap(), 0x1234);
    }

    #[test]
    fn read_i16_be() {
        let data = vec![0x12, 0x34];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_i16(&mut cursor, ByteOrder::BE).unwrap(), 0x1234);
    }

    #[test]
    fn read_i32_le() {
        let data = vec![0x78, 0x56, 0x34, 0x12];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_i32(&mut cursor, ByteOrder::LE).unwrap(), 0x12345678);
    }

    #[test]
    fn read_f32_le() {
        let value = 3.14159f32;
        let bytes = value.to_le_bytes();
        let mut cursor = Cursor::new(bytes);
        let result = read_f32_le(&mut cursor).unwrap();
        assert!((result - value).abs() < 0.0001);
    }

    #[test]
    fn read_string_utf8() {
        let data = b"Hello";
        let mut cursor = Cursor::new(data);
        assert_eq!(read_string(&mut cursor, 5).unwrap(), "Hello");
    }

    #[test]
    fn read_string_cp1252_fallback() {
        // CP1252 character (not valid UTF-8)
        let data = vec![0xE9]; // é in CP1252
        let mut cursor = Cursor::new(data);
        assert_eq!(read_string(&mut cursor, 1).unwrap(), "é");
    }
}
```

**Test command:** `cargo test read::io`

**Commit message:** `Add byte-order aware I/O helpers`

---

### Phase 2: Data Structures (Header, Item, Point types)

#### Task 2.1: Header struct

**Files to create:** `src/types/header.rs`

**Objective:** Define Header struct with public fields and helper methods.

**Implementation:**

```rust
/// CUB file header (first 210 bytes)
#[derive(Debug, Clone)]
pub struct Header {
    // Raw fields (public)
    pub ident: u32,
    pub title: String,
    pub allowed_serials: [u16; 8],
    pub pc_byte_order: u8,
    pub is_secured: u8,
    pub crc32: u32,
    pub key: [u8; 16],
    pub size_of_item: i32,
    pub size_of_point: i32,
    pub hdr_items: i32,
    pub max_pts: i32,
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub max_width: f32,
    pub max_height: f32,
    pub lo_la_scale: f32,
    pub header_offset: i32,
    pub data_offset: i32,
    pub alignment: i32,
}

impl Header {
    /// Get bounding box as (west, south, east, north) in radians
    pub fn bounding_box(&self) -> (f32, f32, f32, f32) {
        (self.left, self.bottom, self.right, self.top)
    }

    /// Check if file is encrypted
    pub fn is_encrypted(&self) -> bool {
        self.is_secured != 0
    }

    /// Get byte order for integers
    pub fn byte_order(&self) -> crate::types::ByteOrder {
        crate::types::ByteOrder::from_pc_byte_order(self.pc_byte_order)
    }
}
```

**Testing approach:**
- No unit tests yet (will test through parsing)

**Commit message:** `Add Header struct`

---

#### Task 2.2: Item struct with bit-packed field accessors

**Files to create:** `src/types/item.rs`

**Objective:** Define Item struct with public fields and getter methods for bit-packed data.

**Implementation:**

```rust
use crate::types::{CubStyle, CubClass, AltStyle, ExtendedType, DaysActive, NotamType, NotamTraffic, NotamScope, NotamCodes};

/// Airspace item (42 bytes minimum, may be larger per Header.size_of_item)
#[derive(Debug, Clone)]
pub struct Item {
    // Bounding box
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,

    // Raw bit-packed fields
    pub type_byte: u8,
    pub alt_style_byte: u8,
    pub min_alt: i16,
    pub max_alt: i16,
    pub points_offset: i32,
    pub time_out: i32,
    pub extra_data: u32,
    pub active_time: u64,
    pub extended_type_byte: u8,
}

impl Item {
    /// Get airspace style/type
    pub fn style(&self) -> CubStyle {
        CubStyle::from_type_byte(self.type_byte)
    }

    /// Get airspace class
    pub fn class(&self) -> CubClass {
        CubClass::from_type_byte(self.type_byte)
    }

    /// Get minimum altitude style
    pub fn min_alt_style(&self) -> AltStyle {
        AltStyle::from_nibble(self.alt_style_byte & 0x0F)
    }

    /// Get maximum altitude style
    pub fn max_alt_style(&self) -> AltStyle {
        AltStyle::from_nibble((self.alt_style_byte >> 4) & 0x0F)
    }

    /// Get extended type if present
    pub fn extended_type(&self) -> Option<ExtendedType> {
        if self.extended_type_byte == 0 {
            None
        } else {
            ExtendedType::from_byte(self.extended_type_byte)
        }
    }

    /// Get active days flags
    pub fn days_active(&self) -> DaysActive {
        let bits = ((self.active_time >> 52) & 0xFFF) as u16;
        DaysActive::from_bits(bits)
    }

    /// Get raw start date (encoded minutes)
    pub fn start_date_raw(&self) -> Option<u32> {
        let value = ((self.active_time >> 26) & 0x3FFFFFF) as u32;
        if value == 0 {
            None
        } else {
            Some(value)
        }
    }

    /// Get raw end date (encoded minutes)
    pub fn end_date_raw(&self) -> Option<u32> {
        let value = (self.active_time & 0x3FFFFFF) as u32;
        if value == 0x3FFFFFF {
            None
        } else {
            Some(value)
        }
    }

    /// Check if ExtraData contains NOTAM data
    pub fn has_notam_data(&self) -> bool {
        (self.extra_data >> 30) == 0 && self.extra_data != 0
    }

    /// Get NOTAM type if ExtraData contains NOTAM data
    pub fn notam_type(&self) -> Option<NotamType> {
        if self.has_notam_data() {
            Some(NotamType::from_bits(self.extra_data))
        } else {
            None
        }
    }

    /// Get NOTAM traffic if ExtraData contains NOTAM data
    pub fn notam_traffic(&self) -> Option<NotamTraffic> {
        if self.has_notam_data() {
            Some(NotamTraffic::from_bits(self.extra_data))
        } else {
            None
        }
    }

    /// Get NOTAM scope if ExtraData contains NOTAM data
    pub fn notam_scope(&self) -> Option<NotamScope> {
        if self.has_notam_data() {
            Some(NotamScope::from_bits(self.extra_data))
        } else {
            None
        }
    }

    /// Get NOTAM subject and action codes if ExtraData contains NOTAM data
    pub fn notam_codes(&self) -> Option<NotamCodes> {
        NotamCodes::from_extra_data(self.extra_data)
    }

    /// Get bounding box as (west, south, east, north) in radians
    pub fn bounding_box(&self) -> (f32, f32, f32, f32) {
        (self.left, self.bottom, self.right, self.top)
    }
}

/// Decode NOTAM time from encoded minutes to datetime components
/// Returns (year, month, day, hour, minute)
pub fn decode_notam_time(encoded: u32) -> (u32, u32, u32, u32, u32) {
    let mut time = encoded;
    let minutes = time % 60;
    time /= 60;
    let hours = time % 24;
    time /= 24;
    let days = (time % 31) + 1;
    time /= 31;
    let months = (time % 12) + 1;
    time /= 12;
    let years = time + 2000;

    (years, months, days, hours, minutes)
}

#[cfg(feature = "datetime")]
use jiff::civil::DateTime;

#[cfg(feature = "datetime")]
impl Item {
    /// Get start date as DateTime (requires "datetime" feature)
    pub fn start_date(&self) -> Option<DateTime> {
        self.start_date_raw().and_then(|raw| {
            let (year, month, day, hour, minute) = decode_notam_time(raw);
            DateTime::new(
                year as i16,
                month as i8,
                day as i8,
                hour as i8,
                minute as i8,
                0,
            ).ok()
        })
    }

    /// Get end date as DateTime (requires "datetime" feature)
    pub fn end_date(&self) -> Option<DateTime> {
        self.end_date_raw().and_then(|raw| {
            let (year, month, day, hour, minute) = decode_notam_time(raw);
            DateTime::new(
                year as i16,
                month as i8,
                day as i8,
                hour as i8,
                minute as i8,
                0,
            ).ok()
        })
    }
}
```

**Add tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_style_and_class() {
        let item = Item {
            type_byte: 0b01010100,  // Class D (0101) + Style 0x04 (DA)
            // ... other fields with defaults
            left: 0.0, top: 0.0, right: 0.0, bottom: 0.0,
            alt_style_byte: 0, min_alt: 0, max_alt: 0,
            points_offset: 0, time_out: 0, extra_data: 0,
            active_time: 0, extended_type_byte: 0,
        };

        assert_eq!(item.style(), CubStyle::DangerArea);
        assert_eq!(item.class(), CubClass::ClassD);
    }

    #[test]
    fn item_alt_styles() {
        let item = Item {
            alt_style_byte: 0x32,  // Max=3 (FL), Min=2 (MSL)
            // ... other fields
            type_byte: 0, left: 0.0, top: 0.0, right: 0.0, bottom: 0.0,
            min_alt: 0, max_alt: 0, points_offset: 0, time_out: 0,
            extra_data: 0, active_time: 0, extended_type_byte: 0,
        };

        assert_eq!(item.min_alt_style(), AltStyle::MeanSeaLevel);
        assert_eq!(item.max_alt_style(), AltStyle::FlightLevel);
    }

    #[test]
    fn decode_notam_time_example() {
        // Example: 2024-07-15 14:30
        // Manually calculated encoded value
        let encoded = 30 + 60 * (14 + 24 * (14 + 31 * (6 + 12 * 24)));
        let (y, m, d, h, min) = decode_notam_time(encoded);
        assert_eq!((y, m, d, h, min), (2024, 7, 15, 14, 30));
    }
}
```

**Test command:** `cargo test types::item`

**Commit message:** `Add Item struct with bit-packed accessors`

---

#### Task 2.3: Point types (stub)

**Files to create:** `src/types/point.rs`

**Objective:** Define types that will be returned by point iterator.

**Implementation:**

```rust
use crate::types::CubDataId;

/// A parsed geometric point with optional attributes
#[derive(Debug, Clone)]
pub struct ParsedPoint {
    /// Longitude in radians
    pub lon: f32,
    /// Latitude in radians
    pub lat: f32,
    /// Optional airspace name (present on first point of sequence)
    pub name: Option<String>,
    /// Optional frequency in Hz
    pub frequency: Option<u32>,
    /// Optional frequency name/label
    pub frequency_name: Option<String>,
    /// Optional additional data
    pub optional_data: Vec<OptionalData>,
}

/// Optional data records found in point sequences
#[derive(Debug, Clone)]
pub enum OptionalData {
    IcaoCode(String),
    SecondaryFrequency(u32),
    ExceptionRules(String),
    NotamRemarks(String),
    NotamId(String),
    NotamInsertTime(u32),  // Raw encoded minutes
}

impl OptionalData {
    pub fn data_id(&self) -> CubDataId {
        match self {
            OptionalData::IcaoCode(_) => CubDataId::IcaoCode,
            OptionalData::SecondaryFrequency(_) => CubDataId::SecondaryFrequency,
            OptionalData::ExceptionRules(_) => CubDataId::ExceptionRules,
            OptionalData::NotamRemarks(_) => CubDataId::NotamRemarks,
            OptionalData::NotamId(_) => CubDataId::NotamId,
            OptionalData::NotamInsertTime(_) => CubDataId::NotamInsertTime,
        }
    }
}

#[cfg(feature = "datetime")]
use jiff::civil::DateTime;

#[cfg(feature = "datetime")]
impl OptionalData {
    /// Get NOTAM insert time as DateTime (requires "datetime" feature)
    pub fn notam_insert_datetime(&self) -> Option<DateTime> {
        match self {
            OptionalData::NotamInsertTime(raw) => {
                let (year, month, day, hour, minute) =
                    crate::types::item::decode_notam_time(*raw);
                DateTime::new(
                    year as i16,
                    month as i8,
                    day as i8,
                    hour as i8,
                    minute as i8,
                    0,
                ).ok()
            }
            _ => None,
        }
    }
}
```

**Testing approach:**
- No tests yet (will test through point parsing)

**Commit message:** `Add ParsedPoint and OptionalData types`

---

#### Task 2.4: CubFile struct

**Files to modify:** `src/types/mod.rs`

**Objective:** Add CubFile struct that owns reader and parsed data.

**Add to `src/types/mod.rs`:**

```rust
use crate::error::Result;
use std::io::{Read, Seek};

/// Parsed CUB file with header, items, and reader for lazy point parsing
pub struct CubFile<R> {
    header: Header,
    items: Vec<Item>,
    reader: R,
}

impl<R> CubFile<R> {
    /// Create new CubFile (used internally by parser)
    pub(crate) fn new(header: Header, items: Vec<Item>, reader: R) -> Self {
        Self { header, items, reader }
    }

    /// Get file header
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Get all airspace items
    pub fn items(&self) -> &[Item] {
        &self.items
    }
}

impl<R: Read + Seek> CubFile<R> {
    /// Parse points for a specific item
    /// Returns iterator that lazily parses CubPoint sequences
    pub fn read_points(&mut self, item: &Item) -> Result<crate::read::PointIterator<'_, R>> {
        crate::read::point::PointIterator::new(
            &mut self.reader,
            &self.header,
            item,
        )
    }
}
```

**Testing approach:**
- Will test through integration tests

**Commit message:** `Add CubFile struct`

---

### Phase 3: Parsing Implementation

#### Task 3.1: Parse header

**Files to create:** `src/read/header.rs`

**Objective:** Implement header parsing with proper byte order handling.

**Implementation:**

```rust
use std::io::{Read, Seek, SeekFrom};
use crate::error::{Error, Result, Warning};
use crate::types::{Header, ByteOrder};
use crate::read::io::*;

/// Parse CUB header (first 210 bytes)
pub fn parse_header<R: Read + Seek>(reader: &mut R) -> Result<(Header, Vec<Warning>)> {
    let mut warnings = Vec::new();

    // Seek to start
    reader.seek(SeekFrom::Start(0))?;

    // Read and validate magic bytes (offset 0-3, always LE)
    let ident = {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        u32::from_le_bytes(buf)
    };

    if ident != 0x425543C2 {
        return Err(Error::InvalidMagicBytes);
    }

    // Read title (offset 4-115, 112 bytes)
    let title = read_string(reader, 112)?.trim_end_matches('\0').to_string();

    // Read allowed serials (offset 116-131, 8 × u16)
    // Parse as LE initially, will re-interpret if needed after reading PcByteOrder
    let allowed_serials = {
        let mut serials = [0u16; 8];
        for serial in &mut serials {
            *serial = read_u16(reader, ByteOrder::LE)?;
        }
        serials
    };

    // Read PcByteOrder (offset 132)
    let pc_byte_order = read_u8(reader)?;
    let byte_order = ByteOrder::from_pc_byte_order(pc_byte_order);

    // Re-read allowed_serials if byte order is BE
    let allowed_serials = if byte_order == ByteOrder::BE {
        reader.seek(SeekFrom::Start(116))?;
        let mut serials = [0u16; 8];
        for serial in &mut serials {
            *serial = read_u16(reader, ByteOrder::BE)?;
        }
        reader.seek(SeekFrom::Start(133))?;  // Skip back to after PcByteOrder
        serials
    } else {
        allowed_serials
    };

    // Read IsSecured (offset 133)
    let is_secured = read_u8(reader)?;

    // Check encryption
    if is_secured != 0 {
        return Err(Error::EncryptedFile);
    }

    // Read Crc32 (offset 134-137)
    let crc32 = read_u32(reader, byte_order)?;

    // Read Key (offset 138-153, 16 bytes)
    let key = {
        let bytes = read_bytes(reader, 16)?;
        let mut key = [0u8; 16];
        key.copy_from_slice(&bytes);
        key
    };

    // Read remaining header fields (all use determined byte_order)
    let size_of_item = read_i32(reader, byte_order)?;
    let size_of_point = read_i32(reader, byte_order)?;
    let hdr_items = read_i32(reader, byte_order)?;
    let max_pts = read_i32(reader, byte_order)?;

    // Floats are always LE
    let left = read_f32_le(reader)?;
    let top = read_f32_le(reader)?;
    let right = read_f32_le(reader)?;
    let bottom = read_f32_le(reader)?;
    let max_width = read_f32_le(reader)?;
    let max_height = read_f32_le(reader)?;
    let lo_la_scale = read_f32_le(reader)?;

    let header_offset = read_i32(reader, byte_order)?;
    let data_offset = read_i32(reader, byte_order)?;
    let alignment = read_i32(reader, byte_order)?;

    // Validate sizes
    if size_of_item < 42 {
        warnings.push(Warning::OversizedItem {
            expected: size_of_item,
            actual: 42,
        });
    }

    if size_of_point < 5 {
        warnings.push(Warning::OversizedItem {
            expected: size_of_point,
            actual: 5,
        });
    }

    let header = Header {
        ident,
        title,
        allowed_serials,
        pc_byte_order,
        is_secured,
        crc32,
        key,
        size_of_item,
        size_of_point,
        hdr_items,
        max_pts,
        left,
        top,
        right,
        bottom,
        max_width,
        max_height,
        lo_la_scale,
        header_offset,
        data_offset,
        alignment,
    };

    Ok((header, warnings))
}
```

**Testing approach:**

Create test helper to build minimal valid header bytes, then test:
- Valid LE header
- Valid BE header
- Invalid magic bytes
- Encrypted file error
- Warning for undersized size_of_item

**Add to `src/read/header.rs`:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn minimal_header_bytes(byte_order: ByteOrder, encrypted: bool) -> Vec<u8> {
        let mut bytes = vec![0u8; 210];

        // Magic bytes (LE)
        bytes[0..4].copy_from_slice(&0x425543C2u32.to_le_bytes());

        // Title (offset 4)
        bytes[4..116].copy_from_slice(b"Test Header\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");

        // Allowed serials (offset 116, skip)

        // PcByteOrder (offset 132)
        bytes[132] = match byte_order {
            ByteOrder::BE => 0,
            ByteOrder::LE => 1,
        };

        // IsSecured (offset 133)
        bytes[133] = if encrypted { 1 } else { 0 };

        // Write minimal valid values for remaining fields
        let write_i32 = |bytes: &mut [u8], offset: usize, value: i32| {
            let val_bytes = match byte_order {
                ByteOrder::LE => value.to_le_bytes(),
                ByteOrder::BE => value.to_be_bytes(),
            };
            bytes[offset..offset+4].copy_from_slice(&val_bytes);
        };

        write_i32(&mut bytes, 154, 42);   // size_of_item
        write_i32(&mut bytes, 158, 5);    // size_of_point
        write_i32(&mut bytes, 162, 0);    // hdr_items
        write_i32(&mut bytes, 166, 100);  // max_pts

        // Floats (always LE)
        bytes[170..174].copy_from_slice(&0.0f32.to_le_bytes()); // left
        bytes[174..178].copy_from_slice(&1.0f32.to_le_bytes()); // top
        bytes[178..182].copy_from_slice(&1.0f32.to_le_bytes()); // right
        bytes[182..186].copy_from_slice(&0.0f32.to_le_bytes()); // bottom
        bytes[186..190].copy_from_slice(&1.0f32.to_le_bytes()); // max_width
        bytes[190..194].copy_from_slice(&1.0f32.to_le_bytes()); // max_height
        bytes[194..198].copy_from_slice(&1.0f32.to_le_bytes()); // lo_la_scale

        write_i32(&mut bytes, 198, 210);  // header_offset
        write_i32(&mut bytes, 202, 210);  // data_offset
        write_i32(&mut bytes, 206, 0);    // alignment

        bytes
    }

    #[test]
    fn parse_valid_le_header() {
        let bytes = minimal_header_bytes(ByteOrder::LE, false);
        let mut cursor = Cursor::new(bytes);
        let (header, warnings) = parse_header(&mut cursor).unwrap();

        assert_eq!(header.ident, 0x425543C2);
        assert_eq!(header.byte_order(), ByteOrder::LE);
        assert!(!header.is_encrypted());
        assert_eq!(header.size_of_item, 42);
        assert!(warnings.is_empty());
    }

    #[test]
    fn parse_valid_be_header() {
        let bytes = minimal_header_bytes(ByteOrder::BE, false);
        let mut cursor = Cursor::new(bytes);
        let (header, warnings) = parse_header(&mut cursor).unwrap();

        assert_eq!(header.byte_order(), ByteOrder::BE);
        assert_eq!(header.size_of_item, 42);
        assert!(warnings.is_empty());
    }

    #[test]
    fn invalid_magic_bytes() {
        let mut bytes = minimal_header_bytes(ByteOrder::LE, false);
        bytes[0] = 0xFF;  // Corrupt magic
        let mut cursor = Cursor::new(bytes);

        match parse_header(&mut cursor) {
            Err(Error::InvalidMagicBytes) => {},
            _ => panic!("Expected InvalidMagicBytes error"),
        }
    }

    #[test]
    fn encrypted_file_error() {
        let bytes = minimal_header_bytes(ByteOrder::LE, true);
        let mut cursor = Cursor::new(bytes);

        match parse_header(&mut cursor) {
            Err(Error::EncryptedFile) => {},
            _ => panic!("Expected EncryptedFile error"),
        }
    }
}
```

**Test command:** `cargo test read::header`

**Commit message:** `Implement header parsing`

---

#### Task 3.2: Parse items

**Files to create:** `src/read/item.rs`

**Objective:** Parse fixed-size array of Item structs.

**Implementation:**

```rust
use std::io::{Read, Seek, SeekFrom};
use crate::error::{Result, Warning};
use crate::types::{Header, Item};
use crate::read::io::*;

/// Parse all items from CUB file
pub fn parse_items<R: Read + Seek>(
    reader: &mut R,
    header: &Header,
) -> Result<(Vec<Item>, Vec<Warning>)> {
    let mut warnings = Vec::new();
    let byte_order = header.byte_order();

    // Seek to items section
    reader.seek(SeekFrom::Start(header.header_offset as u64))?;

    let mut items = Vec::with_capacity(header.hdr_items as usize);

    for _ in 0..header.hdr_items {
        // Read Item fields (42 bytes minimum)
        let left = read_f32_le(reader)?;
        let top = read_f32_le(reader)?;
        let right = read_f32_le(reader)?;
        let bottom = read_f32_le(reader)?;

        let type_byte = read_u8(reader)?;
        let alt_style_byte = read_u8(reader)?;
        let min_alt = read_i16(reader, byte_order)?;
        let max_alt = read_i16(reader, byte_order)?;
        let points_offset = read_i32(reader, byte_order)?;
        let time_out = read_i32(reader, byte_order)?;
        let extra_data = read_u32(reader, byte_order)?;
        let active_time = read_u64(reader, byte_order)?;
        let extended_type_byte = read_u8(reader)?;

        // Skip padding if SizeOfItem > 42
        let padding = header.size_of_item - 42;
        if padding > 0 {
            skip_bytes(reader, padding as usize)?;
        }

        items.push(Item {
            left,
            top,
            right,
            bottom,
            type_byte,
            alt_style_byte,
            min_alt,
            max_alt,
            points_offset,
            time_out,
            extra_data,
            active_time,
            extended_type_byte,
        });
    }

    Ok((items, warnings))
}
```

**Testing approach:**

Build minimal item bytes and test parsing:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use crate::types::ByteOrder;

    fn minimal_header() -> Header {
        Header {
            ident: 0x425543C2,
            title: String::new(),
            allowed_serials: [0; 8],
            pc_byte_order: 1,
            is_secured: 0,
            crc32: 0,
            key: [0; 16],
            size_of_item: 42,
            size_of_point: 5,
            hdr_items: 2,
            max_pts: 100,
            left: 0.0,
            top: 1.0,
            right: 1.0,
            bottom: 0.0,
            max_width: 1.0,
            max_height: 1.0,
            lo_la_scale: 1.0,
            header_offset: 0,
            data_offset: 84,  // 2 items × 42 bytes
            alignment: 0,
        }
    }

    fn build_item_bytes(byte_order: ByteOrder) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Item 1
        bytes.extend_from_slice(&0.1f32.to_le_bytes());  // left
        bytes.extend_from_slice(&0.5f32.to_le_bytes());  // top
        bytes.extend_from_slice(&0.4f32.to_le_bytes());  // right
        bytes.extend_from_slice(&0.2f32.to_le_bytes());  // bottom

        bytes.push(0x04);  // type (DA)
        bytes.push(0x23);  // alt_style (max=FL, min=MSL)

        let write_i16 = |bytes: &mut Vec<u8>, val: i16| {
            bytes.extend_from_slice(&match byte_order {
                ByteOrder::LE => val.to_le_bytes(),
                ByteOrder::BE => val.to_be_bytes(),
            });
        };
        let write_i32 = |bytes: &mut Vec<u8>, val: i32| {
            bytes.extend_from_slice(&match byte_order {
                ByteOrder::LE => val.to_le_bytes(),
                ByteOrder::BE => val.to_be_bytes(),
            });
        };
        let write_u32 = |bytes: &mut Vec<u8>, val: u32| {
            bytes.extend_from_slice(&match byte_order {
                ByteOrder::LE => val.to_le_bytes(),
                ByteOrder::BE => val.to_be_bytes(),
            });
        };
        let write_u64 = |bytes: &mut Vec<u8>, val: u64| {
            bytes.extend_from_slice(&match byte_order {
                ByteOrder::LE => val.to_le_bytes(),
                ByteOrder::BE => val.to_be_bytes(),
            });
        };

        write_i16(&mut bytes, 100);   // min_alt
        write_i16(&mut bytes, 5000);  // max_alt
        write_i32(&mut bytes, 0);     // points_offset
        write_i32(&mut bytes, 0);     // time_out
        write_u32(&mut bytes, 0);     // extra_data
        write_u64(&mut bytes, 0x3FFFFFF);  // active_time (default)
        bytes.push(0);  // extended_type

        // Item 2 (copy of item 1 for simplicity)
        let item1 = bytes.clone();
        bytes.extend_from_slice(&item1);

        bytes
    }

    #[test]
    fn parse_items_le() {
        let header = minimal_header();
        let bytes = build_item_bytes(ByteOrder::LE);
        let mut cursor = Cursor::new(bytes);

        let (items, warnings) = parse_items(&mut cursor, &header).unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].left, 0.1);
        assert_eq!(items[0].min_alt, 100);
        assert_eq!(items[0].max_alt, 5000);
        assert!(warnings.is_empty());
    }
}
```

**Test command:** `cargo test read::item`

**Commit message:** `Implement item array parsing`

---

#### Task 3.3: Point iterator (basic structure)

**Files to create:** `src/read/point.rs`

**Objective:** Create PointIterator structure that maintains parsing state.

**Implementation:**

```rust
use std::io::{Read, Seek, SeekFrom};
use crate::error::{Error, Result, Warning};
use crate::types::{Header, Item, ParsedPoint, OptionalData, CubDataId};
use crate::read::io::*;

/// Iterator that lazily parses CubPoint sequences for an item
pub struct PointIterator<'a, R> {
    reader: &'a mut R,
    header: &'a Header,
    item: Item,
    origin_x: f32,
    origin_y: f32,
    done: bool,
    warnings: Vec<Warning>,
    // Attributes parsed from current point sequence
    current_name: Option<String>,
    current_frequency: Option<u32>,
    current_frequency_name: Option<String>,
    current_optional: Vec<OptionalData>,
}

impl<'a, R: Read + Seek> PointIterator<'a, R> {
    /// Create new point iterator for an item
    pub(crate) fn new(
        reader: &'a mut R,
        header: &'a Header,
        item: &Item,
    ) -> Result<Self> {
        // Seek to first point for this item
        let offset = header.data_offset as u64 + item.points_offset as u64;
        reader.seek(SeekFrom::Start(offset))?;

        // Initialize origin to item's bottom-left
        let origin_x = item.left;
        let origin_y = item.bottom;

        Ok(Self {
            reader,
            header,
            item: item.clone(),
            origin_x,
            origin_y,
            done: false,
            warnings: Vec::new(),
            current_name: None,
            current_frequency: None,
            current_frequency_name: None,
            current_optional: Vec::new(),
        })
    }

    /// Get warnings collected during parsing
    pub fn warnings(&self) -> &[Warning] {
        &self.warnings
    }

    /// Parse next CubPoint
    fn parse_next_point(&mut self) -> Result<Option<ParsedPoint>> {
        if self.done {
            return Ok(None);
        }

        let byte_order = self.header.byte_order();

        loop {
            let flag = read_u8(self.reader)?;

            match flag {
                0x81 => {
                    // Set origin offset
                    let delta_x = read_i16(self.reader, byte_order)? as f32 * self.header.lo_la_scale;
                    let delta_y = read_i16(self.reader, byte_order)? as f32 * self.header.lo_la_scale;
                    self.origin_x += delta_x;
                    self.origin_y += delta_y;

                    // Continue reading
                }

                0x01 => {
                    // New point
                    let x = read_i16(self.reader, byte_order)? as f32 * self.header.lo_la_scale;
                    let y = read_i16(self.reader, byte_order)? as f32 * self.header.lo_la_scale;

                    let lon = self.origin_x + x;
                    let lat = self.origin_y + y;

                    // Build point with current attributes
                    let point = ParsedPoint {
                        lon,
                        lat,
                        name: self.current_name.take(),
                        frequency: self.current_frequency.take(),
                        frequency_name: self.current_frequency_name.take(),
                        optional_data: std::mem::take(&mut self.current_optional),
                    };

                    return Ok(Some(point));
                }

                flag if (flag & 0x40) != 0 => {
                    // Attribute block
                    self.parse_attributes(flag)?;
                }

                0x00 => {
                    // End of points
                    self.done = true;
                    return Ok(None);
                }

                _ => {
                    // Unknown flag - collect warning and skip
                    self.warnings.push(Warning::UnknownPointFlag(flag));
                    // Try to skip this point (size_of_point - 1 for flag already read)
                    skip_bytes(self.reader, (self.header.size_of_point - 1) as usize)?;
                }
            }
        }
    }

    /// Parse attribute records starting with given flag
    fn parse_attributes(&mut self, first_flag: u8) -> Result<()> {
        let byte_order = self.header.byte_order();

        // First attribute: name
        if (first_flag & 0x40) != 0 {
            let name_len = (first_flag & 0x3F) as usize;
            if name_len > 0 {
                let name = read_string(self.reader, name_len)?;
                self.current_name = Some(name.trim_end_matches('\0').to_string());
            }
        }

        // Check for frequency attribute
        let next_flag = read_u8(self.reader)?;
        if (next_flag & 0xC0) == 0xC0 {
            let freq_name_len = (next_flag & 0x3F) as usize;
            let frequency = read_u32(self.reader, byte_order)?;
            self.current_frequency = Some(frequency);

            if freq_name_len > 0 {
                let freq_name = read_string(self.reader, freq_name_len)?;
                self.current_frequency_name = Some(freq_name.trim_end_matches('\0').to_string());
            }

            // Read next flag for optional data
            self.parse_optional_data()?;
        } else if next_flag == 0xA0 {
            // Optional data without frequency
            self.parse_optional_data_with_flag(next_flag)?;
        } else {
            // No more attributes, seek back one byte
            use std::io::SeekFrom;
            self.reader.seek(SeekFrom::Current(-1))?;
        }

        Ok(())
    }

    /// Parse optional data records (0xA0 flags)
    fn parse_optional_data(&mut self) -> Result<()> {
        loop {
            let flag = read_u8(self.reader)?;
            if flag != 0xA0 {
                // Not optional data, seek back
                use std::io::SeekFrom;
                self.reader.seek(SeekFrom::Current(-1))?;
                break;
            }

            self.parse_optional_data_record()?;
        }

        Ok(())
    }

    /// Parse optional data with flag already read
    fn parse_optional_data_with_flag(&mut self, flag: u8) -> Result<()> {
        if flag == 0xA0 {
            self.parse_optional_data_record()?;
            self.parse_optional_data()?;
        }
        Ok(())
    }

    /// Parse single optional data record
    fn parse_optional_data_record(&mut self) -> Result<()> {
        let byte_order = self.header.byte_order();

        let data_id = read_u8(self.reader)?;
        let b1 = read_u8(self.reader)?;
        let b2 = read_u8(self.reader)?;
        let b3 = read_u8(self.reader)?;

        let data_id_enum = CubDataId::from_byte(data_id);

        match data_id_enum {
            Some(CubDataId::IcaoCode) => {
                let len = b3 as usize;
                let icao = read_string(self.reader, len)?;
                self.current_optional.push(OptionalData::IcaoCode(icao));
            }

            Some(CubDataId::SecondaryFrequency) => {
                let value = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);
                self.current_optional.push(OptionalData::SecondaryFrequency(value));
            }

            Some(CubDataId::ExceptionRules) => {
                let len = (((b2 as u16) << 8) | (b3 as u16)) as usize;
                let rules = read_string(self.reader, len)?;
                self.current_optional.push(OptionalData::ExceptionRules(rules));
            }

            Some(CubDataId::NotamRemarks) => {
                let len = (((b2 as u16) << 8) | (b3 as u16)) as usize;
                let remarks = read_string(self.reader, len)?;
                self.current_optional.push(OptionalData::NotamRemarks(remarks));
            }

            Some(CubDataId::NotamId) => {
                let len = b3 as usize;
                let id = read_string(self.reader, len)?;
                self.current_optional.push(OptionalData::NotamId(id));
            }

            Some(CubDataId::NotamInsertTime) => {
                let b4 = read_u8(self.reader)?;
                let value = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);
                let time = (value << 8) | (b4 as u32);
                self.current_optional.push(OptionalData::NotamInsertTime(time));
            }

            None => {
                self.warnings.push(Warning::UnknownPointFlag(data_id));
            }
        }

        Ok(())
    }
}

impl<'a, R: Read + Seek> Iterator for PointIterator<'a, R> {
    type Item = Result<ParsedPoint>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next_point().transpose()
    }
}
```

**Testing approach:**
- Build minimal point byte sequences
- Test origin offset
- Test simple points
- Test points with attributes
- Integration test with real file

**Add basic tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use crate::types::ByteOrder;

    fn minimal_header() -> Header {
        Header {
            ident: 0x425543C2,
            title: String::new(),
            allowed_serials: [0; 8],
            pc_byte_order: 1,
            is_secured: 0,
            crc32: 0,
            key: [0; 16],
            size_of_item: 42,
            size_of_point: 5,
            hdr_items: 1,
            max_pts: 100,
            left: 0.0,
            top: 1.0,
            right: 1.0,
            bottom: 0.0,
            max_width: 1.0,
            max_height: 1.0,
            lo_la_scale: 0.0001,
            header_offset: 210,
            data_offset: 252,  // 210 + 42
            alignment: 0,
        }
    }

    fn minimal_item() -> Item {
        Item {
            left: 0.0,
            top: 1.0,
            right: 1.0,
            bottom: 0.0,
            type_byte: 0x04,
            alt_style_byte: 0,
            min_alt: 0,
            max_alt: 1000,
            points_offset: 0,
            time_out: 0,
            extra_data: 0,
            active_time: 0,
            extended_type_byte: 0,
        }
    }

    #[test]
    fn parse_simple_point() {
        let mut bytes = Vec::new();

        // Point: 0x01 flag + coords
        bytes.push(0x01);
        bytes.extend_from_slice(&100i16.to_le_bytes());  // x offset
        bytes.extend_from_slice(&200i16.to_le_bytes());  // y offset

        // End marker
        bytes.push(0x00);

        let mut cursor = Cursor::new(bytes);
        let header = minimal_header();
        let item = minimal_item();

        let mut iter = PointIterator::new(&mut cursor, &header, &item).unwrap();

        let point = iter.next().unwrap().unwrap();
        assert!((point.lon - (0.0 + 100.0 * 0.0001)).abs() < 0.00001);
        assert!((point.lat - (0.0 + 200.0 * 0.0001)).abs() < 0.00001);
        assert!(point.name.is_none());

        assert!(iter.next().is_none());
    }
}
```

**Test command:** `cargo test read::point`

**Commit message:** `Implement PointIterator with basic parsing`

---

#### Task 3.4: Main parse() function

**Files to modify:** `src/read/mod.rs`

**Objective:** Implement main parse function that orchestrates header and item parsing.

**Replace placeholder in `src/read/mod.rs`:**

```rust
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
```

**Testing approach:**
- Integration test with minimal complete file
- Will test more thoroughly with real file in next phase

**Test command:** `cargo test read::parse`

**Commit message:** `Implement main parse() function`

---

### Phase 4: Integration & Testing

#### Task 4.1: Integration test with real file

**Files to create:** `tests/reader_test.rs`

**Objective:** Test parsing the real France CUB file fixture.

**Implementation:**

```rust
use seeyou_cub::parse;
use std::fs::File;

#[test]
fn parse_france_fixture() {
    let file = File::open("tests/fixtures/france_2024.07.02.cub")
        .expect("Failed to open fixture file");

    let (cub, warnings) = parse(file).expect("Failed to parse CUB file");

    // Basic assertions
    assert_eq!(cub.header().ident, 0x425543C2);
    assert!(cub.items().len() > 0, "Should have at least one airspace");

    // Check bounding box makes sense for France
    let (west, south, east, north) = cub.header().bounding_box();
    println!("Bounding box: W={} S={} E={} N={}", west, south, east, north);

    // Print some stats
    println!("Total airspaces: {}", cub.items().len());
    println!("Warnings: {}", warnings.len());
    for warning in &warnings {
        println!("  Warning: {:?}", warning);
    }

    // Check first few items
    for (i, item) in cub.items().iter().take(5).enumerate() {
        println!("Item {}: style={:?} class={:?} alt={}-{}",
            i, item.style(), item.class(), item.min_alt, item.max_alt);
    }
}

#[test]
fn parse_and_read_points() {
    let file = File::open("tests/fixtures/france_2024.07.02.cub")
        .expect("Failed to open fixture file");

    let (mut cub, _warnings) = parse(file).expect("Failed to parse CUB file");

    // Parse points for first item
    if let Some(first_item) = cub.items().first() {
        let mut points = cub.read_points(first_item).expect("Failed to read points");

        let mut point_count = 0;
        let mut first_point = None;

        for point_result in &mut points {
            let point = point_result.expect("Failed to parse point");
            if first_point.is_none() {
                first_point = Some(point.clone());
            }
            point_count += 1;
        }

        println!("First item has {} points", point_count);

        if let Some(point) = first_point {
            println!("First point: lon={} lat={}", point.lon, point.lat);
            if let Some(name) = point.name {
                println!("  Name: {}", name);
            }
            if let Some(freq) = point.frequency {
                println!("  Frequency: {} Hz", freq);
            }
        }

        // Check warnings from point parsing
        for warning in points.warnings() {
            println!("Point warning: {:?}", warning);
        }

        assert!(point_count > 0, "Should have at least one point");
    }
}

#[test]
fn iterate_all_airspaces() {
    let file = File::open("tests/fixtures/france_2024.07.02.cub")
        .expect("Failed to open fixture file");

    let (mut cub, _warnings) = parse(file).expect("Failed to parse CUB file");

    let mut total_points = 0;

    for item in cub.items() {
        let points = cub.read_points(item).expect("Failed to read points");
        let count = points.count();
        total_points += count;
    }

    println!("Total points across all airspaces: {}", total_points);
    assert!(total_points > 0);
}
```

**Testing approach:**

These tests include `println!()` statements for initial exploration and verification of the real-world file. Run with:

```bash
cargo test --test reader_test -- --nocapture
```

This allows you to inspect:
- Actual bounding box values from the France file
- Number of airspaces and warnings
- Sample airspace data
- Point counts and geometry

In Task 4.2, these `println!()` statements will be converted to proper assertions once we understand what the correct values should be.

**Expected outcome:**
- Tests should pass (or reveal bugs to fix)
- Review printed output to understand file contents
- Collect any warnings for analysis

**Commit message:** `Add integration tests with France fixture`

---

#### Task 4.2: Fix bugs and add assertions

**Objective:** Address issues discovered while parsing real file, then convert exploratory prints to assertions.

**Process:**
1. Run integration tests with `--nocapture` and review output
2. Investigate failures/unexpected results
3. Add unit tests reproducing specific bugs
4. Fix bugs
5. Verify all tests pass
6. **Convert `println!()` statements to exact assertions** based on observed values:
   - Replace bounding box prints with exact value assertions
   - Replace item count prints with exact count assertions (`assert_eq!`)
   - Replace warning prints with exact warning count/type assertions
   - Replace point count prints with exact count assertions
7. Run tests again to ensure assertions pass

**Common issues to watch for:**
- Incorrect byte order handling
- String encoding edge cases
- Point sequence termination
- Attribute parsing edge cases
- Offset calculations

**Example assertion conversions:**

```rust
// Before (Task 4.1):
println!("Total airspaces: {}", cub.items().len());

// After (Task 4.2) - use the exact value observed:
assert_eq!(cub.items().len(), 1234); // Replace 1234 with actual observed value
```

The fixture file is fixed, so we should assert exact values to catch any regression in parsing behavior.

**Commit each bug fix separately with message format:** `Fix [specific issue]`

**Final commit after all fixes:** `Convert integration test prints to assertions`

---

#### Task 4.3: Public API and documentation

**Files to modify:** `src/lib.rs`

**Objective:** Define clean public API surface and add module documentation.

**Implementation:**

```rust
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
pub use read::parse;
pub use types::{CubFile, Header, Item, ParsedPoint, OptionalData};
pub use types::{
    ByteOrder, CubStyle, CubClass, AltStyle, ExtendedType,
    DaysActive, NotamType, NotamTraffic, NotamScope, NotamCodes,
    CubDataId,
};
pub use error::{Error, Warning};

mod error;
mod types;
mod read;
```

**Add documentation to key types:**

Add doc comments to:
- `Header` struct
- `Item` struct and main methods
- `ParsedPoint` struct
- Key enums like `CubStyle`, `CubClass`
- `parse()` function

**Example for `parse()`:**

```rust
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
/// for item in cub.items() {
///     println!("Airspace: {:?}", item.style());
///
///     // Parse geometry for this item
///     for point in cub.read_points(item)? {
///         let pt = point?;
///         println!("  Point: {} {}", pt.lon, pt.lat);
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse<R: Read + Seek>(mut reader: R) -> Result<(CubFile<R>, Vec<Warning>)> {
    // ... implementation
}
```

**Test documentation:**

```bash
cargo doc --open
```

Review generated docs for clarity and completeness.

**Commit message:** `Add public API and documentation`

---

#### Task 4.4: Add README and examples

**Files to create:** `README.md`, `examples/basic.rs`

**README.md:**

````markdown
# seeyou-cub

A Rust parser for the SeeYou CUB binary file format, which stores airspace data for flight navigation software.

## Features

- Parse CUB files from any `Read + Seek` source
- Lenient parsing with warning collection
- Lazy geometry parsing for memory efficiency
- Support for both little-endian and big-endian files
- Optional `jiff` integration for date/time handling

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
seeyou-cub = "0.0.0"
```

Basic example:

```rust
use seeyou_cub::parse;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("airspace.cub")?;
    let (mut cub, warnings) = parse(file)?;

    println!("Loaded {} airspaces", cub.items().len());

    // Inspect warnings
    for warning in warnings {
        eprintln!("Warning: {:?}", warning);
    }

    // Access airspace metadata
    for item in cub.items() {
        println!("{:?} {:?}: {}-{} meters",
            item.style(),
            item.class(),
            item.min_alt,
            item.max_alt,
        );
    }

    // Parse geometry for first airspace
    if let Some(first) = cub.items().first() {
        for point in cub.read_points(first)? {
            let pt = point?;
            println!("  Point: {} {}", pt.lon, pt.lat);
            if let Some(name) = &pt.name {
                println!("    Name: {}", name);
            }
        }
    }

    Ok(())
}
```

## Optional Features

### `datetime`

Enable `jiff` integration for convenient date/time handling:

```toml
[dependencies]
seeyou-cub = { version = "0.0.0", features = ["datetime"] }
```

With this feature enabled, `Item::start_date()` and `Item::end_date()` return `jiff::civil::DateTime`.

## File Format

The CUB format specification is available in the `docs/CUB_file_format.md` file.

## License

TBD
````

**examples/basic.rs:**

```rust
use seeyou_cub::parse;
use std::env;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <cub-file>", args[0]);
        std::process::exit(1);
    }

    let file = File::open(&args[1])?;
    let (mut cub, warnings) = parse(file)?;

    println!("=== CUB File Info ===");
    println!("Header: {}", cub.header().title);
    println!("Airspaces: {}", cub.items().len());

    let (w, s, e, n) = cub.header().bounding_box();
    println!("Bounds: W={:.4} S={:.4} E={:.4} N={:.4}", w, s, e, n);

    if !warnings.is_empty() {
        println!("\n=== Warnings ({}) ===", warnings.len());
        for warning in warnings {
            println!("  {:?}", warning);
        }
    }

    println!("\n=== First 10 Airspaces ===");
    for (i, item) in cub.items().iter().take(10).enumerate() {
        println!("{}. {:?} {:?}", i + 1, item.style(), item.class());
        println!("   Altitude: {} - {} meters ({:?} - {:?})",
            item.min_alt, item.max_alt,
            item.min_alt_style(), item.max_alt_style()
        );

        // Parse points
        let points: Vec<_> = cub.read_points(item)?
            .collect::<Result<Vec<_>, _>>()?;

        println!("   Points: {}", points.len());

        if let Some(first_pt) = points.first() {
            if let Some(name) = &first_pt.name {
                println!("   Name: {}", name);
            }
            if let Some(freq) = first_pt.frequency {
                println!("   Frequency: {} Hz", freq);
            }
        }
    }

    Ok(())
}
```

**Test example:**

```bash
cargo run --example basic tests/fixtures/france_2024.07.02.cub
```

**Commit message:** `Add README and basic example`

---

### Phase 5: Polish & Finalize

#### Task 5.1: Add missing dependencies to Cargo.toml

**Files to modify:** `Cargo.toml`

**Objective:** Ensure all dependencies are properly declared.

**Update `Cargo.toml`:**

```toml
[package]
name = "seeyou-cub"
version = "0.0.0"
edition = "2024"

[dependencies]
encoding_rs = "0.8"
jiff = { version = "0.1", optional = true }
thiserror = "2"

[features]
default = []
datetime = ["jiff"]

[dev-dependencies]
# Add any test-specific deps here if needed
```

**Test all features:**

```bash
cargo test
cargo test --features datetime
cargo test --all-features
```

**Commit message:** `Update Cargo.toml with dependencies`

---

#### Task 5.2: Clippy and formatting

**Objective:** Clean up code quality issues.

**Run checks:**

```bash
# Format code
cargo fmt

# Run Clippy
cargo clippy --all-features -- -D warnings

# Fix any issues found
```

**Common clippy fixes:**
- Remove unused imports
- Simplify boolean expressions
- Use `.copied()` instead of `.cloned()` where appropriate
- Add `#[must_use]` to appropriate functions

**Commit message:** `Fix clippy warnings and format code`

---

#### Task 5.3: Add module-level tests for edge cases

**Objective:** Improve test coverage for edge cases.

**Add tests for:**

1. **Empty files / minimal files**
2. **Files with zero items**
3. **Items with no points**
4. **Maximum values** (largest coordinates, dates, etc.)
5. **String edge cases** (empty strings, null-padded, CP1252 characters)
6. **Point sequences with only origin updates**
7. **All optional data types**

**Example additional test in `src/read/point.rs`:**

```rust
#[test]
fn parse_point_with_all_attributes() {
    // Build point with name, frequency, and all optional data types
    // Test that all are parsed correctly
}

#[test]
fn parse_point_sequence_with_origin_updates() {
    // Build sequence with multiple 0x81 origin updates
    // Verify coordinates calculated correctly
}
```

**Commit message:** `Add edge case tests for point parsing`

---

#### Task 5.4: Final integration test review

**Objective:** Ensure real-world file parses completely and correctly.

**Create comprehensive test in `tests/reader_test.rs`:**

```rust
#[test]
fn comprehensive_france_parse() {
    let file = File::open("tests/fixtures/france_2024.07.02.cub").unwrap();
    let (mut cub, warnings) = parse(file).unwrap();

    // Statistics
    let mut total_points = 0;
    let mut items_with_names = 0;
    let mut items_with_freq = 0;

    let mut style_counts = std::collections::HashMap::new();

    for item in cub.items() {
        *style_counts.entry(item.style()).or_insert(0) += 1;

        let points: Vec<_> = cub.read_points(item)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        total_points += points.len();

        if let Some(first) = points.first() {
            if first.name.is_some() {
                items_with_names += 1;
            }
            if first.frequency.is_some() {
                items_with_freq += 1;
            }
        }
    }

    println!("\n=== Comprehensive Parse Statistics ===");
    println!("Total items: {}", cub.items().len());
    println!("Total points: {}", total_points);
    println!("Items with names: {}", items_with_names);
    println!("Items with frequencies: {}", items_with_freq);
    println!("Warnings: {}", warnings.len());
    println!("\nAirspace types:");
    for (style, count) in style_counts.iter() {
        println!("  {:?}: {}", style, count);
    }

    // Assertions
    assert!(cub.items().len() > 100, "France should have many airspaces");
    assert!(total_points > 1000, "Should have substantial geometry");
}
```

**Test command:** `cargo test --test reader_test -- --nocapture`

**Review output and ensure:**
- No panics or unexpected errors
- Statistics look reasonable
- Warnings are expected/acceptable

**Commit message:** `Add comprehensive integration test`

---

## Testing Strategy

### Unit Tests
- Test each enum conversion function
- Test I/O helpers with both byte orders
- Test bit extraction methods
- Test time decoding

### Integration Tests
- Parse complete real-world file
- Verify statistics (item count, point counts)
- Test reading points for multiple items
- Verify warnings are collected correctly

### Test Data
- Use provided `france_2024.07.02.cub` fixture
- Create minimal synthetic CUB files for specific edge cases

### Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test types::enums
cargo test read::header

# Integration tests
cargo test --test reader_test

# With datetime feature
cargo test --features datetime

# Show output
cargo test -- --nocapture
```

## Implementation Order Summary

1. **Foundation** (Tasks 1.1-1.4): Error types, enums, I/O helpers
2. **Data Structures** (Tasks 2.1-2.4): Header, Item, Point types, CubFile
3. **Parsing** (Tasks 3.1-3.4): Parse header, items, points, main function
4. **Integration** (Tasks 4.1-4.4): Real file tests, bug fixes, API, docs
5. **Polish** (Tasks 5.1-5.4): Dependencies, clippy, edge cases, final review

## Commit Strategy

- Commit after each task completion
- Use clear, descriptive commit messages
- Follow TDD: write tests before/during implementation
- Keep commits focused and atomic

## Definition of Done

For each task:
- [ ] Implementation complete
- [ ] Tests written and passing
- [ ] Documentation added
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings
- [ ] Committed with appropriate message

For the entire project:
- [ ] All unit tests pass
- [ ] Integration test with France fixture passes
- [ ] Documentation complete and clear
- [ ] README with examples
- [ ] Example program runs successfully
- [ ] Both LE and BE support tested
- [ ] Datetime feature tested
- [ ] No outstanding TODOs in code

## Progress Tracking

### Phase 1: Foundation ✅ COMPLETED

- [x] Task 1.1: Error and Warning types - DONE
- [x] Task 1.2: ByteOrder and basic enums - DONE
- [x] Task 1.3: Remaining enums (NOTAM, DaysActive, CubDataId) - DONE
- [x] Task 1.4: I/O helper functions - DONE

**Status:** All 13 tests passing, code compiles successfully.

### Phase 2: Data Structures ✅ COMPLETED

- [x] Task 2.1: Header struct - DONE
- [x] Task 2.2: Item struct with bit-packed accessors - DONE
- [x] Task 2.3: Point types (stub) - DONE
- [x] Task 2.4: CubFile struct - DONE

**Status:** All 16 tests passing with and without datetime feature.

### Phase 3: Parsing Implementation ✅ COMPLETED

- [x] Task 3.1: Parse header - DONE
- [x] Task 3.2: Parse items - DONE
- [x] Task 3.3: Point iterator (basic structure) - DONE
- [x] Task 3.4: Main parse() function - DONE

**Status:** All 22 tests passing.

### Phase 4: Integration & Testing ✅ COMPLETED

- [x] Task 4.1: Integration test with real file - DONE
- [x] Task 4.2: Fix bugs and add assertions - DONE
- [x] Task 4.3: Public API and documentation - DONE
- [x] Task 4.4: Add README and examples - DONE

**Status:** All 25 tests passing (22 unit + 3 integration), 2 doctests passing, working example with France fixture.

### Phase 5: Polish & Finalize - NEXT

- [ ] Task 5.1: Add missing dependencies to Cargo.toml
- [ ] Task 5.2: Clippy and formatting
- [ ] Task 5.3: Add module-level tests for edge cases
- [ ] Task 5.4: Final integration test review

## Notes for Implementer

### File Format Gotchas

1. **Byte Order**: Remember that PcByteOrder affects integers but NOT floats
2. **String Encoding**: Always try UTF-8 first, fall back to CP1252 silently
3. **Point Parsing**: Stateful! Origin accumulates across 0x81 records
4. **Attributes**: Can be on any point, usually first; clear after use
5. **Time Encoding**: Complex bit-packing, follow spec algorithm exactly
6. **Default Values**: `active_time` defaults to `0x3FFFFFF`, check for this

### Design Principles Applied

- **YAGNI**: No speculative features, only what's in the spec
- **TDD**: Tests guide implementation, catch regressions
- **DRY**: I/O helpers reused, no code duplication
- **Lenient Parsing**: Collect warnings, continue when possible
- **Lazy Loading**: Parse geometry only when needed

### Performance Considerations

- Lazy point parsing keeps memory proportional to airspace count, not point count
- Use `Vec::with_capacity` when size is known
- Avoid unnecessary allocations in hot loops
- Reader seeking is acceptable (files stored on disk anyway)

### Error Handling Philosophy

**Return Error for:**
- Invalid magic bytes (not a CUB file)
- Encryption (unsupported)
- I/O failures
- File truncation

**Collect Warning for:**
- Unknown enum values (use default)
- Oversized structures (skip padding)
- Unknown optional flags (skip)
- String encoding fallback (silent, per spec)

Good luck with the implementation! Follow the tasks in order, test thoroughly, and commit frequently.
