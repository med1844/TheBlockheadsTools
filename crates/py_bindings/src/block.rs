use super::{into_py_err, lib, InvalidAccessorError, SharedWorldDb};
use lib::game::{block::BlockType, coord::BlockCoord};
use num_enum::TryFromPrimitive;
use pyo3::prelude::*;
use the_blockheads_tools_lib::game::block::Block;

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

impl From<BlockTypePy> for BlockType {
    fn from(val: BlockTypePy) -> Self {
        BlockType::try_from(val as u8).expect("Enums are out of sync!")
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
        let mut chunk_buffer = Vec::new();
        let block = world_db
            .chunks
            .block_at(self.block_coord, &mut chunk_buffer);
        match block {
            Some(block) => {
                let fg_type = block.map_err(into_py_err)?.fg().map_err(into_py_err)?;
                Ok(fg_type.into())
            }
            None => Err(InvalidAccessorError::new_err(format!(
                "The block at {} doesn't exist.",
                self.block_coord
            ))),
        }
    }
}
