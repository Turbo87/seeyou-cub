use crate::Point;

pub type Result<T> = std::result::Result<T, Error>;

/// Unrecoverable parsing errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Invalid magic bytes in header (expected 0x425543C2)")]
    InvalidMagicBytes,

    #[error("Encrypted CUB files not supported (encryption format undocumented)")]
    EncryptedFile,

    #[error("Unexpected point flag: 0x{0:02X}")]
    UnexpectedPointFlag(u8),

    #[error("SizeOfItem is smaller than the minimum structure size")]
    UndersizedItems { size_of_item: i32 },

    #[error("SizeOfPoint is smaller than the minimum structure size")]
    UndersizedPoints { size_of_point: i32 },

    #[error("Coordinate out of valid range (lat: {}, lon: {})", .point.lat, .point.lon)]
    CoordinateOutOfRange { point: Point },
}

/// Non-fatal issues encountered during lenient parsing
#[derive(Debug, Clone, PartialEq)]
pub enum Warning {
    /// SizeOfItem is larger than the expected structure size
    OversizedItems { size_of_item: i32 },

    /// SizeOfPoint is larger than the expected structure size
    OversizedPoints { size_of_point: i32 },

    /// Unrecognized optional point flag, skipped
    UnknownPointFlag(u8),
}
