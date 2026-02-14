use super::{
    super::{chunk::Chunks, dw::dynamic_world::DynamicWorld},
    world_db_main::WorldDbMain,
};
use crate::{BhError, BhResult};
use lmdb_rs::{
    arch::DynArch,
    codec::types::{Bytes, Str},
    env::{Env, EnvWrite},
};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

#[derive(Debug)]
pub struct WorldDb {
    pub chunks: Chunks,
    pub dw: DynamicWorld,
    pub main: WorldDbMain,
}

impl WorldDb {
    pub fn from_bytes(data: &[u8]) -> BhResult<Self> {
        let env = Env::new(data)?;
        let rtxn = env.read_txn()?;

        let open_db = |name: &str| env.open_database::<Str, Bytes>(&rtxn, Some(name));
        let (Some(blocks), Some(dw), Some(main)) =
            (open_db("blocks")?, open_db("dw")?, open_db("main")?)
        else {
            return Err(BhError::MissingKey(
                "One or more of `block`, `dw` or `main` is missing in the database",
            ));
        };
        let main = WorldDbMain::from_db(&main, &rtxn)?;
        let chunks = Chunks::from_db(&blocks, &rtxn, main.world_v2.world_width_macro)?;
        let dw = DynamicWorld::from_db(&dw, &rtxn)?;

        Ok(Self { chunks, dw, main })
    }

    pub fn write_to<W: Write>(&self, writer: W, arch: DynArch) -> BhResult<()> {
        let mut env = EnvWrite::new(writer, arch);

        let mut wtxn = env.write_txn()?;

        let blocks_db = wtxn.create_database(Some("blocks"))?;
        self.chunks.to_db(&blocks_db, &mut wtxn)?;
        let dw_db = wtxn.create_database(Some("dw"))?;
        self.dw.to_db(&dw_db, &mut wtxn)?;
        let main_db = wtxn.create_database(Some("main"))?;
        self.main.to_db(&main_db, &mut wtxn)?;

        wtxn.commit()?;
        Ok(())
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> BhResult<Self> {
        let mut data = Vec::new();
        {
            let mut file = File::open(path.as_ref().join("data.mdb"))?;
            let _ = file.read_to_end(&mut data)?;
        }
        Self::from_bytes(&data)
    }

    pub fn to_path<P: AsRef<Path>>(&self, path: P, arch: DynArch) -> BhResult<()> {
        let file = File::create_new(path.as_ref().join("data.mdb"))?;
        self.write_to(file, arch)
    }
}
