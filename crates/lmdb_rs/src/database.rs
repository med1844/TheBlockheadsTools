use crate::codec::{BytesDecode, BytesEncode, CodecError};
use crate::cursor::{Cursor, CursorError};
use crate::db_record::DbRecord;
use crate::txn::RoTxn;
use snafu::{ResultExt, Snafu};
use std::io::Write;
use std::marker::PhantomData;

#[derive(Debug, Snafu)]
pub enum DatabaseError {
    /// Cannot read from a database handle opened for writing
    #[snafu(display("Cannot read from a write-only database handle"))]
    WriteOnlyHandle,

    /// Cannot write to a database handle opened for reading
    #[snafu(display("Cannot write to a read-only database handle"))]
    ReadOnlyHandle,

    /// Error during cursor operations
    #[snafu(display("Cursor error"))]
    Cursor { source: CursorError },

    /// Error during key/value codec operations
    #[snafu(display("Codec error"))]
    Codec { source: CodecError },
}

pub type DatabaseResult<T> = std::result::Result<T, DatabaseError>;

#[derive(Debug, Clone)]
pub(crate) enum DbCore {
    Read(DbRecord),
    Write(String), // Name of the DB
}

/// Typed database handle.
#[derive(Debug, Clone)]
pub struct Database<'a, K, V> {
    core: DbCore,
    _marker: PhantomData<&'a (K, V)>,
}

impl<'a, K, V> Database<'a, K, V> {
    pub(crate) fn new(record: DbRecord) -> Self {
        Self {
            core: DbCore::Read(record),
            _marker: PhantomData,
        }
    }

    pub(crate) fn new_write(name: Option<String>) -> Self {
        Self {
            core: DbCore::Write(name.unwrap_or_else(|| "main".to_string())),
            _marker: PhantomData,
        }
    }

    /// Get the underlying DbRecord (Read mode only).
    pub fn record(&self) -> Option<&DbRecord> {
        match &self.core {
            DbCore::Read(r) => Some(r),
            DbCore::Write(_) => None,
        }
    }
}

impl<'a, K, V> Database<'a, K, V>
where
    K: BytesEncode + BytesDecode<'a>,
    V: BytesDecode<'a>,
{
    /// Get entry from database (Read mode only).
    pub fn get<'txn>(
        &self,
        txn: &'txn RoTxn<'a>,
        key: &K::EItem,
    ) -> DatabaseResult<Option<V::DItem>> {
        let record = match &self.core {
            DbCore::Read(r) => r,
            DbCore::Write(_) => return Err(DatabaseError::WriteOnlyHandle),
        };

        let key_bytes = K::bytes_encode(key).context(CodecSnafu)?;
        let env = txn.env();
        let mut cursor = Cursor::new(
            env.raw_data(),
            env.arch(),
            record.root_page,
            env.page_size(),
        );

        if let Some(val_bytes) = cursor.get(&key_bytes).context(CursorSnafu)? {
            let val = V::bytes_decode(val_bytes).context(CodecSnafu)?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    /// Iterator (Read mode only).
    pub fn iter<'txn>(&self, txn: &'txn RoTxn<'a>) -> DatabaseResult<RoIter<'txn, 'a, K, V>> {
        let record = match &self.core {
            DbCore::Read(r) => r,
            DbCore::Write(_) => return Err(DatabaseError::WriteOnlyHandle),
        };

        let env = txn.env();
        let cursor = Cursor::new(
            env.raw_data(),
            env.arch(),
            record.root_page,
            env.page_size(),
        );

        let iter = cursor.iter_start_owned().context(CursorSnafu)?;

        Ok(RoIter {
            iter,
            _marker: PhantomData,
            _txn_marker: PhantomData,
        })
    }
}

// Write Implementation
impl<'a, K, V> Database<'a, K, V>
where
    K: BytesEncode,
    V: BytesEncode,
{
    pub fn put<W: Write>(
        &self,
        txn: &mut crate::txn::RwTxn<'_, W>,
        key: &K::EItem,
        value: &V::EItem,
    ) -> DatabaseResult<()> {
        if let DbCore::Write(name) = &self.core {
            let k_bytes = K::bytes_encode(key).context(CodecSnafu)?;
            let v_bytes = V::bytes_encode(value).context(CodecSnafu)?;
            txn.append(name, &k_bytes, &v_bytes);
            Ok(())
        } else {
            Err(DatabaseError::ReadOnlyHandle)
        }
    }
}

/// Read-only iterator.
pub struct RoIter<'txn, 'a, K, V> {
    iter: crate::cursor::OwnedCursorIter<'a>,
    _marker: PhantomData<(K, V)>,
    _txn_marker: PhantomData<&'txn ()>,
}

impl<'txn, 'a, K, V> Iterator for RoIter<'txn, 'a, K, V>
where
    K: BytesDecode<'a>,
    V: BytesDecode<'a>,
{
    type Item = DatabaseResult<(K::DItem, V::DItem)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok((k, v))) => {
                let decoded_k = match K::bytes_decode(k).context(CodecSnafu) {
                    Ok(val) => val,
                    Err(e) => return Some(Err(e)),
                };
                let decoded_v = match V::bytes_decode(v).context(CodecSnafu) {
                    Ok(val) => val,
                    Err(e) => return Some(Err(e)),
                };
                Some(Ok((decoded_k, decoded_v)))
            }
            Some(Err(e)) => Some(Err(DatabaseError::Cursor { source: e })),
            None => None,
        }
    }
}
