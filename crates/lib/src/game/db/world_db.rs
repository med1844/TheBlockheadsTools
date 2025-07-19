use super::{super::chunk::Chunks, world_db_main::WorldDbMain};
use crate::{BhError, BhResult, util::map::Map};
use heed::{Database, EnvOpenOptions, types::*};
use std::path::Path;

#[derive(Debug)]
pub struct WorldDb {
    pub blocks: Chunks,
    pub dw: Map, // ???
    pub main: WorldDbMain,
}

impl WorldDb {
    pub fn from_path<P: AsRef<Path>>(path: P) -> BhResult<Self> {
        let mut options = EnvOpenOptions::new();
        options.map_size(10 * 1024 * 1024).max_dbs(100);
        let env = unsafe { options.open(path)? };
        let rtxn = env.read_txn()?;
        let open_db = |name: &str| env.open_database::<Str, Bytes>(&rtxn, Some(name));
        let (Some(blocks), Some(dw), Some(main)) =
            (open_db("blocks")?, open_db("dw")?, open_db("main")?)
        else {
            return Err(BhError::MissingKey(
                "One or more of `block`, `dw` or `main` is missing in the database",
            ));
        };
        let blocks = Chunks::from_db(&blocks, &rtxn)?;
        let dw = Map::from_db(&dw, &rtxn)?;
        let main = WorldDbMain::from_db(&main, &rtxn)?;
        Ok(Self { blocks, dw, main })
    }

    pub fn to_path<P: AsRef<Path>>(&self, path: P) -> BhResult<()> {
        let mut options = EnvOpenOptions::new();
        options.map_size(1024 * 1024 * 1024).max_dbs(100);

        let env = unsafe { options.open(path)? };
        let mut wtxn = env.write_txn()?;

        let blocks_db: Database<Str, Bytes> = env.create_database(&mut wtxn, Some("blocks"))?;
        self.blocks.to_db(&blocks_db, &mut wtxn)?;
        let dw_db: Database<Str, Bytes> = env.create_database(&mut wtxn, Some("dw"))?;
        self.dw.to_db(&dw_db, &mut wtxn)?;
        let main_db: Database<Str, Bytes> = env.create_database(&mut wtxn, Some("main"))?;
        self.main.to_db(&main_db, &mut wtxn)?;

        wtxn.commit()?;
        Ok(())
    }
}
