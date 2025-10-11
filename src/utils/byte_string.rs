use std::borrow::Cow;
use std::fmt;
use std::io::{Read, Write};

/// Wrapper around `Vec<u8>` that provides human-readable debug output
///
/// This type is used for string fields in the low-level API where the encoding
/// may be UTF-8 or Extended ASCII (CP1252). The debug output attempts UTF-8
/// decoding and falls back to showing hex bytes for invalid sequences.
#[derive(Clone, PartialEq, Eq)]
pub struct ByteString(Vec<u8>);

impl ByteString {
    /// Create a new `ByteString` from a byte vector
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Read `len` bytes from reader and return as `ByteString`
    pub fn read<R: Read>(reader: &mut R, len: usize) -> std::io::Result<Self> {
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;
        Ok(Self(buf))
    }

    /// Write the bytes to the writer and return number of bytes written
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        writer.write_all(&self.0)?;
        Ok(self.0.len())
    }

    /// Get a reference to the underlying bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert into the underlying byte vector
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    /// Decode raw bytes to string
    ///
    /// Attempts UTF-8 decoding first, falling back to Windows-1252 (CP1252) if UTF-8 fails.
    /// This matches the CUB file format specification which states:
    /// "String encoding: UTF-8, use Extended ASCII if string contains incorrect utf-8 sequence"
    ///
    /// # Returns
    ///
    /// Decoded string (always succeeds with some valid string)
    pub fn decode(&self) -> Cow<'_, str> {
        match str::from_utf8(&self.0) {
            Ok(s) => s.into(),
            Err(_) => encoding_rs::WINDOWS_1252.decode(&self.0).0,
        }
    }
}

impl From<Vec<u8>> for ByteString {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<String> for ByteString {
    fn from(str: String) -> Self {
        Self(str.into_bytes())
    }
}

impl AsRef<[u8]> for ByteString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for ByteString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match std::str::from_utf8(&self.0) {
            Ok(s) => write!(f, "{:?}", s),
            Err(_) => {
                // Show as hex if not valid UTF-8
                write!(f, "b\"")?;
                for &byte in &self.0 {
                    if byte.is_ascii_graphic() || byte == b' ' {
                        write!(f, "{}", byte as char)?;
                    } else {
                        write!(f, "\\x{:02x}", byte)?;
                    }
                }
                write!(f, "\"")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_from_reader() {
        let data = b"Hello World";
        let mut cursor = Cursor::new(data);
        let bs = ByteString::read(&mut cursor, data.len()).unwrap();
        assert_eq!(bs.as_bytes(), b"Hello World");
    }

    #[test]
    fn read_zero_length() {
        let data = b"Hello";
        let mut cursor = Cursor::new(data);
        let bs = ByteString::read(&mut cursor, 0).unwrap();
        assert_eq!(bs.as_bytes(), b"");
    }

    #[test]
    fn debug_valid_utf8() {
        let bs = ByteString::new(b"Hello World".to_vec());
        assert_eq!(format!("{:?}", bs), "\"Hello World\"");
    }

    #[test]
    fn debug_utf8_with_special_chars() {
        let bs = ByteString::new("Zürich".as_bytes().to_vec());
        assert_eq!(format!("{:?}", bs), "\"Zürich\"");
    }

    #[test]
    fn debug_invalid_utf8() {
        // CP1252 character é (0xE9) - not valid UTF-8 on its own
        let bs = ByteString::new(vec![0xE9]);
        assert_eq!(format!("{:?}", bs), "b\"\\xe9\"");
    }

    #[test]
    fn debug_mixed_extended_ascii() {
        // Mix of ASCII and CP1252 characters: "Hello é"
        let bs = ByteString::new(vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0xE9]);
        assert_eq!(format!("{:?}", bs), "b\"Hello \\xe9\"");
    }

    #[test]
    fn debug_empty() {
        let bs = ByteString::new(vec![]);
        assert_eq!(format!("{:?}", bs), "\"\"");
    }

    #[test]
    fn as_bytes() {
        let bs = ByteString::new(b"test".to_vec());
        assert_eq!(bs.as_bytes(), b"test");
    }

    #[test]
    fn into_bytes() {
        let bs = ByteString::new(b"test".to_vec());
        assert_eq!(bs.into_bytes(), b"test");
    }

    #[test]
    fn from_vec() {
        let bs = ByteString::from(b"test".to_vec());
        assert_eq!(bs.as_bytes(), b"test");
    }

    #[test]
    fn decode_utf8_string() {
        let bs = ByteString::new(b"Hello World".to_vec());
        assert_eq!(bs.decode(), "Hello World");
    }

    #[test]
    fn decode_utf8_with_special_chars() {
        let bs = ByteString::new("Zürich".as_bytes().to_vec());
        assert_eq!(bs.decode(), "Zürich");
    }

    #[test]
    fn decode_cp1252_fallback() {
        // CP1252 character é (0xE9) - not valid UTF-8 on its own
        let bs = ByteString::new(vec![0xE9]);
        assert_eq!(bs.decode(), "é");
    }

    #[test]
    fn decode_empty_string() {
        let bs = ByteString::new(vec![]);
        assert_eq!(bs.decode(), "");
    }

    #[test]
    fn decode_mixed_extended_ascii() {
        // Mix of ASCII and CP1252 characters
        let bs = ByteString::new(vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0xE9]); // "Hello é"
        assert_eq!(bs.decode(), "Hello é");
    }

    #[test]
    fn write_to_writer() {
        let bs = ByteString::new(b"Hello World".to_vec());
        let mut buf = Vec::new();
        let written = bs.write(&mut buf).unwrap();
        assert_eq!(written, 11);
        assert_eq!(buf, b"Hello World");
    }

    #[test]
    fn write_empty() {
        let bs = ByteString::new(vec![]);
        let mut buf = Vec::new();
        let written = bs.write(&mut buf).unwrap();
        assert_eq!(written, 0);
        assert_eq!(buf, b"");
    }
}
