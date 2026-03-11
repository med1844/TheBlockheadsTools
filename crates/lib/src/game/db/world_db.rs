use super::{
    super::{chunk::Chunks, dynamic_world::DynamicWorld},
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

#[cfg(test)]
mod tests {
    use super::WorldDb;
    use crate::util::plist::diff_plist_keys;
    use lmdb_rs::{
        arch::DynArch,
        codec::types::{Bytes, Str},
        env::Env,
    };
    use std::fs;

    #[test]
    fn test_world_db_plist_fidelity() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let db_resources_dir = format!("{}/../py_bindings/tests/resources/", manifest_dir);
        let db_names = ["3d7", "9c3"];

        let mut diffs = Vec::new();
        for (db_name, db_path) in
            db_names.map(|s| (s, format!("{}/{}/world_db/data.mdb", db_resources_dir, s)))
        {
            // 1. Load Original DB
            let original_bytes = fs::read(&db_path).unwrap();

            // 2. Round-Trip via WorldDb
            let world_db = WorldDb::from_bytes(&original_bytes).unwrap();
            let mut mem_bytes = Vec::new();
            world_db.write_to(&mut mem_bytes, DynArch::Arch64).unwrap();

            // 3. Load In-Memory Serialized DB
            let mem_env = Env::new(&mem_bytes).unwrap();
            let mem_rtxn = mem_env.read_txn().unwrap();

            // 4. Compare all entries in sub-databases
            let original_env = Env::new(&original_bytes).unwrap();
            let original_rtxn = original_env.read_txn().unwrap();
            for sub_db_name in ["dw", "main", "blocks"] {
                let orig_db = original_env
                    .open_database::<Str, Bytes>(&original_rtxn, Some(sub_db_name))
                    .unwrap()
                    .unwrap();
                let mem_db = mem_env
                    .open_database::<Str, Bytes>(&mem_rtxn, Some(sub_db_name))
                    .unwrap()
                    .unwrap();

                for kv in orig_db.iter(&original_rtxn).unwrap() {
                    let (k, orig_v) = kv.unwrap();

                    // Try to parse the original value as a plist.
                    // If it parses successfully, it means this record is a plist,
                    // and we should verify its structural fidelity.
                    let orig_plist_result = plist::from_bytes::<plist::Value>(orig_v);

                    if let Ok(orig_plist) = orig_plist_result {
                        if let Some(mem_v) = mem_db.get(&mem_rtxn, k).unwrap() {
                            if let Ok(mem_plist) = plist::from_bytes::<plist::Value>(mem_v) {
                                diff_plist_keys(
                                    &format!("{}/{}", sub_db_name, k),
                                    &orig_plist,
                                    &mem_plist,
                                    &mut diffs,
                                );
                            } else {
                                diffs.push(format!(
                                    "Key `{}` in output db for sub-db `{}` in `{}` is not a valid plist",
                                    k, sub_db_name, db_name
                                ));
                            }
                        } else {
                            diffs.push(format!(
                                "Key {} missing in output db for sub-db `{}` in `{}`",
                                k, sub_db_name, db_name
                            ));
                        }
                    }
                }
            }
        }
        assert!(
            diffs.is_empty(),
            "Structural fidelity violations:\n{}",
            diffs.join("\n")
        );
    }
}
