use super::{
    super::util::{
        gzip::{compress_into, decompress},
        plist::to_xml_plist,
    },
    dynamic_object::{AnyDynamicObject, DynamicObjectError},
};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use snafu::prelude::*;
use std::ops::{Deref, DerefMut};
use strum_macros::IntoStaticStr;

#[derive(Debug, Snafu)]
pub enum ItemError {
    #[snafu(display("Invalid item type ID {id}"))]
    InvalidItemTypeId {
        id: u16,
        source: num_enum::TryFromPrimitiveError<ItemType>,
    },
    #[snafu(display("Invalid color type ID {id}"))]
    InvalidColorTypeId {
        id: u8,
        source: num_enum::TryFromPrimitiveError<PigmentColor>,
    },
    #[snafu(display("Failed to deserialize extra bytes as plist::Dictionary"))]
    DeserializeExtra { source: plist::Error },
    #[snafu(display("Failed to serialize extra plist::Dictionary to bytes"))]
    SerializeExtra { source: plist::Error },
    #[snafu(display("Failed to load subItem slots"))]
    LoadSubItems { source: Box<ItemError> },
    #[snafu(display("Failed to save subItem slots"))]
    SaveSubItems { source: Box<ItemError> },
    #[snafu(display("Failed to load dynamicObjectSaveDict as AnyDynamicObject"))]
    LoadDynObjSaveDict { source: DynamicObjectError },
    #[snafu(display("Failed to save AnyDynamicObject to dynamicObjectSaveDict"))]
    SaveDynObjSaveDict { source: DynamicObjectError },
    #[snafu(display(
        "Item data too short: expected at least 8 bytes, got {got} bytes, data: {data:?}"
    ))]
    ItemDataTooShort { got: usize, data: Vec<u8> },
    #[snafu(display("Failed to decompress item extra as gzip"))]
    DecompressExtraBytes { source: std::io::Error },
    #[snafu(display("Failed to compress item extra as gzip"))]
    CompressExtraBytes { source: std::io::Error },
    #[snafu(display(
        "Failed to parse {type_name} because input value is not {target_structure}: {value:?}"
    ))]
    UnexpectedStructure {
        type_name: &'static str,
        target_structure: &'static str,
        value: Box<plist::Value>,
    },
    #[snafu(display("Failed to load slots in inventory"))]
    LoadInventory { source: Box<ItemError> },
    #[snafu(display("Failed to save slots in inventory"))]
    SaveInventory { source: Box<ItemError> },
    #[snafu(display("Num slots mismatch: expected {expected}, got {got}"))]
    NumSlotsMismatch { expected: usize, got: usize },
    #[snafu(display("Failed to load {i}-th slot"))]
    LoadSlots { source: Box<ItemError>, i: usize },
    #[snafu(display("Failed to save {i}-th slot"))]
    SaveSlots { source: Box<ItemError>, i: usize },
    #[snafu(display("Failed to load {i}-th item in slot"))]
    LoadSlot { source: Box<ItemError>, i: usize },
    #[snafu(display("Failed to save {i}-th item in slot"))]
    SaveSlot { source: Box<ItemError>, i: usize },
}

type Result<T> = std::result::Result<T, ItemError>;

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

/// Item as stored in dynamic world XML (DroppedItem, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ItemXml {
    pub item_type: u16,
    pub data_a: u16,
    pub data_b: u16,

    #[serde(
        default,
        deserialize_with = "crate::util::serde::deserialize_some",
        serialize_with = "crate::util::serde::serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub sub_items: Option<plist::Value>,

    #[serde(
        default,
        deserialize_with = "crate::util::serde::deserialize_some",
        serialize_with = "crate::util::serde::serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub dynamic_object_save_dict: Option<plist::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub type_id: u16,
    pub data_a: u16,
    pub data_b: u16,
    pub selected_sub_item_index: u8,
    pub padding: u8,
    pub sub_items: Option<Slots<{ Self::MAX_SUB_ITEMS }>>,
    pub dynamic_object: Option<AnyDynamicObject>,
}

impl Item {
    pub const MAX_SUB_ITEMS: usize = 4;

    fn from_sub_item_values(value: plist::Value) -> Result<Option<Slots<{ Self::MAX_SUB_ITEMS }>>> {
        let sub_item_values = match value {
            plist::Value::Array(values) => values,
            _ => UnexpectedStructureSnafu {
                type_name: "subItems",
                target_structure: "plist::Array",
                value,
            }
            .fail()?,
        };
        (!sub_item_values.is_empty())
            .then_some(
                Slots::from_values(sub_item_values)
                    .map_err(Box::new)
                    .context(LoadSubItemsSnafu),
            )
            .transpose()
    }

    fn to_sub_item_values(&self) -> Result<Option<plist::Value>> {
        Ok(match self.sub_items.as_ref() {
            Some(sub_items) => Some(plist::Value::Array(
                sub_items
                    .to_values()
                    .map_err(Box::new)
                    .context(SaveSubItemsSnafu)?,
            )),
            None => None,
        })
    }

    fn from_dyn_obj_save_dict(value: plist::Value) -> Result<AnyDynamicObject> {
        let dict = match value {
            plist::Value::Dictionary(dict) => dict,
            _ => UnexpectedStructureSnafu {
                type_name: "dynamicObjectSaveDict",
                target_structure: "plist::Array",
                value: value.clone(),
            }
            .fail()?,
        };
        AnyDynamicObject::try_from_save_dict(dict).context(LoadDynObjSaveDictSnafu)
    }

    fn to_dynamic_object_save_dict(&self) -> Result<Option<plist::Value>> {
        match self.dynamic_object.as_ref() {
            Some(dynamic_object) => Some(
                dynamic_object
                    .to_save_dict()
                    .context(SaveDynObjSaveDictSnafu),
            )
            .transpose(),
            None => Ok(None),
        }
    }

    fn try_from_bytes(data: Vec<u8>) -> Result<Self> {
        let bytes = data.as_slice();
        if bytes.len() < 8 {
            return ItemDataTooShortSnafu {
                got: bytes.len(),
                data,
            }
            .fail();
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

        let mut sub_items = None;
        let mut dynamic_object = None;
        if !compressed_extra.is_empty() {
            let extra_bytes = decompress(&compressed_extra).context(DecompressExtraBytesSnafu)?;
            let dict: plist::Dictionary =
                plist::from_reader_xml(extra_bytes.as_slice()).context(DeserializeExtraSnafu)?;
            if let Some(value) = dict.get("s") {
                sub_items = Self::from_sub_item_values(value.to_owned())?;
            }
            if let Some(value) = dict.get("d") {
                dynamic_object = Some(Self::from_dyn_obj_save_dict(value.to_owned())?);
            }
        }
        Ok(Self {
            type_id,
            data_a,
            data_b,
            selected_sub_item_index,
            padding,
            sub_items,
            dynamic_object,
        })
    }

    pub(crate) fn try_from_value(value: plist::Value) -> Result<Self> {
        match value {
            plist::Value::Data(item_data) => Item::try_from_bytes(item_data),
            _ => UnexpectedStructureSnafu {
                type_name: "Item",
                target_structure: "plist::Data",
                value,
            }
            .fail(),
        }
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::with_capacity(8);

        buffer.extend_from_slice(&self.type_id.to_le_bytes());
        buffer.extend_from_slice(&self.data_a.to_le_bytes());
        buffer.extend_from_slice(&self.data_b.to_le_bytes());
        buffer.push(self.selected_sub_item_index);
        buffer.push(self.padding);

        let mut dict = plist::Dictionary::new();
        if let Some(sub_items) = self.sub_items.as_ref() {
            dict.insert(
                "s".to_string(),
                plist::Value::Array(
                    sub_items
                        .to_values()
                        .map_err(Box::new)
                        .context(SaveSubItemsSnafu)?,
                ),
            );
        }
        if let Some(obj) = self.dynamic_object.as_ref() {
            dict.insert(
                "d".to_string(),
                obj.to_save_dict().context(SaveDynObjSaveDictSnafu)?,
            );
        }

        let serialized_extra =
            to_xml_plist(&plist::Value::Dictionary(dict)).context(SerializeExtraSnafu)?;
        compress_into(&serialized_extra, &mut buffer).context(CompressExtraBytesSnafu)?;

        Ok(buffer)
    }

    pub(crate) fn to_value(&self) -> Result<plist::Value> {
        let bytes = self.to_bytes()?;
        Ok(plist::Value::Data(bytes))
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

impl Item {
    pub const MAX_COLORS: usize = 3;

    pub fn item_type_raw(&self) -> u16 {
        self.type_id
    }

    pub fn item_type_raw_mut(&mut self) -> &mut u16 {
        &mut self.type_id
    }

    pub fn item_type(&self) -> Result<ItemType> {
        let raw = self.item_type_raw();
        ItemType::try_from(raw).context(InvalidItemTypeIdSnafu { id: raw })
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

    pub fn decode_colors(mut color_bits: u16) -> Result<[PigmentColor; Self::MAX_COLORS]> {
        let mut colors = [PigmentColor::Transparent; _];
        color_bits >>= 4;
        for color_mut in colors.iter_mut().rev() {
            *color_mut = PigmentColor::try_from((color_bits & 0b1111) as u8)
                .with_context(|e| InvalidColorTypeIdSnafu { id: e.number })?;
            color_bits >>= 4;
        }
        Ok(colors)
    }

    pub fn color(&self) -> Result<[PigmentColor; Self::MAX_COLORS]> {
        Self::decode_colors(self.data_b)
    }

    pub fn set_color(&mut self, colors: [PigmentColor; Self::MAX_COLORS]) {
        *self.color_raw_mut() = Self::encode_colors(colors);
    }

    pub fn new(item_type: ItemType) -> Self {
        Self {
            type_id: item_type as u16,
            data_a: 0,
            data_b: 0,
            selected_sub_item_index: 0,
            padding: 0,
            sub_items: None,
            dynamic_object: None,
        }
    }

    pub(crate) fn from_xml(xml: ItemXml) -> Result<Self> {
        let mut sub_items = None;
        let mut dynamic_object = None;
        if let Some(sub_item_values) = xml.sub_items {
            sub_items = Self::from_sub_item_values(sub_item_values)?;
        }
        if let Some(dict) = xml.dynamic_object_save_dict {
            dynamic_object = Some(Self::from_dyn_obj_save_dict(dict)?);
        }

        Ok(Self {
            type_id: xml.item_type,
            data_a: xml.data_a,
            data_b: xml.data_b,
            selected_sub_item_index: 0,
            padding: 0,
            sub_items,
            dynamic_object,
        })
    }

    pub(crate) fn to_xml(&self) -> Result<ItemXml> {
        let sub_items = self.to_sub_item_values()?;
        let dynamic_object_save_dict = self.to_dynamic_object_save_dict()?;

        Ok(ItemXml {
            item_type: self.type_id,
            data_a: self.data_a,
            data_b: self.data_b,
            sub_items,
            dynamic_object_save_dict,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Slot(pub Vec<Item>); // TODO consider switch to smallvec

impl Slot {
    pub fn new(items: Vec<Item>) -> Self {
        Self(items)
    }

    pub fn try_from_value(value: plist::Value) -> Result<Self> {
        match value {
            plist::Value::Array(values) => Ok(Self(
                values
                    .into_iter()
                    .enumerate()
                    .map(|(i, value)| {
                        Item::try_from_value(value)
                            .map_err(Box::new)
                            .context(LoadSlotSnafu { i })
                    })
                    .collect::<Result<Vec<Item>>>()?,
            )),
            _ => UnexpectedStructureSnafu {
                type_name: "Slot",
                target_structure: "plist::Array",
                value,
            }
            .fail(),
        }
    }

    pub fn to_value(&self) -> Result<plist::Value> {
        Ok(plist::Value::Array(
            self.0
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    item.to_value()
                        .map_err(Box::new)
                        .context(SaveSlotSnafu { i })
                })
                .collect::<Result<Vec<_>>>()?,
        ))
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

#[derive(Debug, Clone, PartialEq)]
pub struct Slots<const N: usize>([Slot; N]);

impl<const N: usize> Slots<N> {
    pub fn from_values(values: Vec<plist::Value>) -> Result<Self> {
        if values.len() != N {
            return NumSlotsMismatchSnafu {
                expected: N,
                got: values.len(),
            }
            .fail();
        }
        let mut arr = core::array::from_fn(|_| Slot(Vec::new()));
        for (i, value) in values.into_iter().take(N).enumerate() {
            arr[i] = Slot::try_from_value(value)
                .map_err(Box::new)
                .context(LoadSlotsSnafu { i })?;
        }
        Ok(Self(arr))
    }

    pub fn to_values(&self) -> Result<Vec<plist::Value>> {
        let slot_values = self
            .0
            .iter()
            .enumerate()
            .map(|(i, slot)| {
                slot.to_value()
                    .map_err(Box::new)
                    .context(SaveSlotsSnafu { i })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(slot_values)
    }
}

impl<const N: usize> Deref for Slots<N> {
    type Target = [Slot; N];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for Slots<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// An inventory of a blockhead
#[derive(Debug, Clone, PartialEq)]
pub struct Inventory(Slots<{ Self::NUM_SLOTS }>);

impl Inventory {
    pub const NUM_SLOTS: usize = 8;

    pub fn new(slots: Slots<{ Self::NUM_SLOTS }>) -> Self {
        Self(slots)
    }

    pub fn try_from_value(value: plist::Value) -> Result<Self> {
        match value {
            plist::Value::Array(values) => Ok(Self(
                Slots::from_values(values)
                    .map_err(Box::new)
                    .context(LoadInventorySnafu)?,
            )),
            _ => UnexpectedStructureSnafu {
                type_name: "Inventory",
                target_structure: "plist::Array",
                value,
            }
            .fail(),
        }
    }

    pub fn to_value(&self) -> Result<plist::Value> {
        Ok(plist::Value::Array(
            self.0
                .to_values()
                .map_err(Box::new)
                .context(SaveInventorySnafu)?,
        ))
    }
}

impl Deref for Inventory {
    type Target = Slots<{ Self::NUM_SLOTS }>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Inventory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Inventory {
    type Item = Slot;
    type IntoIter = std::array::IntoIter<Self::Item, { Self::NUM_SLOTS }>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::dynamic_object::{
            AnyDynamicObject, DynamicObject, InteractionObject, InteractionObjectType, UniqueID,
            workbench::{Workbench, WorkbenchType},
        },
        Inventory, Item, ItemType, PigmentColor, Slot, Slots,
    };
    use crate::util::plist::{diff_plist_keys, to_xml_plist};

    #[test]
    fn test_color_round_trip() {
        let mut item = Item {
            type_id: ItemType::Flint as u16,
            data_a: 0,
            data_b: 0,
            selected_sub_item_index: 0,
            padding: 0,
            sub_items: None,
            dynamic_object: None,
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
                sub_items: None,
                dynamic_object: None,
            },
            Item {
                type_id: ItemType::SteelPickaxe as u16,
                data_a: 100,
                data_b: 0,
                selected_sub_item_index: 0,
                padding: 0,
                sub_items: None,
                dynamic_object: None,
            },
        ];

        for item in items {
            let serialized = item.to_value().unwrap();
            let deserialized = Item::try_from_value(serialized).unwrap();
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
            sub_items: None,
            dynamic_object: None,
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
            sub_items: None,
            dynamic_object: None,
        };
        let mut basket_items = [const { Slot(vec![]) }; 4];
        basket_items[0] = Slot(vec![item_in_basket]);

        let item = Item {
            type_id: ItemType::Basket as u16,
            data_a: 0,
            data_b: 0,
            selected_sub_item_index: 0,
            padding: 0,
            sub_items: Some(Slots(basket_items)),
            dynamic_object: None,
        };

        let serialized = item.to_value().unwrap();
        let deserialized = Item::try_from_value(serialized).unwrap();
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
            sub_items: None,
            dynamic_object: None,
        };
        let slot = Slot(vec![item]);
        let serialized = slot.to_value().unwrap();
        let deserialized = Slot::try_from_value(serialized).unwrap();
        assert_eq!(slot, deserialized);
    }

    #[test]
    fn test_standard_chest_extra_deserialization() {
        let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
        <key>chestType</key>
        <integer>0</integer>
        <key>flipped</key>
        <false/>
        <key>floatPos</key>
        <array>
                <real>11191.5</real>
                <real>670</real>
        </array>
        <key>interactionObjectType</key>
        <integer>2</integer>
        <key>isInUse</key>
        <false/>
        <key>ownerID</key>
        <string>server</string>
        <key>paintColor</key>
        <integer>0</integer>
        <key>pos_x</key>
        <integer>11191</integer>
        <key>pos_y</key>
        <integer>670</integer>
        <key>saveItemSlots</key>
        <array>
                <array/>
                <array/>
                <array/>
                <array/>
                <array/>
                <array/>
                <array/>
                <array/>
                <array/>
                <array/>
                <array/>
                <array/>
                <array/>
                <array/>
                <array/>
                <array/>
        </array>
        <key>saveTime</key>
        <real>5018.8335087001324</real>
        <key>uniqueID</key>
        <integer>5952</integer>
</dict>
</plist>";
        let dict = plist::from_reader_xml(xml.as_bytes()).unwrap();
        let dyn_obj = AnyDynamicObject::try_from_save_dict(dict).unwrap();
        assert!(matches!(dyn_obj, AnyDynamicObject::Chest(..)));
    }

    #[test]
    fn test_shelf_extra_deserialization() {
        let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
        <key>chestType</key>
        <integer>2</integer>
        <key>flipped</key>
        <false/>
        <key>floatPos</key>
        <array>
                <real>11191.5</real>
                <real>668</real>
        </array>
        <key>interactionObjectType</key>
        <integer>2</integer>
        <key>isInUse</key>
        <false/>
        <key>ownerID</key>
        <string>server</string>
        <key>paintColor</key>
        <integer>0</integer>
        <key>pos_x</key>
        <integer>11191</integer>
        <key>pos_y</key>
        <integer>668</integer>
        <key>saveItemSlots</key>
        <array>
                <array>
                        <data>
                        8wAAAAAAAAA=
                        </data>
                </array>
                <array>
                        <data>
                        +QAAAAAAAAA=
                        </data>
                </array>
                <array>
                        <data>
                        LgQAAAAAAAA=
                        </data>
                </array>
                <array/>
        </array>
        <key>saveTime</key>
        <real>5034.406163521111</real>
        <key>uniqueID</key>
        <integer>836</integer>
</dict>
</plist>";
        let dict = plist::from_reader_xml(xml.as_bytes()).unwrap();
        let dyn_obj = AnyDynamicObject::try_from_save_dict(dict).unwrap();
        assert!(matches!(dyn_obj, AnyDynamicObject::Chest(..)));
    }

    #[test]
    fn test_portal_chest_extra_deserialization() {
        let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
        <key>chestType</key>
        <integer>4</integer>
        <key>flipped</key>
        <false/>
        <key>floatPos</key>
        <array>
                <real>11193.5</real>
                <real>668</real>
        </array>
        <key>interactionObjectType</key>
        <integer>2</integer>
        <key>isInUse</key>
        <false/>
        <key>ownerID</key>
        <string>server</string>
        <key>paintColor</key>
        <integer>0</integer>
        <key>pos_x</key>
        <integer>11193</integer>
        <key>pos_y</key>
        <integer>668</integer>
        <key>saveTime</key>
        <real>5061.7033485174179</real>
        <key>uniqueID</key>
        <integer>838</integer>
</dict>
</plist>";
        let dict = plist::from_reader_xml(xml.as_bytes()).unwrap();
        let dyn_obj = AnyDynamicObject::try_from_save_dict(dict).unwrap();
        assert!(matches!(dyn_obj, AnyDynamicObject::Chest(..)));
    }

    #[test]
    fn test_basket_with_bed_deserialization() {
        let xml = "
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
    <data>
        DAAAAAAAAZIfiwgAAAAAAAAH7Zlbc6JIFMefJ5/C9ZVKwBvolMlUcxMQuSio
        +IagaARBQNF8+gUzk93Z2kqfhzztwouN/s6F0/8+bRfDH9cobFw2abaPj8/N
        1hPVbGyOXuzvj8Fz07bEx37zx8vD8A9e5yzHEBpJuM/yhmGzqsw1mo8kiZIk
        3JAkb/ENQ5VnVqP0QZKC1mw0d3mefCfJoiie3Ip68uKoAjPSSONkk+Y3tXT2
        WBo8+bnfLMO8e/8tnfJbf+/lLw/fhofN7SUbktVHeeemqVsN/jb6NvTd3L2P
        jAK9X4q83RfBzxskva4Uf92VC1F2L4O5VzAnmxalURIJLa+ycyI5um3Z/Zw6
        izGRjfLRJWXJAcFaJKJL29yi+j2ixSzjRCLezGu+R4fKbuUfTUbc5VQXZc6r
        pLUO2ptg0Z2lZBy07cg3roS07ktCF0XzWxAsLMZIKrs2z645vnU7Biy3PQfr
        YvFGj3aJi2hJ2bcjR6LSK9udts+9lshlp/FCPKHKTggWnDdFp2M7uoYRXTi7
        xZpITjwR2/7SaSWJeo1QZ8Ayme2ITkZI7lmq7Po7dV9cxo5i37w9cqPIkVvc
        mNCoRci4YRCGki6POytRCKJ4kFBT0WJXlR2yTdk+d90FYpWLzPg+QefSYR2e
        3KTDLFfzgnj15lm80IKZErK9y+VyEyq7mWz3du0LP0jnu/FpuzKz8XZ3iriB
        6joepWuMJZs2Q3DJVKYNiuAoPXyt7Hz+tvEEmioMku/ZN4KhSWWSSmtf6mnh
        25FY7Zcr8/BqtZNpSB1Xna7hZPd6ElEvYBjL989tm8xEdn1UaMMb0YG+TYmw
        w+420YQ/9dNRnC/p5ZJEYlzZvQnhVEWTUiLPz3clkb+kNCQxSkPcAemTSmn/
        sPwdCpF21+SnkIm0AAtNkXbAQjowXB8SLsZCKiTxCdK6EMjDQbyEVHzioJxm
        kJxUSJ1uSBOwEIU0BwtdkYawUAdpNmSC8SpQkJZBSoAPp0AqrsOmRceXYAFI
        vJIKtgT8GKlYFVQQ3tMEAHEB5Ol2kLnbwDwBoO29iWHDAXKaQFaLCukqOkSZ
        oHAG0iiIJ3w4AxLOhIRTIcU0IP1pBiwmPhxoWiYQyAQmju8FGsQTaCubATsd
        HpIgJRAhKjC/bDfXgNMCWgigPg4qJj6nGeTpQHOnQXYpUE5zgJ7KvWUif03r
        Mb6sTibEE6iJ6RBlypDFCVot5aaIL2a53+H/ZAUQSAG2aPxCkP5dmbiDg2ai
        9+uzg8MH9FkONVRDNVRDNVRDNVRDNVRDNVRDNfRfh3BnTPmX3WdnzA/osxz+
        r9BfFf4YlT/e30gPyfv76peHPwEV755kRh8AAA==
    </data>
</plist>";
        let data: plist::Data = plist::from_reader_xml(xml.as_bytes()).unwrap();
        let bytes: Vec<u8> = data.into();

        let _ = Item::try_from_bytes(bytes).expect("should parse");
    }

    fn inventory_round_trip(inventory_data: &[u8]) {
        let inventory_value =
            plist::from_reader_xml(inventory_data).expect("should be able to deserialize");
        let inventory =
            Inventory::try_from_value(inventory_value).expect("should be able to parse");
        let round_trip_inventory_data = to_xml_plist(
            &inventory
                .to_value()
                .expect("inventory should be converted to value"),
        )
        .expect("should serialize");
        let round_trip_inventory_value =
            plist::from_reader_xml(inventory_data).expect("should deserialize");
        let round_trip_inventory =
            Inventory::try_from_value(round_trip_inventory_value).expect("should parse");
        assert_eq!(inventory, round_trip_inventory);

        let inventory_value: plist::Value =
            plist::from_reader_xml(inventory_data).expect("should deserialize");
        let round_trip_inventory_value: plist::Value =
            plist::from_reader_xml(round_trip_inventory_data.as_slice())
                .expect("should deserialize");
        let mut diffs = Vec::new();
        diff_plist_keys(
            "",
            &inventory_value,
            &round_trip_inventory_value,
            &mut diffs,
        );
        assert!(
            diffs.is_empty(),
            "structural fidelity violations:\n{}",
            diffs.join("\n")
        );
    }

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
        inventory_round_trip(inventory_data);
    }

    #[test]
    fn real_inventory_bronze_age_round_trip_test() {
        let inventory_data = b"
        <?xml version=\"1.0\" encoding=\"UTF-8\"?>
        <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
        <plist version=\"1.0\">
        <array>
                <array>
                        <data>
                        AQAAAAAAAJI=
                        </data>
                </array>
                <array>
                        <data>
                        DAAAAAAAAZIfiwgAAAAAAAAH7Zlbc6JIFMefJ5/C9ZVKwBvolMlUcxMQuSio
                        +IagaARBQNF8+gUzk93Z2kqfhzztwouN/s6F0/8+bRfDH9cobFw2abaPj8/N
                        1hPVbGyOXuzvj8Fz07bEx37zx8vD8A9e5yzHEBpJuM/yhmGzqsw1mo8kiZIk
                        3JAkb/ENQ5VnVqP0QZKC1mw0d3mefCfJoiie3Ip68uKoAjPSSONkk+Y3tXT2
                        WBo8+bnfLMO8e/8tnfJbf+/lLw/fhofN7SUbktVHeeemqVsN/jb6NvTd3L2P
                        jAK9X4q83RfBzxskva4Uf92VC1F2L4O5VzAnmxalURIJLa+ycyI5um3Z/Zw6
                        izGRjfLRJWXJAcFaJKJL29yi+j2ixSzjRCLezGu+R4fKbuUfTUbc5VQXZc6r
                        pLUO2ptg0Z2lZBy07cg3roS07ktCF0XzWxAsLMZIKrs2z645vnU7Biy3PQfr
                        YvFGj3aJi2hJ2bcjR6LSK9udts+9lshlp/FCPKHKTggWnDdFp2M7uoYRXTi7
                        xZpITjwR2/7SaSWJeo1QZ8Ayme2ITkZI7lmq7Po7dV9cxo5i37w9cqPIkVvc
                        mNCoRci4YRCGki6POytRCKJ4kFBT0WJXlR2yTdk+d90FYpWLzPg+QefSYR2e
                        3KTDLFfzgnj15lm80IKZErK9y+VyEyq7mWz3du0LP0jnu/FpuzKz8XZ3iriB
                        6joepWuMJZs2Q3DJVKYNiuAoPXyt7Hz+tvEEmioMku/ZN4KhSWWSSmtf6mnh
                        25FY7Zcr8/BqtZNpSB1Xna7hZPd6ElEvYBjL989tm8xEdn1UaMMb0YG+TYmw
                        w+420YQ/9dNRnC/p5ZJEYlzZvQnhVEWTUiLPz3clkb+kNCQxSkPcAemTSmn/
                        sPwdCpF21+SnkIm0AAtNkXbAQjowXB8SLsZCKiTxCdK6EMjDQbyEVHzioJxm
                        kJxUSJ1uSBOwEIU0BwtdkYawUAdpNmSC8SpQkJZBSoAPp0AqrsOmRceXYAFI
                        vJIKtgT8GKlYFVQQ3tMEAHEB5Ol2kLnbwDwBoO29iWHDAXKaQFaLCukqOkSZ
                        oHAG0iiIJ3w4AxLOhIRTIcU0IP1pBiwmPhxoWiYQyAQmju8FGsQTaCubATsd
                        HpIgJRAhKjC/bDfXgNMCWgigPg4qJj6nGeTpQHOnQXYpUE5zgJ7KvWUif03r
                        Mb6sTibEE6iJ6RBlypDFCVot5aaIL2a53+H/ZAUQSAG2aPxCkP5dmbiDg2ai
                        9+uzg8MH9FkONVRDNVRDNVRDNVRDNVRDNVRDNfRfh3BnTPmX3WdnzA/osxz+
                        r9BfFf4YlT/e30gPyfv76peHPwEV755kRh8AAA==
                        </data>
                </array>
                <array>
                        <data>
                        DAAAAAAAAJIfiwgAAAAAAAAH7ZXBT4MwFMbP46+ovcPTmzGFpcLUKlGI7LBj
                        QxskMmhKY91/b2HRuGhc4mmH9tLv9f2+rz31keX7tkNvUo/t0Mf4IjrHSPb1
                        INq+ifG6ugkv8TIJyFn2lFabYoVU144GFevrnKUIhwBUqU4CZFWGipw9V8hl
                        AKweMcIvxqgrAGttxCcqqoftBI5Q6EFJbXa5CwudIRJGYHfNPv3gOe5UtLVJ
                        ggV5lbtkJDBtruJa80l8UwsiuOGzYmVK70pK6T2L5w58tgj87Sxp1tw2/3Hm
                        lu7XD+dvEKUe8pCHPOShE4COfe4PNqXsyED5Uq45zywC80RLgg87lbwFaAcA
                        AA==
                        </data>
                </array>
                <array>
                        <data>
                        DAAAAAAAApIfiwgAAAAAAAAH7dlPa4MwFADwc/spstzr62CHMlLLW/+Ao3GW
                        2UOPotLaWhXN5vrtFx0rGwwjO5bnxaf55eXxyCVEzD7OKXuPyyrJsym/t8ac
                        xVmYR0m2n/KtvxpN+MweirvFy9zfeUtWpEmlmLd9WjtzxkcAWBRpDLDwF8xb
                        O68+0zkAli5n/KBU8QhQ17UVNMoK83MDK/DKvIhLdVnrZCM9wYpUxPUyX9l/
                        laP/Rkmo7OFAnOKLXQloXvorKMugCX5EAxEFKmgjxA22z7MzbUfgOvQHQiRE
                        iBAhQoQIESJEiNANIgHdB4fVZn5AWXUfHDTKUD5016BRinJiRPs+mfZtTWZk
                        Xi5B1zGiEuXWiN5QboyoQLnrg0IjOvUpXLegNqIjyrERHfp0/NQHHdH9z050
                        v7dw1068oq4aCBEiRIgQIUKECBEiRIjQrSM6Y5r6co30YHvPLKC9hbaHnx7/
                        tIkcHwAA
                        </data>
                </array>
                <array>
                        <data>
                        DAAAAAAAA5IfiwgAAAAAAAAH7dhNa4MwGAfwc/spslxLzQZj60ZqeaqWWnSz
                        zB56FJVO+qJomOu3X3TFdTCSMHrYIF6M5pfHv9GASCfv+x16S8sqyw9jfGNc
                        Y5Qe4jzJDpsxXoWz4QhPzD69sp+tcB04qNhlFUPBauq5FsJDQqAodikhdmij
                        wHNfQsRrEOI8YYRfGSseCanr2ogaZcT5voEVCcq8SEt29HixIR9gJCzB/DKf
                        1b/F4WeTLGZmv0e36dGsKGl2/Cgqy6hpnLV6NIlY1LbCpQ3gLwEW7rjtIV3X
                        ObIewBsB30TIdsAHKbLAX0kRz+RK0Uyxkn+RTHwKFNBABd2Dv5Wiu/axyDPF
                        KlOwUQl+oUwj8NfSTHMVNFN5CxyFu2uQI0WLnytRIl5COUwH4LriJZSDZcP8
                        VpyBI4B5/YsMUzhtogwdEmXQSCONNNJII4000kijP4hkH8TO8jRO9EHcIVEG
                        jf4Z+no1uhbvbP+NUNL+OTH7Hy3t4NzQEQAA
                        </data>
                </array>
                <array>
                        <data>
                        DAAAAAAAA5IfiwgAAAAAAAAH7dhBb4IwFADg8/wVjLtWM5eYBTFv1jlGEdww
                        2Y6MMiQqECBj/vsVTNyWGF+zk4dysdqP1ye0zUuNydduq31GRZlk6Vgf9Pq6
                        FqVhxpM0Husr/6E70idmx7im7tR/82Zavk3KSvNW98yaanqXEMjzbUQI9anm
                        MevF10QMQmYLXdPXVZXfEVLXdS9oVC/Mdg0siVdkeVRUeyaCdcUNPV5xXQxz
                        iP4nHfErT8LK7FwZm2hvlgZpPsS3oCiCpvGrdWXwoAra1qKGw/Vkjdsecuw6
                        gQAUUkghhRRSSCGFLgMZBKlyYnoDboZUOTEdglsiOcT0FtxaBmUo6oPbR9Ec
                        GJ6TAyxEkS0TiQPDXkNMQxn0AfZQBo1QtAEbf+I7YI5EJInEU5nh1sBmKHqX
                        GU4qEpfJKZKJJBYCPlUegeEz05ZBA5klFck8pwRsfDguM+lE4jgSK3jzj61m
                        vlwetqhzW80RnctBIYUUUkghhRS6NITWATV9BidE6oCaLsGJkRxq+goOUvoI
                        RMG1UDQF91SV/PNvji3R2Z4nG6Q9bTY73+flnIYEFwAA
                        </data>
                </array>
                <array>
                        <data>
                        DAAAAAAAA5IfiwgAAAAAAAAHvVHRboIwFH2Wr+j6Dte9LUuBdOASFlTM8MHH
                        hjaODKEpzdC/t60x2bKpxIf1pefee87paS6J97sGfQnV110b4sdgipFoq47X
                        7TbE6/LVf8Jx5JGHdJmUm2KGZFP3GhXrlzxLEPYBqJSNAEjLFBV59l4i4wEw
                        W2CEP7SWzwDDMATMsoKq21liD4XqpFD6kBsz3wgCrjk2z5zcf8QxXV5XOvIm
                        5FMcop6AvUzFlGIWfEMTwplmDs1XCXXnLQvdBM4jAteVYkUvKP8iUXoP6WYG
                        OiYDHZPhH0i/fgN2P+em4bn9EXDbjbwjbCEt8HQCAAA=
                        </data>
                </array>
                <array>
                        <data>
                        DAAAAAAAAJIfiwgAAAAAAAAHjU/BDoIwDD3LV8zdoXozZkAmqMGgYsSDx4Ut
                        SERYxiL69w6MRi/GXvra1/faEv92KdFVqKaoKxePnRFGospqXlS5iw/pwp5g
                        37PIMNwG6TGZI1kWjUbJYRZHAcI2AJWyFABhGqIkjvYpMh4A8w1G+KS1nAK0
                        beuwbsrJ6ks32ECiaimUvsfGzDYCh2uOzZqn+9c5psuLTHvWgJzF3WsIdMlU
                        TCnWgQ80IJxp1qN1TumMmlhFbs/AiyLwW7mkO0r/U8I3fPNG0p9MoH/Isx72
                        38H+ZwEAAA==
                        </data>
                </array>
        </array>
        </plist>";
        inventory_round_trip(inventory_data);
    }

    #[test]
    fn test_extra_workbench_isolation() {
        let wb_data = Workbench::new(
            InteractionObject::new(
                DynamicObject {
                    float_pos: [0.0f32.try_into().unwrap(), 0.0f32.try_into().unwrap()],
                    pos_x: 5,
                    pos_y: 5,
                    unique_id: UniqueID::new(456),
                    owner_id: Some("wb_owner".to_string()),
                },
                InteractionObjectType::Workbench,
                false,
                false,
                0,
            ),
            0,
            0.0f32.try_into().unwrap(),
            0.0f32.try_into().unwrap(),
            0.0f32.try_into().unwrap(),
            false,
            0,
            0.0f32.try_into().unwrap(),
            0.0f32.try_into().unwrap(),
            false,
            0.0f32.try_into().unwrap(),
            1,
            100.0f32.try_into().unwrap(),
            0,
            WorkbenchType::Workbench,
            0.0f32.try_into().unwrap(),
            None,
        );

        let item = Item {
            type_id: ItemType::WorkBench as u16,
            data_a: 0,
            data_b: 0,
            selected_sub_item_index: 0,
            padding: 0,
            sub_items: None,
            dynamic_object: Some(AnyDynamicObject::Workbench(Box::new(wb_data))),
        };

        let serialized = item.to_value().unwrap();
        let deserialized = Item::try_from_value(serialized).unwrap();
        assert_eq!(item, deserialized);
    }
}
