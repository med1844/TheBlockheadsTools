use super::dynamic_object::{chest::ChestData, workbench::Workbench};
use crate::{
    BhError, BhResult,
    util::gzip::{compress_into, decompress},
};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize, de::Error as DeError, ser::Error as SerError};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};
use strum_macros::IntoStaticStr;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    IntoStaticStr,
    TryFromPrimitive,
    Serialize_repr,
    Deserialize_repr,
)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Extra {
    Basket([Slot; Self::NUM_SLOT_BASKET]),
    Chest(Box<ChestData>),
    Workbench(Box<Workbench>),
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

pub(crate) struct AsDisplay<'a, T>(pub(crate) &'a T);

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub type_id: u16,
    pub data_a: u16,
    pub data_b: u16,
    pub selected_sub_item_index: u8,
    pub padding: u8,
    pub extra: Option<Extra>,
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
            let extra_bytes = decompress(&compressed_extra).map_err(|e| {
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
            compress_into(&serialized_extra, &mut buffer).map_err(|e| {
                S::Error::custom(format!("Failed to compress item extra data: {}", e))
            })?;
        }
        // Wrap in plist::Data so it renders as <data> in the XML
        plist::Data::new(buffer).serialize(serializer)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
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

pub fn item_type_to_str(item_type: BhResult<ItemType>) -> String {
    item_type
        .map(|item_type| {
            let item_type_str: &'static str = item_type.into();
            item_type_str.to_string()
        })
        .unwrap_or_else(|e| e.to_string())
}

pub trait ItemView {
    fn type_id(&self) -> u16;
    fn fmt_extra(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    fn has_extra(&self) -> bool;
}

pub fn fmt_item_display<T: ItemView + ?Sized>(
    item: &T,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let type_id = item.type_id();
    let item_type = ItemType::try_from(type_id).map_err(|e| BhError::InvalidItemTypeId(e.number));
    let type_name = item_type_to_str(item_type);
    if item.has_extra() {
        f.write_str(&type_name)?;
        f.write_str("(")?;
        item.fmt_extra(f)?;
        f.write_str(")")
    } else {
        f.write_str(&type_name)
    }
}

pub trait SlotView {
    fn len(&self) -> usize;
    fn item_type_id(&self, index: usize) -> u16;
    fn item_has_extra(&self, index: usize) -> bool;
    fn fmt_item(&self, index: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

struct SlotItemDisplay<'a, S: SlotView + ?Sized> {
    slot: &'a S,
    index: usize,
}

impl<'a, S: SlotView + ?Sized> std::fmt::Debug for SlotItemDisplay<'a, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.slot.fmt_item(self.index, f)
    }
}

pub fn fmt_slot_display<S: SlotView + ?Sized>(
    slot: &S,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    if !slot.is_empty() {
        let count = slot.len();
        let first_type = slot.item_type_id(0);

        let all_same_type = (0..count).all(|i| slot.item_type_id(i) == first_type);
        let any_extra = (0..count).any(|i| slot.item_has_extra(i));
        let is_stacked = count > 1;

        let item_type =
            ItemType::try_from(first_type).map_err(|e| BhError::InvalidItemTypeId(e.number));
        let type_name = item_type_to_str(item_type);

        match (all_same_type, any_extra, is_stacked) {
            (true, false, _) => {
                write!(f, "{} {}", count, type_name)
            }
            (_, _, false) => slot.fmt_item(0, f),
            _ => {
                let mut list = f.debug_list();
                for i in 0..count {
                    list.entry(&SlotItemDisplay { slot, index: i });
                }
                list.finish()
            }
        }
    } else {
        f.write_str("Empty")
    }
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

    pub fn encode_colors(colors: [PigmentColor; Self::MAX_COLORS]) -> u16 {
        let mut color_bits = 0;
        for color in colors.into_iter() {
            color_bits <<= 4;
            color_bits |= color as u16;
        }
        color_bits << 4
    }

    pub fn decode_colors(mut color_bits: u16) -> BhResult<[PigmentColor; Self::MAX_COLORS]> {
        let mut colors = [PigmentColor::Transparent; _];
        color_bits >>= 4;
        for color_mut in colors.iter_mut().rev() {
            *color_mut = PigmentColor::try_from((color_bits & 0b1111) as u8)
                .map_err(|e| BhError::InvalidColorId(e.number))?;
            color_bits >>= 4;
        }
        Ok(colors)
    }

    pub fn color(&self) -> BhResult<[PigmentColor; Self::MAX_COLORS]> {
        Self::decode_colors(self.data_b)
    }

    pub fn set_color(&mut self, colors: [PigmentColor; Self::MAX_COLORS]) {
        *self.color_raw_mut() = Self::encode_colors(colors);
    }

    pub fn extra(&self) -> &Option<Extra> {
        &self.extra
    }

    pub fn new(item_type: ItemType) -> Self {
        Self {
            type_id: item_type as u16,
            data_a: 0,
            data_b: 0,
            selected_sub_item_index: 0,
            padding: 0,
            extra: None,
        }
    }
}

impl ItemView for Item {
    fn type_id(&self) -> u16 {
        self.type_id
    }

    fn has_extra(&self) -> bool {
        self.extra.is_some()
    }

    fn fmt_extra(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(extra) = &self.extra {
            Display::fmt(extra, f)
        } else {
            Ok(())
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_item_display(self, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Slot(pub Vec<Item>); // TODO consider switch to smallvec

impl Slot {
    pub fn new(items: Vec<Item>) -> Self {
        Self(items)
    }
}

impl Deref for Slot {
    type Target = Vec<Item>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Slot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SlotView for Slot {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn item_type_id(&self, index: usize) -> u16 {
        self.0[index].type_id
    }

    fn item_has_extra(&self, index: usize) -> bool {
        self.0[index].extra.is_some()
    }

    fn fmt_item(&self, index: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0[index], f)
    }
}

impl Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_slot_display(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::{Extra, Item, ItemType, PigmentColor, Slot};

    #[test]
    fn test_color_round_trip() {
        let mut item = Item {
            type_id: ItemType::Flint as u16,
            data_a: 0,
            data_b: 0,
            selected_sub_item_index: 0,
            padding: 0,
            extra: None,
        };

        let colors = [
            PigmentColor::RedOchre,
            PigmentColor::EmeraldGreen,
            PigmentColor::UltraMarineBlue,
        ];

        item.set_color(colors);
        let recovered = item.color().unwrap();
        assert_eq!(recovered, colors);
    }

    #[test]
    fn test_item_serialization_basic() {
        let items = vec![
            Item {
                type_id: ItemType::Flint as u16,
                data_a: 10,
                data_b: 20,
                selected_sub_item_index: 1,
                padding: 0,
                extra: None,
            },
            Item {
                type_id: ItemType::SteelPickaxe as u16,
                data_a: 100,
                data_b: 0,
                selected_sub_item_index: 0,
                padding: 0,
                extra: None,
            },
        ];

        for item in items {
            let serialized = plist::to_value(&item).unwrap();
            let deserialized: Item = plist::from_value(&serialized).unwrap();
            assert_eq!(item, deserialized);
        }
    }

    #[test]
    fn test_invalid_item_id() {
        let item = Item {
            type_id: 9999,
            data_a: 0,
            data_b: 0,
            selected_sub_item_index: 0,
            padding: 0,
            extra: None,
        };
        assert!(item.item_type().is_err());
    }

    #[test]
    fn test_extra_basket_isolation() {
        let item_in_basket = Item {
            type_id: ItemType::Apple as u16,
            data_a: 0,
            data_b: 0,
            selected_sub_item_index: 0,
            padding: 0,
            extra: None,
        };
        let mut basket_items = [const { Slot(vec![]) }; 4];
        basket_items[0] = Slot(vec![item_in_basket]);

        let item = Item {
            type_id: ItemType::Basket as u16,
            data_a: 0,
            data_b: 0,
            selected_sub_item_index: 0,
            padding: 0,
            extra: Some(Extra::Basket(basket_items)),
        };

        let serialized = plist::to_value(&item).unwrap();
        let deserialized: Item = plist::from_value(&serialized).unwrap();
        assert_eq!(item, deserialized);
    }

    #[test]
    fn test_slot_serialization() {
        let item = Item {
            type_id: ItemType::Flint as u16,
            data_a: 0,
            data_b: 0,
            selected_sub_item_index: 0,
            padding: 0,
            extra: None,
        };
        let slot = Slot(vec![item]);
        let serialized = plist::to_value(&slot).unwrap();
        let deserialized: Slot = plist::from_value(&serialized).unwrap();
        assert_eq!(slot, deserialized);
    }
}
