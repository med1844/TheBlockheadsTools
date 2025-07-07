use thiserror::Error;

#[derive(Debug, Error)]
pub enum BhError {
    #[error("Database error: {0}")]
    DbError(#[from] heed::Error),
    #[error("Plist deserialization error: {0}")]
    PlistError(#[from] plist::Error),
    #[error("Gzip I/O error: {0}")]
    GzipError(#[from] std::io::Error),
    #[error("Coord out of bound error: input {input} >= limit {limit}")]
    CoordError { input: u64, limit: u64 },
    #[error("Parse error: {0}")]
    ParseError(String), // New error variant for parsing issues
    #[error("Invalid block ID: {0}")]
    InvalidBlockIdError(u8),
    #[error("Invalid block content ID: {0}")]
    InvalidBlockContentIdError(u8),
    #[error("Missing key: {0}")]
    MissingKey(&'static str),
}

pub type BhResult<T> = Result<T, BhError>;
