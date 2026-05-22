use num_enum::TryFromPrimitive;
use snafu::prelude::*;
use strum::{Display, IntoStaticStr};

#[derive(Debug, Snafu)]
pub enum BlockError {
    #[snafu(display("Unknown block type ID {id}"))]
    UnknownBlockTypeId {
        id: u8,
        source: num_enum::TryFromPrimitiveError<BlockType>,
    },
    #[snafu(display("Unknown block content type ID {id}"))]
    UnknownBlockContentTypeId {
        id: u8,
        source: num_enum::TryFromPrimitiveError<BlockContentType>,
    },
}

type Result<T> = std::result::Result<T, BlockError>;

/// An enumeration of block types.
///
/// Records the ID of each type of block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr, Display, TryFromPrimitive)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr, Display, TryFromPrimitive)]
#[repr(u8)]
pub enum BlockContentType {
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
    Sprite = 48,
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
    Wire = 96,
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

impl From<BlockContentType> for u8 {
    fn from(value: BlockContentType) -> Self {
        value as u8
    }
}

impl BlockContentType {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

const FG: usize = 0;
const BG: usize = 1;
const CONTENT: usize = 3;
const HEIGHT: usize = 4;
const DAMAGE: usize = 5;
const VISIBILITY: usize = 6;
const BRIGHTNESS: usize = 7;

pub trait Block {
    // required
    fn as_bytes(&self) -> &[u8; 64];

    // provided
    fn fg_raw(&self) -> u8 {
        self.as_bytes()[FG]
    }
    fn bg_raw(&self) -> u8 {
        self.as_bytes()[BG]
    }
    fn content_raw(&self) -> u8 {
        self.as_bytes()[CONTENT]
    }

    fn fg(&self) -> Result<BlockType> {
        let raw = self.fg_raw();
        BlockType::try_from(raw).context(UnknownBlockTypeIdSnafu { id: raw })
    }
    fn bg(&self) -> Result<BlockType> {
        let raw = self.bg_raw();
        BlockType::try_from(raw).context(UnknownBlockTypeIdSnafu { id: raw })
    }
    fn content(&self) -> Result<BlockContentType> {
        let raw = self.content_raw();
        BlockContentType::try_from(raw).context(UnknownBlockContentTypeIdSnafu { id: raw })
    }
    fn height(&self) -> u8 {
        self.as_bytes()[HEIGHT]
    }
    fn damage(&self) -> u8 {
        self.as_bytes()[DAMAGE]
    }
    fn visibility(&self) -> u8 {
        self.as_bytes()[VISIBILITY]
    }
    fn brightness(&self) -> u8 {
        self.as_bytes()[BRIGHTNESS]
    }
}

pub trait BlockMut {
    // required
    fn as_mut_bytes(&mut self) -> &mut [u8; 64];

    // provided
    fn fg_raw_mut(&mut self) -> &mut u8 {
        &mut self.as_mut_bytes()[FG]
    }
    fn set_fg<I: Into<u8>>(&mut self, value: I) {
        self.as_mut_bytes()[FG] = value.into();
    }
    fn bg_raw_mut(&mut self) -> &mut u8 {
        &mut self.as_mut_bytes()[BG]
    }
    fn set_bg<I: Into<u8>>(&mut self, value: I) {
        self.as_mut_bytes()[BG] = value.into();
    }
    fn content_raw_mut(&mut self) -> &mut u8 {
        &mut self.as_mut_bytes()[CONTENT]
    }
    fn set_content<I: Into<u8>>(&mut self, value: I) {
        self.as_mut_bytes()[CONTENT] = value.into();
    }
    fn height_mut(&mut self) -> &mut u8 {
        &mut self.as_mut_bytes()[HEIGHT]
    }
    fn set_height(&mut self, value: u8) {
        self.as_mut_bytes()[HEIGHT] = value;
    }
    fn damage_mut(&mut self) -> &mut u8 {
        &mut self.as_mut_bytes()[DAMAGE]
    }
    fn set_damage(&mut self, value: u8) {
        self.as_mut_bytes()[DAMAGE] = value;
    }
    fn visibility_mut(&mut self) -> &mut u8 {
        &mut self.as_mut_bytes()[VISIBILITY]
    }
    fn set_visibility(&mut self, value: u8) {
        self.as_mut_bytes()[VISIBILITY] = value;
    }
    fn brightness_mut(&mut self) -> &mut u8 {
        &mut self.as_mut_bytes()[BRIGHTNESS]
    }
    fn set_brightness(&mut self, value: u8) {
        self.as_mut_bytes()[BRIGHTNESS] = value;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BlockView<'chunk>(&'chunk [u8; 64]);

impl<'chunk> BlockView<'chunk> {
    pub(crate) fn new(slice: &'chunk [u8; 64]) -> Self {
        Self(slice)
    }
}

impl<'chunk> Block for BlockView<'chunk> {
    fn as_bytes(&self) -> &[u8; 64] {
        self.0
    }
}

pub struct BlockViewMut<'chunk>(&'chunk mut [u8; 64]);

impl<'chunk> BlockViewMut<'chunk> {
    pub(crate) fn new(slice: &'chunk mut [u8; 64]) -> Self {
        Self(slice)
    }
}

impl<'chunk> Block for BlockViewMut<'chunk> {
    fn as_bytes(&self) -> &[u8; 64] {
        self.0
    }
}

impl<'chunk> BlockMut for BlockViewMut<'chunk> {
    fn as_mut_bytes(&mut self) -> &mut [u8; 64] {
        self.0
    }
}
