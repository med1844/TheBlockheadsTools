use crate::{BhError, BhResult};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::ops::{Deref, DerefMut};
use strum_macros::{Display, IntoStaticStr};

/// An enumeration of block types.
///
/// Records the ID of each type of block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr, Display, FromPrimitive)]
#[repr(u8)]
pub enum BlockType {
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

impl BlockType {
    /// Converts a `u8` integer into a `BlockType` enum variant.
    ///
    /// This function returns a `Result` to handle cases where the integer
    /// does not correspond to a valid `BlockType`.
    ///
    /// # Examples
    ///
    /// ```
    /// use the_blockheads_tools_lib::game::block::BlockType;
    ///
    /// let block_type = BlockType::try_from_u8(1).unwrap();
    /// assert_eq!(block_type, BlockType::Stone);
    ///
    /// let error = BlockType::try_from_u8(99).unwrap_err();
    /// ```
    pub fn try_from_u8(value: u8) -> Result<Self, BhError> {
        BlockType::from_u8(value).ok_or(BhError::InvalidBlockIdError(value))
    }

    /// Converts the `BlockType` enum variant to its corresponding string slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use the_blockheads_tools_lib::game::block::BlockType;
    ///
    /// assert_eq!(BlockType::Stone.as_str(), "Stone");
    /// ```
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

impl From<BlockType> for u8 {
    fn from(value: BlockType) -> Self {
        value as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr, Display, FromPrimitive)]
#[repr(u8)]
pub enum BlockContent {
    None = 0,
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

impl BlockContent {
    pub fn try_from_u8(value: u8) -> Result<Self, BhError> {
        Self::from_u8(value).ok_or(BhError::InvalidBlockIdError(value))
    }

    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Block<'chunk>(&'chunk [u8; 64]);

impl<'chunk> Block<'chunk> {
    pub(crate) fn new(slice: &'chunk [u8; 64]) -> Self {
        Self(slice)
    }
}

impl<'chunk> Deref for Block<'chunk> {
    type Target = [u8; 64];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

pub struct BlockMut<'chunk>(&'chunk mut [u8; 64]);

impl<'chunk> BlockMut<'chunk> {
    pub(crate) fn new(slice: &'chunk mut [u8; 64]) -> Self {
        Self(slice)
    }
}

impl<'chunk> Deref for BlockMut<'chunk> {
    type Target = [u8; 64];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'chunk> DerefMut for BlockMut<'chunk> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

pub trait BlockView {
    fn fg(&self) -> BhResult<BlockType>;
    fn fg_raw(&self) -> u8;
    fn content(&self) -> BhResult<BlockContent>;
    fn content_raw(&self) -> u8;
    fn bg(&self) -> BhResult<BlockType>;
    fn bg_raw(&self) -> u8;
}

impl<T: Deref<Target = [u8; 64]>> BlockView for T {
    fn fg(&self) -> BhResult<BlockType> {
        BlockType::try_from_u8(self.fg_raw())
    }

    fn fg_raw(&self) -> u8 {
        self.deref()[0]
    }

    fn content(&self) -> BhResult<BlockContent> {
        BlockContent::try_from_u8(self.content_raw())
    }

    fn content_raw(&self) -> u8 {
        self.deref()[3]
    }

    fn bg(&self) -> BhResult<BlockType> {
        BlockType::try_from_u8(self.bg_raw())
    }

    fn bg_raw(&self) -> u8 {
        self.deref()[1]
    }
}

pub trait BlockViewMut {
    fn set_fg<I: Into<u8>>(&mut self, value: I);
    fn set_fg_content<I: Into<u8>>(&mut self, value: I);
    fn set_bg<I: Into<u8>>(&mut self, value: I);
}

impl<T: DerefMut<Target = [u8; 64]>> BlockViewMut for T {
    fn set_fg<I: Into<u8>>(&mut self, value: I) {
        self.deref_mut()[0] = value.into();
    }

    fn set_fg_content<I: Into<u8>>(&mut self, value: I) {
        self.deref_mut()[3] = value.into();
    }

    fn set_bg<I: Into<u8>>(&mut self, value: I) {
        self.deref_mut()[1] = value.into();
    }
}
