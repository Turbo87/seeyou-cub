use crate::error::{Error, Result, Warning};
use crate::read::io::*;
use crate::types::{ByteOrder, Header};
use std::io::{Read, Seek, SeekFrom};

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
        reader.seek(SeekFrom::Start(133))?; // Skip back to after PcByteOrder
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn minimal_header_bytes(byte_order: ByteOrder, encrypted: bool) -> Vec<u8> {
        let mut bytes = vec![0u8; 210];

        // Magic bytes (LE)
        bytes[0..4].copy_from_slice(&0x425543C2u32.to_le_bytes());

        // Title (offset 4)
        bytes[4..15].copy_from_slice(b"Test Header");

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
            bytes[offset..offset + 4].copy_from_slice(&val_bytes);
        };

        write_i32(&mut bytes, 154, 42); // size_of_item
        write_i32(&mut bytes, 158, 5); // size_of_point
        write_i32(&mut bytes, 162, 0); // hdr_items
        write_i32(&mut bytes, 166, 100); // max_pts

        // Floats (always LE)
        bytes[170..174].copy_from_slice(&0.0f32.to_le_bytes()); // left
        bytes[174..178].copy_from_slice(&1.0f32.to_le_bytes()); // top
        bytes[178..182].copy_from_slice(&1.0f32.to_le_bytes()); // right
        bytes[182..186].copy_from_slice(&0.0f32.to_le_bytes()); // bottom
        bytes[186..190].copy_from_slice(&1.0f32.to_le_bytes()); // max_width
        bytes[190..194].copy_from_slice(&1.0f32.to_le_bytes()); // max_height
        bytes[194..198].copy_from_slice(&1.0f32.to_le_bytes()); // lo_la_scale

        write_i32(&mut bytes, 198, 210); // header_offset
        write_i32(&mut bytes, 202, 210); // data_offset
        write_i32(&mut bytes, 206, 0); // alignment

        bytes
    }

    #[test]
    fn parse_valid_le_header() {
        let bytes = minimal_header_bytes(ByteOrder::LE, false);
        let mut cursor = Cursor::new(bytes);
        let (header, warnings) = parse_header(&mut cursor).unwrap();

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
        bytes[0] = 0xFF; // Corrupt magic
        let mut cursor = Cursor::new(bytes);

        match parse_header(&mut cursor) {
            Err(Error::InvalidMagicBytes) => {}
            _ => panic!("Expected InvalidMagicBytes error"),
        }
    }

    #[test]
    fn encrypted_file_error() {
        let bytes = minimal_header_bytes(ByteOrder::LE, true);
        let mut cursor = Cursor::new(bytes);

        match parse_header(&mut cursor) {
            Err(Error::EncryptedFile) => {}
            _ => panic!("Expected EncryptedFile error"),
        }
    }
}
