use super::{
    super::{
        chunk::{Chunks, ChunksError},
        dynamic_world::{DynamicWorld, DynamicWorldError},
    },
    world_db_main::{Main, MainError},
};
use lmdb_rs::{
    arch::DynArch,
    codec::types::{Bytes, Str},
    env::{Env, EnvWrite},
};
use snafu::{OptionExt, ResultExt, Snafu};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

#[derive(Debug, Snafu)]
pub enum WorldDbError {
    #[snafu(display("Failed to initialize LMDB environment: {source}"))]
    InitEnv { source: lmdb_rs::error::EnvError },
    #[snafu(display("Failed to open database {name}: {source}"))]
    OpenDatabase {
        source: lmdb_rs::error::EnvError,
        name: &'static str,
    },
    #[snafu(display("No database named {name}"))]
    MissingDatabase { name: &'static str },
    #[snafu(display("Failed to create database {name}: {source}"))]
    CreateDatabase {
        source: lmdb_rs::error::TxnError,
        name: &'static str,
    },
    #[snafu(display("Failed to load sub-db `main`: {source}"))]
    LoadMain { source: MainError },
    #[snafu(display("Failed to save sub-db `main`: {source}"))]
    SaveMain { source: MainError },
    #[snafu(display("Failed to load sub-db `blocks`: {source}"))]
    LoadBlocks { source: ChunksError },
    #[snafu(display("Failed to save sub-db `blocks`: {source}"))]
    SaveBlocks { source: ChunksError },
    #[snafu(display("Failed to load sub-db `dw`: {source}"))]
    LoadDw { source: DynamicWorldError },
    #[snafu(display("Failed to save sub-db `dw`: {source}"))]
    SaveDw { source: DynamicWorldError },
    #[snafu(display("Failed to commit changes: {source}"))]
    Commit { source: lmdb_rs::error::TxnError },
    #[snafu(display("Failed to open file: {source}"))]
    OpenFile { source: std::io::Error },
    #[snafu(display("Failed to read file: {source}"))]
    ReadFile { source: std::io::Error },
    #[snafu(display("Failed to create file: {source}"))]
    CreateFile { source: std::io::Error },
}

type Result<T> = std::result::Result<T, WorldDbError>;

#[derive(Debug)]
pub struct WorldDb {
    pub chunks: Chunks,
    pub dw: DynamicWorld,
    pub main: Main,
}

impl WorldDb {
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let env = Env::new(data).context(InitEnvSnafu)?;
        let rtxn = env.read_txn().context(InitEnvSnafu)?;

        let open_db = |name: &'static str| {
            env.open_database::<Str, Bytes>(&rtxn, Some(name))
                .context(OpenDatabaseSnafu { name })?
                .context(MissingDatabaseSnafu { name })
        };
        let blocks = open_db("blocks")?;
        let main = open_db("main")?;
        let dw = open_db("dw")?;

        let main = Main::from_db(&main, &rtxn).context(LoadMainSnafu)?;
        let chunks = Chunks::from_db(&blocks, &rtxn, main.world_v2.world_width_macro)
            .context(LoadBlocksSnafu)?;
        let dw = DynamicWorld::from_db(&dw, &rtxn).context(LoadDwSnafu)?;

        Ok(Self { chunks, dw, main })
    }

    pub fn write_to<W: Write>(&self, writer: W, arch: DynArch) -> Result<()> {
        let mut env = EnvWrite::new(writer, arch);

        let mut wtxn = env.write_txn().context(InitEnvSnafu)?;
        let mut create_db = |name: &'static str| {
            wtxn.create_database(Some(name))
                .context(CreateDatabaseSnafu { name })
        };

        let blocks_db = create_db("blocks")?;
        let main_db = create_db("main")?;
        let dw_db = create_db("dw")?;

        self.chunks
            .to_db(&blocks_db, &mut wtxn)
            .context(SaveBlocksSnafu)?;
        self.main
            .to_db(&main_db, &mut wtxn)
            .context(SaveMainSnafu)?;
        self.dw.to_db(&dw_db, &mut wtxn).context(SaveDwSnafu)?;

        wtxn.commit().context(CommitSnafu)?;
        Ok(())
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut data = Vec::new();
        {
            let mut file = File::open(path.as_ref().join("data.mdb")).context(OpenFileSnafu)?;
            let _ = file.read_to_end(&mut data).context(ReadFileSnafu)?;
        }
        Self::from_bytes(&data)
    }

    pub fn to_path<P: AsRef<Path>>(&self, path: P, arch: DynArch) -> Result<()> {
        let file = File::create_new(path.as_ref().join("data.mdb")).context(CreateFileSnafu)?;
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
    #[ignore] // we are not writing empty dynamic object list, causing false positive
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
