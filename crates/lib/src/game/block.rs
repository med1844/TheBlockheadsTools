use std::ops::{Deref, DerefMut};

use crate::{BhError, BhResult};

/// An enumeration of block types.
///
/// Records the ID of each type of block.
#[derive(Debug, PartialEq, Eq, Hash)]
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
        match value {
            1 => Ok(BlockType::Stone),
            2 => Ok(BlockType::Air),
            3 => Ok(BlockType::Water),
            4 => Ok(BlockType::Ice),
            5 => Ok(BlockType::Snow),
            6 => Ok(BlockType::Dirt),
            7 => Ok(BlockType::DesertSand),
            8 => Ok(BlockType::BeachSand),
            9 => Ok(BlockType::Wood),
            10 => Ok(BlockType::MinedStone),
            11 => Ok(BlockType::RedBrick),
            12 => Ok(BlockType::Limestone),
            13 => Ok(BlockType::MinedLimestone),
            14 => Ok(BlockType::Marble),
            15 => Ok(BlockType::MinedMarble),
            16 => Ok(BlockType::TimeCrystal),
            17 => Ok(BlockType::SandStone),
            18 => Ok(BlockType::MinedSandStone),
            19 => Ok(BlockType::RedMarble),
            20 => Ok(BlockType::MinedRedMarble),
            24 => Ok(BlockType::Glass),
            25 => Ok(BlockType::SpawnPortalBase),
            26 => Ok(BlockType::GoldBlock),
            27 => Ok(BlockType::GrassDirt),
            28 => Ok(BlockType::SnowDirt),
            29 => Ok(BlockType::LapisLazuli),
            30 => Ok(BlockType::MinedLapisLazuli),
            31 => Ok(BlockType::Lava),
            32 => Ok(BlockType::ReinforcedPlatform),
            33 => Ok(BlockType::SpawnPortalBaseAmethyst),
            34 => Ok(BlockType::SpawnPortalBaseSapphire),
            35 => Ok(BlockType::SpawnPortalBaseEmerald),
            36 => Ok(BlockType::SpawnPortalBaseRuby),
            37 => Ok(BlockType::SpawnPortalBaseDiamond),
            38 => Ok(BlockType::NorthPole),
            39 => Ok(BlockType::SouthPole),
            40 => Ok(BlockType::WestPole),
            41 => Ok(BlockType::EastPole),
            42 => Ok(BlockType::PortalBase),
            43 => Ok(BlockType::PortalBaseAmethyst),
            44 => Ok(BlockType::PortalBaseSapphire),
            45 => Ok(BlockType::PortalBaseEmerald),
            46 => Ok(BlockType::PortalBaseRuby),
            47 => Ok(BlockType::PortalBaseDiamond),
            48 => Ok(BlockType::Compost),
            49 => Ok(BlockType::GrassCompost),
            50 => Ok(BlockType::SnowCompost),
            51 => Ok(BlockType::Basalt),
            52 => Ok(BlockType::MinedBasalt),
            53 => Ok(BlockType::CopperBlock),
            54 => Ok(BlockType::TinBlock),
            55 => Ok(BlockType::BronzeBlock),
            56 => Ok(BlockType::IronBlock),
            57 => Ok(BlockType::SteelBlock),
            58 => Ok(BlockType::BlackSand),
            59 => Ok(BlockType::BlackGlass),
            60 => Ok(BlockType::TradePortalBase),
            61 => Ok(BlockType::TradePortalBaseAmethyst),
            62 => Ok(BlockType::TradePortalBaseSapphire),
            63 => Ok(BlockType::TradePortalBaseEmerald),
            64 => Ok(BlockType::TradePortalBaseRuby),
            65 => Ok(BlockType::TradePortalBaseDiamond),
            67 => Ok(BlockType::PlatinumBlock),
            68 => Ok(BlockType::TitaniumBlock),
            69 => Ok(BlockType::CarbonFiberBlock),
            70 => Ok(BlockType::Gravel),
            71 => Ok(BlockType::AmethystBlock),
            72 => Ok(BlockType::SapphireBlock),
            73 => Ok(BlockType::EmeraldBlock),
            74 => Ok(BlockType::RubyBlock),
            75 => Ok(BlockType::DiamondBlock),
            76 => Ok(BlockType::Plaster),
            77 => Ok(BlockType::LuminousPlaster),
            _ => Err(BhError::InvalidBlockIdError(value)),
        }
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
        match self {
            BlockType::Stone => "Stone",
            BlockType::Air => "Air",
            BlockType::Water => "Water",
            BlockType::Ice => "Ice",
            BlockType::Snow => "Snow",
            BlockType::Dirt => "Dirt",
            BlockType::DesertSand => "DesertSand",
            BlockType::BeachSand => "BeachSand",
            BlockType::Wood => "Wood",
            BlockType::MinedStone => "MinedStone",
            BlockType::RedBrick => "RedBrick",
            BlockType::Limestone => "Limestone",
            BlockType::MinedLimestone => "MinedLimestone",
            BlockType::Marble => "Marble",
            BlockType::MinedMarble => "MinedMarble",
            BlockType::TimeCrystal => "TimeCrystal",
            BlockType::SandStone => "SandStone",
            BlockType::MinedSandStone => "MinedSandStone",
            BlockType::RedMarble => "RedMarble",
            BlockType::MinedRedMarble => "MinedRedMarble",
            BlockType::Glass => "Glass",
            BlockType::SpawnPortalBase => "SpawnPortalBase",
            BlockType::GoldBlock => "GoldBlock",
            BlockType::GrassDirt => "GrassDirt",
            BlockType::SnowDirt => "SnowDirt",
            BlockType::LapisLazuli => "LapisLazuli",
            BlockType::MinedLapisLazuli => "MinedLapisLazuli",
            BlockType::Lava => "Lava",
            BlockType::ReinforcedPlatform => "ReinforcedPlatform",
            BlockType::SpawnPortalBaseAmethyst => "SpawnPortalBaseAmethyst",
            BlockType::SpawnPortalBaseSapphire => "SpawnPortalBaseSapphire",
            BlockType::SpawnPortalBaseEmerald => "SpawnPortalBaseEmerald",
            BlockType::SpawnPortalBaseRuby => "SpawnPortalBaseRuby",
            BlockType::SpawnPortalBaseDiamond => "SpawnPortalBaseDiamond",
            BlockType::NorthPole => "NorthPole",
            BlockType::SouthPole => "SouthPole",
            BlockType::WestPole => "WestPole",
            BlockType::EastPole => "EastPole",
            BlockType::PortalBase => "PortalBase",
            BlockType::PortalBaseAmethyst => "PortalBaseAmethyst",
            BlockType::PortalBaseSapphire => "PortalBaseSapphire",
            BlockType::PortalBaseEmerald => "PortalBaseEmerald",
            BlockType::PortalBaseRuby => "PortalBaseRuby",
            BlockType::PortalBaseDiamond => "PortalBaseDiamond",
            BlockType::Compost => "Compost",
            BlockType::GrassCompost => "GrassCompost",
            BlockType::SnowCompost => "SnowCompost",
            BlockType::Basalt => "Basalt",
            BlockType::MinedBasalt => "MinedBasalt",
            BlockType::CopperBlock => "CopperBlock",
            BlockType::TinBlock => "TinBlock",
            BlockType::BronzeBlock => "BronzeBlock",
            BlockType::IronBlock => "IronBlock",
            BlockType::SteelBlock => "SteelBlock",
            BlockType::BlackSand => "BlackSand",
            BlockType::BlackGlass => "BlackGlass",
            BlockType::TradePortalBase => "TradePortalBase",
            BlockType::TradePortalBaseAmethyst => "TradePortalBaseAmethyst",
            BlockType::TradePortalBaseSapphire => "TradePortalBaseSapphire",
            BlockType::TradePortalBaseEmerald => "TradePortalBaseEmerald",
            BlockType::TradePortalBaseRuby => "TradePortalBaseRuby",
            BlockType::TradePortalBaseDiamond => "TradePortalBaseDiamond",
            BlockType::PlatinumBlock => "PlatinumBlock",
            BlockType::TitaniumBlock => "TitaniumBlock",
            BlockType::CarbonFiberBlock => "CarbonFiberBlock",
            BlockType::Gravel => "Gravel",
            BlockType::AmethystBlock => "AmethystBlock",
            BlockType::SapphireBlock => "SapphireBlock",
            BlockType::EmeraldBlock => "EmeraldBlock",
            BlockType::RubyBlock => "RubyBlock",
            BlockType::DiamondBlock => "DiamondBlock",
            BlockType::Plaster => "Plaster",
            BlockType::LuminousPlaster => "LuminousPlaster",
        }
    }
}

impl From<BlockType> for u8 {
    fn from(value: BlockType) -> Self {
        value as u8
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
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
    PortalGate = 47,
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
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Flint),
            2 => Ok(Self::Clay),
            3 => Ok(Self::AppleTreeLeaf),
            4 => Ok(Self::AppleTreeTrunk),
            5 => Ok(Self::AppleTreeTrunkLeaf),
            6 => Ok(Self::PineTreeLeaf),
            7 => Ok(Self::PineTreeTrunk),
            8 => Ok(Self::PineTreeTrunkLeaf),
            9 => Ok(Self::MapleTreeLeaf),
            10 => Ok(Self::MapleTreeTrunk),
            11 => Ok(Self::MapleTreeTrunkLeaf),
            12 => Ok(Self::MangoTreeLeaf),
            13 => Ok(Self::MangoTreeTrunk),
            14 => Ok(Self::MangoTreeTrunkLeaf),
            15 => Ok(Self::CoconutTreeLeaf),
            16 => Ok(Self::CoconutTreeTrunk),
            18 => Ok(Self::OrangeTreeLeaf),
            19 => Ok(Self::OrangeTreeTrunk),
            20 => Ok(Self::OrangeTreeTrunkLeaf),
            21 => Ok(Self::CherryTreeLeaf),
            22 => Ok(Self::CherryTreeTrunk),
            23 => Ok(Self::CherryTreeTrunkLeaf),
            24 => Ok(Self::CoffeeTreeLeaf),
            25 => Ok(Self::CoffeeTreeTrunk),
            26 => Ok(Self::CoffeeTreeTrunkLeaf),
            29 => Ok(Self::DeadPineTreeTrunk),
            34 => Ok(Self::DeadPineTreeLeaf),
            37 => Ok(Self::DeadOrangeTreeLeaf),
            38 => Ok(Self::DeadOrangeTreeTrunk),
            39 => Ok(Self::DeadCherryTreeLeaf),
            40 => Ok(Self::DeadCherryTreeTrunk),
            43 => Ok(Self::Cactus),
            44 => Ok(Self::DeadCactus),
            46 => Ok(Self::Workbench),
            47 => Ok(Self::PortalGate),
            61 => Ok(Self::CopperOre),
            62 => Ok(Self::TinOre),
            63 => Ok(Self::IronOre),
            64 => Ok(Self::Oil),
            65 => Ok(Self::Coal),
            77 => Ok(Self::GoldNuggets),
            89 => Ok(Self::LimeTreeLeaf),
            90 => Ok(Self::LimeTreeTrunk),
            91 => Ok(Self::LimeTreeTrunkLeaf),
            92 => Ok(Self::DeadLimeTreeLeaf),
            93 => Ok(Self::DeadLimeTreeTrunk),
            106 => Ok(Self::PlatinumOre),
            107 => Ok(Self::TitaniumOre),
            109 => Ok(Self::AmethystTreeTrunk),
            110 => Ok(Self::AmethystTreeLeaf),
            111 => Ok(Self::AmethystTreeTrunkLeaf),
            112 => Ok(Self::SapphireTreeTrunk),
            113 => Ok(Self::SapphireTreeLeaf),
            114 => Ok(Self::SapphireTreeTrunkLeaf),
            115 => Ok(Self::EmeraldTreeTrunk),
            116 => Ok(Self::EmeraldTreeLeaf),
            117 => Ok(Self::EmeraldTreeTrunkLeaf),
            118 => Ok(Self::RubyTreeTrunk),
            119 => Ok(Self::RubyTreeLeaf),
            120 => Ok(Self::RubyTreeTrunkLeaf),
            121 => Ok(Self::DiamondTreeTrunk),
            122 => Ok(Self::DiamondTreeLeaf),
            123 => Ok(Self::DiamondTreeTrunkLeaf),
            _ => Err(BhError::InvalidBlockContentIdError(value)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Block<'chunk>(&'chunk [u8; 64]);

impl<'chunk> Block<'chunk> {
    pub(crate) fn new(slice: &'chunk [u8; 64]) -> Self {
        Self(slice)
    }

    pub fn to_hex_string_single_allocation(&self) -> String {
        fn to_hex(four_bit: u8) -> &'static str {
            match four_bit {
                0 => "0",
                1 => "1",
                2 => "2",
                3 => "3",
                4 => "4",
                5 => "5",
                6 => "6",
                7 => "7",
                8 => "8",
                9 => "9",
                10 => "A",
                11 => "B",
                12 => "C",
                13 => "D",
                14 => "E",
                15 => "F",
                _ => "G",
            }
        }

        let estimated_len = 64 * 2 + (8 * 7) + 7; // 128 + 56 + 7 = 191

        let mut result = Vec::with_capacity(estimated_len);

        for (i, &byte) in self.0.iter().enumerate() {
            result.push(to_hex(byte >> 4));
            result.push(to_hex(byte & 15));

            // Add a space after every byte, except the last in a group of 8
            if i & 7 != 7 {
                result.push(" ");
            }

            // Add a newline after every 8 bytes (i.e., at the end of each line),
            // but not after the very last byte in the entire block.
            if i & 7 == 7 && i < 63 {
                result.push("\n");
            }
        }
        result.join("")
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
