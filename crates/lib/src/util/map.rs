use heed::{Database, RoTxn, RwTxn, types::*};
use std::{collections::HashMap, ops::Deref};

#[derive(Debug)]
pub struct Map(HashMap<String, Vec<u8>>);

impl Map {
    pub fn from_db(db: &Database<Str, Bytes>, rtxn: &RoTxn) -> heed::Result<Self> {
        Ok(Self(
            db.iter(rtxn)?
                .filter_map(|v| v.ok())
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        ))
    }

    pub fn to_db(&self, db: &Database<Str, Bytes>, wtxn: &mut RwTxn) -> heed::Result<()> {
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
