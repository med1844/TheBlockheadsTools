use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum BuildError {
    /// Page is full and cannot take more entries.
    #[snafu(display("Page is full"))]
    PageFull,

    /// Buffer is too small for the operation.
    #[snafu(display("Buffer too small: expected {expected}, available {available}"))]
    BufferTooSmall { expected: usize, available: usize },

    /// I/O error during build.
    #[snafu(display("I/O error"))]
    Io { source: std::io::Error },
}

pub type BuildResult<T> = std::result::Result<T, BuildError>;

pub mod btree;
pub mod database;
pub mod meta;
pub mod overflow;
pub mod page;
