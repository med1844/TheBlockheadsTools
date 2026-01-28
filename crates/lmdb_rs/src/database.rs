use crate::codec::BytesDecode;
use crate::codec::BytesEncode;
use crate::cursor::Cursor;
use crate::db_record::DbRecord;
use crate::error::Result;
use crate::txn::RoTxn;
use std::io::Write;
use std::marker::PhantomData;

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
    pub fn get<'txn>(&self, txn: &'txn RoTxn<'a>, key: &K::EItem) -> Result<Option<V::DItem>> {
        let record = match &self.core {
            DbCore::Read(r) => r,
            DbCore::Write(_) => {
                return Err(crate::error::Error::Io(std::io::Error::other(
                    "Cannot read from Write DB",
                )));
            }
        };

        let key_bytes = K::bytes_encode(key)?;
        let env = txn.env();
        let mut cursor = Cursor::new(
            env.raw_data(),
            env.arch(),
            record.root_page,
            env.page_size(),
        );

        if let Some(val_bytes) = cursor.get(&key_bytes)? {
            let val = V::bytes_decode(val_bytes)?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    /// Iterator (Read mode only).
    pub fn iter<'txn>(&self, txn: &'txn RoTxn<'a>) -> Result<RoIter<'txn, 'a, K, V>> {
        let record = match &self.core {
            DbCore::Read(r) => r,
            DbCore::Write(_) => {
                return Err(crate::error::Error::Io(std::io::Error::other(
                    "Cannot iterate Write DB",
                )));
            }
        };

        let env = txn.env();
        let cursor = Cursor::new(
            env.raw_data(),
            env.arch(),
            record.root_page,
            env.page_size(),
        );

        let iter = cursor.iter_start_owned()?;

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
    ) -> Result<()> {
        if let DbCore::Write(name) = &self.core {
            let k_bytes = K::bytes_encode(key)?;
            let v_bytes = V::bytes_encode(value)?;
            txn.append(name, &k_bytes, &v_bytes);
            Ok(())
        } else {
            Err(crate::error::Error::Io(std::io::Error::other(
                "Cannot write to Read DB",
            )))
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
    type Item = Result<(K::DItem, V::DItem)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok((k, v))) => {
                let decoded_k = match K::bytes_decode(k) {
                    Ok(val) => val,
                    Err(e) => return Some(Err(e)),
                };
                let decoded_v = match V::bytes_decode(v) {
                    Ok(val) => val,
                    Err(e) => return Some(Err(e)),
                };
                Some(Ok((decoded_k, decoded_v)))
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }
}
