use crate::arch::DynArch;
use crate::codec::BytesDecode;
use crate::database::Database;
use crate::db_record::DbRecord;
use crate::page::meta::MetaPage;
use crate::txn::{RoTxn, RwTxn, TxnError};
use snafu::{ResultExt, Snafu};
use std::io::Write;

#[derive(Debug, Snafu)]
pub enum EnvError {
    #[snafu(display("Failed to parse meta page during environment bootstrap"))]
    MetaParse { source: crate::page::PageError },

    #[snafu(display("Failed to initialize read transaction"))]
    ReadTransactionStart { source: TxnError },

    #[snafu(display("Failed to initialize write transaction"))]
    WriteTransactionStart { source: TxnError },

    #[snafu(display("Error while opening database '{name:?}'"))]
    DatabaseOpen {
        name: Option<String>,
        source: TxnError,
    },

    #[snafu(display("IO error during environment operation"))]
    Io { source: std::io::Error },
}

pub type EnvResult<T> = std::result::Result<T, EnvError>;

/// Database environment (read-only, from bytes).
pub struct Env<'a> {
    data: &'a [u8],
    page_size: usize,
    arch: DynArch,
    meta: MetaPage<'a>,
}

impl<'a> Env<'a> {
    /// Open environment from bytes, auto-detecting architecture.
    pub fn new(data: &'a [u8]) -> EnvResult<Self> {
        // 1. Read Meta 0 to determine Page Size
        // We must successfully read Meta 0 to bootstrap.
        let (mut active_meta, mut active_arch): (MetaPage, DynArch) =
            MetaPage::parse(&data[0..]).context(MetaParseSnafu)?;

        let page_size = active_meta.page_size() as usize;
        let meta1_offset = page_size;

        // 2. Try to read Meta 1
        // We slice the data starting at meta1_offset to effectively pass "offset" to parse
        if data.len() >= meta1_offset + 4096
            && let Ok((meta1, arch1)) = MetaPage::parse(&data[meta1_offset..])
        {
            // 3. Compare TxnID
            if meta1.txn_id() > active_meta.txn_id() {
                active_meta = meta1;
                active_arch = arch1;
            }
        }

        Ok(Self {
            arch: active_arch,
            page_size,
            data,
            meta: active_meta,
        })
    }

    /// Get detected architecture.
    pub fn arch(&self) -> DynArch {
        self.arch
    }

    /// Get page size.
    pub fn page_size(&self) -> usize {
        self.page_size
    }

    /// Get raw data.
    pub fn raw_data(&self) -> &'a [u8] {
        self.data
    }

    /// Get the Main DB root page.
    pub fn main_db_root(&self) -> u64 {
        self.meta.main_db().root_page
    }

    /// Get the Main DB record.
    pub fn main_db(&self) -> DbRecord {
        self.meta.main_db()
    }

    /// Create read transaction.
    pub fn read_txn(&'a self) -> EnvResult<RoTxn<'a>> {
        RoTxn::new(self).context(ReadTransactionStartSnafu)
    }

    /// Open a typed database.
    pub fn open_database<K, V>(
        &self,
        txn: &RoTxn<'a>,
        name: Option<&str>,
    ) -> EnvResult<Option<Database<'a, K, V>>>
    where
        K: BytesDecode<'a>,
        V: BytesDecode<'a>,
    {
        txn.open_database(name).with_context(|_| DatabaseOpenSnafu {
            name: name.map(|s| s.to_string()),
        })
    }
}

/// Entry point for creating a new LMDB file.
/// Wraps a writer (File, Vec<u8>, etc).
///
/// This builder acts as the "Environment" for write operations.
/// It accumulates data via transactions and writes the final LMDB structure
/// to the underlying writer in one go upon commit.
pub struct EnvWrite<W: Write> {
    pub(crate) writer: W,
    pub(crate) arch: DynArch,
    pub(crate) page_size: usize,
}

impl<W: Write> EnvWrite<W> {
    /// Create a new environment writer.
    ///
    /// # Arguments
    /// * `writer` - The sink to write the final DB bytes to.
    /// * `arch` - Target architecture (Arch64 is standard).
    pub fn new(writer: W, arch: DynArch) -> Self {
        Self {
            writer,
            arch,
            page_size: 4096, // Default standard page size
        }
    }

    /// Create a write transaction.
    ///
    /// Note: Since this library implements a "bulk loader" logic,
    /// effectively only one write transaction is supported per file creation.
    /// The transaction will buffer all writes in memory until `commit()` is called.
    pub fn write_txn(&mut self) -> EnvResult<RwTxn<'_, W>> {
        Ok(RwTxn::new(self))
    }
}
