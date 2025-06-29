use std::collections::HashMap;
use std::io::Read;
use std::ops::Deref;
use std::path::Path;

use block::BlockView;
use flate2::read::GzDecoder;
use heed::Database;
use heed::EnvOpenOptions;
use heed::RoTxn;
use heed::types::*;
use serde::Deserialize;

mod block;
mod block_type;
mod error;

use block::{Block, BlockMut};
use error::{BhError, BhResult};

#[derive(Debug)]
struct Map(HashMap<String, Vec<u8>>);

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
    blockheads: Vec<u8>,       // Vec<Blockheads>
    dynamic_world_v2: Vec<u8>, // ???
    world_v2: WorldV2,
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

// helper function to check if given coord is smaller than max value.
fn check_coord_limit(val: u64, max_val: u64) -> BhResult<()> {
    match val < max_val {
        false => Err(BhError::CoordError {
            input: val,
            limit: max_val,
        }),
        true => Ok(()),
    }
}

/// Block coordinate within a chunk. 0 <= x < 32, 0 <= y < 32.
/// Block coords within the chunk and their corresponding offset:
/// ```
/// 31| 992| 993| 994|     1023|
/// 30| 960| 961| 962|      991|
///              ...
///  2|  64|  65|  66|       95|
///  1|  32|  33|  34|       63|
///  0|   0|   1|   2|       31|
///  Y`----|----|----|---------|
///   X   0|   1|   2|  ...  31|
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ChunkBlockCoord {
    x: u8,
    y: u8,
}

impl ChunkBlockCoord {
    pub fn new(x: u8, y: u8) -> BhResult<Self> {
        check_coord_limit(x as u64, 32)?;
        check_coord_limit(y as u64, 32)?;
        Ok(Self { x, y })
    }

    fn to_offset(&self) -> usize {
        ((self.y as usize) << 5 | (self.x as usize)) << 6
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

    fn inner(&self) -> &[u8] {
        &self.0
    }

    fn inner_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    fn block_at(&self, coord: &ChunkBlockCoord) -> Block {
        let offset = coord.to_offset();
        let slice = <&[u8; 64]>::try_from(&self.inner()[offset..offset + NUM_BYTES_PER_BLOCK])
            .expect(
                "Return value of `ChunkBlockCoord.to_offset()` is guaranteed \
                to be smaller than chunk bytes len - 32",
            );
        Block::new(slice)
    }

    fn block_at_mut(&mut self, coord: &ChunkBlockCoord) -> BlockMut {
        let offset = coord.to_offset();
        let slice =
            <&mut [u8; 64]>::try_from(&mut self.inner_mut()[offset..offset + NUM_BYTES_PER_BLOCK])
                .expect(
                    "Return value of `ChunkBlockCoord.to_offset()` is guaranteed \
                to be smaller than chunk bytes len - 32",
                );
        BlockMut::new(slice)
    }

    fn format_grid(&self) -> String {
        let mut result = String::new();

        for y in (0..NUM_BLOCK_PER_COL).rev() {
            let mut row_strings = Vec::new();
            for x in 0..NUM_BLOCK_PER_ROW {
                let coord = ChunkBlockCoord::new(x as u8, y as u8).expect("Must be valid coord");
                let block = self.block_at(&coord);

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

/// Chunk coordinates in world. 0 <= x < world_v2.world_width_macro, 0 <= y < 32
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ChunkCoord {
    x: u64,
    y: u8,
}

impl ChunkCoord {
    /// Attempts to create a `ChunkCoord` from a string in the format "x_y".
    /// Returns `Err(BhError::ParseError)` for malformed strings or invalid numbers,
    /// or `Err(BhError::CoordError)` if coordinates are out of their initial type bounds.
    pub fn from_str<S: AsRef<str>>(s: S) -> BhResult<Self> {
        let s = s.as_ref();
        let mut parts = s.split('_');

        let x_str = parts
            .next()
            .ok_or_else(|| BhError::ParseError(format!("Missing x coordinate in {}", s)))?;
        let y_str = parts
            .next()
            .ok_or_else(|| BhError::ParseError(format!("Missing y coordinate in {}", s)))?;

        if parts.next().is_some() {
            return Err(BhError::ParseError(format!(
                "Too many parts in coordinate {}. Expected 'x_y' format.",
                s
            )));
        }

        let x = x_str.parse::<u64>().map_err(|e| {
            BhError::ParseError(format!("Failed to parse x coordinate as u64: {}", e))
        })?;
        let y = y_str.parse::<u8>().map_err(|e| {
            BhError::ParseError(format!("Failed to parse y coordinate as u8: {}", e))
        })?;

        Self::new(x, y)
    }

    /// Creates a new `ChunkCoord` after validating its coordinates.
    /// Returns `Err(BhError::CoordError)` if `y` is out of its valid range (0..32).
    pub fn new(x: u64, y: u8) -> BhResult<Self> {
        check_coord_limit(y as u64, 32)?;
        Ok(Self { x, y })
    }
}

/// Block coordinates in world. 0 <= x < world_v2.world_width_macro * 32, 0 <= y < 1024
#[derive(Debug, Clone, Copy)]
pub struct BlockCoord {
    x: u64,
    y: u16,
}

impl BlockCoord {
    /// Creates a new `BlockCoord` after validating its coordinates.
    /// Returns `Err(BhError::CoordError)` if `y` is out of its valid range (0..1024).
    pub fn new(x: u64, y: u16) -> BhResult<Self> {
        check_coord_limit(y as u64, 1024)?;
        Ok(Self { x, y: y as u16 })
    }

    pub fn into_decomposed(self) -> (ChunkCoord, ChunkBlockCoord) {
        (
            ChunkCoord::new(self.x >> 5, (self.y >> 5) as u8).expect("y < 1024, thus y >> 5 < 32"),
            ChunkBlockCoord::new((self.x & 31) as u8, (self.y & 31) as u8)
                .expect("x & 31 < 32 for all x: u64"),
        )
    }
}

impl Into<ChunkCoord> for BlockCoord {
    fn into(self) -> ChunkCoord {
        self.into_decomposed().0
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
        let (chunk_coord, chunk_block_coord) = block_coord.into_decomposed();
        self.0.get_mut(&chunk_coord).map(|v| {
            v.as_uncompressed_mut()
                .map(|chunk| chunk.block_at(&chunk_block_coord))
        })
    }

    pub fn block_at_mut<I: Into<BlockCoord>>(
        &mut self,
        coord: I,
    ) -> Option<std::io::Result<BlockMut>> {
        let block_coord = coord.into();
        let (chunk_coord, chunk_block_coord) = block_coord.into_decomposed();
        self.0.get_mut(&chunk_coord).map(|v| {
            v.as_uncompressed_mut()
                .map(|chunk| chunk.block_at_mut(&chunk_block_coord))
        })
    }
}

#[derive(Debug)]
pub struct WorldDb {
    blocks: Chunks,
    dw: Map, // ???
    main: WorldDbMain,
}

impl WorldDb {
    fn from_path<P: AsRef<Path>>(path: P) -> BhResult<Option<Self>> {
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

fn dump_to_stdout(bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    std::io::stdout().write_all(bytes)?;
    std::io::stdout().flush()?; // Make sure all bytes are written immediately
    Ok(())
}

fn main() -> BhResult<()> {
    let world_db = WorldDb::from_path("test_data/saves/3d716d9bbf89c77ef5001e9cd227ec29/world_db")?;
    if let Some(mut world_db) = world_db {
        // dump_to_stdout(
        //     world_db
        //         .main
        //         .world_v2
        //         .circum_navigate_booleans_data
        //         .as_ref(),
        // )
        // .unwrap();
        // let a = plist::from_bytes::<plist::Value>(world_db.main.world_v2.found_items.as_ref());
        // dbg!(a);
        // dbg!(world_db.main.world_v2);
        // dump_to_stdout(.as_ref());
        // dbg!(world_db.blocks.keys().collect::<Vec<_>>());
        // world_db.blocks.at_mut(coord)
        // let world_v2 = &mut world_db.main.world_v2;
        let x = world_db.main.world_v2.start_portal_pos_x;
        let y = world_db.main.world_v2.start_portal_pos_y;
        dbg!(x, y);
        let start_portal_pos = BlockCoord::new(x, (y - 1) as u16)?;
        let block = world_db.blocks.block_at(start_portal_pos).unwrap()?;
        let block_type = block.fg()?;
        dbg!(block_type.as_str());
        let chunk = world_db.blocks.chunk_at(start_portal_pos).unwrap()?;
        println!("{}", chunk.format_grid());
        // let chunk = world_db.blocks.chunk_at().unwrap()?;
        // chunk.block_at(&ChunkBlockCoord::new(x & 31, y & 31));
    }
    Ok(())
}
