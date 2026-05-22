use super::{
    super::{
        dynamic_object::{
            DynamicObjectList,
            blockhead::{Blockhead, BlockheadXml},
        },
        item::{Inventory, ItemError},
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
use std::io::Write;

#[derive(Debug, Snafu)]
pub enum MainError {
    #[snafu(display("Failed to get entry {key} from database"))]
    GetEntry {
        source: lmdb_rs::error::DatabaseError,
        key: String,
    },
    #[snafu(display("Key {key} doesn't exist in database"))]
    MissingKey { key: &'static str },
    #[snafu(display("Failed to iterate over database"))]
    IterateDatabase {
        source: lmdb_rs::error::DatabaseError,
    },
    #[snafu(display("Failed to decode database entry"))]
    DecodeEntry {
        source: lmdb_rs::error::DatabaseError,
    },
    #[snafu(display("Failed to put entry with key {key} in database"))]
    PutEntry {
        key: String,
        source: lmdb_rs::error::DatabaseError,
    },
    #[snafu(display("Failed to deserialize inventory of blockhead with unique id = {unique_id}"))]
    DeserializeBlockheadInventory {
        unique_id: u64,
        source: plist::Error,
    },
    #[snafu(display("Failed to serialize inventory of blockhead with unique id = {unique_id}"))]
    SerializeBlockheadInventory {
        unique_id: u64,
        source: plist::Error,
    },
    #[snafu(display("Failed to save inventory of blockhead with unique id = {unique_id}"))]
    SaveBlockheadInventory { unique_id: u64, source: ItemError },
    #[snafu(display("Failed to parse inventory of blockhead with unique id = {unique_id}"))]
    ParseBlockheadInventory { unique_id: u64, source: ItemError },
    #[snafu(display("Can't find inventory of blockhead with unique id = {unique_id} in db"))]
    BlockheadInventoryNotFound { unique_id: u64 },
    #[snafu(display("Failed to deserialize `blockheads`"))]
    DeserializeBlockheads { source: plist::Error },
    #[snafu(display("Failed to serialize `blockheads`"))]
    SerializeBlockheads { source: plist::Error },
    #[snafu(display("Failed to deserialize `dynamic_world_v2`"))]
    DeserializeDynamicWorldV2 { source: plist::Error },
    #[snafu(display("Failed to serialize `dynamic_world_v2`"))]
    SerializeDynamicWorldV2 { source: plist::Error },
    #[snafu(display("Failed to deserialize `world_v2`"))]
    DeserializeWorldV2 { source: plist::Error },
    #[snafu(display("Failed to serialize `world_v2`"))]
    SerializeWorldV2 { source: plist::Error },
}

type Result<T> = std::result::Result<T, MainError>;

#[derive(Debug)]
pub struct Main {
    pub blockheads: Vec<Blockhead>,
    pub dynamic_world_v2: DynamicWorldV2,
    pub world_v2: WorldV2,
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

        let blockhead_xmls: DynamicObjectList<BlockheadXml> =
            plist::from_reader_xml(blockheads).context(DeserializeBlockheadsSnafu)?;
        let blockheads = blockhead_xmls
            .into_iter()
            .map(|xml| -> Result<Blockhead> {
                let blockhead_id = *xml.unique_id.inner();
                let key = format!("blockhead_{}_inventory", blockhead_id);
                let inventory_bytes = db
                    .get(rtxn, &key)
                    .context(GetEntrySnafu { key: key.clone() })?
                    .context(BlockheadInventoryNotFoundSnafu {
                        unique_id: blockhead_id,
                    })?;
                let value = plist::from_reader_xml(inventory_bytes).context(
                    DeserializeBlockheadInventorySnafu {
                        unique_id: blockhead_id,
                    },
                )?;
                let inventory =
                    Inventory::try_from_value(value).context(ParseBlockheadInventorySnafu {
                        unique_id: blockhead_id,
                    })?;
                Ok(Blockhead::from_xml_and_inventory(xml, inventory))
            })
            .collect::<Result<Vec<Blockhead>>>()?;
        Ok(Self {
            blockheads,
            dynamic_world_v2: plist::from_reader_xml(dynamic_world_v2)
                .context(DeserializeDynamicWorldV2Snafu)?,
            world_v2: plist::from_bytes(world_v2).context(DeserializeWorldV2Snafu)?,
        })
    }

    pub fn to_db<W: Write>(&self, db: &Database<Str, Bytes>, wtxn: &mut RwTxn<W>) -> Result<()> {
        let put = |key: &'static str, bytes: &[u8], wtxn: &mut RwTxn<W>| {
            db.put(wtxn, key, bytes).context(PutEntrySnafu {
                key: key.to_owned(),
            })
        };
        put(
            "dynamicWorldv2",
            to_xml_plist(&self.dynamic_world_v2)
                .context(SerializeDynamicWorldV2Snafu)?
                .as_slice(),
            wtxn,
        )?;
        put(
            "worldv2",
            to_xml_plist(&self.world_v2)
                .context(SerializeWorldV2Snafu)?
                .as_slice(),
            wtxn,
        )?;

        let blockhead_xmls = self
            .blockheads
            .iter()
            .map(|blockhead| {
                let unique_id = *blockhead.unique_id.inner();
                let xml: BlockheadXml = blockhead.into();
                let inventory_value = blockhead
                    .inventory
                    .to_value()
                    .context(SaveBlockheadInventorySnafu { unique_id })?;
                let inventory_bytes = to_xml_plist(&inventory_value)
                    .context(SerializeBlockheadInventorySnafu { unique_id })?;
                let key = format!("blockhead_{}_inventory", unique_id);
                db.put(wtxn, key.as_str(), inventory_bytes.as_slice())
                    .context(PutEntrySnafu { key })?;
                Ok(xml)
            })
            .collect::<Result<DynamicObjectList<BlockheadXml>>>()?;
        put(
            "blockheads",
            to_xml_plist(&blockhead_xmls)
                .context(SerializeBlockheadsSnafu)?
                .as_slice(),
            wtxn,
        )?;
        Ok(())
    }
}
