use pyo3::{
    create_exception,
    exceptions::{PyException, PyValueError},
    prelude::*,
};
use std::sync::{Arc, RwLock};
use the_blockheads_tools_lib as lib;

pub type SharedWorldDb = Arc<RwLock<lib::game::db::world_db::WorldDb>>;

pub fn into_py_err(err: lib::BhError) -> PyErr {
    let error_message = err.to_string();

    match err {
        lib::BhError::LmdbError(_)
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
    use num_enum::TryFromPrimitive;
    use pyo3::prelude::*;

    #[pyclass(eq, eq_int, name = "BlockType")]
    #[derive(PartialEq, TryFromPrimitive)]
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
            Self::try_from(value as u8).expect("Enums are out of sync!")
        }
    }

    impl Into<BlockType> for BlockTypePy {
        fn into(self) -> BlockType {
            BlockType::try_from(self as u8).expect("Enums are out of sync!")
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
            let mut world_db = self.world_db.write().unwrap();
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
            let mut world_db = self.world_db.write().unwrap();
            let chunk = world_db.chunks.chunk_at_mut(self.coord);
            match chunk {
                Some(chunk) => Ok(Cow::Owned(chunk.as_uncompressed()?.inner().to_owned())),
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
            let world_db = self.world_db.read().unwrap();
            world_db.chunks.contains_key(coord.inner)
        }

        fn keys(&self) -> HashSet<ChunkCoordPy> {
            // Ton's of allocation, but I guess python users will be ok with that.
            let world_db = self.world_db.read().unwrap();
            HashSet::from_iter(
                world_db
                    .chunks
                    .keys()
                    .into_iter()
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
            let world_db = self.world_db.read().unwrap();
            world_db.chunks.contains_key(chunk_coord).then(|| ChunkPy {
                world_db: self.world_db.clone(),
                coord: chunk_coord,
            })
        }

        fn block_at(&self, coord: BlockCoordPy) -> Option<BlockPy> {
            let (chunk_coord, _) = coord.inner.decompose();
            let world_db = self.world_db.read().unwrap();
            world_db
                .chunks
                .contains_key(chunk_coord)
                .then_some(BlockPy {
                    world_db: self.world_db.clone(),
                    block_coord: coord.inner,
                })
        }
    }
}

pub use chunk::{ChunkPy, ChunksPy};

mod world_db {
    use super::{Arc, RwLock};
    use crate::{into_py_err, lib, ChunksPy, SharedWorldDb};
    use lib::game::{db::world_db::WorldDb, dw::dynamic_object::Blockhead};
    use pyo3::prelude::*;
    use std::{
        borrow::Cow,
        sync::{RwLockReadGuard, RwLockWriteGuard},
    };

    #[pyclass(name = "WorldV2")]
    pub struct WorldV2Py {
        inner: SharedWorldDb,
    }

    #[pymethods]
    impl WorldV2Py {
        #[getter]
        fn get_blockhead_datas_v2(&self) -> String {
            let world_db = self.inner.read().unwrap();
            format!("{:?}", world_db.main.world_v2.blockhead_datas_v2)
        }

        #[getter]
        fn get_circum_navigate_booleans_data(&'_ self) -> Cow<'_, [u8]> {
            let world_db = self.inner.read().unwrap();
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
            let mut world_db = self.inner.write().unwrap();
            world_db.main.world_v2.circum_navigate_booleans_data = value.into();
        }

        #[getter]
        fn get_creation_date(&self) -> String {
            let world_db = self.inner.read().unwrap();
            format!("{:?}", world_db.main.world_v2.creation_date)
        }

        #[getter]
        fn get_distance_ordered_food_types(&self) -> Cow<'_, [u8]> {
            let world_db = self.inner.read().unwrap();
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
            let mut world_db = self.inner.write().unwrap();
            world_db.main.world_v2.distance_ordered_food_types = value.into();
        }

        #[getter]
        fn get_expert_mode(&self) -> bool {
            self.inner.read().unwrap().main.world_v2.expert_mode
        }

        #[setter]
        fn set_expert_mode(&self, value: bool) {
            self.inner.write().unwrap().main.world_v2.expert_mode = value;
        }

        #[getter]
        fn get_found_items(&'_ self) -> Cow<'_, [u8]> {
            let world_db = self.inner.read().unwrap();
            Cow::Owned(world_db.main.world_v2.found_items.as_ref().to_vec())
        }

        #[setter]
        fn set_found_items(&self, value: Vec<u8>) {
            self.inner.write().unwrap().main.world_v2.found_items = value.into();
        }

        #[getter]
        fn get_host_port(&self) -> Option<String> {
            self.inner.read().unwrap().main.world_v2.host_port.clone()
        }

        #[setter]
        fn set_host_port(&self, value: Option<&str>) {
            self.inner.write().unwrap().main.world_v2.host_port = value.map(ToString::to_string);
        }

        #[getter]
        fn get_max_players(&self) -> Option<String> {
            self.inner.read().unwrap().main.world_v2.max_players.clone()
        }

        #[setter]
        fn set_max_players(&self, value: Option<&str>) {
            self.inner.write().unwrap().main.world_v2.max_players = value.map(ToString::to_string);
        }

        #[getter]
        fn get_migration_complete_v1_7(&self) -> bool {
            self.inner
                .read()
                .unwrap()
                .main
                .world_v2
                .migration_complete_v1_7
        }

        #[setter]
        fn set_migration_complete_v1_7(&self, value: bool) {
            self.inner
                .write()
                .unwrap()
                .main
                .world_v2
                .migration_complete_v1_7 = value;
        }

        #[getter]
        fn get_no_rain_timer(&self) -> f64 {
            self.inner.read().unwrap().main.world_v2.no_rain_timer
        }

        #[setter]
        fn set_no_rain_timer(&self, value: f64) {
            self.inner.write().unwrap().main.world_v2.no_rain_timer = value;
        }

        #[getter]
        fn get_portal_level(&self) -> u64 {
            self.inner.read().unwrap().main.world_v2.portal_level
        }

        #[setter]
        fn set_portal_level(&self, value: u64) {
            self.inner.write().unwrap().main.world_v2.portal_level = value;
        }

        #[getter]
        fn get_random_seed(&self) -> u64 {
            self.inner.read().unwrap().main.world_v2.random_seed
        }

        #[setter]
        fn set_random_seed(&self, value: u64) {
            self.inner.write().unwrap().main.world_v2.random_seed = value;
        }

        #[getter]
        fn get_remote_game(&self) -> bool {
            self.inner.read().unwrap().main.world_v2.remote_game
        }

        #[setter]
        fn set_remote_game(&self, value: bool) {
            self.inner.write().unwrap().main.world_v2.remote_game = value;
        }

        #[getter]
        fn get_run_at_launch(&self) -> bool {
            self.inner.read().unwrap().main.world_v2.run_at_launch
        }

        #[setter]
        fn set_run_at_launch(&self, value: bool) {
            self.inner.write().unwrap().main.world_v2.run_at_launch = value;
        }

        #[getter]
        fn get_save_date(&self) -> String {
            let world_db = self.inner.read().unwrap();
            format!("{:?}", world_db.main.world_v2.save_date)
        }

        #[getter]
        fn get_save_id(&self) -> String {
            self.inner.read().unwrap().main.world_v2.save_id.clone()
        }

        #[setter]
        fn set_save_id(&self, value: &str) {
            self.inner.write().unwrap().main.world_v2.save_id = value.to_string();
        }

        #[getter]
        fn get_save_version(&self) -> u64 {
            self.inner.read().unwrap().main.world_v2.save_version
        }

        #[setter]
        fn set_save_version(&self, value: u64) {
            self.inner.write().unwrap().main.world_v2.save_version = value;
        }

        #[getter]
        fn get_start_portal_pos_x(&self) -> u64 {
            self.inner.read().unwrap().main.world_v2.start_portal_pos_x
        }

        #[setter]
        fn set_start_portal_pos_x(&self, value: u64) {
            self.inner.write().unwrap().main.world_v2.start_portal_pos_x = value;
        }

        #[getter]
        fn get_start_portal_pos_y(&self) -> u64 {
            self.inner.read().unwrap().main.world_v2.start_portal_pos_y
        }

        #[setter]
        fn set_start_portal_pos_y(&self, value: u64) {
            self.inner.write().unwrap().main.world_v2.start_portal_pos_y = value;
        }

        #[getter]
        fn get_translation(&self) -> (f64, f64) {
            self.inner.read().unwrap().main.world_v2.translation
        }

        #[setter]
        fn set_translation(&self, value: (f64, f64)) {
            self.inner.write().unwrap().main.world_v2.translation = value;
        }

        #[getter]
        fn get_world_name(&self) -> String {
            self.inner.read().unwrap().main.world_v2.world_name.clone()
        }

        #[setter]
        fn set_world_name(&self, value: &str) {
            self.inner.write().unwrap().main.world_v2.world_name = value.to_string();
        }

        #[getter]
        fn get_world_time(&self) -> f64 {
            self.inner.read().unwrap().main.world_v2.world_time
        }

        #[setter]
        fn set_world_time(&self, value: f64) {
            self.inner.write().unwrap().main.world_v2.world_time = value;
        }

        #[getter]
        fn get_world_width_macro(&self) -> u32 {
            self.inner.read().unwrap().main.world_v2.world_width_macro
        }

        #[setter]
        fn set_world_width_macro(&self, value: u32) {
            self.inner.write().unwrap().main.world_v2.world_width_macro = value;
        }

        fn __repr__(&self) -> String {
            format!("{:?}", self.inner.read().unwrap().main.world_v2)
        }
    }

    #[pyclass(name = "DynamicWorldV2")]
    pub struct DynamicWorldV2Py {
        inner: SharedWorldDb,
    }

    impl DynamicWorldV2Py {
        fn read(&self) -> RwLockReadGuard<'_, WorldDb> {
            self.inner.read().unwrap()
        }

        fn write(&self) -> RwLockWriteGuard<'_, WorldDb> {
            self.inner.write().unwrap()
        }
    }

    #[pymethods]
    impl DynamicWorldV2Py {
        #[getter]
        fn get_active_blockhead_index(&self) -> u64 {
            self.read().main.dynamic_world_v2.active_blockhead_index
        }

        #[setter]
        fn set_active_blockhead_index(&self, value: u64) {
            self.write().main.dynamic_world_v2.active_blockhead_index = value;
        }

        #[getter]
        fn get_dynamic_object_id_count(&self) -> u64 {
            self.read().main.dynamic_world_v2.dynamic_object_id_count
        }

        #[setter]
        fn set_dynamic_object_id_count(&self, value: u64) {
            self.write().main.dynamic_world_v2.dynamic_object_id_count = value;
        }

        #[getter]
        fn get_save_version(&self) -> u8 {
            self.read().main.dynamic_world_v2.save_version
        }

        #[setter]
        fn set_save_version(&self, value: u8) {
            self.write().main.dynamic_world_v2.save_version = value;
        }

        #[getter]
        fn get_saved_glow_indices(&'_ self) -> Cow<'_, [u8]> {
            Cow::Owned(
                self.read()
                    .main
                    .dynamic_world_v2
                    .saved_glow_indices
                    .as_ref()
                    .to_vec(),
            )
        }

        #[setter]
        fn set_saved_glow_indices(&self, value: Vec<u8>) {
            self.write().main.dynamic_world_v2.saved_glow_indices = value.into();
        }

        #[getter]
        fn get_workbench_has_been_crafted(&self) -> bool {
            self.read().main.dynamic_world_v2.workbench_has_been_crafted
        }

        #[setter]
        fn set_workbench_has_been_crafted(&self, value: bool) {
            self.write()
                .main
                .dynamic_world_v2
                .workbench_has_been_crafted = value;
        }

        fn __repr__(&self) -> String {
            format!("{:?}", self.inner.read().unwrap().main.dynamic_world_v2)
        }
    }

    #[pyclass(name = "Blockhead")]
    pub struct BlockheadPy {
        inner: SharedWorldDb,
        index: usize,
    }

    impl BlockheadPy {
        fn read(&self) -> RwLockReadGuard<'_, WorldDb> {
            self.inner.read().unwrap()
        }

        fn write(&self) -> RwLockWriteGuard<'_, WorldDb> {
            self.inner.write().unwrap()
        }
    }

    #[pymethods]
    impl BlockheadPy {
        #[getter]
        fn get_name(&self) -> String {
            self.read().main.blockheads[self.index].name.clone()
        }

        #[setter]
        fn set_name(&self, value: String) {
            self.write().main.blockheads[self.index].name = value;
        }

        #[getter]
        fn get_clothing_increment_timer(&self) -> u64 {
            self.read().main.blockheads[self.index].clothing_increment_timer
        }

        #[setter]
        fn set_clothing_increment_timer(&self, value: u64) {
            self.write().main.blockheads[self.index].clothing_increment_timer = value;
        }

        #[getter]
        fn get_double_time_unlocked(&self) -> bool {
            self.read().main.blockheads[self.index].double_time_unlocked
        }

        #[setter]
        fn set_double_time_unlocked(&self, value: bool) {
            self.write().main.blockheads[self.index].double_time_unlocked = value;
        }

        #[getter]
        fn get_skin_options(&self) -> Cow<'_, [u8]> {
            Cow::Owned(
                self.read().main.blockheads[self.index]
                    .skin_options
                    .as_ref()
                    .to_vec(),
            )
        }

        #[setter]
        fn set_skin_options(&self, value: Vec<u8>) {
            self.write().main.blockheads[self.index].skin_options = value.into();
        }

        #[getter]
        fn get_state(&self) -> Cow<'_, [u8]> {
            Cow::Owned(
                self.read().main.blockheads[self.index]
                    .state
                    .as_ref()
                    .to_vec(),
            )
        }

        #[setter]
        fn set_state(&self, value: Vec<u8>) {
            self.write().main.blockheads[self.index].state = value.into();
        }

        fn __repr__(&self) -> String {
            format!("{:?}", self.read().main.blockheads[self.index])
        }
    }

    #[pyclass(name = "WorldDbMain")]
    pub struct WorldDbMainPy {
        inner: SharedWorldDb,
    }

    #[pymethods]
    impl WorldDbMainPy {
        #[getter]
        fn get_blockheads(&'_ self) -> Vec<BlockheadPy> {
            self.inner
                .read()
                .unwrap()
                .main
                .blockheads
                .iter()
                .enumerate()
                .map(|(index, _)| BlockheadPy {
                    inner: self.inner.clone(),
                    index,
                })
                .collect()
        }

        #[getter]
        fn get_dynamic_world_v2(&'_ self) -> DynamicWorldV2Py {
            DynamicWorldV2Py {
                inner: self.inner.clone(),
            }
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
                inner: Arc::new(RwLock::new(world_db)),
            })
        }

        fn save(&self, path: &str) -> PyResult<()> {
            self.inner
                .read()
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
