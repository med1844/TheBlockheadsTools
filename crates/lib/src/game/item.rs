use crate::{
    BhError, BhResult,
    game::dw::dynamic_object::DynamicObject,
    util::gzip::{compress_gzip_to, decompress_gzip},
};
use num_enum::{FromPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize, de::Error as DeError, ser::Error as SerError};
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};
use strum_macros::IntoStaticStr;
use typed_floats::NonNaNFinite;

#[derive(Debug, IntoStaticStr, TryFromPrimitive)]
#[repr(u16)]
pub enum ItemType {
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

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionObject {
    #[serde(flatten)]
    parent: DynamicObject,
    interaction_object_type: u64,
    is_in_use: bool,
    flipped: bool,
    paint_color: u16,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, IntoStaticStr, FromPrimitive,
)]
#[serde(from = "u8", into = "u8")]
#[repr(u8)]
pub enum ChestType {
    #[default]
    Standard = 0,
    Safe = 1,
    Shelf = 2,
    Gold = 3,
    Portal = 4,
    DisplayCabinet = 5,
    Feeder = 6,
}

impl From<ChestType> for u8 {
    fn from(value: ChestType) -> Self {
        value as u8
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChestData {
    #[serde(flatten)]
    parent: InteractionObject,
    chest_type: ChestType,
    save_item_slots: [StackedItem; Self::NUM_SLOTS],
    #[serde(rename = "ownerID")]
    pub owner_id: String,
}

impl ChestData {
    pub const NUM_SLOTS: usize = 16;
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, IntoStaticStr, FromPrimitive,
)]
#[serde(from = "u8", into = "u8")]
#[repr(u8)]
pub enum WorkbenchType {
    #[default]
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

impl From<WorkbenchType> for u8 {
    fn from(value: WorkbenchType) -> Self {
        value as u8
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchData {
    #[serde(flatten)]
    parent: InteractionObject,
    available_electricity: u64,
    craft_progress_count: NonNaNFinite<f32>,
    fire_spread_timer: NonNaNFinite<f32>,
    fuel_fraction: NonNaNFinite<f32>,
    has_fuel: bool,
    hurry_cost: u64,
    hurry_seconds: NonNaNFinite<f32>,
    hurry_timer: NonNaNFinite<f32>,
    hurrying: bool,
    last_world_time: NonNaNFinite<f32>,
    level: u8,
    save_time: NonNaNFinite<f64>,
    #[serde(rename = "ownerID")]
    owner_id: String,
    selected_index: u8,
    workbench_type: WorkbenchType,
    x_scroll: NonNaNFinite<f32>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Extra {
    Basket([StackedItem; Self::NUM_SLOT_BASKET]),
    Chest(ChestData),
    Workbench(WorkbenchData),
}

impl Extra {
    pub const NUM_SLOT_BASKET: usize = 4;
}

impl<'de> Deserialize<'de> for Extra {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let dict = plist::Dictionary::deserialize(deserializer)?;
        if let Some(value) = dict.get("s") {
            Ok(Self::Basket(plist::from_value(value).map_err(|e| {
                D::Error::custom(format!("plist error: {}", e))
            })?))
        } else if let Some(value @ plist::Value::Dictionary(d)) = dict.get("d")
            && d.contains_key("chestType")
        {
            Ok(Self::Chest(plist::from_value(value).map_err(|e| {
                D::Error::custom(format!("plist error: {}", e))
            })?))
        } else if let Some(value @ plist::Value::Dictionary(d)) = dict.get("d")
            && d.contains_key("workbenchType")
        {
            Ok(Self::Workbench(plist::from_value(value).map_err(|e| {
                D::Error::custom(format!("plist error: {}", e))
            })?))
        } else {
            dbg!(dict);
            Err(D::Error::custom("No known key in extra"))
        }
    }
}

impl Serialize for Extra {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut dict = plist::Dictionary::new();
        match self {
            Self::Basket(items) => {
                dict.insert(
                    "s".to_string(),
                    plist::to_value(items).map_err(|e| S::Error::custom(e.to_string()))?,
                );
            }
            Self::Chest(chest) => {
                dict.insert(
                    "d".to_string(),
                    plist::to_value(chest).map_err(|e| S::Error::custom(e.to_string()))?,
                );
            }
            Self::Workbench(workbench) => {
                dict.insert(
                    "d".to_string(),
                    plist::to_value(workbench).map_err(|e| S::Error::custom(e.to_string()))?,
                );
            }
        }
        dict.serialize(serializer)
    }
}

struct AsDisplay<'a, T>(&'a T);

impl<'a, T: Display> std::fmt::Debug for AsDisplay<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.0, f)
    }
}

struct ListDisplay<'a, T>(&'a [T]);
impl<'a, T: Display> Display for ListDisplay<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(AsDisplay))
            .finish()
    }
}

impl Display for Extra {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Extra::Basket(items) => f.debug_list().entries(items.iter().map(AsDisplay)).finish(),
            Extra::Chest(chest) => f
                .debug_struct("ChestData")
                .field("type", &chest.chest_type)
                .field("items", &AsDisplay(&ListDisplay(&chest.save_item_slots)))
                .finish(),
            Extra::Workbench(workbench) => std::fmt::Debug::fmt(workbench, f),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Item {
    type_id: u16,
    data_a: u16,
    data_b: u16,
    selected_sub_item_index: u8,
    padding: u8,
    extra: Option<Extra>,
}

impl<'de> Deserialize<'de> for Item {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data: Vec<u8> = plist::Data::deserialize(deserializer)?.into();
        let bytes = data.as_slice();
        if bytes.len() < 8 {
            return Err(D::Error::custom(
                "Item data too short: expected at least 8 bytes",
            ));
        }
        // SAFETY: from_le_bytes expects [u8; 2], we provide slice exactly that long
        let type_id = u16::from_le_bytes(bytes[0..2].try_into().unwrap());
        let data_a = u16::from_le_bytes(bytes[2..4].try_into().unwrap());
        let data_b = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
        let selected_sub_item_index = bytes[6];
        let padding = bytes[7];

        // Pop first 8 bytes and avoid malloc
        let mut compressed_extra = data;
        compressed_extra.drain(0..8);

        let extra = if compressed_extra.is_empty() {
            None
        } else {
            let extra_bytes = decompress_gzip(&compressed_extra).map_err(|e| {
                D::Error::custom(format!("Failed to decompress item extra as gzip: {:?}", e))
            })?;
            Some(
                plist::from_reader_xml(extra_bytes.as_slice())
                    .map_err(|e| D::Error::custom(format!("plist error: {}", e)))?,
            )
        };

        Ok(Self {
            type_id,
            data_a,
            data_b,
            selected_sub_item_index,
            padding,
            extra,
        })
    }
}

impl Serialize for Item {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buffer = Vec::with_capacity(8);

        buffer.extend_from_slice(&self.type_id.to_le_bytes());
        buffer.extend_from_slice(&self.data_a.to_le_bytes());
        buffer.extend_from_slice(&self.data_b.to_le_bytes());
        buffer.push(self.selected_sub_item_index);
        buffer.push(self.padding);
        if let Some(extra) = self.extra.as_ref() {
            let mut serialized_extra = Vec::new();
            plist::to_writer_xml(&mut serialized_extra, extra).map_err(|e| {
                S::Error::custom(format!("Failed to serialize item extra data: {}", e))
            })?;
            compress_gzip_to(&serialized_extra, &mut buffer).map_err(|e| {
                S::Error::custom(format!("Failed to compress item extra data: {}", e))
            })?;
        }
        // Wrap in plist::Data so it renders as <data> in the XML
        plist::Data::new(buffer).serialize(serializer)
    }
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum PigmentColor {
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

fn item_type_to_str(item_type: BhResult<ItemType>) -> String {
    item_type
        .map(|item_type| {
            let item_type_str: &'static str = item_type.into();
            item_type_str.to_string()
        })
        .unwrap_or_else(|e| e.to_string())
}

impl Item {
    pub const MAX_COLORS: usize = 3;

    pub fn item_type_raw(&self) -> u16 {
        self.type_id
    }

    pub fn item_type_raw_mut(&mut self) -> &mut u16 {
        &mut self.type_id
    }

    pub fn item_type(&self) -> BhResult<ItemType> {
        ItemType::try_from(self.item_type_raw()).map_err(|e| BhError::InvalidItemTypeId(e.number))
    }

    pub fn set_item_type(&mut self, item_type: ItemType) {
        *self.item_type_raw_mut() = item_type as u16;
    }

    pub fn damage(&self) -> u16 {
        self.data_a
    }

    pub fn damage_mut(&mut self) -> &mut u16 {
        &mut self.data_a
    }

    pub fn color_raw(&self) -> u16 {
        self.data_b
    }

    pub fn color_raw_mut(&mut self) -> &mut u16 {
        &mut self.data_b
    }

    pub fn color(&self) -> BhResult<[PigmentColor; Self::MAX_COLORS]> {
        let mut color_bits = self.data_b;
        let mut colors = [PigmentColor::Transparent; _];
        for color_mut in colors.iter_mut() {
            *color_mut = PigmentColor::try_from((color_bits & 0b1111) as u8)
                .map_err(|e| BhError::InvalidColorId(e.number))?;
            color_bits >>= 4;
        }
        Ok(colors)
    }

    pub fn set_color(&mut self, colors: [PigmentColor; Self::MAX_COLORS]) {
        let mut color_bits = 0;
        for color in colors {
            color_bits |= color as u16;
            color_bits <<= 4;
        }
        *self.color_raw_mut() = color_bits;
    }

    pub fn extra(&self) -> &Option<Extra> {
        &self.extra
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = item_type_to_str(self.item_type());
        if let Some(extra) = self.extra.as_ref() {
            f.debug_tuple(&type_name).field(&AsDisplay(extra)).finish()
        } else {
            f.write_str(&type_name)
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StackedItem(Vec<Item>); // TODO consider switch to smallvec

impl Deref for StackedItem {
    type Target = Vec<Item>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StackedItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for StackedItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(item) = self.first() {
            let all_same_type = self.iter().all(|other| item.type_id == other.type_id);
            let any_extra = self.iter().any(|item| item.extra().is_some());
            let is_stacked = self.len() > 1;
            let type_name = item_type_to_str(item.item_type());
            match (all_same_type, any_extra, is_stacked) {
                (true, false, _) => {
                    write!(f, "{} {}", self.len(), type_name)
                }
                (_, _, false) => item.fmt(f),
                _ => f.debug_list().entries(self.iter().map(AsDisplay)).finish(),
            }
        } else {
            f.write_str("Empty")
        }
    }
}

// An inventory of a blockhead
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Inventory([StackedItem; Self::NUM_SLOTS]);

impl Inventory {
    pub const NUM_SLOTS: usize = 8;
}

impl Deref for Inventory {
    type Target = [StackedItem; Self::NUM_SLOTS];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Inventory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Inventory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter().map(AsDisplay)).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::Inventory;

    #[test]
    fn inventory_round_trip_test() {
        let inventory_data = b"
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<array>
        <array>
                <data>
                AQAAAAAAAAw=
                </data>
        </array>
        <array>
                <data>
                DAAAAAAAAgwfiwgAAAAAAAAH7dyxbtNQFAbguX0K4z25sCHkpApNQZUiSNV0
                gM2KrWCRJpZtYfL22KmaFAGpGBiQPi/3v77fOb7rWZxcfL9fR9/yqi62m1H8
                avgyjvLNcpsVm9Uovlu8G7yOL8bnyYvpx8vFp/lVVK6Luonmd29n15dRPAhh
                UpbrPITpYhrNZ9e3i6jrEcLVhziKvzRN+SaEtm2Haa+Gy+19D+swr7ZlXjW7
                Wdds0BUMsyaLu888dP/pOt3brFg24/Oz5Gu+G9dJ6Jdul1ZV2ocn6SzJ0ibd
                p9nqZvLwtKP9STgc/Q5NIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiC
                IAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiC
                IAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAj6S5SEtKrS3T4e0pPK
                z6vHwvbP7Y/oxB0gCIIgCIIgCIL+X/Ts7NTuq25/nZ2eq3x/8zivnZi6jujE
                7SEIgiAIgiAIgiAIgiAIgiAIgiAIgiAIgiAIgiAIgiAIgiAIgiAIgiAIgiAI
                giAIgiAIgiAI+ufo+GOhQ+oOi2XTr+W6qLvwA2esHopJ5wAA
                </data>
        </array>
        <array>
                <data>
                DAAAAAAAAgwfiwgAAAAAAAAH7ZZLk6JIFEbXXb/CcWtUJYKITljVwdMSBAFB
                gR2SKCgvIXn++gZ7pucRHRPR2wlzw5fccy65uUGuvjZxNKr8vAjT5H08fcPG
                Iz/xUhgml/exaQivi/HXj5fVb9yONWyVH2VRWKCRajLbDTsavwJAZ1nkA8AZ
                3EjdbvbGqO8BAK+MR+MAoex3AOq6fnMH6s1L4wEsgJqnmZ+jdts3e+2FN4jg
                uP/M9+7/OE7/FoYe+nj5srr57UexAsOj37l57g7hb+nLCrrIfaSjxtH8he5X
                /f6ogD9LK/DfplP3Ekvrv27ytUZ/X/U5rC9/ZPqTcgL9RBY1L/vnGwYYn17s
                XChcNpGGDR4UMGzKSAd112SLgo1OGwbiLaEgcsKIChuHsCkTQztYOVedKxtC
                yuUHD1QnUSNn2XIWtgIDjQM8r++pJLEyzG5md3DOAelIadIhbjITiF0dA2bw
                YlNqo08+OorUfEHG4fXC1Kx302iBSiWbB1WdHgrNo53Gv6eIrI3r7OGZohsx
                IZJCgvEvWs0LjMJPvNSKZwRPh0Rc7HNW2RZgf4P2rDPCLmEuj3Nim1ykHGap
                Sx06mY3e+ZuFxQiYbC8LZcemta47eMt1FsNZuJdt9kE5ePixtGVNm58vDXkR
                bluNzwJVz21B/pQxe4IK13SUe5DkEJsluYav8dYavClER/dA295ZQFG3nKRx
                oNaZCgNBxrekLVMmZkf1MfTmYCpjpMA0+Hrw7mwYc/BkcOb8wnIct3adPbmX
                7LMFbNEvXJWfVLCM5/Jt6TAl0BNJXQyeVKiLyfFOaEuhadd8t1RwoSTm8VWb
                YF1yXkOTcohkg0RZxQgRNXw8dQcv+hQDv6TQbnPDExP5+G3nddJBcZAx7Q4w
                kQ6xnCQlSGStKVFYqkEbDx45pXZ7xwBuu7VIh6LOXSiibBHnVRH5Z6ut1Paq
                EV45OVWG1xEE6uuDZxPxqdrpJj1LGuNyrWrZykoLcPBwNMRalETd511I6Zqe
                NvrVK8ylnAxeeTXx672+Yt2GWt8agQisjNKJ+zxUTpynzlyO2su8FiKXMqsY
                KW2QTAaPiatMXxZ3vp+oX5sk+8fw/HsGfwrRT+gJPaEn9ISe0P8ZWvwc+ut3
                +iP1xceteQUed+qPl29sHDTy6gsAAA==
                </data>
        </array>
        <array>
                <data>
                DAAAAAAAAAwfiwgAYA9lXgL/7dlBT4MwFAfw8/gUtXd4ejOmsGxjJkuIw8gO
                89ZBo0QGpDTWfXsLmqkXx8nD/HPh0ffrn97eoWL6tq/Yq9Jd2dQhvwouOVN1
                3hRl/RTyTXbrX/Np5ImLeL3ItumStVXZGZZu5slqwbhPNGvbShHFWczSZPWQ
                MZdBtLzjjD8b094QWWsD2asgb/Y97CjVTau0OSQuzHcbgsIU3P3mI/3Hcdxq
                UeYm8ibiRR2iTlD/cl9Sa9kX36qJKKSRQ7W7n30+Nhw6dGwBAQEBAQH9AyTo
                90m5HhMPBAQEBAR0tujUpNzaEfFAQEBAQEBni05NyscxgxgICAgICOjv0Nfo
                OlauOdwwChruHyPvHUCTe1QWHQAA
                </data>
        </array>
        <array>
                <data>
                DAAAAAAAAAwfiwgAAAAAAAAH7d3JjqNWFAbgdfdTON5aXRiXx8hVLcxswGAb
                PO3MjBnNaNfTx1R3p5Moqt5FivSzuQf4/stdcAS7O/96i6NO7eRFkCYvXfKp
                3+04iZXaQeK9dA2d+zLtfn39PP+NUWn9qLGdLAqKsqMZC1mkO90vBEFlWeQQ
                BKMzHU0Wt3rnMQdBsKtup+uXZfY7QTRN83Ru1ZOVxi0sCC1PMycv7/Jjsi+P
                wJNd2t3HY77N/rflPK7agVW+fv40D537azEn2uFxds7zc1v8pfo0t8/l+b3i
                vTX17WjcoPG+15Rw2enmpUwbViCdOpBX6rNAu+ey2MV9qs3R66TcBFI6jlZR
                6AnWyjzlKvnWq8OhxzdXf8pvEqsyldhRb4TtzvwoM9tcKGbNLF5OLnFpNgvJ
                YfPTuHDcW8XEzWGo8sTgMBSKI5GO2EjihHNQmW6by0bausgK0d2oNBXKjMP7
                XH8aFqXXCOur8jyYyId9WtW0Sg7Uxk0k2vXbXHy7pwa9HVFSqBl9bb+iIvUa
                aWbExHfirgkbTt2OJeY2EghKNPxZRBNxmxvf1jO/ZvfsMD1w7NFgA3K5rrf+
                hpmMfD46euubJYwOpnLhN44Ur5Klwb6v05AcZTeIthbVtxYDfee8SRG93ygL
                JY6UU0gRYi/xRV+UyGbbb3aprBFtTlAr8dpcqczSuUmhZ/ZpYnu2WjoscTOv
                Nrnzylnmk5kyXlyCzarM5Xve5qzj826gLupzdEsMf0Tn9YbJfDPlt1src7Le
                ePM2jZcKR7JvsX6KRyGnyW1uMqzdky8H3nCR3+0xeZLE2JQm3vKicdua8EL/
                lLiBYFiziqnL1WySJLM2V5b1dZZcGXrWN8rKdaL8YJJv8vCS8hZDV4VSaMou
                ZenK6q/UZLC3RHbV5taWl0VNzw56zJWcTPhLcp+OS5aMxt6xCquiiarUmpq5
                nVzV4qzXTjXYt7kBwVYXjl8fSFtc3u8cH7OkvN8qhJJ4z5Kgyfltnz337ELV
                7scB75TC4X2d0krvF9k0CiNtnAySft8VblfbDIbr9Ow2HL+/7Ht6Yzxe9ZeX
                944gfrTEnPi4Y5QfPUI1/0j+K6KAgICAgICAgICA/gOE31ggICAgICAgICAg
                ICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg
                ICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg
                ICAgICAgICAgICAgICAgICAgICAgICCg/yn61Yb12pr9Hvxgw/qf6IM1AAEB
                AQEB/Qr9/Cr9WT1uBlbZjlkUFI/iD75cbuYdlAAA
                </data>
        </array>
        <array>
                <data>
                DAAAAAAAAAwfiwgAYA9lXgL/nVBNC8IwDD27X1F736I3kW6jbgrC0InzoLey
                Fh3uo3TF6r+3myh6ETGXvOTlvYSQ8FqV6CJUWzS1j8feCCNR5w0v6qOPd9nC
                neAwcMgwXkfZPp0jWRatRululiwjhF0AKmUpAOIsRmmy3GbIegDMVxjhk9Zy
                CmCM8Vg35eVN1Q22kKpGCqVviTVzrcDjmmO75uH+cY7t8iLXgTMgZ3ELWgJd
                shVTinXgDQ0IZ5r16GCojdmRUuP3DDwpAj8pN/8qqflb+XXnC1myfwiB/l2B
                cwc3T8VwxQEAAA==
                </data>
        </array>
        <array>
                <data>
                DAAAAAAAAwwfiwgAYA9lXgL/nZBRC4IwFIWf81esveutt4hpmBYEUkL2UG/D
                jZJMxxyt/n1Xo6iXCAdjZzv3O3dcNrtdSnKVuinqyqdjb0SJrPJaFNXRp7ts
                6U7oLHDYMN5E2T5dEFUWjSHpbp6sIkJdgFCpUgLEWUzSZLXNCGYALNaU0JMx
                agpgrfV4W+Xl9aUtbCDVtZLa3BMMcxHwhBEU2zzTv76Dr6LITeAM2Fneg4ZB
                e+CNa81b8aEGTHDDO3WwIa4It/U7B14Wg3/IeX/S9iXD4y/yrdDsBsKgG1fg
                PABFr9BBxQEAAA==
                </data>
        </array>
        <array>
                <data>
                DAAAAAAAAAwfiwgAYA9lXgL/7dlBT4MwFAfw8/gUtXd4ejOmsOCYyRLiSGQH
                d0PaTCKDpjTivr0FzdSL42jmnwuPvl//9PYOFfO3fc1elemqtgn5VXDJmWrK
                VlbNLuSb/M6/5vPIExfJepE/Zkum66qzLNvcpqsF4z5RrHWtiJI8YVm6esiZ
                yyBa3nPGn63VN0R93wfFoIKy3Q+wo8y0Whl7SF2Y7zYE0krufvOR/uM4blVW
                pY28mXhRh6gTNLzcV2FMMRTfqpmQhS3GSvbx59OHY4eOLSAgICAgoH+ABP0+
                KZ92E+KBgICAgIDOFp2alOt4QjwQEBAQENDZolOTcjslHggICAgI6I+hr/l2
                rFxzvIYUNF5SRt47sNC68TsdAAA=
                </data>
        </array>
</array>
</plist>";
        let inventory: Inventory = plist::from_reader_xml(inventory_data.as_slice())
            .expect("should be able to deserialize");
        let mut round_trip_inventory_data = Vec::new();
        plist::to_writer_xml(&mut round_trip_inventory_data, &inventory).expect("should serialize");
        let round_trip_inventory: Inventory =
            plist::from_reader_xml(round_trip_inventory_data.as_slice())
                .expect("should be able to deserialize");
        assert_eq!(inventory, round_trip_inventory);
    }
}
