use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChestError {
    #[error("Incomplete shelf_render_items array")]
    IncompleteShelfRenderItems,
    #[error("Incomplete shelf_item_data_bs array")]
    IncompleteItemDataBs,
    #[error("Num slots mismatch: expected {0}, got {1} for type {2}")]
    NumSlotsMismatch(usize, usize, &'static str),
    #[error("Get save_item_slot when portal chest shouldn't have one")]
    PortalChestHaveSlots,
    #[error("No save_item_slot when chest type {0} should have one")]
    NoSaveItemSlot(&'static str),
}

#[derive(Debug, Error)]
pub enum BhError {
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
    #[error("Invalid dynamic object ID: {0}")]
    InvalidDynamicOjectId(u16),
    #[error("Invalid item type ID: {0}")]
    InvalidItemTypeId(u16),
    #[error("Invalid color ID: {0}")]
    InvalidColorId(u8),
    #[error("Invalid chunk size: {0}")]
    InvalidChunkSize(#[from] std::array::TryFromSliceError),
    #[error("Missing key: {0}")]
    MissingKey(&'static str),
    #[error("Lmdb error: {0}")]
    LmdbError(#[from] lmdb_rs::error::Error),
    #[error("Failed to parse chest")]
    ChestError(#[from] ChestError),
}

pub type BhResult<T> = Result<T, BhError>;
