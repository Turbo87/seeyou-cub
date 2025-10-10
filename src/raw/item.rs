use crate::error::Result;
use crate::raw::Header;
use crate::utils::io::{read_f32_le, read_i16, read_i32, read_u8, read_u32, read_u64};
use crate::{
    AltStyle, CubClass, CubStyle, DateTime, DaysActive, ExtendedType, NotamCodes, NotamScope,
    NotamTraffic, NotamType,
};
use std::io::{Cursor, Read};

/// Airspace item (26 bytes minimum, may be larger per `Header::size_of_item`)
///
/// Represents a single airspace with its bounding box, altitude limits,
/// classification, and metadata. Contains bit-packed fields that are
/// decoded through accessor methods.
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
    /// Read a single airspace item from current position
    ///
    /// Reads exactly `header.size_of_item` bytes and parses them into an `Item`.
    ///
    /// # Arguments
    ///
    /// * `reader` - Must be positioned at the start of an item
    /// * `header` - Header containing byte order and item size
    ///
    /// # Returns
    ///
    /// The parsed `Item` or an error if reading/parsing fails
    pub fn read<R: Read>(reader: &mut R, header: &Header) -> Result<Self> {
        // Create 43-byte zero-filled buffer and read `size_of_item` bytes into it
        // Per spec: remaining bytes should be set to `0` if `size_of_item < 43`
        let mut item_buffer = [0u8; 43];
        let bytes_to_read = std::cmp::min(header.size_of_item as usize, 43);

        reader.read_exact(&mut item_buffer[..bytes_to_read])?;

        // If size_of_item > 43, read and discard the extra bytes
        if header.size_of_item > 43 {
            let extra_bytes = (header.size_of_item - 43) as usize;
            let mut discard = vec![0u8; extra_bytes];
            reader.read_exact(&mut discard)?;
        }

        // Parse from the buffer using a cursor (full 43-byte zero-padded buffer)
        let mut cursor = Cursor::new(&item_buffer);

        // Read bounding box
        let left = read_f32_le(&mut cursor)?;
        let top = read_f32_le(&mut cursor)?;
        let right = read_f32_le(&mut cursor)?;
        let bottom = read_f32_le(&mut cursor)?;

        // Read bit-packed fields
        let byte_order = header.byte_order();
        let type_byte = read_u8(&mut cursor)?;
        let alt_style_byte = read_u8(&mut cursor)?;
        let min_alt = read_i16(&mut cursor, byte_order)?;
        let max_alt = read_i16(&mut cursor, byte_order)?;
        let points_offset = read_i32(&mut cursor, byte_order)?;
        let time_out = read_i32(&mut cursor, byte_order)?;
        let extra_data = read_u32(&mut cursor, byte_order)?;

        let mut active_time = read_u64(&mut cursor, byte_order)?;
        if active_time == 0 {
            active_time = 0x3FFFFFF;
        }

        let extended_type_byte = read_u8(&mut cursor)?;

        Ok(Self {
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
        })
    }

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
        ExtendedType::from_byte(self.extended_type_byte)
    }

    /// Get active days flags
    pub fn days_active(&self) -> DaysActive {
        let bits = ((self.active_time >> 52) & 0xFFF) as u16;
        DaysActive::from_bits(bits)
    }

    /// Get start date as DateTime
    pub fn start_date(&self) -> Option<DateTime> {
        let value = ((self.active_time >> 26) & 0x3FFFFFF) as u32;
        if value == 0 {
            None
        } else {
            Some(decode_notam_time(value))
        }
    }

    /// Get end date as DateTime
    pub fn end_date(&self) -> Option<DateTime> {
        let value = (self.active_time & 0x3FFFFFF) as u32;
        if value == 0x3FFFFFF {
            None
        } else {
            Some(decode_notam_time(value))
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

/// Decode NOTAM time from encoded minutes to DateTime
pub fn decode_notam_time(encoded: u32) -> DateTime {
    let mut time = encoded;
    let minute = (time % 60) as u8;
    time /= 60;
    let hour = (time % 24) as u8;
    time /= 24;
    let day = (time % 31) as u8 + 1;
    time /= 31;
    let month = (time % 12) as u8 + 1;
    time /= 12;
    let year = time + 2000;

    DateTime {
        day,
        month,
        year,
        hour,
        minute,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn item_style_and_class() {
        let item = Item {
            type_byte: 0b01000100, // Class D (0100) + Style 0x04 (DA)
            // ... other fields with defaults
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            alt_style_byte: 0,
            min_alt: 0,
            max_alt: 0,
            points_offset: 0,
            time_out: 0,
            extra_data: 0,
            active_time: 0,
            extended_type_byte: 0,
        };

        assert_eq!(item.style(), CubStyle::DangerArea);
        assert_eq!(item.class(), CubClass::ClassD);
    }

    #[test]
    fn item_alt_styles() {
        let item = Item {
            alt_style_byte: 0x32, // Max=3 (FL), Min=2 (MSL)
            // ... other fields
            type_byte: 0,
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            min_alt: 0,
            max_alt: 0,
            points_offset: 0,
            time_out: 0,
            extra_data: 0,
            active_time: 0,
            extended_type_byte: 0,
        };

        assert_eq!(item.min_alt_style(), AltStyle::MeanSeaLevel);
        assert_eq!(item.max_alt_style(), AltStyle::FlightLevel);
    }

    #[test]
    fn decode_notam_time_example() {
        // Example: 2024-07-15 14:30
        // Manually calculated encoded value
        let encoded = 30 + 60 * (14 + 24 * (14 + 31 * (6 + 12 * 24)));
        let dt = decode_notam_time(encoded);
        assert_eq!(dt.year, 2024);
        assert_eq!(dt.month, 7);
        assert_eq!(dt.day, 15);
        assert_eq!(dt.hour, 14);
        assert_eq!(dt.minute, 30);
    }

    #[test]
    fn read_item_from_fixture() {
        let mut file =
            File::open("tests/fixtures/france_2024.07.02.cub").expect("Failed to open fixture");

        // Read header first to get item offset
        let header = Header::read(&mut file).unwrap();

        // Seek to first item
        file.seek(SeekFrom::Start(header.header_offset as u64))
            .unwrap();

        // Read item
        let item = Item::read(&mut file, &header).expect("Failed to read item");
        insta::assert_debug_snapshot!(item);
    }
}
