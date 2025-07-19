use super::world_v2::WorldV2;
use crate::{BhError, BhResult};
use heed::{Database, RoTxn, RwTxn, types::*};

#[derive(Debug)]
pub struct WorldDbMain {
    pub blockheads: Vec<u8>,       // Vec<Blockheads>
    pub dynamic_world_v2: Vec<u8>, // ???
    pub world_v2: WorldV2,
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
        Ok(Self {
            blockheads: blockheads.to_vec(),
            dynamic_world_v2: dynamic_world_v2.to_vec(),
            world_v2: plist::from_bytes::<WorldV2>(world_v2)?, // TODO this should be handled by WorldV2
        })
    }

    pub fn to_db(&self, db: &Database<Str, Bytes>, wtxn: &mut RwTxn) -> BhResult<()> {
        db.put(wtxn, "dynamicWorldv2", self.dynamic_world_v2.as_slice())?;
        db.put(wtxn, "blockheads", self.blockheads.as_slice())?;
        let mut world_v2_bytes = Vec::new();
        plist::to_writer_xml(&mut world_v2_bytes, &self.world_v2)?;
        db.put(wtxn, "worldv2", world_v2_bytes.as_slice())?;
        Ok(())
    }
}
