use super::error::BhResult;
use lmdb_rs::{
    codec::types::{Bytes, Str},
    database::Database,
    txn::{RoTxn, RwTxn},
};
use std::{collections::HashMap, io::Write, ops::Deref};

#[derive(Debug)]
pub struct Map(HashMap<String, Vec<u8>>);

impl Map {
    pub fn from_db(db: &Database<Str, Bytes>, rtxn: &RoTxn) -> BhResult<Self> {
        Ok(Self(
            db.iter(rtxn)?
                .filter_map(|v| v.ok())
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        ))
    }

    pub fn to_db<W: Write>(&self, db: &Database<Str, Bytes>, wtxn: &mut RwTxn<W>) -> BhResult<()> {
        for (k, v) in self.0.iter() {
            db.put(wtxn, k, v)?;
        }
        Ok(())
    }
}

impl Deref for Map {
    type Target = HashMap<String, Vec<u8>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
