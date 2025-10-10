use crate::types::ByteOrder;
use std::io::{Read, Result, Write};

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

/// Write u8
pub fn write_u8<W: Write>(writer: &mut W, value: u8) -> Result<()> {
    writer.write_all(&[value])
}

/// Write i16 with specified byte order
pub fn write_i16<W: Write>(writer: &mut W, value: i16, order: ByteOrder) -> Result<()> {
    let buf = match order {
        ByteOrder::LE => value.to_le_bytes(),
        ByteOrder::BE => value.to_be_bytes(),
    };
    writer.write_all(&buf)
}

/// Write u16 with specified byte order
pub fn write_u16<W: Write>(writer: &mut W, value: u16, order: ByteOrder) -> Result<()> {
    let buf = match order {
        ByteOrder::LE => value.to_le_bytes(),
        ByteOrder::BE => value.to_be_bytes(),
    };
    writer.write_all(&buf)
}

/// Write i32 with specified byte order
pub fn write_i32<W: Write>(writer: &mut W, value: i32, order: ByteOrder) -> Result<()> {
    let buf = match order {
        ByteOrder::LE => value.to_le_bytes(),
        ByteOrder::BE => value.to_be_bytes(),
    };
    writer.write_all(&buf)
}

/// Write u32 with specified byte order
pub fn write_u32<W: Write>(writer: &mut W, value: u32, order: ByteOrder) -> Result<()> {
    let buf = match order {
        ByteOrder::LE => value.to_le_bytes(),
        ByteOrder::BE => value.to_be_bytes(),
    };
    writer.write_all(&buf)
}

/// Write u64 with specified byte order
pub fn write_u64<W: Write>(writer: &mut W, value: u64, order: ByteOrder) -> Result<()> {
    let buf = match order {
        ByteOrder::LE => value.to_le_bytes(),
        ByteOrder::BE => value.to_be_bytes(),
    };
    writer.write_all(&buf)
}

/// Write f32 (always little-endian per spec)
pub fn write_f32_le<W: Write>(writer: &mut W, value: f32) -> Result<()> {
    writer.write_all(&value.to_le_bytes())
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
    fn test_write_u8() {
        let mut buf = Vec::new();
        write_u8(&mut buf, 0x42).unwrap();
        assert_eq!(buf, vec![0x42]);
    }

    #[test]
    fn test_write_i16_le() {
        let mut buf = Vec::new();
        write_i16(&mut buf, 0x1234, ByteOrder::LE).unwrap();
        assert_eq!(buf, vec![0x34, 0x12]);
    }

    #[test]
    fn test_write_i16_be() {
        let mut buf = Vec::new();
        write_i16(&mut buf, 0x1234, ByteOrder::BE).unwrap();
        assert_eq!(buf, vec![0x12, 0x34]);
    }

    #[test]
    fn test_write_u16_le() {
        let mut buf = Vec::new();
        write_u16(&mut buf, 0x1234, ByteOrder::LE).unwrap();
        assert_eq!(buf, vec![0x34, 0x12]);
    }

    #[test]
    fn test_write_u16_be() {
        let mut buf = Vec::new();
        write_u16(&mut buf, 0x1234, ByteOrder::BE).unwrap();
        assert_eq!(buf, vec![0x12, 0x34]);
    }

    #[test]
    fn test_write_i32_le() {
        let mut buf = Vec::new();
        write_i32(&mut buf, 0x12345678, ByteOrder::LE).unwrap();
        assert_eq!(buf, vec![0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_write_i32_be() {
        let mut buf = Vec::new();
        write_i32(&mut buf, 0x12345678, ByteOrder::BE).unwrap();
        assert_eq!(buf, vec![0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn test_write_u32_le() {
        let mut buf = Vec::new();
        write_u32(&mut buf, 0x12345678, ByteOrder::LE).unwrap();
        assert_eq!(buf, vec![0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_write_u32_be() {
        let mut buf = Vec::new();
        write_u32(&mut buf, 0x12345678, ByteOrder::BE).unwrap();
        assert_eq!(buf, vec![0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn test_write_u64_le() {
        let mut buf = Vec::new();
        write_u64(&mut buf, 0x123456789ABCDEF0, ByteOrder::LE).unwrap();
        assert_eq!(buf, vec![0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_write_u64_be() {
        let mut buf = Vec::new();
        write_u64(&mut buf, 0x123456789ABCDEF0, ByteOrder::BE).unwrap();
        assert_eq!(buf, vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0]);
    }

    #[test]
    fn test_write_f32_le() {
        let value = std::f32::consts::PI;
        let mut buf = Vec::new();
        write_f32_le(&mut buf, value).unwrap();
        assert_eq!(buf, value.to_le_bytes());
    }
}
