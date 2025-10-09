use crate::error::Result;
use crate::types::ByteOrder;
use std::io::Read;

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
            let (decoded, _encoding, _had_errors) = encoding_rs::WINDOWS_1252.decode(&bytes);
            Ok(decoded.into_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_i16_le() {
        let data = vec![0x34, 0x12];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_i16(&mut cursor, ByteOrder::LE).unwrap(), 0x1234);
    }

    #[test]
    fn test_read_i16_be() {
        let data = vec![0x12, 0x34];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_i16(&mut cursor, ByteOrder::BE).unwrap(), 0x1234);
    }

    #[test]
    fn test_read_i32_le() {
        let data = vec![0x78, 0x56, 0x34, 0x12];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_i32(&mut cursor, ByteOrder::LE).unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_f32_le() {
        let value = std::f32::consts::PI;
        let bytes = value.to_le_bytes();
        let mut cursor = Cursor::new(bytes);
        let result = read_f32_le(&mut cursor).unwrap();
        assert!((result - value).abs() < 0.0001);
    }

    #[test]
    fn test_read_string_utf8() {
        let data = b"Hello";
        let mut cursor = Cursor::new(data);
        assert_eq!(read_string(&mut cursor, 5).unwrap(), "Hello");
    }

    #[test]
    fn test_read_string_cp1252_fallback() {
        // CP1252 character (not valid UTF-8)
        let data = vec![0xE9]; // é in CP1252
        let mut cursor = Cursor::new(data);
        assert_eq!(read_string(&mut cursor, 1).unwrap(), "é");
    }
}
