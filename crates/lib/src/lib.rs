use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;

use coords::ChunkOffset;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use heed::Database;
use heed::EnvOpenOptions;
use heed::RoTxn;
use heed::RwTxn;
use heed::types::*;
use maybe_owned::MaybeOwned;
use plist::XmlWriteOptions;
use serde::Deserialize;

mod block;
mod block_type;
mod coords;
mod error;

pub use block::{Block, BlockMut, BlockView, BlockViewMut};
pub use block_type::{BlockContent, BlockType};
pub use coords::{BlockCoord, ChunkBlockCoord, ChunkCoord};
pub use error::{BhError, BhResult};
use serde::Serialize;

#[derive(Debug)]
pub struct Map(HashMap<String, Vec<u8>>);

impl Map {
    fn from_db(db: &Database<Str, Bytes>, rtxn: &RoTxn) -> heed::Result<Self> {
        Ok(Self(
            db.iter(rtxn)?
                .filter_map(|v| v.ok())
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        ))
    }

    fn to_db(&self, db: &Database<Str, Bytes>, wtxn: &mut RwTxn) -> heed::Result<()> {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct WorldV2 {
    #[serde(rename = "blockheadDatasv2")]
    pub blockhead_datasv2: plist::Value,
    #[serde(rename = "circumNavigateBooleansData")]
    pub circum_navigate_booleans_data: plist::Data, // bplist dict
    #[serde(rename = "creationDate")]
    pub creation_date: plist::Date,
    #[serde(rename = "distanceOrderedFoodTypes")]
    pub distance_ordered_food_types: plist::Data, // suspect: Vec<ItemId>, where ItemId = u32
    #[serde(rename = "expertMode")]
    pub expert_mode: bool,
    #[serde(rename = "foundItems")]
    pub found_items: plist::Data, // bplist dict
    #[serde(rename = "hostPort")]
    pub host_port: String,
    #[serde(rename = "maxPlayers")]
    pub max_players: String,
    #[serde(rename = "migrationComplete_1.7")]
    pub migration_complete_v1_7: bool,
    #[serde(rename = "noRainTimer")]
    pub no_rain_timer: f64,
    #[serde(rename = "portalLevel")]
    pub portal_level: u64,
    #[serde(rename = "randomSeed")]
    pub random_seed: u64,
    #[serde(rename = "remoteGame")]
    pub remote_game: bool,
    #[serde(rename = "runAtLaunch")]
    pub run_at_launch: bool,
    #[serde(rename = "saveDate")]
    pub save_date: plist::Date,
    #[serde(rename = "saveID")]
    pub save_id: String,
    #[serde(rename = "saveVersion")]
    pub save_version: u64,
    #[serde(rename = "startPortalPos.x")]
    pub start_portal_pos_x: u64,
    #[serde(rename = "startPortalPos.y")]
    pub start_portal_pos_y: u64,
    #[serde(rename = "translation")]
    pub translation: (f64, f64),
    #[serde(rename = "worldName")]
    pub world_name: String,
    #[serde(rename = "worldTime")]
    pub world_time: f64,
    #[serde(rename = "worldWidthMacro")]
    pub world_width_macro: u32,
}

#[derive(Debug)]
pub struct WorldDbMain {
    pub blockheads: Vec<u8>,       // Vec<Blockheads>
    pub dynamic_world_v2: Vec<u8>, // ???
    pub world_v2: WorldV2,
}

impl WorldDbMain {
    fn from_db(db: &Database<Str, Bytes>, rtxn: &RoTxn) -> BhResult<Self> {
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
            world_v2: plist::from_bytes::<WorldV2>(world_v2)?,
        })
    }

    fn to_db(&self, db: &Database<Str, Bytes>, wtxn: &mut RwTxn) -> BhResult<()> {
        db.put(wtxn, "dynamicWorldv2", self.dynamic_world_v2.as_slice())?;
        db.put(wtxn, "blockheads", self.blockheads.as_slice())?;
        let mut world_v2_bytes = Vec::new();
        plist::to_writer_xml(&mut world_v2_bytes, &self.world_v2)?;
        db.put(wtxn, "worldv2", world_v2_bytes.as_slice())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Chunk(
    [u8; Self::NUM_BLOCK_PER_ROW * Self::NUM_BLOCK_PER_COL * Self::NUM_BYTES_PER_BLOCK + 5],
); // 5 unknown bytes

impl Chunk {
    pub const NUM_BLOCK_PER_ROW: usize = 32;
    pub const NUM_BLOCK_PER_COL: usize = 32;
    pub const NUM_BYTES_PER_BLOCK: usize = 64;

    fn new_empty() -> Self {
        Self([0; 32 * 32 * 64 + 5])
    }

    pub fn inner(&self) -> &[u8] {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    pub fn block_at<O: ChunkOffset>(&self, coord: O) -> Block {
        let offset = coord.to_offset();
        let slice =
            <&[u8; 64]>::try_from(&self.inner()[offset..offset + Self::NUM_BYTES_PER_BLOCK])
                .expect(
                    "Return value of `ChunkBlockCoord.to_offset()` is guaranteed \
                to be smaller than chunk bytes len - 32",
                );
        Block::new(slice)
    }

    pub fn block_at_mut<O: ChunkOffset>(&mut self, coord: O) -> BlockMut {
        let offset = coord.to_offset();
        let slice = <&mut [u8; 64]>::try_from(
            &mut self.inner_mut()[offset..offset + Self::NUM_BYTES_PER_BLOCK],
        )
        .expect(
            "Return value of `ChunkBlockCoord.to_offset()` is guaranteed \
                to be smaller than chunk bytes len - 32",
        );
        BlockMut::new(slice)
    }
}

impl AsRef<[u8]> for Chunk {
    fn as_ref(&self) -> &[u8] {
        self.inner()
    }
}

pub trait FromGzip: Sized {
    fn from_compressed_gzip(bytes: &[u8]) -> Result<Self, std::io::Error>;
}

pub trait ToGzip: Sized {
    fn to_gzip(&self) -> std::io::Result<Vec<u8>>;
}

impl<B: AsRef<[u8]>> ToGzip for B {
    fn to_gzip(&self) -> std::io::Result<Vec<u8>> {
        let a = self.as_ref();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(a)?;
        encoder.finish()
    }
}

impl FromGzip for Chunk {
    fn from_compressed_gzip(bytes: &[u8]) -> Result<Self, std::io::Error> {
        let mut decoder = GzDecoder::new(bytes);
        let mut chunk = Self::new_empty();
        decoder.read_exact(chunk.inner_mut())?;
        Ok(chunk)
    }
}

#[derive(Debug)]
pub enum Gzip<T> {
    Compressed(Vec<u8>),
    Uncompressed(T),
}

impl<T: FromGzip> Gzip<T> {
    fn ensure_decompressed(&mut self) -> Result<(), std::io::Error> {
        let current_state = std::mem::replace(self, Gzip::Compressed(Vec::new()));
        *self = match current_state {
            Gzip::Compressed(vec) => match T::from_compressed_gzip(vec.as_slice()) {
                Ok(obj) => Gzip::Uncompressed(obj),
                Err(e) => {
                    *self = Gzip::Compressed(vec);
                    return Err(e);
                }
            },
            obj @ Gzip::Uncompressed(_) => obj,
        };
        Ok(())
    }

    fn as_uncompressed(&mut self) -> Result<&T, std::io::Error> {
        self.ensure_decompressed()?;
        if let Self::Uncompressed(val) = self {
            Ok(val)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Internal error: Gzip state unexpected after decompression",
            ))
        }
    }

    fn as_uncompressed_mut(&mut self) -> Result<&mut T, std::io::Error> {
        self.ensure_decompressed()?;
        if let Self::Uncompressed(val) = self {
            Ok(val)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Internal error: Gzip state unexpected after decompression",
            ))
        }
    }
}

impl<T: ToGzip> Gzip<T> {
    fn into_compressed<'s>(&'s self) -> std::io::Result<MaybeOwned<'s, Vec<u8>>> {
        match self {
            Gzip::Compressed(vec) => Ok(MaybeOwned::Borrowed(vec)),
            Gzip::Uncompressed(t) => Ok(MaybeOwned::Owned(t.to_gzip()?)),
        }
    }
}

impl<T> Gzip<T> {
    fn from_compressed(bytes: Vec<u8>) -> Self {
        Self::Compressed(bytes)
    }
}

#[derive(Debug)]
pub struct Chunks(HashMap<ChunkCoord, Gzip<Chunk>>);

impl Chunks {
    pub fn from_db(db: &Database<Str, Bytes>, rtxn: &RoTxn) -> Result<Self, heed::Error> {
        Ok(Self(
            db.iter(rtxn)?
                .filter_map(|v| v.ok())
                .filter_map(|(k, v)| {
                    ChunkCoord::from_str(k)
                        .ok()
                        .map(|k| (k, Gzip::from_compressed(v.to_owned())))
                })
                .collect(),
        ))
    }

    pub fn to_db(&self, db: &Database<Str, Bytes>, wtxn: &mut RwTxn) -> BhResult<()> {
        for (key, value) in self.0.iter() {
            let data = value.into_compressed()?;
            db.put(wtxn, key.to_string().as_str(), &data)?;
        }
        Ok(())
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, ChunkCoord, Gzip<Chunk>> {
        self.0.keys()
    }

    pub fn contains_key(&self, key: &ChunkCoord) -> bool {
        self.0.contains_key(key)
    }

    pub fn chunk_at<I: Into<ChunkCoord>>(&mut self, coord: I) -> Option<std::io::Result<&Chunk>> {
        self.0.get_mut(&coord.into()).map(|v| v.as_uncompressed())
    }

    pub fn chunk_at_mut<I: Into<ChunkCoord>>(
        &mut self,
        coord: I,
    ) -> Option<std::io::Result<&mut Chunk>> {
        self.0
            .get_mut(&coord.into())
            .map(|v| v.as_uncompressed_mut())
    }

    pub fn block_at<I: Into<BlockCoord>>(&mut self, coord: I) -> Option<std::io::Result<Block>> {
        let block_coord = coord.into();
        let (chunk_coord, chunk_block_coord) = block_coord.decompose();
        self.0.get_mut(&chunk_coord).map(|v| {
            v.as_uncompressed_mut()
                .map(|chunk| chunk.block_at(chunk_block_coord))
        })
    }

    pub fn block_at_mut<I: Into<BlockCoord>>(
        &mut self,
        coord: I,
    ) -> Option<std::io::Result<BlockMut>> {
        let block_coord = coord.into();
        let (chunk_coord, chunk_block_coord) = block_coord.decompose();
        self.0.get_mut(&chunk_coord).map(|v| {
            v.as_uncompressed_mut()
                .map(|chunk| chunk.block_at_mut(&chunk_block_coord))
        })
    }
}

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
