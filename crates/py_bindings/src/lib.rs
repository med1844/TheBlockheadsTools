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
        | lib::BhError::InvalidDynamicOjectId(_)
        | lib::BhError::InvalidItemTypeId(_)
        | lib::BhError::InvalidColorId(_)
        | lib::BhError::InvalidChunkSize(_) => PyValueError::new_err(error_message),
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
}

pub use block::{BlockPy, BlockTypePy};

mod chunk {
    use std::{borrow::Cow, collections::HashSet};

    use crate::{
        into_py_err, lib, BlockCoordPy, BlockPy, ChunkBlockCoordPy, ChunkCoordPy,
        InvalidAccessorError, SharedWorldDb,
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
            let world_db = self.world_db.read().unwrap();
            let chunk = world_db.chunks.chunk_at(self.coord);
            match chunk {
                Some(chunk) => Ok(Cow::Owned(
                    chunk.decompress().map_err(into_py_err)?.inner().to_vec(),
                )),
                None => Err(InvalidAccessorError::new_err(format!(
                    "The chunk at {} doesn't exist.",
                    self.coord
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

pub mod item {
    use crate::{into_py_err, lib};
    use lib::game::{
        dw::dynamic_object::{DynamicObject, UniqueID},
        item::{
            ChestData, ChestType, Extra, InteractionObject, Inventory, Item, ItemType,
            PigmentColor, StackedItem, WorkbenchData, WorkbenchType,
        },
    };
    use num_enum::TryFromPrimitive;
    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;

    #[pyclass(eq, eq_int, name = "ItemType")]
    #[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
    #[repr(u16)]
    pub enum ItemTypePy {
        Unknown = 0,
        Clothing = 1,
        DeprecatedDirtBlock = 2,
        Flint = 3,
        Stick = 4,
        DeprecatedWoodBlock = 5,
        FlintAxe = 6,
        FlintSpear = 7,
        FlintPickaxe = 8,
        DoubleTime = 9,
        DeprecatedWorkbench = 10,
        TimeCrystal = 11,
        Basket = 12,
        Ember = 13,
        Charcoal = 14,
        Campfire = 15,
        FlintSpade = 16,
        Torch = 17,
        DeprecatedSand = 18,
        Blockhead = 19,
        Food = 20,
        Apple = 21,
        Mango = 22,
        MapleSeed = 23,
        PricklyPear = 24,
        FlintMachete = 25,
        DeprecatedStoneWorkbench = 26,
        Pinecone = 27,
        Clay = 28,
        DodoMeat = 29,
        DodoFeather = 30,
        CopperOre = 31,
        IronOre = 32,
        StoneAxe = 33,
        StonePickaxe = 34,
        CopperIngot = 35,
        TinOre = 36,
        TinIngot = 37,
        BronzeIngot = 38,
        CopperSpear = 39,
        TinSpade = 40,
        CopperArrow = 41,
        CopperBowAndArrows = 42,
        BronzePickaxe = 43,
        String = 44,
        ClayJug = 45,
        Coconut = 46,
        OilLantern = 47,
        Oil = 48,
        BronzeMachete = 49,
        BronzeSword = 50,
        Coal = 51,
        Door = 52,
        Ladder = 53,
        FlaxSeed = 54,
        Flax = 55,
        IndianYellow = 56,
        RedOchre = 57,
        Window = 58,
        CookedDodoMeat = 59,
        Orange = 60,
        SunflowerSeed = 61,
        Corn = 62,
        Bed = 63,
        StoneSpade = 64,
        IronIngot = 65,
        IronPickaxe = 66,
        IronMachete = 67,
        IronSword = 68,
        Trapdoor = 69,
        IronAxe = 70,
        Carrot = 71,
        GoldIngot = 72,
        GoldNugget = 73,
        CarrotOnAStick = 74,
        Ruby = 75,
        Emerald = 76,
        Cherry = 77,
        CoffeeCherry = 78,
        GreenCoffeeBean = 79,
        Cup = 80,
        Coffee = 81,
        RoastedCoffeeBean = 82,
        Linen = 83,
        LinenPants = 84,
        LinenShirt = 85,
        Sapphire = 86,
        Amethyst = 87,
        Diamond = 88,
        GoldSpade = 89,
        GoldPickaxe = 90,
        DodoEgg = 91,
        SteelIngot = 92,
        SteelPickaxe = 93,
        AmethystPickaxe = 94,
        SapphirePickaxe = 95,
        EmeraldPickaxe = 96,
        RubyPickaxe = 97,
        DiamondPickaxe = 98,
        UltramarineBlue = 99,
        CarbonBlack = 100,
        MarbleWhite = 101,
        TinBucket = 102,
        Paint = 103,
        PaintStripper = 104,
        BucketOfWater = 105,
        Pigment = 106,
        RainbowPaintCap = 107,
        InvalidPigment = 108,
        EmeraldGreen = 109,
        TyrianPurple = 110,
        Boat = 111,
        Chilli = 112,
        RainbowLinenPants = 113,
        RainbowShirt = 114,
        LinenCap = 115,
        RainbowCap = 116,
        LinenBrimmedHat = 117,
        RainbowBrimmedHat = 118,
        CopperBlue = 119,
        Leather = 120,
        Fur = 121,
        LeatherJacket = 122,
        RainbowJacket = 123,
        LeatherBoots = 124,
        RainbowLeatherBoots = 125,
        FurCoat = 126,
        FurBoots = 127,
        RainbowCoat = 128,
        RainbowFurBoots = 129,
        LeatherPants = 130,
        RainbowLeatherPants = 131,
        Upgrade = 132,
        Camera = 133,
        Portal = 134,
        AmethystPortal = 135,
        SapphirePortal = 136,
        EmeraldPortal = 137,
        RubyPortal = 138,
        DiamondPortal = 139,
        SunriseHatOfFullness = 140,
        SunsetSkirtOfHappiness = 141,
        NorthPoleHatOfWarmth = 142,
        SouthPoleBootsOfSpeed = 143,
        Kelp = 144,
        AmethystChandelier = 145,
        SapphireChandelier = 146,
        EmeraldChandelier = 147,
        RubyChandelier = 148,
        DiamondChandelier = 149,
        SteelLantern = 150,
        RawFish = 151,
        CookedFish = 152,
        TinFoil = 153,
        TinFoilHat = 154,
        Worm = 155,
        FishingRod = 156,
        SharkJaw = 157,
        FishBucket = 158,
        SharkBucket = 159,
        Lime = 160,
        Shelf = 161,
        TeleportHere = 162,
        Sign = 163,
        IronDoor = 164,
        IronTrapdoor = 165,
        CopperCoin = 166,
        GoldCoin = 167,
        Shop = 168,
        SoftBed = 169,
        GoldenBed = 170,
        BedBlanket = 171,
        RainbowSoftBed = 172,
        RainbowGoldenBed = 173,
        BlackWindow = 174,
        Magnet = 175,
        CopperBoiler = 176,
        ElectronicMotor = 177,
        CopperWire = 178,
        SteamEngine = 179,
        IronPot = 180,
        FishCurry = 181,
        DodoStew = 182,
        IceTorch = 183,
        SiliconIngot = 184,
        SiliconCrystal = 185,
        SiliconWafer = 186,
        TinArmorLeggings = 187,
        TinChestPlate = 188,
        TinHelmet = 189,
        TinBoots = 190,
        IronArmorLeggings = 191,
        IronChestPlate = 192,
        IronHelmet = 193,
        IronBoots = 194,
        IceArmorLeggings = 195,
        IceChestPlate = 196,
        IceHelmet = 197,
        IceBoots = 198,
        Rail = 199,
        TrainStation = 200,
        PigIron = 201,
        CrushedLimestone = 202,
        TrainWheel = 203,
        RailHandcar = 204,
        SteamLocomotive = 205,
        FreightCar = 206,
        DisplayCabinet = 207,
        PassengerCar = 208,
        Crowbar = 209,
        TradePortal = 210,
        DeprecatedGoldChest = 211,
        LargeSquarePainting = 212,
        LargeLandscapePainting = 213,
        LargePortraitPainting = 214,
        MedSquarePainting = 215,
        MedLandscapePainting = 216,
        MedPortraitPainting = 217,
        SmallSquarePainting = 218,
        SmallLandscapePainting = 219,
        SmallPortraitPainting = 220,
        Easel = 221,
        StoneColumn = 222,
        LimestoneColumn = 223,
        MarbleColumn = 224,
        SandstoneColumn = 225,
        RedMarbleColumn = 226,
        LapisLazuliColumn = 227,
        BasaltColumn = 228,
        StoneStairs = 229,
        LimestoneStairs = 230,
        MarbleStairs = 231,
        SandstoneStairs = 232,
        RedMarbleStairs = 233,
        LapisLazuliStairs = 234,
        BasaltStairs = 235,
        CopperColumn = 236,
        TinColumn = 237,
        BronzeColumn = 238,
        IronColumn = 239,
        SteelColumn = 240,
        GoldColumn = 241,
        WoodColumn = 242,
        BrickColumn = 243,
        IceColumn = 244,
        CopperStairs = 245,
        TinStairs = 246,
        BronzeStairs = 247,
        IronStairs = 248,
        SteelStairs = 249,
        GoldStairs = 250,
        WoodStairs = 251,
        BrickStairs = 252,
        IceStairs = 253,
        SteelDownlight = 254,
        Poison = 255,
        PoisonArrow = 256,
        GoldBowAndPoisonArrows = 257,
        SteelUplight = 258,
        WorldCredit = 259,
        PlatiumCoin = 260,
        PlatiumNugget = 261,
        PlatiumIngot = 262,
        PlatiumStairs = 263,
        PlatiumColumn = 264,
        GlassStairs = 265,
        GlassColumn = 266,
        BlackGlassStairs = 267,
        BlackGlassColumn = 268,
        Fuel = 269,
        Refinery = 270,
        Epoxy = 271,
        RawResin = 272,
        CarbonFibers = 273,
        CarbonFiberSheet = 274,
        CarbonFiberWing = 275,
        JetpackChassis = 276,
        JetEngine = 277,
        Jetpack = 278,
        TitaniumOre = 279,
        TitaniumIngot = 280,
        TitaniumStairs = 281,
        TitaniumColumn = 282,
        CarbonFiberStairs = 283,
        CarbonFiberColumn = 284,
        TitaniumPickaxe = 285,
        TitaniumSword = 286,
        TitaniumLeggings = 287,
        TitaniumChestPlate = 288,
        TitaniumHelmet = 289,
        TitaniumBoots = 290,
        CarbonFiberLeggings = 291,
        CarbonFiberChestPlate = 292,
        CarbonFiberHelmet = 293,
        CarbonFiberBoots = 294,
        Vine = 295,
        TulipBulb = 296,
        TulipSeed = 297,
        Coins = 298,
        RandomOre = 299,
        ElectricSluice = 300,
        OwnershipSign = 301,
        Cage = 302,
        CagedDodo = 303,
        WoodenGate = 304,
        AmethystShard = 305,
        SapphireShard = 306,
        EmeraldShard = 307,
        RubyShard = 308,
        DiamondShard = 309,
        Wheat = 310,
        Flour = 311,
        Yeast = 312,
        Salt = 313,
        Dough = 314,
        Bread = 315,
        Tomato = 316,
        Pizza = 317,
        Flatbread = 318,
        Milk = 319,
        Mozzarella = 320,
        YakHorn = 321,
        Razor = 322,
        YakShavings = 323,
        CagedDonkey = 324,
        CagedYak = 325,
        CagedDropbear = 326,
        CagedScorpion = 327,
        RainbowCake = 328,
        RainbowEssence = 329,
        CagedUnicorn = 330,
        Mirror = 331,
        PlasterColumn = 332,
        PlasterStairs = 333,
        AmethystColumn = 334,
        SapphireColumn = 335,
        EmeraldColumn = 336,
        RubyColumn = 337,
        DiamondColumn = 338,
        AmethystStairs = 339,
        SapphireStairs = 340,
        EmeraldStairs = 341,
        RubyStairs = 342,
        DiamondStairs = 343,

        Stone = 1024,
        Kiln = 1025,
        Brick = 1026,
        Limestone = 1027,
        MinedLimestone = 1028,
        Marble = 1029,
        MinedMarble = 1030,
        Furnace = 1031,
        WoodworkBench = 1032,
        TaylorsBench = 1033,
        Press = 1034,
        Sandstone = 1035,
        MinedSandstone = 1036,
        RedMarble = 1037,
        MinedRedMarble = 1038,
        WovenFlaxMat = 1039,
        YellowFlaxMat = 1040,
        RedFlaxMat = 1041,
        Glass = 1042,
        Chest = 1043,
        DeprecatedFood = 1044,
        GoldBlock = 1045,
        DeprecatedMango = 1046,
        Rock = 1047,
        Dirt = 1048,
        Wood = 1049,
        WorkBench = 1050,
        Sand = 1051,
        ToolBench = 1052,
        LapisLazuli = 1053,
        MinedLapisLazuli = 1054,
        CraftBench = 1055,
        MixingBench = 1056,
        ReinforcedPlatform = 1057,
        DeprecatedStonePickaxe = 1058,
        DeprecatedCopperIngot = 1059,
        Ice = 1060,
        DyeBench = 1061,
        Compost = 1062,
        Basalt = 1063,
        MinedBasalt = 1064,
        Safe = 1065,
        CopperBlock = 1066,
        TinBlock = 1067,
        BronzeBlock = 1068,
        IronBlock = 1069,
        SteelBlock = 1070,
        MetalworkBench = 1071,
        GoldenChest = 1072,
        DeprecatedBronzeMachete = 1073,
        PortalChest = 1074,
        BlackSand = 1075,
        BlackGlass = 1076,
        SteamGenerator = 1077,
        ElectricKiln = 1078,
        ElectricFurnace = 1079,
        ElectricMetalworkBench = 1080,
        ElectricStove = 1081,
        SolarPanel = 1082,
        Flywheel = 1083,
        ArmorBench = 1084,
        TrainYard = 1085,
        BuildersBench = 1086,
        ElevatorShaft = 1087,
        ElectricElevatorMotor = 1088,
        PlatiumBlock = 1089,
        CarbonFiberBlock = 1090,
        TitaniumBlock = 1091,
        DeprecatedIronSword = 1092,
        ElectricPress = 1093,
        Gravel = 1094,
        CompostBin = 1095,
        EggExtractor = 1096,
        PizzaOven = 1097,
        AmethystBlock = 1098,
        SapphireBlock = 1099,
        EmeraldBlock = 1100,
        RubyBlock = 1101,
        DiamondBlock = 1102,
        Plaster = 1103,
        FeederChest = 1104,
        LuminousPlaster = 1105,
    }

    impl From<ItemType> for ItemTypePy {
        fn from(value: ItemType) -> Self {
            Self::try_from(value as u16).expect("Enums are out of sync!")
        }
    }

    impl From<ItemTypePy> for ItemType {
        fn from(val: ItemTypePy) -> Self {
            ItemType::try_from(val as u16).expect("Enums are out of sync!")
        }
    }

    #[pyclass(eq, eq_int, name = "ChestType")]
    #[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum ChestTypePy {
        Standard = 0,
        Safe = 1,
        Shelf = 2,
        Gold = 3,
        Portal = 4,
        DisplayCabinet = 5,
        Feeder = 6,
    }

    impl From<ChestType> for ChestTypePy {
        fn from(value: ChestType) -> Self {
            Self::try_from(value as u8).expect("Enums are out of sync!")
        }
    }

    impl From<ChestTypePy> for ChestType {
        fn from(val: ChestTypePy) -> Self {
            ChestType::from(val as u8)
        }
    }

    #[pyclass(eq, eq_int, name = "WorkbenchType")]
    #[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum WorkbenchTypePy {
        Undefined = 0,
        BasicPortal = 1,
        Workbench = 2,
        Campfire = 3,
        Weave = 4,
        Wood = 5,
        Tool = 6,
        Press = 7,
        Kiln = 8,
        Furnace = 9,
        Craft = 10,
        Mix = 11,
        Dye = 12,
        PlacedPortal = 13,
        Metalwork = 14,
        SteamGenerator = 15,
        ElectricKiln = 16,
        ElectricFurnace = 17,
        ElectricMetalworkBench = 18,
        ElectricStove = 19,
        SolarPanel = 20,
        Flywheel = 21,
        ArmorBench = 22,
        TrainYard = 23,
        Easel = 24,
        Build = 25,
        Refinery = 26,
        ElectricPress = 27,
        CompostBin = 28,
        Sluice = 29,
        EggExtractor = 30,
        PizzaOven = 31,
    }

    impl From<WorkbenchType> for WorkbenchTypePy {
        fn from(value: WorkbenchType) -> Self {
            Self::try_from(value as u8).expect("Enums are out of sync!")
        }
    }

    impl From<WorkbenchTypePy> for WorkbenchType {
        fn from(val: WorkbenchTypePy) -> Self {
            WorkbenchType::from(val as u8)
        }
    }

    #[pyclass(eq, eq_int, name = "PigmentColor")]
    #[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum PigmentColorPy {
        Transparent = 0,
        MarbleWhite = 1,
        CarbonBlack = 2,
        RedOchre = 3,
        IndianYellow = 4,
        UltraMarineBlue = 5,
        EmeraldGreen = 6,
        TyrianPurple = 7,
        CopperBlue = 8,
    }

    impl From<PigmentColor> for PigmentColorPy {
        fn from(value: PigmentColor) -> Self {
            Self::try_from(value as u8).expect("Enums are out of sync!")
        }
    }

    impl From<PigmentColorPy> for PigmentColor {
        fn from(val: PigmentColorPy) -> Self {
            PigmentColor::try_from(val as u8).expect("Enums are out of sync!")
        }
    }

    #[pyclass(name = "BasketExtra")]
    pub struct BasketExtraPy {
        #[pyo3(get, set)]
        pub items: Vec<Py<StackedItemPy>>,
    }

    impl std::fmt::Debug for BasketExtraPy {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("BasketExtraPy")
                .field("items", &"<items>")
                .finish()
        }
    }

    #[pymethods]
    impl BasketExtraPy {
        #[new]
        #[pyo3(signature = (items=None))]
        fn new(items: Option<Vec<Py<StackedItemPy>>>) -> PyResult<Self> {
            match items {
                Some(items) => {
                    if items.len() != Extra::NUM_SLOT_BASKET {
                        return Err(PyValueError::new_err(format!(
                            "BasketExtra must have exactly {} slots",
                            Extra::NUM_SLOT_BASKET
                        )));
                    }
                    Ok(Self { items })
                }
                None => Python::attach(|py| {
                    let mut items = Vec::with_capacity(Extra::NUM_SLOT_BASKET);
                    for _ in 0..Extra::NUM_SLOT_BASKET {
                        items.push(Py::new(py, StackedItemPy::default())?);
                    }
                    Ok(Self { items })
                }),
            }
        }

        fn __len__(&self) -> usize {
            self.items.len()
        }

        fn __getitem__(&self, py: Python<'_>, mut index: isize) -> PyResult<Py<StackedItemPy>> {
            if index < 0 {
                index += self.items.len() as isize;
            }
            if index < 0 || index >= self.items.len() as isize {
                return Err(pyo3::exceptions::PyIndexError::new_err(
                    "index out of range",
                ));
            }
            Ok(self.items[index as usize].clone_ref(py))
        }

        fn __setitem__(&mut self, mut index: isize, value: Py<StackedItemPy>) -> PyResult<()> {
            if index < 0 {
                index += self.items.len() as isize;
            }
            if index < 0 || index >= self.items.len() as isize {
                return Err(pyo3::exceptions::PyIndexError::new_err(
                    "index out of range",
                ));
            }
            self.items[index as usize] = value;
            Ok(())
        }

        fn __repr__(&self, py: Python<'_>) -> String {
            let items_repr: Vec<String> = self
                .items
                .iter()
                .map(|item| {
                    item.bind(py)
                        .repr()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|_| "<repr error>".to_string())
                })
                .collect();
            format!("BasketExtra(items=[{}])", items_repr.join(", "))
        }
    }

    #[pyclass(name = "ChestExtra")]
    pub struct ChestExtraPy {
        #[pyo3(get, set)]
        pub chest_type: ChestTypePy,
        #[pyo3(get, set)]
        pub items: Vec<Py<StackedItemPy>>,
        #[pyo3(get, set)]
        pub owner_id: String,
        #[pyo3(get, set)]
        pub is_in_use: bool,
        #[pyo3(get, set)]
        pub flipped: bool,
        #[pyo3(get, set)]
        pub paint_color: u16,
        #[pyo3(get, set)]
        pub pos_x: u64,
        #[pyo3(get, set)]
        pub pos_y: u16,
        #[pyo3(get, set)]
        pub float_pos: [f32; 2],
        #[pyo3(get, set)]
        pub unique_id: u64,
    }

    impl std::fmt::Debug for ChestExtraPy {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ChestExtraPy")
                .field("chest_type", &self.chest_type)
                .field("items", &"<items>")
                .field("owner_id", &self.owner_id)
                .finish()
        }
    }

    impl ChestExtraPy {
        pub fn inflate(py: Python<'_>, chest: ChestData) -> PyResult<Py<Self>> {
            let mut items = Vec::with_capacity(ChestData::NUM_SLOTS);
            for slot in chest.save_item_slots {
                items.push(StackedItemPy::inflate(py, slot)?);
            }
            Py::new(
                py,
                Self {
                    chest_type: chest.chest_type.into(),
                    items,
                    owner_id: chest.owner_id,
                    is_in_use: chest.parent.is_in_use,
                    flipped: chest.parent.flipped,
                    paint_color: chest.parent.paint_color,
                    pos_x: chest.parent.parent.pos_x,
                    pos_y: chest.parent.parent.pos_y,
                    float_pos: [
                        chest.parent.parent.float_pos[0].into(),
                        chest.parent.parent.float_pos[1].into(),
                    ],
                    unique_id: *chest.parent.parent.unique_id.inner(),
                },
            )
        }

        pub fn deflate(&self, py: Python<'_>) -> ChestData {
            let mut save_item_slots = [const { StackedItem(vec![]) }; ChestData::NUM_SLOTS];
            for (i, si_py) in self.items.iter().enumerate() {
                if i < ChestData::NUM_SLOTS {
                    save_item_slots[i] = si_py.bind(py).borrow().deflate(py);
                }
            }
            ChestData {
                parent: InteractionObject {
                    parent: DynamicObject {
                        float_pos: [
                            self.float_pos[0].try_into().unwrap_or_default(),
                            self.float_pos[1].try_into().unwrap_or_default(),
                        ],
                        pos_x: self.pos_x,
                        pos_y: self.pos_y,
                        unique_id: UniqueID::new(self.unique_id),
                    },
                    interaction_object_type: 0, // Placeholder
                    is_in_use: self.is_in_use,
                    flipped: self.flipped,
                    paint_color: self.paint_color,
                },
                chest_type: self.chest_type.into(),
                save_item_slots,
                owner_id: self.owner_id.clone(),
            }
        }

        pub fn clone_ref(&self, py: Python<'_>) -> Py<Self> {
            Py::new(
                py,
                Self {
                    chest_type: self.chest_type,
                    items: self.items.iter().map(|i| i.clone_ref(py)).collect(),
                    owner_id: self.owner_id.clone(),
                    is_in_use: self.is_in_use,
                    flipped: self.flipped,
                    paint_color: self.paint_color,
                    pos_x: self.pos_x,
                    pos_y: self.pos_y,
                    float_pos: self.float_pos,
                    unique_id: self.unique_id,
                },
            )
            .unwrap()
        }
    }

    #[pymethods]
    impl ChestExtraPy {
        #[new]
        #[pyo3(signature = (chest_type=ChestTypePy::Standard, owner_id="server".to_string()))]
        fn new(py: Python<'_>, chest_type: ChestTypePy, owner_id: String) -> PyResult<Py<Self>> {
            let mut items = Vec::with_capacity(ChestData::NUM_SLOTS);
            for _ in 0..ChestData::NUM_SLOTS {
                items.push(Py::new(py, StackedItemPy::default())?);
            }
            Py::new(
                py,
                Self {
                    chest_type,
                    items,
                    owner_id,
                    is_in_use: false,
                    flipped: false,
                    paint_color: 0,
                    pos_x: 0,
                    pos_y: 0,
                    float_pos: [0.0, 0.0],
                    unique_id: 0,
                },
            )
        }

        fn __len__(&self) -> usize {
            self.items.len()
        }

        fn __getitem__(&self, index: isize, py: Python<'_>) -> PyResult<Py<StackedItemPy>> {
            let len = self.items.len() as isize;
            let idx = if index < 0 { index + len } else { index };
            if idx < 0 || idx >= len {
                return Err(pyo3::exceptions::PyIndexError::new_err(
                    "index out of range",
                ));
            }
            Ok(self.items[idx as usize].clone_ref(py))
        }

        fn __setitem__(&mut self, index: isize, item: Py<StackedItemPy>) -> PyResult<()> {
            let len = self.items.len() as isize;
            let idx = if index < 0 { index + len } else { index };
            if idx < 0 || idx >= len {
                return Err(pyo3::exceptions::PyIndexError::new_err(
                    "index out of range",
                ));
            }
            self.items[idx as usize] = item;
            Ok(())
        }

        fn __repr__(&self, py: Python<'_>) -> String {
            let items_repr: Vec<String> = self
                .items
                .iter()
                .map(|item| {
                    item.bind(py)
                        .repr()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|_| "<repr error>".to_string())
                })
                .collect();
            format!(
                "ChestExtra(type={:?}, owner_id=\"{}\", items=[{}])",
                self.chest_type,
                self.owner_id,
                items_repr.join(", ")
            )
        }
    }

    #[pyclass(name = "WorkbenchExtra")]
    #[derive(Debug)]
    pub struct WorkbenchExtraPy {
        #[pyo3(get, set)]
        pub workbench_type: WorkbenchTypePy,
        #[pyo3(get, set)]
        pub level: u8,
        #[pyo3(get, set)]
        pub owner_id: String,
        #[pyo3(get, set)]
        pub is_in_use: bool,
        #[pyo3(get, set)]
        pub flipped: bool,
        #[pyo3(get, set)]
        pub paint_color: u16,
        #[pyo3(get, set)]
        pub pos_x: u64,
        #[pyo3(get, set)]
        pub pos_y: u16,
        #[pyo3(get, set)]
        pub float_pos: [f32; 2],
        #[pyo3(get, set)]
        pub unique_id: u64,
        #[pyo3(get, set)]
        pub available_electricity: u64,
        #[pyo3(get, set)]
        pub craft_progress_count: f32,
        #[pyo3(get, set)]
        pub fire_spread_timer: f32,
        #[pyo3(get, set)]
        pub fuel_fraction: f32,
        #[pyo3(get, set)]
        pub has_fuel: bool,
        #[pyo3(get, set)]
        pub hurry_cost: u64,
        #[pyo3(get, set)]
        pub hurry_seconds: f32,
        #[pyo3(get, set)]
        pub hurry_timer: f32,
        #[pyo3(get, set)]
        pub hurrying: bool,
        #[pyo3(get, set)]
        pub last_world_time: f32,
        #[pyo3(get, set)]
        pub save_time: f32,
        #[pyo3(get, set)]
        pub selected_index: u8,
        #[pyo3(get, set)]
        pub x_scroll: f32,
    }

    impl WorkbenchExtraPy {
        pub fn inflate(py: Python<'_>, workbench: WorkbenchData) -> PyResult<Py<Self>> {
            Py::new(
                py,
                Self {
                    workbench_type: workbench.workbench_type.into(),
                    level: workbench.level,
                    owner_id: workbench.owner_id,
                    is_in_use: workbench.parent.is_in_use,
                    flipped: workbench.parent.flipped,
                    paint_color: workbench.parent.paint_color,
                    pos_x: workbench.parent.parent.pos_x,
                    pos_y: workbench.parent.parent.pos_y,
                    float_pos: [
                        workbench.parent.parent.float_pos[0].get(),
                        workbench.parent.parent.float_pos[1].get(),
                    ],
                    unique_id: *workbench.parent.parent.unique_id.inner(),

                    available_electricity: workbench.available_electricity,
                    craft_progress_count: workbench.craft_progress_count.get(),
                    fire_spread_timer: workbench.fire_spread_timer.get(),
                    fuel_fraction: workbench.fuel_fraction.get(),
                    has_fuel: workbench.has_fuel,
                    hurry_cost: workbench.hurry_cost,
                    hurry_seconds: workbench.hurry_seconds.get(),
                    hurry_timer: workbench.hurry_timer.get(),
                    hurrying: workbench.hurrying,
                    last_world_time: workbench.last_world_time.get(),
                    save_time: workbench.save_time.get(),
                    selected_index: workbench.selected_index,
                    x_scroll: workbench.x_scroll.get(),
                },
            )
        }

        pub fn deflate(&self) -> WorkbenchData {
            WorkbenchData {
                parent: InteractionObject {
                    parent: DynamicObject {
                        float_pos: [
                            self.float_pos[0].try_into().unwrap_or_default(),
                            self.float_pos[1].try_into().unwrap_or_default(),
                        ],
                        pos_x: self.pos_x,
                        pos_y: self.pos_y,
                        unique_id: UniqueID::new(self.unique_id),
                    },
                    interaction_object_type: 0,
                    is_in_use: self.is_in_use,
                    flipped: self.flipped,
                    paint_color: self.paint_color,
                },
                available_electricity: self.available_electricity,
                craft_progress_count: self.craft_progress_count.try_into().unwrap(),
                fire_spread_timer: self.fire_spread_timer.try_into().unwrap(),
                fuel_fraction: self.fuel_fraction.try_into().unwrap(),
                has_fuel: self.has_fuel,
                hurry_cost: self.hurry_cost,
                hurry_seconds: self.hurry_seconds.try_into().unwrap(),
                hurry_timer: self.hurry_timer.try_into().unwrap(),
                hurrying: self.hurrying,
                last_world_time: self.last_world_time.try_into().unwrap(),
                level: self.level,
                save_time: self.save_time.try_into().unwrap(),
                owner_id: self.owner_id.clone(),
                selected_index: self.selected_index,
                workbench_type: self.workbench_type.into(),
                x_scroll: self.x_scroll.try_into().unwrap(),
            }
        }

        pub fn clone_ref(&self, py: Python<'_>) -> Py<Self> {
            Py::new(
                py,
                Self {
                    workbench_type: self.workbench_type,
                    level: self.level,
                    owner_id: self.owner_id.clone(),
                    is_in_use: self.is_in_use,
                    flipped: self.flipped,
                    paint_color: self.paint_color,
                    pos_x: self.pos_x,
                    pos_y: self.pos_y,
                    float_pos: self.float_pos,
                    unique_id: self.unique_id,

                    available_electricity: self.available_electricity,
                    craft_progress_count: self.craft_progress_count,
                    fire_spread_timer: self.fire_spread_timer,
                    fuel_fraction: self.fuel_fraction,
                    has_fuel: self.has_fuel,
                    hurry_cost: self.hurry_cost,
                    hurry_seconds: self.hurry_seconds,
                    hurry_timer: self.hurry_timer,
                    hurrying: self.hurrying,
                    last_world_time: self.last_world_time,
                    save_time: self.save_time,
                    selected_index: self.selected_index,
                    x_scroll: self.x_scroll,
                },
            )
            .unwrap()
        }
    }

    #[pymethods]
    impl WorkbenchExtraPy {
        #[new]
        #[pyo3(signature = (workbench_type=WorkbenchTypePy::Workbench, level=1, owner_id="server".to_string()))]
        fn new(
            py: Python<'_>,
            workbench_type: WorkbenchTypePy,
            level: u8,
            owner_id: String,
        ) -> PyResult<Py<Self>> {
            Py::new(
                py,
                Self {
                    workbench_type,
                    level,
                    owner_id,
                    is_in_use: false,
                    flipped: false,
                    paint_color: 0,
                    pos_x: 0,
                    pos_y: 0,
                    float_pos: [0.0, 0.0],
                    unique_id: 0,

                    available_electricity: 0,
                    craft_progress_count: 0.0,
                    fire_spread_timer: 0.0,
                    fuel_fraction: 0.0,
                    has_fuel: false,
                    hurry_cost: 0,
                    hurry_seconds: 0.0,
                    hurry_timer: 0.0,
                    hurrying: false,
                    last_world_time: 0.0,
                    save_time: 0.0,
                    selected_index: 0,
                    x_scroll: 0.0,
                },
            )
        }

        fn __repr__(&self) -> String {
            format!(
                "WorkbenchExtra(type={:?}, level={}, owner_id=\"{}\", pos_x={}, pos_y={})",
                self.workbench_type, self.level, self.owner_id, self.pos_x, self.pos_y
            )
        }
    }

    #[derive(FromPyObject, IntoPyObject)]
    pub enum ItemExtraPy {
        #[pyo3(transparent)]
        Basket(Py<BasketExtraPy>),
        #[pyo3(transparent)]
        Chest(Py<ChestExtraPy>),
        #[pyo3(transparent)]
        Workbench(Py<WorkbenchExtraPy>),
    }

    impl ItemExtraPy {
        pub fn clone_ref(&self, py: Python<'_>) -> Self {
            match self {
                Self::Basket(b) => Self::Basket(b.clone_ref(py)),
                Self::Chest(c) => Self::Chest(c.clone_ref(py)),
                Self::Workbench(w) => Self::Workbench(w.clone_ref(py)),
            }
        }
    }

    impl std::fmt::Debug for ItemExtraPy {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Python::attach(|py| match self {
                Self::Basket(basket_py) => f
                    .debug_tuple("BasketExtra")
                    .field(&basket_py.bind(py).borrow())
                    .finish(),
                Self::Chest(chest_py) => f
                    .debug_tuple("ChestExtra")
                    .field(&chest_py.bind(py).borrow())
                    .finish(),
                Self::Workbench(bench_py) => f
                    .debug_tuple("WorkbenchExtra")
                    .field(&bench_py.bind(py).borrow())
                    .finish(),
            })
        }
    }

    impl ItemExtraPy {
        pub fn inflate(py: Python<'_>, extra: Extra) -> PyResult<Self> {
            match extra {
                Extra::Basket(items) => {
                    let py_items = items
                        .into_iter()
                        .map(|si| StackedItemPy::inflate(py, si))
                        .collect::<PyResult<Vec<_>>>()?;
                    Ok(Self::Basket(Py::new(
                        py,
                        BasketExtraPy { items: py_items },
                    )?))
                }
                Extra::Chest(chest) => Ok(Self::Chest(ChestExtraPy::inflate(py, *chest)?)),
                Extra::Workbench(bench) => {
                    Ok(Self::Workbench(WorkbenchExtraPy::inflate(py, *bench)?))
                }
            }
        }

        pub fn deflate(&self, py: Python<'_>) -> Extra {
            match self {
                Self::Basket(basket_py) => {
                    let basket = basket_py.bind(py).borrow();
                    let mut items = [const { StackedItem(vec![]) }; Extra::NUM_SLOT_BASKET];
                    for (i, si) in basket.items.iter().enumerate() {
                        items[i] = si.bind(py).borrow().deflate(py);
                    }
                    Extra::Basket(items)
                }
                Self::Chest(chest_py) => {
                    let chest = chest_py.bind(py).borrow();
                    Extra::Chest(Box::new(chest.deflate(py)))
                }
                Self::Workbench(bench_py) => {
                    let bench = bench_py.bind(py).borrow();
                    Extra::Workbench(Box::new(bench.deflate()))
                }
            }
        }
    }

    #[pyclass(name = "Item")]
    pub struct ItemPy {
        #[pyo3(get, set)]
        pub type_id: u16,
        #[pyo3(get, set)]
        pub data_a: u16,
        #[pyo3(get, set)]
        pub data_b: u16,
        #[pyo3(get, set)]
        pub selected_sub_item_index: u8,
        #[pyo3(get, set)]
        pub padding: u8,
        pub extra: Option<ItemExtraPy>,
    }

    impl std::fmt::Debug for ItemPy {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ItemPy")
                .field("type_id", &self.type_id)
                .field("data_a", &self.data_a)
                .field("data_b", &self.data_b)
                .field("sub_index", &self.selected_sub_item_index)
                .field("padding", &self.padding)
                .field("extra", &self.extra)
                .finish()
        }
    }

    impl ItemPy {
        pub fn inflate(py: Python<'_>, item: Item) -> PyResult<Py<Self>> {
            let extra = match item.extra {
                Some(e) => Some(ItemExtraPy::inflate(py, e)?),
                None => None,
            };
            Py::new(
                py,
                Self {
                    type_id: item.type_id,
                    data_a: item.data_a,
                    data_b: item.data_b,
                    selected_sub_item_index: item.selected_sub_item_index,
                    padding: item.padding,
                    extra,
                },
            )
        }

        pub fn deflate(&self, py: Python<'_>) -> Item {
            let extra = self.extra.as_ref().map(|e| e.deflate(py));
            Item {
                type_id: self.type_id,
                data_a: self.data_a,
                data_b: self.data_b,
                selected_sub_item_index: self.selected_sub_item_index,
                padding: self.padding,
                extra,
            }
        }
    }

    impl From<Item> for ItemPy {
        fn from(item: Item) -> Self {
            Self {
                type_id: item.type_id,
                data_a: item.data_a,
                data_b: item.data_b,
                selected_sub_item_index: item.selected_sub_item_index,
                padding: item.padding,
                extra: None,
            }
        }
    }

    #[pymethods]
    impl ItemPy {
        #[new]
        #[pyo3(signature = (item_type, extra=None))]
        fn new(item_type: ItemTypePy, extra: Option<ItemExtraPy>) -> Self {
            let mut s = Self::from(Item::new(item_type.into()));
            s.extra = extra;
            s
        }

        #[getter]
        fn get_item_type(&self) -> PyResult<ItemTypePy> {
            ItemTypePy::try_from(self.type_id).map_err(|_| {
                PyValueError::new_err(format!("Invalid item type id: {}", self.type_id))
            })
        }

        #[setter]
        fn set_item_type(&mut self, item_type: ItemTypePy) {
            self.type_id = item_type as u16;
        }

        #[getter]
        fn get_damage(&self) -> u16 {
            self.data_a
        }

        #[setter]
        fn set_damage(&mut self, damage: u16) {
            self.data_a = damage;
        }

        #[getter]
        fn get_colors(&self) -> PyResult<Vec<PigmentColorPy>> {
            let colors = Item::decode_colors(self.data_b).map_err(into_py_err)?;
            Ok(colors.into_iter().map(Into::into).collect())
        }

        #[setter]
        fn set_colors(&mut self, colors: Vec<PigmentColorPy>) -> PyResult<()> {
            if colors.len() != 3 {
                return Err(PyValueError::new_err("colors must have exactly 3 elements"));
            }
            let mut array = [PigmentColor::Transparent; 3];
            for (i, color) in colors.into_iter().enumerate() {
                array[i] = color.into();
            }
            self.data_b = Item::encode_colors(array);
            Ok(())
        }

        #[getter]
        fn get_extra(&self, py: Python<'_>) -> Option<ItemExtraPy> {
            self.extra.as_ref().map(|e| e.clone_ref(py))
        }

        #[setter]
        fn set_extra(&mut self, extra: Option<ItemExtraPy>) {
            self.extra = extra;
        }

        fn __repr__(&self, py: Python<'_>) -> String {
            let extra_repr = match &self.extra {
                Some(e) => match e {
                    ItemExtraPy::Basket(b) => b
                        .bind(py)
                        .repr()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|_| "<repr error>".to_string()),
                    ItemExtraPy::Chest(c) => c
                        .bind(py)
                        .repr()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|_| "<repr error>".to_string()),
                    ItemExtraPy::Workbench(w) => w
                        .bind(py)
                        .repr()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|_| "<repr error>".to_string()),
                },
                None => "None".to_string(),
            };
            format!(
                "Item(type_id={}, data_a={}, data_b={}, sub_index={}, padding={}, extra={})",
                self.type_id,
                self.data_a,
                self.data_b,
                self.selected_sub_item_index,
                self.padding,
                extra_repr
            )
        }
    }

    #[pyclass(name = "Inventory")]
    #[derive(Debug)]
    pub struct InventoryPy {
        #[pyo3(get, set)]
        pub slots: Vec<Py<StackedItemPy>>,
    }

    impl InventoryPy {
        pub fn inflate(py: Python<'_>, inventory: Inventory) -> PyResult<Py<Self>> {
            let mut slots = Vec::with_capacity(Inventory::NUM_SLOTS);
            for slot in inventory.0 {
                slots.push(StackedItemPy::inflate(py, slot)?);
            }
            Py::new(py, Self { slots })
        }

        pub fn deflate(&self, py: Python<'_>) -> Inventory {
            let mut slots = [const { StackedItem(vec![]) }; Inventory::NUM_SLOTS];
            for (i, slot_py) in self.slots.iter().enumerate() {
                if i < Inventory::NUM_SLOTS {
                    slots[i] = slot_py.bind(py).borrow().deflate(py);
                }
            }
            Inventory(slots)
        }

        pub fn clone_ref(&self, py: Python<'_>) -> Py<Self> {
            Py::new(
                py,
                Self {
                    slots: self.slots.iter().map(|s| s.clone_ref(py)).collect(),
                },
            )
            .unwrap()
        }
    }

    #[pymethods]
    impl InventoryPy {
        #[new]
        #[pyo3(signature = (slots=None))]
        fn new(py: Python<'_>, slots: Option<Vec<Py<StackedItemPy>>>) -> PyResult<Py<Self>> {
            let slots = match slots {
                Some(s) => {
                    if s.len() != Inventory::NUM_SLOTS {
                        return Err(PyValueError::new_err(format!(
                            "Inventory must have exactly {} slots",
                            Inventory::NUM_SLOTS
                        )));
                    }
                    s
                }
                None => {
                    let mut s = Vec::with_capacity(Inventory::NUM_SLOTS);
                    for _ in 0..Inventory::NUM_SLOTS {
                        s.push(Py::new(py, StackedItemPy::default())?);
                    }
                    s
                }
            };
            Py::new(py, Self { slots })
        }

        fn __len__(&self) -> usize {
            self.slots.len()
        }

        fn __getitem__(&self, index: isize, py: Python<'_>) -> PyResult<Py<StackedItemPy>> {
            let len = self.slots.len() as isize;
            let idx = if index < 0 { index + len } else { index };
            if idx < 0 || idx >= len {
                return Err(pyo3::exceptions::PyIndexError::new_err(
                    "index out of range",
                ));
            }
            Ok(self.slots[idx as usize].clone_ref(py))
        }

        fn __setitem__(&mut self, index: isize, item: Py<StackedItemPy>) -> PyResult<()> {
            let len = self.slots.len() as isize;
            let idx = if index < 0 { index + len } else { index };
            if idx < 0 || idx >= len {
                return Err(pyo3::exceptions::PyIndexError::new_err(
                    "index out of range",
                ));
            }
            self.slots[idx as usize] = item;
            Ok(())
        }

        fn __repr__(&self, py: Python<'_>) -> String {
            let slots_repr: Vec<String> = self
                .slots
                .iter()
                .map(|slot| {
                    slot.bind(py)
                        .repr()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|_| "<repr error>".to_string())
                })
                .collect();
            format!("Inventory(slots=[{}])", slots_repr.join(", "))
        }
    }

    #[pyclass(name = "StackedItem")]
    #[derive(Default)]
    pub struct StackedItemPy {
        #[pyo3(get, set)]
        pub items: Vec<Py<ItemPy>>,
    }

    impl std::fmt::Debug for StackedItemPy {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("StackedItemPy")
                .field("items", &"<items>")
                .finish()
        }
    }

    impl StackedItemPy {
        pub fn inflate(py: Python<'_>, stacked: StackedItem) -> PyResult<Py<Self>> {
            let mut items = Vec::with_capacity(stacked.0.len());
            for item in stacked.0 {
                items.push(ItemPy::inflate(py, item)?);
            }
            Py::new(py, Self { items })
        }

        pub fn deflate(&self, py: Python<'_>) -> StackedItem {
            StackedItem(
                self.items
                    .iter()
                    .map(|item_py| item_py.bind(py).borrow().deflate(py))
                    .collect(),
            )
        }
    }

    #[pymethods]
    impl StackedItemPy {
        #[new]
        #[pyo3(signature = (items=None))]
        fn new(items: Option<Vec<Py<ItemPy>>>) -> Self {
            Self {
                items: items.unwrap_or_default(),
            }
        }

        fn __len__(&self) -> usize {
            self.items.len()
        }

        fn __getitem__(&self, py: Python<'_>, mut index: isize) -> PyResult<Py<ItemPy>> {
            if index < 0 {
                index += self.items.len() as isize;
            }
            if index < 0 || index >= self.items.len() as isize {
                return Err(pyo3::exceptions::PyIndexError::new_err(
                    "index out of range",
                ));
            }
            Ok(self.items[index as usize].clone_ref(py))
        }

        fn __setitem__(&mut self, mut index: isize, value: Py<ItemPy>) -> PyResult<()> {
            if index < 0 {
                index += self.items.len() as isize;
            }
            if index < 0 || index >= self.items.len() as isize {
                return Err(pyo3::exceptions::PyIndexError::new_err(
                    "index out of range",
                ));
            }
            self.items[index as usize] = value;
            Ok(())
        }

        fn __repr__(&self, py: Python<'_>) -> String {
            let items_repr: Vec<String> = self
                .items
                .iter()
                .map(|item| format!("{:?}", item.bind(py).borrow()))
                .collect();
            format!("StackedItem(items=[{}])", items_repr.join(", "))
        }
    }
}

pub use item::{
    BasketExtraPy, ChestExtraPy, ChestTypePy, InventoryPy, ItemPy, ItemTypePy, PigmentColorPy,
    StackedItemPy, WorkbenchExtraPy, WorkbenchTypePy,
};

mod world_db {
    use super::{into_py_err, item::InventoryPy, lib, ChunksPy, SharedWorldDb};
    use lib::game::db::world_db::WorldDb;
    use pyo3::prelude::*;
    use std::{
        borrow::Cow,
        sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
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

        #[getter]
        fn get_inventory(&self, py: Python<'_>) -> PyResult<Option<Py<InventoryPy>>> {
            let world_db = self.read();
            let unique_id = world_db.main.blockheads[self.index].obj.unique_id.clone();
            if let Some(inv) = world_db.main.blockhead_inventories.get(&unique_id) {
                Ok(Some(InventoryPy::inflate(py, inv.clone())?))
            } else {
                Ok(None)
            }
        }

        #[setter]
        fn set_inventory(
            &self,
            py: Python<'_>,
            inventory: Option<Py<InventoryPy>>,
        ) -> PyResult<()> {
            let mut world_db = self.write();
            let unique_id = world_db.main.blockheads[self.index].obj.unique_id.clone();
            if let Some(inv_py) = inventory {
                let inv = inv_py.bind(py).borrow().deflate(py);
                world_db.main.blockhead_inventories.insert(unique_id, inv);
            } else {
                world_db.main.blockhead_inventories.remove(&unique_id);
            }
            Ok(())
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

        #[getter]
        fn blockhead_inventory_keys(&self) -> std::collections::HashSet<u64> {
            self.inner
                .read()
                .unwrap()
                .main
                .blockhead_inventories
                .keys()
                .map(|id| *id.inner())
                .collect()
        }

        fn get_blockhead_inventory(
            &self,
            py: Python<'_>,
            id: u64,
        ) -> PyResult<Option<Py<InventoryPy>>> {
            let world_db = self.inner.read().unwrap();
            if let Some(inv) = world_db
                .main
                .blockhead_inventories
                .get(&lib::game::dw::dynamic_object::UniqueID::new(id))
            {
                Ok(Some(InventoryPy::inflate(py, inv.clone())?))
            } else {
                Ok(None)
            }
        }

        fn set_blockhead_inventory(
            &self,
            py: Python<'_>,
            id: u64,
            inventory: Option<Py<InventoryPy>>,
        ) -> PyResult<()> {
            let mut world_db = self.inner.write().unwrap();
            let unique_id = lib::game::dw::dynamic_object::UniqueID::new(id);
            if let Some(inv_py) = inventory {
                let inv = inv_py.bind(py).borrow().deflate(py);
                world_db.main.blockhead_inventories.insert(unique_id, inv);
            } else {
                world_db.main.blockhead_inventories.remove(&unique_id);
            }
            Ok(())
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

    m.add_class::<ItemTypePy>()?;
    m.add_class::<ChestTypePy>()?;
    m.add_class::<WorkbenchTypePy>()?;
    m.add_class::<PigmentColorPy>()?;

    m.add_class::<ItemPy>()?;
    m.add_class::<StackedItemPy>()?;
    m.add_class::<BasketExtraPy>()?;
    m.add_class::<InventoryPy>()?;
    m.add_class::<ChestExtraPy>()?;
    m.add_class::<WorkbenchExtraPy>()?;

    m.add_class::<world_db::WorldDbMainPy>()?;
    m.add_class::<world_db::WorldV2Py>()?;
    m.add_class::<world_db::DynamicWorldV2Py>()?;
    m.add_class::<world_db::BlockheadPy>()?;

    Ok(())
}
