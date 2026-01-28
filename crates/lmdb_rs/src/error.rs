use crate::arch::DynArch;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// Invalid magic number in header
    InvalidMagic { expected: u32, found: u32 },
    /// Unsupported data format version
    UnsupportedVersion { version: u32 },
    /// Data too short for expected structure
    UnexpectedEof { expected: usize, available: usize },
    /// Page number out of bounds
    InvalidPageNumber { pgno: u64, max: u64 },
    /// Invalid page type for operation
    InvalidPageType { expected: u16, found: u16 },
    /// Key not found in database
    KeyNotFound,
    /// Database not found
    DatabaseNotFound { name: Option<String> },
    /// Invalid UTF-8 in string key
    InvalidUtf8,
    /// Page size mismatch or invalid
    InvalidPageSize { size: u32 },
    /// Corrupted B+tree structure
    CorruptedTree { message: &'static str },
    /// Architecture mismatch
    ArchMismatch { expected: DynArch, found: DynArch },
    /// Codec error during encode/decode
    Codec(String),
    /// Write buffer too small
    BufferTooSmall { needed: usize, available: usize },
    /// Page is full and cannot accept more entries
    PageFull,
    /// IO Error
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidMagic { expected, found } => {
                write!(f, "Invalid magic number: expected {:#x}, found {:#x}", expected, found)
            }
            Error::UnsupportedVersion { version } => {
                write!(f, "Unsupported data version: {}", version)
            }
            Error::UnexpectedEof { expected, available } => {
                write!(f, "Unexpected EOF: expected {} bytes, available {}", expected, available)
            }
            Error::InvalidPageNumber { pgno, max } => {
                write!(f, "Invalid page number {}: max is {}", pgno, max)
            }
            Error::InvalidPageType { expected, found } => {
                write!(f, "Invalid page type: expected {:#x}, found {:#x}", expected, found)
            }
            Error::KeyNotFound => write!(f, "Key not found"),
            Error::DatabaseNotFound { name } => {
                write!(f, "Database not found: {:?}", name)
            }
            Error::InvalidUtf8 => write!(f, "Invalid UTF-8 sequence"),
            Error::InvalidPageSize { size } => write!(f, "Invalid page size: {}", size),
            Error::CorruptedTree { message } => write!(f, "Corrupted tree: {}", message),
            Error::ArchMismatch { expected, found } => {
                write!(f, "Architecture mismatch: expected {:?}, found {:?}", expected, found)
            }
            Error::Codec(msg) => write!(f, "Codec error: {}", msg),
            Error::BufferTooSmall { needed, available } => {
                write!(f, "Buffer too small: needed {}, available {}", needed, available)
            }

            Error::PageFull => write!(f, "Page full"),
            Error::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
