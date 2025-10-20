use std::sync::{Arc, Mutex};

use pyo3::{
    create_exception,
    exceptions::{PyException, PyValueError},
    prelude::*,
};
use the_blockheads_tools_lib as lib;

pub type SharedWorldDb = Arc<Mutex<lib::game::db::world_db::WorldDb>>;

pub fn into_py_err(err: lib::BhError) -> PyErr {
    let error_message = err.to_string();

    match err {
        lib::BhError::DbError(_)
        | lib::BhError::PlistError(_)
        | lib::BhError::GzipError(_)
        | lib::BhError::MissingKey(_) => PyException::new_err(error_message),

        lib::BhError::CoordError { .. }
        | lib::BhError::ParseError(_)
        | lib::BhError::InvalidBlockIdError(_)
        | lib::BhError::InvalidBlockContentIdError(_)
        | lib::BhError::InvalidDynamicOjectId(_) => PyValueError::new_err(error_message),
    }
}

// For accessors, they might point to no-longer valid resources.
// ```py
// world_db = WorldDb.open("save_file")
// chunk = world_db.chunks.chunk_at((432, 10))
// world_db.chunks.remove((432, 10))
// chunk.block_at((7, 13))  # InvalidAccessorError!
// ```
create_exception!(
    the_blockheads_tools_py,
    InvalidAccessorError,
    pyo3::exceptions::PyException
);

mod coord {
    use std::hash::Hash;

    use crate::{into_py_err, lib};
    use lib::game::coord::{BlockCoord, ChunkBlockCoord, ChunkCoord};
    use pyo3::prelude::*;

    #[derive(Clone, Hash, PartialEq, Eq)]
    #[pyclass(frozen, eq, hash, name = "ChunkCoord")]
    pub struct ChunkCoordPy {
        pub(crate) inner: ChunkCoord,
    }

    #[pymethods]
    impl ChunkCoordPy {
        #[new]
        fn new(x: u32, y: u8) -> PyResult<Self> {
            Ok(Self {
                inner: ChunkCoord::new(x, y).map_err(into_py_err)?,
            })
        }

        #[getter]
        fn get_x(&self) -> u32 {
            self.inner.x()
        }

        #[getter]
        fn get_y(&self) -> u8 {
            self.inner.y()
        }

        fn __str__(&self) -> String {
            self.inner.to_string()
        }

        fn __repr__(&self) -> String {
            format!("ChunkCoord({}, {})", self.inner.x(), self.inner.y())
        }
    }

    #[derive(Clone, Hash, PartialEq, Eq)]
    #[pyclass(frozen, eq, hash, name = "ChunkBlockCoord")]
    pub struct ChunkBlockCoordPy {
        pub(crate) inner: ChunkBlockCoord,
    }

    #[pymethods]
    impl ChunkBlockCoordPy {
        #[new]
        fn new(x: u8, y: u8) -> PyResult<Self> {
            Ok(Self {
                inner: ChunkBlockCoord::new(x, y).map_err(into_py_err)?,
            })
        }

        #[getter]
        fn get_x(&self) -> u8 {
            self.inner.x()
        }

        #[getter]
        fn get_y(&self) -> u8 {
            self.inner.y()
        }

        fn __str__(&self) -> String {
            self.inner.to_string()
        }

        fn __repr__(&self) -> String {
            self.__str__()
        }
    }

    #[derive(Clone, Hash, PartialEq, Eq)]
    #[pyclass(frozen, eq, hash, name = "BlockCoord")]
    pub struct BlockCoordPy {
        pub(crate) inner: BlockCoord,
    }

    #[pymethods]
    impl BlockCoordPy {
        #[new]
        fn new(x: u32, y: u16) -> PyResult<Self> {
            Ok(Self {
                inner: BlockCoord::new(x, y).map_err(into_py_err)?,
            })
        }

        #[getter]
        fn get_x(&self) -> u32 {
            self.inner.x()
        }

        #[getter]
        fn get_y(&self) -> u16 {
            self.inner.y()
        }

        fn __str__(&self) -> String {
            self.inner.to_string()
        }

        fn __repr__(&self) -> String {
            self.__str__()
        }
    }
}

pub use coord::{BlockCoordPy, ChunkBlockCoordPy, ChunkCoordPy};

mod block {
    use crate::{into_py_err, lib, InvalidAccessorError, SharedWorldDb};
    use lib::game::{
        block::{BlockType, BlockView},
        coord::BlockCoord,
    };
    use pyo3::prelude::*;

    #[pyclass(eq, eq_int, name = "BlockType")]
    #[derive(PartialEq)]
    #[repr(u8)]
    pub enum BlockTypePy {
        Stone = 1,
        Air = 2,
        Water = 3,
        Ice = 4,
        Snow = 5,
        Dirt = 6,
        DesertSand = 7,
        BeachSand = 8,
        Wood = 9,
        MinedStone = 10,
        RedBrick = 11,
        Limestone = 12,
        MinedLimestone = 13,
        Marble = 14,
        MinedMarble = 15,
        TimeCrystal = 16,
        SandStone = 17,
        MinedSandStone = 18,
        RedMarble = 19,
        MinedRedMarble = 20,
        Glass = 24,
        SpawnPortalBase = 25,
        GoldBlock = 26,
        GrassDirt = 27,
        SnowDirt = 28,
        LapisLazuli = 29,
        MinedLapisLazuli = 30,
        Lava = 31,
        ReinforcedPlatform = 32,
        SpawnPortalBaseAmethyst = 33,
        SpawnPortalBaseSapphire = 34,
        SpawnPortalBaseEmerald = 35,
        SpawnPortalBaseRuby = 36,
        SpawnPortalBaseDiamond = 37,
        NorthPole = 38,
        SouthPole = 39,
        WestPole = 40,
        EastPole = 41,
        PortalBase = 42,
        PortalBaseAmethyst = 43,
        PortalBaseSapphire = 44,
        PortalBaseEmerald = 45,
        PortalBaseRuby = 46,
        PortalBaseDiamond = 47,
        Compost = 48,
        GrassCompost = 49,
        SnowCompost = 50,
        Basalt = 51,
        MinedBasalt = 52,
        CopperBlock = 53,
        TinBlock = 54,
        BronzeBlock = 55,
        IronBlock = 56,
        SteelBlock = 57,
        BlackSand = 58,
        BlackGlass = 59,
        TradePortalBase = 60,
        TradePortalBaseAmethyst = 61,
        TradePortalBaseSapphire = 62,
        TradePortalBaseEmerald = 63,
        TradePortalBaseRuby = 64,
        TradePortalBaseDiamond = 65,
        PlatinumBlock = 67,
        TitaniumBlock = 68,
        CarbonFiberBlock = 69,
        Gravel = 70,
        AmethystBlock = 71,
        SapphireBlock = 72,
        EmeraldBlock = 73,
        RubyBlock = 74,
        DiamondBlock = 75,
        Plaster = 76,
        LuminousPlaster = 77,
    }

    impl From<BlockType> for BlockTypePy {
        fn from(value: BlockType) -> Self {
            match value {
                BlockType::Stone => Self::Stone,
                BlockType::Air => Self::Air,
                BlockType::Water => Self::Water,
                BlockType::Ice => Self::Ice,
                BlockType::Snow => Self::Snow,
                BlockType::Dirt => Self::Dirt,
                BlockType::DesertSand => Self::DesertSand,
                BlockType::BeachSand => Self::BeachSand,
                BlockType::Wood => Self::Wood,
                BlockType::MinedStone => Self::MinedStone,
                BlockType::RedBrick => Self::RedBrick,
                BlockType::Limestone => Self::Limestone,
                BlockType::MinedLimestone => Self::MinedLimestone,
                BlockType::Marble => Self::Marble,
                BlockType::MinedMarble => Self::MinedMarble,
                BlockType::TimeCrystal => Self::TimeCrystal,
                BlockType::SandStone => Self::SandStone,
                BlockType::MinedSandStone => Self::MinedSandStone,
                BlockType::RedMarble => Self::RedMarble,
                BlockType::MinedRedMarble => Self::MinedRedMarble,
                BlockType::Glass => Self::Glass,
                BlockType::SpawnPortalBase => Self::SpawnPortalBase,
                BlockType::GoldBlock => Self::GoldBlock,
                BlockType::GrassDirt => Self::GrassDirt,
                BlockType::SnowDirt => Self::SnowDirt,
                BlockType::LapisLazuli => Self::LapisLazuli,
                BlockType::MinedLapisLazuli => Self::MinedLapisLazuli,
                BlockType::Lava => Self::Lava,
                BlockType::ReinforcedPlatform => Self::ReinforcedPlatform,
                BlockType::SpawnPortalBaseAmethyst => Self::SpawnPortalBaseAmethyst,
                BlockType::SpawnPortalBaseSapphire => Self::SpawnPortalBaseSapphire,
                BlockType::SpawnPortalBaseEmerald => Self::SpawnPortalBaseEmerald,
                BlockType::SpawnPortalBaseRuby => Self::SpawnPortalBaseRuby,
                BlockType::SpawnPortalBaseDiamond => Self::SpawnPortalBaseDiamond,
                BlockType::NorthPole => Self::NorthPole,
                BlockType::SouthPole => Self::SouthPole,
                BlockType::WestPole => Self::WestPole,
                BlockType::EastPole => Self::EastPole,
                BlockType::PortalBase => Self::PortalBase,
                BlockType::PortalBaseAmethyst => Self::PortalBaseAmethyst,
                BlockType::PortalBaseSapphire => Self::PortalBaseSapphire,
                BlockType::PortalBaseEmerald => Self::PortalBaseEmerald,
                BlockType::PortalBaseRuby => Self::PortalBaseRuby,
                BlockType::PortalBaseDiamond => Self::PortalBaseDiamond,
                BlockType::Compost => Self::Compost,
                BlockType::GrassCompost => Self::GrassCompost,
                BlockType::SnowCompost => Self::SnowCompost,
                BlockType::Basalt => Self::Basalt,
                BlockType::MinedBasalt => Self::MinedBasalt,
                BlockType::CopperBlock => Self::CopperBlock,
                BlockType::TinBlock => Self::TinBlock,
                BlockType::BronzeBlock => Self::BronzeBlock,
                BlockType::IronBlock => Self::IronBlock,
                BlockType::SteelBlock => Self::SteelBlock,
                BlockType::BlackSand => Self::BlackSand,
                BlockType::BlackGlass => Self::BlackGlass,
                BlockType::TradePortalBase => Self::TradePortalBase,
                BlockType::TradePortalBaseAmethyst => Self::TradePortalBaseAmethyst,
                BlockType::TradePortalBaseSapphire => Self::TradePortalBaseSapphire,
                BlockType::TradePortalBaseEmerald => Self::TradePortalBaseEmerald,
                BlockType::TradePortalBaseRuby => Self::TradePortalBaseRuby,
                BlockType::TradePortalBaseDiamond => Self::TradePortalBaseDiamond,
                BlockType::PlatinumBlock => Self::PlatinumBlock,
                BlockType::TitaniumBlock => Self::TitaniumBlock,
                BlockType::CarbonFiberBlock => Self::CarbonFiberBlock,
                BlockType::Gravel => Self::Gravel,
                BlockType::AmethystBlock => Self::AmethystBlock,
                BlockType::SapphireBlock => Self::SapphireBlock,
                BlockType::EmeraldBlock => Self::EmeraldBlock,
                BlockType::RubyBlock => Self::RubyBlock,
                BlockType::DiamondBlock => Self::DiamondBlock,
                BlockType::Plaster => Self::Plaster,
                BlockType::LuminousPlaster => Self::LuminousPlaster,
            }
        }
    }

    impl Into<BlockType> for BlockTypePy {
        fn into(self) -> BlockType {
            match self {
                Self::Stone => BlockType::Stone,
                Self::Air => BlockType::Air,
                Self::Water => BlockType::Water,
                Self::Ice => BlockType::Ice,
                Self::Snow => BlockType::Snow,
                Self::Dirt => BlockType::Dirt,
                Self::DesertSand => BlockType::DesertSand,
                Self::BeachSand => BlockType::BeachSand,
                Self::Wood => BlockType::Wood,
                Self::MinedStone => BlockType::MinedStone,
                Self::RedBrick => BlockType::RedBrick,
                Self::Limestone => BlockType::Limestone,
                Self::MinedLimestone => BlockType::MinedLimestone,
                Self::Marble => BlockType::Marble,
                Self::MinedMarble => BlockType::MinedMarble,
                Self::TimeCrystal => BlockType::TimeCrystal,
                Self::SandStone => BlockType::SandStone,
                Self::MinedSandStone => BlockType::MinedSandStone,
                Self::RedMarble => BlockType::RedMarble,
                Self::MinedRedMarble => BlockType::MinedRedMarble,
                Self::Glass => BlockType::Glass,
                Self::SpawnPortalBase => BlockType::SpawnPortalBase,
                Self::GoldBlock => BlockType::GoldBlock,
                Self::GrassDirt => BlockType::GrassDirt,
                Self::SnowDirt => BlockType::SnowDirt,
                Self::LapisLazuli => BlockType::LapisLazuli,
                Self::MinedLapisLazuli => BlockType::MinedLapisLazuli,
                Self::Lava => BlockType::Lava,
                Self::ReinforcedPlatform => BlockType::ReinforcedPlatform,
                Self::SpawnPortalBaseAmethyst => BlockType::SpawnPortalBaseAmethyst,
                Self::SpawnPortalBaseSapphire => BlockType::SpawnPortalBaseSapphire,
                Self::SpawnPortalBaseEmerald => BlockType::SpawnPortalBaseEmerald,
                Self::SpawnPortalBaseRuby => BlockType::SpawnPortalBaseRuby,
                Self::SpawnPortalBaseDiamond => BlockType::SpawnPortalBaseDiamond,
                Self::NorthPole => BlockType::NorthPole,
                Self::SouthPole => BlockType::SouthPole,
                Self::WestPole => BlockType::WestPole,
                Self::EastPole => BlockType::EastPole,
                Self::PortalBase => BlockType::PortalBase,
                Self::PortalBaseAmethyst => BlockType::PortalBaseAmethyst,
                Self::PortalBaseSapphire => BlockType::PortalBaseSapphire,
                Self::PortalBaseEmerald => BlockType::PortalBaseEmerald,
                Self::PortalBaseRuby => BlockType::PortalBaseRuby,
                Self::PortalBaseDiamond => BlockType::PortalBaseDiamond,
                Self::Compost => BlockType::Compost,
                Self::GrassCompost => BlockType::GrassCompost,
                Self::SnowCompost => BlockType::SnowCompost,
                Self::Basalt => BlockType::Basalt,
                Self::MinedBasalt => BlockType::MinedBasalt,
                Self::CopperBlock => BlockType::CopperBlock,
                Self::TinBlock => BlockType::TinBlock,
                Self::BronzeBlock => BlockType::BronzeBlock,
                Self::IronBlock => BlockType::IronBlock,
                Self::SteelBlock => BlockType::SteelBlock,
                Self::BlackSand => BlockType::BlackSand,
                Self::BlackGlass => BlockType::BlackGlass,
                Self::TradePortalBase => BlockType::TradePortalBase,
                Self::TradePortalBaseAmethyst => BlockType::TradePortalBaseAmethyst,
                Self::TradePortalBaseSapphire => BlockType::TradePortalBaseSapphire,
                Self::TradePortalBaseEmerald => BlockType::TradePortalBaseEmerald,
                Self::TradePortalBaseRuby => BlockType::TradePortalBaseRuby,
                Self::TradePortalBaseDiamond => BlockType::TradePortalBaseDiamond,
                Self::PlatinumBlock => BlockType::PlatinumBlock,
                Self::TitaniumBlock => BlockType::TitaniumBlock,
                Self::CarbonFiberBlock => BlockType::CarbonFiberBlock,
                Self::Gravel => BlockType::Gravel,
                Self::AmethystBlock => BlockType::AmethystBlock,
                Self::SapphireBlock => BlockType::SapphireBlock,
                Self::EmeraldBlock => BlockType::EmeraldBlock,
                Self::RubyBlock => BlockType::RubyBlock,
                Self::DiamondBlock => BlockType::DiamondBlock,
                Self::Plaster => BlockType::Plaster,
                Self::LuminousPlaster => BlockType::LuminousPlaster,
            }
        }
    }

    #[pyclass(name = "Block")]
    pub struct BlockPy {
        pub(crate) world_db: SharedWorldDb,
        pub(crate) block_coord: BlockCoord,
    }

    #[pymethods]
    impl BlockPy {
        fn fg(&self) -> PyResult<BlockTypePy> {
            let mut world_db = self.world_db.lock().unwrap();
            let block = world_db.chunks.block_at(self.block_coord);
            match block {
                Some(block) => {
                    let fg_type = block?.fg().map_err(into_py_err)?;
                    Ok(fg_type.into())
                }
                None => Err(InvalidAccessorError::new_err(format!(
                    "The block at {} doesn't exist.",
                    self.block_coord.to_string()
                ))),
            }
        }
    }
}

pub use block::{BlockPy, BlockTypePy};

mod chunk {
    use std::{borrow::Cow, collections::HashSet};

    use crate::{
        lib, BlockCoordPy, BlockPy, ChunkBlockCoordPy, ChunkCoordPy, InvalidAccessorError,
        SharedWorldDb,
    };
    use lib::game::coord::{BlockCoord, ChunkCoord};
    use pyo3::prelude::*;

    #[pyclass(name = "Chunk")]
    pub struct ChunkPy {
        world_db: SharedWorldDb,
        coord: ChunkCoord,
    }

    #[pymethods]
    impl ChunkPy {
        fn as_bytes(&'_ self) -> PyResult<Cow<'_, [u8]>> {
            let mut world_db = self.world_db.lock().unwrap();
            let chunk = world_db.chunks.chunk_at(self.coord);
            match chunk {
                Some(chunk) => Ok(Cow::Owned(chunk?.inner().to_owned())),
                None => Err(InvalidAccessorError::new_err(format!(
                    "The chunk at {} doesn't exist.",
                    self.coord.to_string()
                ))),
            }
        }

        fn block_at(&self, coord: ChunkBlockCoordPy) -> BlockPy {
            BlockPy {
                world_db: self.world_db.clone(),
                block_coord: BlockCoord::from_decomposed(self.coord, coord.inner),
            }
        }
    }

    #[derive(FromPyObject)]
    enum IntoChunkCoord {
        BlockCoord(BlockCoordPy),
        ChunkCoord(ChunkCoordPy),
    }

    #[pyclass(name = "Chunks")]
    pub struct ChunksPy {
        pub(crate) world_db: SharedWorldDb,
    }

    #[pymethods]
    impl ChunksPy {
        fn __contains__(&self, coord: &ChunkCoordPy) -> bool {
            let world_db = self.world_db.lock().unwrap();
            world_db.chunks.contains_key(&coord.inner)
        }

        fn keys(&self) -> HashSet<ChunkCoordPy> {
            // Ton's of allocation, but I guess python users will be ok with that.
            let world_db = self.world_db.lock().unwrap();
            HashSet::from_iter(
                world_db
                    .chunks
                    .keys()
                    .into_iter()
                    .copied()
                    .map(|value| ChunkCoordPy { inner: value }),
            )
        }

        fn chunk_at(&self, coord: IntoChunkCoord) -> Option<ChunkPy> {
            let chunk_coord = match coord {
                IntoChunkCoord::BlockCoord(block_coord_py) => {
                    let (chunk_coord, _) = block_coord_py.inner.decompose();
                    chunk_coord
                }
                IntoChunkCoord::ChunkCoord(chunk_coord_py) => chunk_coord_py.inner,
            };
            let world_db = self.world_db.lock().unwrap();
            world_db.chunks.contains_key(&chunk_coord).then(|| ChunkPy {
                world_db: self.world_db.clone(),
                coord: chunk_coord,
            })
        }

        fn block_at(&self, coord: BlockCoordPy) -> Option<BlockPy> {
            let (chunk_coord, _) = coord.inner.decompose();
            let world_db = self.world_db.lock().unwrap();
            world_db
                .chunks
                .contains_key(&chunk_coord)
                .then_some(BlockPy {
                    world_db: self.world_db.clone(),
                    block_coord: coord.inner,
                })
        }
    }
}

pub use chunk::{ChunkPy, ChunksPy};

mod world_db {
    use std::{
        borrow::Cow,
        sync::{Arc, Mutex},
    };

    use crate::{into_py_err, lib, ChunksPy, SharedWorldDb};
    use lib::game::db::world_db::WorldDb;
    use pyo3::prelude::*;

    #[pyclass(name = "WorldV2")]
    pub struct WorldV2Py {
        inner: SharedWorldDb,
    }

    #[pymethods]
    impl WorldV2Py {
        #[getter]
        fn get_blockhead_datas_v2(&self) -> String {
            let world_db = self.inner.lock().unwrap();
            format!("{:?}", world_db.main.world_v2.blockhead_datas_v2)
        }

        #[getter]
        fn get_circum_navigate_booleans_data(&'_ self) -> Cow<'_, [u8]> {
            let world_db = self.inner.lock().unwrap();
            Cow::Owned(
                world_db
                    .main
                    .world_v2
                    .circum_navigate_booleans_data
                    .as_ref()
                    .to_vec(),
            )
        }

        #[setter]
        fn set_circum_navigate_booleans_data(&self, value: Vec<u8>) {
            let mut world_db = self.inner.lock().unwrap();
            world_db.main.world_v2.circum_navigate_booleans_data = value.into();
        }

        #[getter]
        fn get_creation_date(&self) -> String {
            let world_db = self.inner.lock().unwrap();
            format!("{:?}", world_db.main.world_v2.creation_date)
        }

        #[getter]
        fn get_distance_ordered_food_types(&self) -> Cow<'_, [u8]> {
            let world_db = self.inner.lock().unwrap();
            Cow::Owned(
                world_db
                    .main
                    .world_v2
                    .distance_ordered_food_types
                    .as_ref()
                    .to_vec(),
            )
        }

        #[setter]
        fn set_distance_ordered_food_types(&self, value: Vec<u8>) {
            let mut world_db = self.inner.lock().unwrap();
            world_db.main.world_v2.distance_ordered_food_types = value.into();
        }

        #[getter]
        fn get_expert_mode(&self) -> bool {
            self.inner.lock().unwrap().main.world_v2.expert_mode
        }

        #[setter]
        fn set_expert_mode(&self, value: bool) {
            self.inner.lock().unwrap().main.world_v2.expert_mode = value;
        }

        #[getter]
        fn get_found_items(&'_ self) -> Cow<'_, [u8]> {
            let world_db = self.inner.lock().unwrap();
            Cow::Owned(world_db.main.world_v2.found_items.as_ref().to_vec())
        }

        #[setter]
        fn set_found_items(&self, value: Vec<u8>) {
            self.inner.lock().unwrap().main.world_v2.found_items = value.into();
        }

        #[getter]
        fn get_host_port(&self) -> String {
            self.inner.lock().unwrap().main.world_v2.host_port.clone()
        }

        #[setter]
        fn set_host_port(&self, value: &str) {
            self.inner.lock().unwrap().main.world_v2.host_port = value.to_string();
        }

        #[getter]
        fn get_max_players(&self) -> String {
            self.inner.lock().unwrap().main.world_v2.max_players.clone()
        }

        #[setter]
        fn set_max_players(&self, value: &str) {
            self.inner.lock().unwrap().main.world_v2.max_players = value.to_string();
        }

        #[getter]
        fn get_migration_complete_v1_7(&self) -> bool {
            self.inner
                .lock()
                .unwrap()
                .main
                .world_v2
                .migration_complete_v1_7
        }

        #[setter]
        fn set_migration_complete_v1_7(&self, value: bool) {
            self.inner
                .lock()
                .unwrap()
                .main
                .world_v2
                .migration_complete_v1_7 = value;
        }

        #[getter]
        fn get_no_rain_timer(&self) -> f64 {
            self.inner.lock().unwrap().main.world_v2.no_rain_timer
        }

        #[setter]
        fn set_no_rain_timer(&self, value: f64) {
            self.inner.lock().unwrap().main.world_v2.no_rain_timer = value;
        }

        #[getter]
        fn get_portal_level(&self) -> u64 {
            self.inner.lock().unwrap().main.world_v2.portal_level
        }

        #[setter]
        fn set_portal_level(&self, value: u64) {
            self.inner.lock().unwrap().main.world_v2.portal_level = value;
        }

        #[getter]
        fn get_random_seed(&self) -> u64 {
            self.inner.lock().unwrap().main.world_v2.random_seed
        }

        #[setter]
        fn set_random_seed(&self, value: u64) {
            self.inner.lock().unwrap().main.world_v2.random_seed = value;
        }

        #[getter]
        fn get_remote_game(&self) -> bool {
            self.inner.lock().unwrap().main.world_v2.remote_game
        }

        #[setter]
        fn set_remote_game(&self, value: bool) {
            self.inner.lock().unwrap().main.world_v2.remote_game = value;
        }

        #[getter]
        fn get_run_at_launch(&self) -> bool {
            self.inner.lock().unwrap().main.world_v2.run_at_launch
        }

        #[setter]
        fn set_run_at_launch(&self, value: bool) {
            self.inner.lock().unwrap().main.world_v2.run_at_launch = value;
        }

        #[getter]
        fn get_save_date(&self) -> String {
            let world_db = self.inner.lock().unwrap();
            format!("{:?}", world_db.main.world_v2.save_date)
        }

        #[getter]
        fn get_save_id(&self) -> String {
            self.inner.lock().unwrap().main.world_v2.save_id.clone()
        }

        #[setter]
        fn set_save_id(&self, value: &str) {
            self.inner.lock().unwrap().main.world_v2.save_id = value.to_string();
        }

        #[getter]
        fn get_save_version(&self) -> u64 {
            self.inner.lock().unwrap().main.world_v2.save_version
        }

        #[setter]
        fn set_save_version(&self, value: u64) {
            self.inner.lock().unwrap().main.world_v2.save_version = value;
        }

        #[getter]
        fn get_start_portal_pos_x(&self) -> u64 {
            self.inner.lock().unwrap().main.world_v2.start_portal_pos_x
        }

        #[setter]
        fn set_start_portal_pos_x(&self, value: u64) {
            self.inner.lock().unwrap().main.world_v2.start_portal_pos_x = value;
        }

        #[getter]
        fn get_start_portal_pos_y(&self) -> u64 {
            self.inner.lock().unwrap().main.world_v2.start_portal_pos_y
        }

        #[setter]
        fn set_start_portal_pos_y(&self, value: u64) {
            self.inner.lock().unwrap().main.world_v2.start_portal_pos_y = value;
        }

        #[getter]
        fn get_translation(&self) -> (f64, f64) {
            self.inner.lock().unwrap().main.world_v2.translation
        }

        #[setter]
        fn set_translation(&self, value: (f64, f64)) {
            self.inner.lock().unwrap().main.world_v2.translation = value;
        }

        #[getter]
        fn get_world_name(&self) -> String {
            self.inner.lock().unwrap().main.world_v2.world_name.clone()
        }

        #[setter]
        fn set_world_name(&self, value: &str) {
            self.inner.lock().unwrap().main.world_v2.world_name = value.to_string();
        }

        #[getter]
        fn get_world_time(&self) -> f64 {
            self.inner.lock().unwrap().main.world_v2.world_time
        }

        #[setter]
        fn set_world_time(&self, value: f64) {
            self.inner.lock().unwrap().main.world_v2.world_time = value;
        }

        #[getter]
        fn get_world_width_macro(&self) -> u32 {
            self.inner.lock().unwrap().main.world_v2.world_width_macro
        }

        #[setter]
        fn set_world_width_macro(&self, value: u32) {
            self.inner.lock().unwrap().main.world_v2.world_width_macro = value;
        }

        fn __repr__(&self) -> String {
            format!("{:?}", self.inner.lock().unwrap().main.world_v2)
        }
    }

    #[pyclass(name = "WorldDbMain")]
    pub struct WorldDbMainPy {
        inner: SharedWorldDb,
    }

    #[pymethods]
    impl WorldDbMainPy {
        #[getter]
        fn get_blockheads(&'_ self) -> Cow<'_, [u8]> {
            Cow::Owned(self.inner.lock().unwrap().main.blockheads.to_vec())
        }

        #[getter]
        fn get_dynamic_world_v2(&'_ self) -> Cow<'_, [u8]> {
            Cow::Owned(self.inner.lock().unwrap().main.dynamic_world_v2.to_vec())
        }

        #[getter]
        fn get_world_v2(&self) -> WorldV2Py {
            WorldV2Py {
                inner: self.inner.clone(),
            }
        }
    }

    #[pyclass(name = "WorldDb")]
    pub struct WorldDbPy {
        // Python doesn't care about lifetimes. Thus we model the save file in the pythonic way.
        // This imposes severe runtime expense - each time some downstream accessor accesses some data in world_db,
        // we need to get the mutex lock, which is slow as hell.
        // Every object other than trivial ones like coords will hold a shared reference.
        inner: SharedWorldDb,
    }

    #[pymethods]
    impl WorldDbPy {
        #[staticmethod]
        fn open(path: &str) -> PyResult<Self> {
            let world_db = WorldDb::from_path(path).map_err(into_py_err)?;
            Ok(Self {
                inner: Arc::new(Mutex::new(world_db)),
            })
        }

        fn save(&self, path: &str) -> PyResult<()> {
            self.inner
                .lock()
                .unwrap()
                .to_path(path)
                .map_err(into_py_err)?;
            Ok(())
        }

        #[getter]
        fn get_chunks(&self) -> ChunksPy {
            ChunksPy {
                world_db: self.inner.clone(),
            }
        }

        #[getter]
        fn get_main(&self) -> WorldDbMainPy {
            WorldDbMainPy {
                inner: self.inner.clone(),
            }
        }
    }
}

pub use world_db::WorldDbPy;

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn the_blockheads_tools_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<BlockCoordPy>()?;
    m.add_class::<ChunkBlockCoordPy>()?;
    m.add_class::<ChunkCoordPy>()?;

    m.add_class::<BlockTypePy>()?;
    m.add_class::<BlockPy>()?;

    m.add_class::<ChunkPy>()?;
    m.add_class::<ChunksPy>()?;

    m.add_class::<WorldDbPy>()?;
    Ok(())
}
