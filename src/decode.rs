use std::borrow::Cow;

/// Decode raw bytes to string
///
/// Attempts UTF-8 decoding first, falling back to Windows-1252 (CP1252) if UTF-8 fails.
/// This matches the CUB file format specification which states:
/// "String encoding: UTF-8, use Extended ASCII if string contains incorrect utf-8 sequence"
///
/// # Arguments
///
/// * `bytes` - Raw bytes from CUB file (e.g., airspace name, ICAO code)
///
/// # Returns
///
/// Decoded string (always succeeds with some valid string)
pub fn decode_string(bytes: &[u8]) -> Cow<'_, str> {
    match str::from_utf8(bytes) {
        Ok(s) => s.into(),
        Err(_) => encoding_rs::WINDOWS_1252.decode(bytes).0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_utf8_string() {
        let bytes = b"Hello World";
        assert_eq!(decode_string(bytes), "Hello World");
    }

    #[test]
    fn decode_utf8_with_special_chars() {
        let bytes = "Zürich".as_bytes();
        assert_eq!(decode_string(bytes), "Zürich");
    }

    #[test]
    fn decode_cp1252_fallback() {
        // CP1252 character é (0xE9) - not valid UTF-8 on its own
        let bytes = vec![0xE9];
        assert_eq!(decode_string(&bytes), "é");
    }

    #[test]
    fn decode_empty_string() {
        let bytes = b"";
        assert_eq!(decode_string(bytes), "");
    }

    #[test]
    fn decode_mixed_extended_ascii() {
        // Mix of ASCII and CP1252 characters
        let bytes = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0xE9]; // "Hello é"
        assert_eq!(decode_string(&bytes), "Hello é");
    }
}
