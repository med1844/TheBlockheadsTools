use super::{chunk::ChunkPy, into_py_err, lib};
use lib::game::{
    block::{Block, BlockContentType, BlockMut, BlockType, BlockView, BlockViewMut},
    coord::ChunkBlockCoord,
};
use num_enum::TryFromPrimitive;
use pyo3::prelude::*;

#[pyclass(eq, eq_int, name = "BlockType")]
#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
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

#[pymethods]
impl BlockTypePy {
    #[new]
    fn new(value: u8) -> PyResult<Self> {
        Self::try_from(value).map_err(|_| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid BlockType value: {}", value))
        })
    }
}

impl From<BlockType> for BlockTypePy {
    fn from(value: BlockType) -> Self {
        Self::try_from(value as u8).expect("Enums are out of sync!")
    }
}

impl From<BlockTypePy> for BlockType {
    fn from(val: BlockTypePy) -> Self {
        Self::try_from(val as u8).expect("Enums are out of sync!")
    }
}

impl From<BlockTypePy> for u8 {
    fn from(value: BlockTypePy) -> Self {
        value as u8
    }
}

#[pyclass(eq, eq_int, name = "BlockContentType")]
#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum BlockContentTypePy {
    Nothing = 0,
    Flint = 1,
    Clay = 2,
    AppleTreeLeaf = 3,
    AppleTreeTrunk = 4,
    AppleTreeTrunkLeaf = 5,
    PineTreeLeaf = 6,
    PineTreeTrunk = 7,
    PineTreeTrunkLeaf = 8,
    MapleTreeLeaf = 9,
    MapleTreeTrunk = 10,
    MapleTreeTrunkLeaf = 11,
    MangoTreeLeaf = 12,
    MangoTreeTrunk = 13,
    MangoTreeTrunkLeaf = 14,
    CoconutTreeLeaf = 15,
    CoconutTreeTrunk = 16,
    OrangeTreeLeaf = 18,
    OrangeTreeTrunk = 19,
    OrangeTreeTrunkLeaf = 20,
    CherryTreeLeaf = 21,
    CherryTreeTrunk = 22,
    CherryTreeTrunkLeaf = 23,
    CoffeeTreeLeaf = 24,
    CoffeeTreeTrunk = 25,
    CoffeeTreeTrunkLeaf = 26,
    DeadPineTreeTrunk = 29,
    DeadPineTreeLeaf = 34,
    DeadOrangeTreeLeaf = 37,
    DeadOrangeTreeTrunk = 38,
    DeadCherryTreeLeaf = 39,
    DeadCherryTreeTrunk = 40,
    Cactus = 43,
    DeadCactus = 44,
    Workbench = 46,
    WorkbenchSprite = 47,
    CopperOre = 61,
    TinOre = 62,
    IronOre = 63,
    Oil = 64,
    Coal = 65,
    GoldNuggets = 77,
    LimeTreeLeaf = 89,
    LimeTreeTrunk = 90,
    LimeTreeTrunkLeaf = 91,
    DeadLimeTreeLeaf = 92,
    DeadLimeTreeTrunk = 93,
    GoldChest = 94,
    PlatinumOre = 106,
    TitaniumOre = 107,
    AmethystTreeTrunk = 109,
    AmethystTreeLeaf = 110,
    AmethystTreeTrunkLeaf = 111,
    SapphireTreeTrunk = 112,
    SapphireTreeLeaf = 113,
    SapphireTreeTrunkLeaf = 114,
    EmeraldTreeTrunk = 115,
    EmeraldTreeLeaf = 116,
    EmeraldTreeTrunkLeaf = 117,
    RubyTreeTrunk = 118,
    RubyTreeLeaf = 119,
    RubyTreeTrunkLeaf = 120,
    DiamondTreeTrunk = 121,
    DiamondTreeLeaf = 122,
    DiamondTreeTrunkLeaf = 123,
}

#[pymethods]
impl BlockContentTypePy {
    #[new]
    fn new(value: u8) -> PyResult<Self> {
        Self::try_from(value).map_err(|_| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid BlockType value: {}", value))
        })
    }
}

impl From<BlockContentType> for BlockContentTypePy {
    fn from(value: BlockContentType) -> Self {
        Self::try_from(value as u8).expect("Enums are out of sync!")
    }
}

impl From<BlockContentTypePy> for BlockContentType {
    fn from(val: BlockContentTypePy) -> Self {
        Self::try_from(val as u8).expect("Enums are out of sync!")
    }
}

impl From<BlockContentTypePy> for u8 {
    fn from(value: BlockContentTypePy) -> Self {
        value as u8
    }
}

#[pyclass(name = "Block")]
pub struct BlockPy {
    pub(crate) chunk: Py<ChunkPy>,
    pub(crate) coord: ChunkBlockCoord,
}

impl BlockPy {
    fn read<O, F: Fn(BlockView) -> O>(&self, py: Python<'_>, f: F) -> O {
        let chunk_py = self.chunk.borrow(py);
        let chunk = chunk_py.inner();
        f(chunk.view().block_at(self.coord))
    }

    fn write<O, F: FnOnce(BlockViewMut) -> O>(&self, py: Python<'_>, f: F) -> O {
        let mut chunk_py = self.chunk.borrow_mut(py);
        let chunk = chunk_py.inner_mut();
        f(chunk.view_mut().block_at_mut(self.coord))
    }
}

#[pymethods]
impl BlockPy {
    fn fg_raw(&self, py: Python<'_>) -> u8 {
        self.read(py, |block_view| block_view.fg_raw())
    }

    fn bg_raw(&self, py: Python<'_>) -> u8 {
        self.read(py, |block_view| block_view.bg_raw())
    }

    fn content_raw(&self, py: Python<'_>) -> u8 {
        self.read(py, |block_view| block_view.content_raw())
    }

    fn fg(&self, py: Python<'_>) -> PyResult<BlockTypePy> {
        self.read(py, |block_view| {
            block_view.fg().map(Into::into).map_err(into_py_err)
        })
    }

    fn set_fg(&self, py: Python<'_>, block_type: &BlockTypePy) {
        self.write(py, |mut block_view_mut| block_view_mut.set_fg(*block_type))
    }

    fn bg(&self, py: Python<'_>) -> PyResult<BlockTypePy> {
        self.read(py, |block_view| {
            block_view.bg().map(Into::into).map_err(into_py_err)
        })
    }

    fn set_bg(&self, py: Python<'_>, block_type: &BlockTypePy) {
        self.write(py, |mut block_view_mut| block_view_mut.set_bg(*block_type))
    }

    fn content(&self, py: Python<'_>) -> PyResult<BlockContentTypePy> {
        self.read(py, |block_view| {
            block_view.content().map(Into::into).map_err(into_py_err)
        })
    }

    fn set_content(&self, py: Python<'_>, block_content_type: &BlockContentTypePy) {
        self.write(py, |mut block_view_mut| {
            block_view_mut.set_content(*block_content_type)
        })
    }

    fn height(&self, py: Python<'_>) -> u8 {
        self.read(py, |block_view| block_view.height())
    }

    fn set_height(&self, py: Python<'_>, height: u8) {
        self.write(py, |mut block_view_mut| block_view_mut.set_height(height))
    }

    fn damage(&self, py: Python<'_>) -> u8 {
        self.read(py, |block_view| block_view.damage())
    }

    fn set_damage(&self, py: Python<'_>, damage: u8) {
        self.write(py, |mut block_view_mut| block_view_mut.set_damage(damage))
    }

    fn visibility(&self, py: Python<'_>) -> u8 {
        self.read(py, |block_view| block_view.visibility())
    }

    fn set_visibility(&self, py: Python<'_>, visibility: u8) {
        self.write(py, |mut block_view_mut| {
            block_view_mut.set_visibility(visibility)
        })
    }

    fn brightness(&self, py: Python<'_>) -> u8 {
        self.read(py, |block_view| block_view.brightness())
    }

    fn set_brightness(&self, py: Python<'_>, brightness: u8) {
        self.write(py, |mut block_view_mut| {
            block_view_mut.set_brightness(brightness)
        })
    }
}
