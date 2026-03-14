use crate::arch::DynArch;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum PageError {
    /// Slice was shorter than the structure being parsed.
    #[snafu(display("Unexpected EOF parsing page: expected {expected} bytes, available {available}"))]
    UnexpectedEof { expected: usize, available: usize },

    /// Page flags indicate the wrong page type for this operation.
    #[snafu(display("Invalid page type: expected {expected:#x}, found {found:#x}"))]
    InvalidPageType { expected: u16, found: u16 },

    /// B-tree structure is internally inconsistent.
    #[snafu(display("Corrupted B-tree: {message}"))]
    CorruptedTree { message: &'static str },

    /// Meta-page magic number did not match `MDB_MAGIC`.
    #[snafu(display("Invalid magic number: expected {expected:#x}, found {found:#x}"))]
    InvalidMagic { expected: u32, found: u32 },

    /// Meta-page data format version is not supported.
    #[snafu(display("Unsupported data version: {version}"))]
    UnsupportedVersion { version: u32 },

    /// Page size field in the meta page is invalid.
    #[snafu(display("Invalid page size: {size}"))]
    InvalidPageSize { size: u32 },

    /// File was created with a different pointer-size architecture.
    #[snafu(display("Architecture mismatch: expected {expected:?}, found {found:?}"))]
    ArchMismatch { expected: DynArch, found: DynArch },
}

pub type PageResult<T> = std::result::Result<T, PageError>;

pub mod branch;
pub mod generic;
pub mod header;
pub mod leaf;
pub mod meta;
pub mod node;
pub mod overflow;
