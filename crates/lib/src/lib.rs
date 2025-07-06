use std::collections::HashMap;
use std::io::Read;
use std::ops::Deref;
use std::path::Path;

use coords::ChunkOffset;
use flate2::read::GzDecoder;
use heed::Database;
use heed::EnvOpenOptions;
use heed::RoTxn;
use heed::types::*;
use serde::Deserialize;

mod block;
mod block_type;
mod coords;
mod error;

pub use block::{Block, BlockMut, BlockView, BlockViewMut};
pub use block_type::{BlockContent, BlockType};
pub use coords::{BlockCoord, ChunkBlockCoord, ChunkCoord};
pub use error::{BhError, BhResult};

#[derive(Debug)]
pub struct Map(HashMap<String, Vec<u8>>);

impl Map {
    fn from_db(db: &Database<Str, Bytes>, rtxn: &RoTxn) -> Result<Self, heed::Error> {
        Ok(Self(
            db.iter(rtxn)?
                .filter_map(|v| v.ok())
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        ))
    }
}

impl Deref for Map {
    type Target = HashMap<String, Vec<u8>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize)]
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
    pub world_width_macro: u64,
}

#[derive(Debug)]
pub struct WorldDbMain {
    pub blockheads: Vec<u8>,       // Vec<Blockheads>
    pub dynamic_world_v2: Vec<u8>, // ???
    pub world_v2: WorldV2,
}

impl WorldDbMain {
    fn from_db(db: &Database<Str, Bytes>, rtxn: &RoTxn) -> BhResult<Option<Self>> {
        let (Some(blockheads), Some(dynamic_world_v2), Some(world_v2)) = (
            db.get(rtxn, "blockheads")?,
            db.get(rtxn, "dynamicWorldv2")?,
            db.get(rtxn, "worldv2")?,
        ) else {
            return Ok(None);
        };
        Ok(Some(Self {
            blockheads: blockheads.to_vec(),
            dynamic_world_v2: dynamic_world_v2.to_vec(),
            world_v2: plist::from_bytes::<WorldV2>(world_v2)?,
        }))
    }
}

const NUM_BLOCK_PER_ROW: usize = 32;
const NUM_BLOCK_PER_COL: usize = 32;
const NUM_BYTES_PER_BLOCK: usize = 64;

#[derive(Debug)]
pub struct Chunk([u8; NUM_BLOCK_PER_ROW * NUM_BLOCK_PER_COL * NUM_BYTES_PER_BLOCK + 5]); // 5 unknown bytes

impl Chunk {
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
        let slice = <&[u8; 64]>::try_from(&self.inner()[offset..offset + NUM_BYTES_PER_BLOCK])
            .expect(
                "Return value of `ChunkBlockCoord.to_offset()` is guaranteed \
                to be smaller than chunk bytes len - 32",
            );
        Block::new(slice)
    }

    pub fn block_at_mut<O: ChunkOffset>(&mut self, coord: O) -> BlockMut {
        let offset = coord.to_offset();
        let slice =
            <&mut [u8; 64]>::try_from(&mut self.inner_mut()[offset..offset + NUM_BYTES_PER_BLOCK])
                .expect(
                    "Return value of `ChunkBlockCoord.to_offset()` is guaranteed \
                to be smaller than chunk bytes len - 32",
                );
        BlockMut::new(slice)
    }

    pub fn display_chunk_by_fg(&self) -> String {
        let mut result = String::new();

        for y in (0..NUM_BLOCK_PER_COL).rev() {
            let mut row_strings = Vec::new();
            for x in 0..NUM_BLOCK_PER_ROW {
                let coord = ChunkBlockCoord::new(x as u8, y as u8).expect("Must be valid coord");
                let block = self.block_at(coord);

                let display_text = match block.fg() {
                    Ok(block_type) => block_type.as_str().chars().take(5).collect::<String>(),
                    Err(_) => {
                        format!("{}", block.fg_raw())
                    }
                };

                row_strings.push(format!("{:>7}", display_text));
            }
            result.push_str(&row_strings.join(""));

            if y > 0 {
                result.push('\n');
            }
        }
        result
    }
}

pub trait FromCompressedGzip: Sized {
    fn from_compressed_gzip(bytes: &[u8]) -> Result<Self, std::io::Error>;
}

impl FromCompressedGzip for Chunk {
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

impl<T: FromCompressedGzip> Gzip<T> {
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
    pub fn from_path<P: AsRef<Path>>(path: P) -> BhResult<Option<Self>> {
        let mut options = EnvOpenOptions::new();
        options.map_size(10 * 1024 * 1024).max_dbs(100);
        let env = unsafe { options.open(path)? };
        let rtxn = env.read_txn()?;
        let open_db = |name: &str| env.open_database::<Str, Bytes>(&rtxn, Some(name));
        let (Some(blocks), Some(dw), Some(main)) =
            (open_db("blocks")?, open_db("dw")?, open_db("main")?)
        else {
            return Ok(None);
        };
        let blocks = Chunks::from_db(&blocks, &rtxn)?;
        let dw = Map::from_db(&dw, &rtxn)?;
        let main = WorldDbMain::from_db(&main, &rtxn)?;
        let Some(main) = main else {
            return Ok(None);
        };
        Ok(Some(Self { blocks, dw, main }))
    }
}
