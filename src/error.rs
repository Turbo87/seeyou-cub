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
    OversizedItem { expected: i32, actual: i32 },

    /// Unrecognized optional point flag, skipped
    UnknownPointFlag(u8),

    /// Data appears truncated but parsing continued
    TruncatedData { context: String },
}

pub type Result<T> = std::result::Result<T, Error>;
