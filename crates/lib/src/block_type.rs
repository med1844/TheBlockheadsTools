use crate::error::BhError;

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
    MinedSand = 7,
    Sand = 8,
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
    /// use crate::BlockType;
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
            7 => Ok(BlockType::MinedSand),
            8 => Ok(BlockType::Sand),
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
    /// ```
    /// use crate::BlockType;
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
            BlockType::Sand => "Sand",
            BlockType::MinedSand => "MinedSand",
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

impl Into<u8> for BlockType {
    fn into(self) -> u8 {
        // We can safely cast to u8 because the values are within the u8 range.
        self as u8
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BlockContent {
    None = 0,
    Flint = 1,
    Clay = 2,
    Workbench = 46,
    PortalGate = 47,
    CopperOre = 61,
    TinOre = 62,
    IronOre = 63,
    Oil = 64,
    Coal = 65,
    GoldNuggets = 77,
    PlatinumOre = 106,
    TitaniumOre = 107,
}

impl BlockContent {
    pub fn try_from_u8(value: u8) -> Result<Self, BhError> {
        match value {
            0 => Ok(BlockContent::None),
            1 => Ok(BlockContent::Flint),
            2 => Ok(BlockContent::Clay),
            46 => Ok(BlockContent::Workbench),
            47 => Ok(BlockContent::PortalGate),
            61 => Ok(BlockContent::CopperOre),
            62 => Ok(BlockContent::TinOre),
            63 => Ok(BlockContent::IronOre),
            64 => Ok(BlockContent::Oil),
            65 => Ok(BlockContent::Coal),
            66 => Ok(BlockContent::GoldNuggets),
            67 => Ok(BlockContent::PlatinumOre),
            68 => Ok(BlockContent::TitaniumOre),
            _ => Err(BhError::InvalidBlockContentIdError(value)),
        }
    }
}
