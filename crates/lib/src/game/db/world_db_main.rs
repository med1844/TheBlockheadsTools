use super::{
    super::{
        dw::dynamic_object::{Blockhead, DynamicObjectList, UniqueID},
        item::Inventory,
    },
    dynamic_world_v2::DynamicWorldV2,
    world_v2::WorldV2,
};
use crate::{BhError, BhResult};
use lmdb_rs::{
    codec::types::{Bytes, Str},
    database::Database,
    txn::{RoTxn, RwTxn},
};
use std::{collections::HashMap, io::Write};

#[derive(Debug)]
pub struct WorldDbMain {
    pub blockheads: DynamicObjectList<Blockhead>,
    pub dynamic_world_v2: DynamicWorldV2,
    pub world_v2: WorldV2,
    pub blockhead_inventories: HashMap<UniqueID, Inventory>,
}

impl WorldDbMain {
    pub fn from_db(db: &Database<Str, Bytes>, rtxn: &RoTxn) -> BhResult<Self> {
        let (Some(blockheads), Some(dynamic_world_v2), Some(world_v2)) = (
            db.get(rtxn, "blockheads")?,
            db.get(rtxn, "dynamicWorldv2")?,
            db.get(rtxn, "worldv2")?,
        ) else {
            return Err(BhError::MissingKey(
                "One or more of `blockheads`, `dynamicWorldv2`, `worldv2` is missing from `main` database",
            ));
        };
        let mut blockhead_inventories = HashMap::new();
        for entry in db.iter(rtxn)? {
            let (key, value) = entry?;
            if let Some(blockhead_id_str) = key
                .strip_prefix("blockhead_")
                .and_then(|key| key.strip_suffix("_inventory"))
                && let Ok(blockhead_id) = blockhead_id_str.parse()
            {
                let _ = blockhead_inventories
                    .insert(UniqueID::new(blockhead_id), plist::from_reader_xml(value)?);
            }
        }
        Ok(Self {
            blockheads: plist::from_reader_xml(blockheads)?,
            dynamic_world_v2: plist::from_reader_xml(dynamic_world_v2)?,
            world_v2: plist::from_bytes(world_v2)?,
            blockhead_inventories,
        })
    }

    pub fn to_db<W: Write>(&self, db: &Database<Str, Bytes>, wtxn: &mut RwTxn<W>) -> BhResult<()> {
        let mut dynamic_world_v2_bytes = Vec::new();
        plist::to_writer_xml(&mut dynamic_world_v2_bytes, &self.dynamic_world_v2)?;
        db.put(wtxn, "dynamicWorldv2", dynamic_world_v2_bytes.as_slice())?;
        let mut blockheads_bytes = Vec::new();
        plist::to_writer_xml(&mut blockheads_bytes, &self.blockheads)?;
        db.put(wtxn, "blockheads", blockheads_bytes.as_slice())?;
        let mut world_v2_bytes = Vec::new();
        plist::to_writer_binary(&mut world_v2_bytes, &self.world_v2)?;
        db.put(wtxn, "worldv2", world_v2_bytes.as_slice())?;
        Ok(())
    }
}
