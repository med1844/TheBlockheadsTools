pub const MDB_MAGIC: u32 = 0xBEEFC0DE;
pub const MDB_DATA_VERSION: u32 = 1;

/// Default map size matching C LMDB's default (100MB).
/// This is stored in the meta page as the maximum environment size.
pub const DEFAULT_MAPSIZE: u64 = 100 * 1024 * 1024;

// Page flags
pub const P_BRANCH: u16 = 0x01;
pub const P_LEAF: u16 = 0x02;
pub const P_OVERFLOW: u16 = 0x04;
pub const P_META: u16 = 0x08;
pub const P_LEAF2: u16 = 0x20;
pub const P_SUBP: u16 = 0x40;

// Node flags
pub const F_BIGDATA: u16 = 0x01;
pub const F_SUBDATA: u16 = 0x02;
pub const F_DUPDATA: u16 = 0x04;

// Database flags
pub const MDB_INTEGERKEY: u16 = 0x08;

// Sizes

pub const NODE_HEADER_SIZE: usize = 8;
pub const MIN_PAGE_SIZE: usize = 512;
pub const MAX_PAGE_SIZE: usize = 65536;

// Special page numbers
pub const FREE_DBI: usize = 0;
pub const MAIN_DBI: usize = 1;
