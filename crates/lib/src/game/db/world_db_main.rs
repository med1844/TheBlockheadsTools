use super::{
    super::dynamic_object::{
        DynamicObjectList, UniqueID,
        blockhead::{Blockhead, Inventory},
    },
    dynamic_world_v2::DynamicWorldV2,
    world_v2::WorldV2,
};
use crate::util::plist::to_xml_plist;
use lmdb_rs::{
    codec::types::{Bytes, Str},
    database::Database,
    txn::{RoTxn, RwTxn},
};
use snafu::{OptionExt, ResultExt, Snafu};
use std::{collections::HashMap, io::Write};

#[derive(Debug, Snafu)]
pub enum MainError {
    #[snafu(display("Failed to get entry {key} from database: {source}"))]
    GetEntry {
        source: lmdb_rs::error::DatabaseError,
        key: &'static str,
    },
    #[snafu(display("Key {key} doesn't exist in database"))]
    MissingKey { key: &'static str },
    #[snafu(display("Failed to iterate over database: {source}"))]
    IterateDatabase {
        source: lmdb_rs::error::DatabaseError,
    },
    #[snafu(display("Failed to decode database entry: {source}"))]
    DecodeEntry {
        source: lmdb_rs::error::DatabaseError,
    },
    #[snafu(display("Failed to put entry with key {key} in database: {source}"))]
    PutEntry {
        key: String,
        source: lmdb_rs::error::DatabaseError,
    },
    #[snafu(display(
        "Failed to deserialize inventory of blockhead with unique id = {unique_id}: {source}"
    ))]
    DeserializeBlockheadInventory {
        unique_id: u64,
        source: plist::Error,
    },
    #[snafu(display(
        "Failed to serialize inventory of blockhead with unique id = {unique_id}: {source}"
    ))]
    SerializeBlockheadInventory {
        unique_id: u64,
        source: plist::Error,
    },
    #[snafu(display("Failed to deserialize `blockheads` : {source}"))]
    DeserializeBlockheads { source: plist::Error },
    #[snafu(display("Failed to serialize `blockheads` : {source}"))]
    SerializeBlockheads { source: plist::Error },
    #[snafu(display("Failed to deserialize `dynamic_world_v2` : {source}"))]
    DeserializeDynamicWorldV2 { source: plist::Error },
    #[snafu(display("Failed to serialize `dynamic_world_v2` : {source}"))]
    SerializeDynamicWorldV2 { source: plist::Error },
    #[snafu(display("Failed to deserialize `world_v2` : {source}"))]
    DeserializeWorldV2 { source: plist::Error },
    #[snafu(display("Failed to serialize `world_v2` : {source}"))]
    SerializeWorldV2 { source: plist::Error },
}

type Result<T> = std::result::Result<T, MainError>;

#[derive(Debug)]
pub struct Main {
    pub blockheads: DynamicObjectList<Blockhead>,
    pub dynamic_world_v2: DynamicWorldV2,
    pub world_v2: WorldV2,
    pub blockhead_inventories: HashMap<UniqueID, Inventory>,
}

impl Main {
    pub fn from_db(db: &Database<Str, Bytes>, rtxn: &RoTxn) -> Result<Self> {
        let get = |key: &'static str| {
            db.get(rtxn, key)
                .context(GetEntrySnafu { key })?
                .context(MissingKeySnafu { key })
        };
        let blockheads = get("blockheads")?;
        let dynamic_world_v2 = get("dynamicWorldv2")?;
        let world_v2 = get("worldv2")?;
        let mut blockhead_inventories = HashMap::new();
        for entry in db.iter(rtxn).context(IterateDatabaseSnafu)? {
            let (key, value) = entry.context(DecodeEntrySnafu)?;
            if let Some(blockhead_id_str) = key
                .strip_prefix("blockhead_")
                .and_then(|key| key.strip_suffix("_inventory"))
                && let Ok(blockhead_id) = blockhead_id_str.parse()
            {
                let _ = blockhead_inventories.insert(
                    UniqueID::new(blockhead_id),
                    plist::from_reader_xml(value).context(DeserializeBlockheadInventorySnafu {
                        unique_id: blockhead_id,
                    })?,
                );
            }
        }
        Ok(Self {
            blockheads: plist::from_reader_xml(blockheads).context(DeserializeBlockheadsSnafu)?,
            dynamic_world_v2: plist::from_reader_xml(dynamic_world_v2)
                .context(DeserializeDynamicWorldV2Snafu)?,
            world_v2: plist::from_bytes(world_v2).context(DeserializeWorldV2Snafu)?,
            blockhead_inventories,
        })
    }

    pub fn to_db<W: Write>(&self, db: &Database<Str, Bytes>, wtxn: &mut RwTxn<W>) -> Result<()> {
        let mut put = |key: &'static str, bytes: &[u8]| {
            db.put(wtxn, key, bytes).context(PutEntrySnafu {
                key: key.to_owned(),
            })
        };
        put(
            "dynamicWorldv2",
            to_xml_plist(&self.dynamic_world_v2)
                .context(SerializeDynamicWorldV2Snafu)?
                .as_slice(),
        )?;
        put(
            "blockheads",
            to_xml_plist(&self.blockheads)
                .context(SerializeBlockheadsSnafu)?
                .as_slice(),
        )?;
        put(
            "worldv2",
            to_xml_plist(&self.world_v2)
                .context(SerializeWorldV2Snafu)?
                .as_slice(),
        )?;
        for (unique_id, inventory) in self.blockhead_inventories.iter() {
            let inventory_bytes =
                to_xml_plist(inventory).context(SerializeBlockheadInventorySnafu {
                    unique_id: *unique_id.inner(),
                })?;
            let key = format!("blockhead_{}_inventory", unique_id.inner());
            db.put(wtxn, key.as_str(), inventory_bytes.as_slice())
                .context(PutEntrySnafu { key })?;
        }
        Ok(())
    }
}
