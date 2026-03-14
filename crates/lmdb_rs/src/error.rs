//! Error types for LMDB-RS.

pub use crate::build::BuildError;
pub use crate::codec::CodecError;
pub use crate::cursor::CursorError;
pub use crate::database::DatabaseError;
pub use crate::env::EnvError;
pub use crate::page::PageError;
pub use crate::txn::TxnError;

/// Crate-level Result types for each module.
pub type PageResult<T> = crate::page::PageResult<T>;
pub type CursorResult<T> = crate::cursor::CursorResult<T>;
pub type BuildResult<T> = crate::build::BuildResult<T>;
pub type DatabaseResult<T> = crate::database::DatabaseResult<T>;
pub type TxnResult<T> = crate::txn::TxnResult<T>;
pub type EnvResult<T> = crate::env::EnvResult<T>;
