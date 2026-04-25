use super::{lib, ItemSnafu};
use lib::game::{
    dynamic_object::{
        chest::{Chest, ChestSlots, ChestType, NUM_SHELF_SLOTS, NUM_STANDARD_SLOTS},
        workbench::{Workbench, WorkbenchType},
        AnyDynamicObject, DynamicObject, InteractionObject, InteractionObjectType, UniqueID,
    },
    item::{Inventory, Item, ItemType, PigmentColor, Slot, Slots},
};
use num_enum::TryFromPrimitive;
use pyo3::exceptions::{PyIndexError, PyValueError};
use pyo3::prelude::*;
use snafu::ResultExt;

#[pyclass(eq, eq_int, name = "ItemType")]
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive)]
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
        Self::try_from(val as u16).expect("Enums are out of sync!")
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
    fn from(value: ChestTypePy) -> Self {
        Self::try_from(value as u8).expect("Enums are out of sync!")
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
    fn from(value: WorkbenchTypePy) -> Self {
        Self::try_from(value as u8).expect("Enums are out of sync!")
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
    fn from(value: PigmentColorPy) -> Self {
        Self::try_from(value as u8).expect("Enums are out of sync!")
    }
}

#[pyclass(name = "Slots")]
pub struct SlotsPy {
    slots: Vec<Py<SlotPy>>,
}

impl std::fmt::Debug for SlotsPy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SlotsPy")
            .field("slots", &"<slots>")
            .finish()
    }
}

#[pymethods]
impl SlotsPy {
    #[new]
    #[pyo3(signature = (slots=None))]
    fn new(slots: Option<Vec<Py<SlotPy>>>) -> PyResult<Self> {
        match slots {
            Some(slots) => {
                if slots.len() != Item::MAX_SUB_ITEMS {
                    return Err(PyValueError::new_err(format!(
                        "BasketSlots must have exactly {} slots",
                        Item::MAX_SUB_ITEMS
                    )));
                }
                Ok(Self { slots })
            }
            None => Python::attach(|py| {
                let mut slots = Vec::with_capacity(Item::MAX_SUB_ITEMS);
                for _ in 0..Item::MAX_SUB_ITEMS {
                    slots.push(Py::new(py, SlotPy::default())?);
                }
                Ok(Self { slots })
            }),
        }
    }

    fn __len__(&self) -> usize {
        self.slots.len()
    }

    fn __getitem__(&self, py: Python<'_>, mut index: isize) -> PyResult<Py<SlotPy>> {
        if index < 0 {
            index += self.slots.len() as isize;
        }
        if index < 0 || index >= self.slots.len() as isize {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "index out of range",
            ));
        }
        Ok(self.slots[index as usize].clone_ref(py))
    }

    fn __setitem__(&mut self, mut index: isize, value: Py<SlotPy>) -> PyResult<()> {
        if index < 0 {
            index += self.slots.len() as isize;
        }
        if index < 0 || index >= self.slots.len() as isize {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "index out of range",
            ));
        }
        self.slots[index as usize] = value;
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
        format!("Slots(slots=[{}])", slots_repr.join(", "))
    }

    fn __str__(&self, py: Python<'_>) -> String {
        let slots_str: Vec<String> = self
            .slots
            .iter()
            .map(|slot| {
                slot.bind(py)
                    .str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| "<str error>".to_string())
            })
            .collect();
        format!("[{}]", slots_str.join(", "))
    }
}

#[pyclass(subclass, name = "DynamicObject")]
pub struct DynamicObjectPy {
    #[pyo3(get, set)]
    pub float_pos: [f32; 2],
    #[pyo3(get, set)]
    pub pos_x: u32,
    #[pyo3(get, set)]
    pub pos_y: u16,
    #[pyo3(get, set)]
    pub unique_id: u64,
    #[pyo3(get, set)]
    pub owner_id: Option<String>,
}

#[pymethods]
impl DynamicObjectPy {
    #[new]
    #[pyo3(signature = (float_pos=[0.0, 0.0], pos_x=0, pos_y=0, unique_id=0, owner_id=None))]
    fn new(
        float_pos: [f32; 2],
        pos_x: u32,
        pos_y: u16,
        unique_id: u64,
        owner_id: Option<String>,
    ) -> Self {
        Self {
            float_pos,
            pos_x,
            pos_y,
            unique_id,
            owner_id,
        }
    }
}

#[pyclass(extends=DynamicObjectPy, subclass, name = "InteractionObject")]
pub struct InteractionObjectPy {
    #[pyo3(get, set)]
    pub interaction_object_type: u8,
    #[pyo3(get, set)]
    pub is_in_use: bool,
    #[pyo3(get, set)]
    pub flipped: bool,
    #[pyo3(get, set)]
    pub paint_color: u16,
}

#[pymethods]
impl InteractionObjectPy {
    #[new]
    #[pyo3(signature = (interaction_object_type=0, is_in_use=false, flipped=false, paint_color=0, float_pos=[0.0, 0.0], pos_x=0, pos_y=0, unique_id=0, owner_id=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        interaction_object_type: u8,
        is_in_use: bool,
        flipped: bool,
        paint_color: u16,
        float_pos: [f32; 2],
        pos_x: u32,
        pos_y: u16,
        unique_id: u64,
        owner_id: Option<String>,
    ) -> pyo3::PyClassInitializer<Self> {
        let base = DynamicObjectPy::new(float_pos, pos_x, pos_y, unique_id, owner_id);
        pyo3::PyClassInitializer::from(base).add_subclass(Self {
            interaction_object_type,
            is_in_use,
            flipped,
            paint_color,
        })
    }
}

#[pyclass(extends=InteractionObjectPy, subclass, name = "Chest")]
pub struct ChestPy {}

#[pymethods]
impl ChestPy {
    fn __repr__(&self) -> String {
        "Chest()".to_string()
    }

    fn __str__(&self) -> String {
        format!("{:?}", self)
    }
}

impl std::fmt::Debug for ChestPy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chest").finish()
    }
}

macro_rules! define_standard_chest {
    ($name:ident, $py_name:expr) => {
        #[pyclass(extends=ChestPy, name = $py_name)]
        pub struct $name {
            pub slots: [Py<SlotPy>; NUM_STANDARD_SLOTS],
        }

        #[pymethods]
        impl $name {
            #[new]
            #[pyo3(signature = (slots=None, owner_id=None))]
            fn new(
                py: Python<'_>,
                slots: Option<Vec<Py<SlotPy>>>,
                owner_id: Option<String>,
            ) -> PyResult<pyo3::PyClassInitializer<Self>> {
                let slots = if let Some(s) = slots {
                    if s.len() != NUM_STANDARD_SLOTS {
                        return Err(pyo3::exceptions::PyValueError::new_err(
                            "slots must have exactly 16 elements",
                        ));
                    }
                    s.try_into().unwrap()
                } else {
                    std::array::from_fn(|_| Py::new(py, SlotPy::default()).unwrap())
                };

                let dyn_obj = DynamicObjectPy::new([0.0, 0.0], 0, 0, 0, owner_id);
                let int_obj = InteractionObjectPy {
                    interaction_object_type: InteractionObjectType::Chest as u8,
                    is_in_use: false,
                    flipped: false,
                    paint_color: 0,
                };
                let base = ChestPy {};
                Ok(pyo3::PyClassInitializer::from(dyn_obj)
                    .add_subclass(int_obj)
                    .add_subclass(base)
                    .add_subclass(Self { slots }))
            }

            fn __len__(&self) -> usize {
                16
            }

            fn __getitem__(&self, index: isize, py: Python<'_>) -> PyResult<Py<SlotPy>> {
                let len = NUM_STANDARD_SLOTS as isize;
                let idx = if index < 0 { index + len } else { index };
                if idx < 0 || idx >= len {
                    return Err(pyo3::exceptions::PyIndexError::new_err(
                        "index out of range",
                    ));
                }
                Ok(self.slots[idx as usize].clone_ref(py))
            }

            fn __setitem__(&mut self, index: isize, item: Py<SlotPy>) -> PyResult<()> {
                let len = NUM_STANDARD_SLOTS as isize;
                let idx = if index < 0 { index + len } else { index };
                if idx < 0 || idx >= len {
                    return Err(pyo3::exceptions::PyIndexError::new_err(
                        "index out of range",
                    ));
                }
                self.slots[idx as usize] = item;
                Ok(())
            }
        }
    };
}

define_standard_chest!(StandardChestPy, "StandardChest");
define_standard_chest!(SafeChestPy, "SafeChest");
define_standard_chest!(GoldChestPy, "GoldChest");
define_standard_chest!(FeederChestPy, "FeederChest");

macro_rules! define_shelf_chest {
    ($name:ident, $py_name:expr) => {
        #[pyclass(extends=ChestPy, name = $py_name)]
        pub struct $name {
            pub slots: [Py<SlotPy>; NUM_SHELF_SLOTS],
            #[pyo3(get, set)]
            pub render_items: Option<[ItemTypePy; NUM_SHELF_SLOTS]>,
            #[pyo3(get, set)]
            pub item_data_bs: Option<[u16; NUM_SHELF_SLOTS]>,
        }

        #[pymethods]
        impl $name {
            #[new]
            #[pyo3(signature = (slots=None, render_items=None, item_data_bs=None, owner_id=None))]
            fn new(
                py: Python<'_>,
                slots: Option<Vec<Py<SlotPy>>>,
                render_items: Option<Vec<ItemTypePy>>,
                item_data_bs: Option<Vec<u16>>,
                owner_id: Option<String>,
            ) -> PyResult<pyo3::PyClassInitializer<Self>> {
                let slots = if let Some(s) = slots {
                    if s.len() != NUM_SHELF_SLOTS {
                        return Err(pyo3::exceptions::PyValueError::new_err(
                            "slots must have exactly 4 elements",
                        ));
                    }
                    s.try_into().unwrap()
                } else {
                    std::array::from_fn(|_| Py::new(py, SlotPy::default()).unwrap())
                };

                let render_items = if let Some(r) = render_items {
                    if r.len() != 4 {
                        return Err(pyo3::exceptions::PyValueError::new_err(
                            "render_items must have exactly 4 elements",
                        ));
                    }
                    Some(r.try_into().unwrap())
                } else {
                    None
                };

                let item_data_bs = if let Some(i) = item_data_bs {
                    if i.len() != NUM_SHELF_SLOTS {
                        return Err(pyo3::exceptions::PyValueError::new_err(
                            "item_data_bs must have exactly 4 elements",
                        ));
                    }
                    Some(i.try_into().unwrap())
                } else {
                    None
                };

                let dyn_obj = DynamicObjectPy::new([0.0, 0.0], 0, 0, 0, owner_id);
                let int_obj = InteractionObjectPy {
                    interaction_object_type: InteractionObjectType::Chest as u8,
                    is_in_use: false,
                    flipped: false,
                    paint_color: 0,
                };
                let base = ChestPy {};

                Ok(pyo3::PyClassInitializer::from(dyn_obj)
                    .add_subclass(int_obj)
                    .add_subclass(base)
                    .add_subclass(Self {
                        slots,
                        render_items,
                        item_data_bs,
                    }))
            }

            fn __len__(&self) -> usize {
                NUM_SHELF_SLOTS
            }

            fn __getitem__(&self, index: isize, py: Python<'_>) -> PyResult<Py<SlotPy>> {
                let len = NUM_SHELF_SLOTS as isize;
                let idx = if index < 0 { index + len } else { index };
                if idx < 0 || idx >= len {
                    return Err(pyo3::exceptions::PyIndexError::new_err(
                        "index out of range",
                    ));
                }
                Ok(self.slots[idx as usize].clone_ref(py))
            }

            fn __setitem__(&mut self, index: isize, item: Py<SlotPy>) -> PyResult<()> {
                let len = NUM_SHELF_SLOTS as isize;
                let idx = if index < 0 { index + len } else { index };
                if idx < 0 || idx >= len {
                    return Err(pyo3::exceptions::PyIndexError::new_err(
                        "index out of range",
                    ));
                }
                self.slots[idx as usize] = item;
                Ok(())
            }
        }
    };
}

define_shelf_chest!(ShelfChestPy, "ShelfChest");
define_shelf_chest!(CabinetPy, "Cabinet");

#[pyclass(extends=ChestPy, name = "PortalChest")]
pub struct PortalChestPy {}

#[pymethods]
impl PortalChestPy {
    #[new]
    #[pyo3(signature = (owner_id=None))]
    fn new(owner_id: Option<String>) -> PyResult<pyo3::PyClassInitializer<Self>> {
        let dyn_obj = DynamicObjectPy::new([0.0, 0.0], 0, 0, 0, owner_id);
        let int_obj = InteractionObjectPy {
            interaction_object_type: InteractionObjectType::Chest as u8,
            is_in_use: false,
            flipped: false,
            paint_color: 0,
        };
        let base = ChestPy {};
        Ok(pyo3::PyClassInitializer::from(dyn_obj)
            .add_subclass(int_obj)
            .add_subclass(base)
            .add_subclass(Self {}))
    }
}

impl ChestPy {
    pub fn inflate(py: Python<'_>, chest: Chest) -> PyResult<Py<PyAny>> {
        let dyn_obj = DynamicObjectPy::new(
            [chest.float_pos[0], chest.float_pos[1]],
            chest.pos_x,
            chest.pos_y,
            *chest.unique_id.inner(),
            chest.owner_id.clone(),
        );
        let int_obj = InteractionObjectPy {
            interaction_object_type: chest.interaction_object_type as u8,
            is_in_use: chest.is_in_use,
            flipped: chest.flipped,
            paint_color: chest.paint_color,
        };
        let base = ChestPy {};

        let init = pyo3::PyClassInitializer::from(dyn_obj)
            .add_subclass(int_obj)
            .add_subclass(base);

        match chest.slots {
            ChestSlots::Standard(slots) => {
                let slots = Self::inflate_standard(py, slots.into_inner())?;
                Ok(Py::new(py, init.add_subclass(StandardChestPy { slots }))?.into_any())
            }
            ChestSlots::Safe(slots) => {
                let slots = Self::inflate_standard(py, slots.into_inner())?;
                Ok(Py::new(py, init.add_subclass(SafeChestPy { slots }))?.into_any())
            }
            ChestSlots::Gold(slots) => {
                let slots = Self::inflate_standard(py, slots.into_inner())?;
                Ok(Py::new(py, init.add_subclass(GoldChestPy { slots }))?.into_any())
            }
            ChestSlots::Feeder(slots) => {
                let slots = Self::inflate_standard(py, slots.into_inner())?;
                Ok(Py::new(py, init.add_subclass(FeederChestPy { slots }))?.into_any())
            }
            ChestSlots::Shelf {
                slots,
                render_items,
                item_data_bs,
            } => {
                let slots = Self::inflate_shelf(py, slots.into_inner())?;
                Ok(Py::new(
                    py,
                    init.add_subclass(ShelfChestPy {
                        slots,
                        render_items: render_items.map(|ri| ri.map(|i| i.into())),
                        item_data_bs,
                    }),
                )?
                .into_any())
            }
            ChestSlots::Cabinet {
                slots,
                render_items,
                item_data_bs,
            } => {
                let slots = Self::inflate_shelf(py, slots.into_inner())?;
                Ok(Py::new(
                    py,
                    init.add_subclass(CabinetPy {
                        slots,
                        render_items: render_items.map(|ri| ri.map(|i| i.into())),
                        item_data_bs,
                    }),
                )?
                .into_any())
            }
            ChestSlots::Portal => Ok(Py::new(py, init.add_subclass(PortalChestPy {}))?.into_any()),
        }
    }

    fn inflate_standard(py: Python<'_>, slots: [Slot; 16]) -> PyResult<[Py<SlotPy>; 16]> {
        let mut py_slots: [Py<SlotPy>; 16] =
            std::array::from_fn(|_| Py::new(py, SlotPy::default()).unwrap());
        for (i, slot) in slots.into_iter().enumerate() {
            py_slots[i] = SlotPy::inflate(py, slot)?;
        }
        Ok(py_slots)
    }

    fn inflate_shelf(py: Python<'_>, slots: [Slot; 4]) -> PyResult<[Py<SlotPy>; 4]> {
        let mut py_slots: [Py<SlotPy>; 4] =
            std::array::from_fn(|_| Py::new(py, SlotPy::default()).unwrap());
        for (i, slot) in slots.into_iter().enumerate() {
            py_slots[i] = SlotPy::inflate(py, slot)?;
        }
        Ok(py_slots)
    }

    pub fn deflate(py: Python<'_>, py_obj: Py<PyAny>) -> PyResult<Chest> {
        let any = py_obj.bind(py);
        let _base_ref = any.extract::<pyo3::PyRef<ChestPy>>()?;
        let int_ref = any.extract::<pyo3::PyRef<InteractionObjectPy>>()?;
        let dyn_ref = any.extract::<pyo3::PyRef<DynamicObjectPy>>()?;

        let slots = if let Ok(c) = any.extract::<pyo3::PyRef<StandardChestPy>>() {
            ChestSlots::Standard(Slots::new(Self::deflate_standard(py, &c.slots)))
        } else if let Ok(c) = any.extract::<pyo3::PyRef<SafeChestPy>>() {
            ChestSlots::Safe(Slots::new(Self::deflate_standard(py, &c.slots)))
        } else if let Ok(c) = any.extract::<pyo3::PyRef<GoldChestPy>>() {
            ChestSlots::Gold(Slots::new(Self::deflate_standard(py, &c.slots)))
        } else if let Ok(c) = any.extract::<pyo3::PyRef<FeederChestPy>>() {
            ChestSlots::Feeder(Slots::new(Self::deflate_standard(py, &c.slots)))
        } else if let Ok(c) = any.extract::<pyo3::PyRef<ShelfChestPy>>() {
            ChestSlots::Shelf {
                slots: Slots::new(Self::deflate_shelf(py, &c.slots)),
                render_items: c.render_items.map(|ri| ri.map(|i| i.into())),
                item_data_bs: c.item_data_bs,
            }
        } else if let Ok(c) = any.extract::<pyo3::PyRef<CabinetPy>>() {
            ChestSlots::Cabinet {
                slots: Slots::new(Self::deflate_shelf(py, &c.slots)),
                render_items: c.render_items.map(|ri| ri.map(|i| i.into())),
                item_data_bs: c.item_data_bs,
            }
        } else if any.extract::<pyo3::PyRef<PortalChestPy>>().is_ok() {
            ChestSlots::Portal
        } else {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Invalid chest subclass",
            ));
        };

        Ok(Chest::new(
            InteractionObject::new(
                DynamicObject {
                    float_pos: [dyn_ref.float_pos[0], dyn_ref.float_pos[1]],
                    pos_x: dyn_ref.pos_x,
                    pos_y: dyn_ref.pos_y,
                    unique_id: UniqueID::new(dyn_ref.unique_id),
                    owner_id: dyn_ref.owner_id.clone(),
                },
                InteractionObjectType::Chest,
                int_ref.is_in_use,
                int_ref.flipped,
                int_ref.paint_color,
            ),
            0.0, // dummy save_time
            slots,
        ))
    }

    fn deflate_standard(py: Python<'_>, slots: &[Py<SlotPy>; 16]) -> [Slot; 16] {
        std::array::from_fn(|i| slots[i].bind(py).borrow().deflate(py))
    }

    fn deflate_shelf(py: Python<'_>, slots: &[Py<SlotPy>; 4]) -> [Slot; 4] {
        std::array::from_fn(|i| slots[i].bind(py).borrow().deflate(py))
    }
}

#[pyclass(extends=InteractionObjectPy, subclass, name = "Workbench")]
#[derive(Debug)]
pub struct WorkbenchPy {
    #[pyo3(get, set)]
    pub workbench_type: WorkbenchTypePy,
    #[pyo3(get, set)]
    pub level: u8,
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

impl WorkbenchPy {
    pub fn inflate(py: Python<'_>, workbench: Workbench) -> PyResult<Py<Self>> {
        let dyn_obj = DynamicObjectPy::new(
            [workbench.float_pos[0], workbench.float_pos[1]],
            workbench.pos_x,
            workbench.pos_y,
            *workbench.unique_id.inner(),
            workbench.owner_id.clone(),
        );
        let int_obj = InteractionObjectPy {
            interaction_object_type: workbench.interaction_object_type as u8,
            is_in_use: workbench.is_in_use,
            flipped: workbench.flipped,
            paint_color: workbench.paint_color,
        };
        let bench = Self {
            workbench_type: workbench.workbench_type.into(),
            level: workbench.level,

            available_electricity: workbench.available_electricity,
            craft_progress_count: workbench.craft_progress_count,
            fire_spread_timer: workbench.fire_spread_timer,
            fuel_fraction: workbench.fuel_fraction,
            has_fuel: workbench.has_fuel,
            hurry_cost: workbench.hurry_cost,
            hurry_seconds: workbench.hurry_seconds,
            hurry_timer: workbench.hurry_timer,
            hurrying: workbench.hurrying,
            last_world_time: workbench.last_world_time,
            save_time: workbench.save_time,
            selected_index: workbench.selected_index,
            x_scroll: workbench.x_scroll,
        };

        let init = pyo3::PyClassInitializer::from(dyn_obj)
            .add_subclass(int_obj)
            .add_subclass(bench);

        Py::new(py, init)
    }

    pub fn deflate(py: Python<'_>, py_obj: Py<PyAny>) -> PyResult<Workbench> {
        let any = py_obj.bind(py);
        let slf = any.extract::<pyo3::PyRef<WorkbenchPy>>()?;
        let int_ref = any.extract::<pyo3::PyRef<InteractionObjectPy>>()?;
        let dyn_ref = any.extract::<pyo3::PyRef<DynamicObjectPy>>()?;

        Ok(Workbench::new(
            InteractionObject::new(
                DynamicObject {
                    float_pos: [dyn_ref.float_pos[0], dyn_ref.float_pos[1]],
                    pos_x: dyn_ref.pos_x,
                    pos_y: dyn_ref.pos_y,
                    unique_id: UniqueID::new(dyn_ref.unique_id),
                    owner_id: dyn_ref.owner_id.clone(),
                },
                InteractionObjectType::Workbench,
                int_ref.is_in_use,
                int_ref.flipped,
                int_ref.paint_color,
            ),
            slf.available_electricity,
            slf.craft_progress_count,
            slf.fire_spread_timer,
            slf.fuel_fraction,
            slf.has_fuel,
            slf.hurry_cost,
            slf.hurry_seconds,
            slf.hurry_timer,
            slf.hurrying,
            slf.last_world_time,
            slf.level,
            slf.save_time,
            slf.selected_index,
            slf.workbench_type.into(),
            slf.x_scroll,
            None, // TODO add artificial dict
        ))
    }

    pub fn clone_ref(py: Python<'_>, py_obj: Py<PyAny>) -> PyResult<Py<Self>> {
        let any = py_obj.bind(py);
        let slf = any.extract::<pyo3::PyRef<WorkbenchPy>>()?;
        let int_ref = any.extract::<pyo3::PyRef<InteractionObjectPy>>()?;
        let dyn_ref = any.extract::<pyo3::PyRef<DynamicObjectPy>>()?;
        let dyn_obj = DynamicObjectPy::new(
            [dyn_ref.float_pos[0], dyn_ref.float_pos[1]],
            dyn_ref.pos_x,
            dyn_ref.pos_y,
            dyn_ref.unique_id,
            dyn_ref.owner_id.clone(),
        );
        let int_obj = InteractionObjectPy {
            interaction_object_type: int_ref.interaction_object_type,
            is_in_use: int_ref.is_in_use,
            flipped: int_ref.flipped,
            paint_color: int_ref.paint_color,
        };
        let bench = Self {
            workbench_type: slf.workbench_type,
            level: slf.level,

            available_electricity: slf.available_electricity,
            craft_progress_count: slf.craft_progress_count,
            fire_spread_timer: slf.fire_spread_timer,
            fuel_fraction: slf.fuel_fraction,
            has_fuel: slf.has_fuel,
            hurry_cost: slf.hurry_cost,
            hurry_seconds: slf.hurry_seconds,
            hurry_timer: slf.hurry_timer,
            hurrying: slf.hurrying,
            last_world_time: slf.last_world_time,
            save_time: slf.save_time,
            selected_index: slf.selected_index,
            x_scroll: slf.x_scroll,
        };
        let init = pyo3::PyClassInitializer::from(dyn_obj)
            .add_subclass(int_obj)
            .add_subclass(bench);
        Ok(Py::new(py, init).unwrap())
    }
}

#[pymethods]
impl WorkbenchPy {
    #[new]
    #[pyo3(signature = (workbench_type=WorkbenchTypePy::Workbench, level=1, owner_id=None))]
    fn new(
        workbench_type: WorkbenchTypePy,
        level: u8,
        owner_id: Option<String>,
    ) -> PyResult<pyo3::PyClassInitializer<Self>> {
        let dyn_obj = DynamicObjectPy::new([0.0, 0.0], 0, 0, 0, owner_id);
        let int_obj = InteractionObjectPy {
            interaction_object_type: InteractionObjectType::Workbench as u8,
            is_in_use: false,
            flipped: false,
            paint_color: 0,
        };
        let bench = Self {
            workbench_type,
            level,

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
        };
        Ok(pyo3::PyClassInitializer::from(dyn_obj)
            .add_subclass(int_obj)
            .add_subclass(bench))
    }

    fn __repr__(&self) -> String {
        format!(
            "Workbench(type={:?}, level={})",
            self.workbench_type, self.level
        )
    }

    fn __str__(&self) -> String {
        format!("{:?}", self)
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
    pub sub_items: Option<Py<SlotsPy>>,
    pub dynamic_object: Option<Py<PyAny>>,
}

impl std::fmt::Debug for ItemPy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ItemPy")
            .field("type_id", &self.type_id)
            .field("data_a", &self.data_a)
            .field("data_b", &self.data_b)
            .field("sub_index", &self.selected_sub_item_index)
            .field("padding", &self.padding)
            .field("sub_items", &"<sub_items>")
            .field("dynamic_object", &"<dynamic_object>")
            .finish()
    }
}

impl ItemPy {
    pub fn inflate(py: Python<'_>, item: Item) -> PyResult<Py<Self>> {
        let sub_items = match item.sub_items {
            Some(si) => {
                let py_items = si
                    .into_inner()
                    .into_iter()
                    .map(|s| SlotPy::inflate(py, s))
                    .collect::<PyResult<Vec<_>>>()?;
                Some(Py::new(py, SlotsPy::new(Some(py_items))?)?)
            }
            None => None,
        };

        let dynamic_object = match item.dynamic_object {
            Some(AnyDynamicObject::Chest(chest)) => Some(ChestPy::inflate(py, *chest)?),
            Some(AnyDynamicObject::Workbench(bench)) => {
                Some(WorkbenchPy::inflate(py, *bench)?.into_any())
            }
            _ => None,
        };

        Py::new(
            py,
            Self {
                type_id: item.type_id,
                data_a: item.data_a,
                data_b: item.data_b,
                selected_sub_item_index: item.selected_sub_item_index,
                padding: item.padding,
                sub_items,
                dynamic_object,
            },
        )
    }

    pub fn deflate(&self, py: Python<'_>) -> Item {
        let sub_items = self.sub_items.as_ref().map(|si| {
            let slots_py = si.bind(py).borrow();
            let mut slots = [const { Slot(vec![]) }; Item::MAX_SUB_ITEMS];
            for (i, slot) in slots_py.slots.iter().enumerate() {
                if i < Item::MAX_SUB_ITEMS {
                    slots[i] = slot.bind(py).borrow().deflate(py);
                }
            }
            Slots::new(slots)
        });

        let dynamic_object = self.dynamic_object.as_ref().and_then(|obj| {
            if obj.bind(py).is_instance_of::<ChestPy>() {
                Some(AnyDynamicObject::Chest(Box::new(
                    ChestPy::deflate(py, obj.clone_ref(py)).ok()?,
                )))
            } else if obj.bind(py).is_instance_of::<WorkbenchPy>() {
                Some(AnyDynamicObject::Workbench(Box::new(
                    WorkbenchPy::deflate(py, obj.clone_ref(py)).ok()?,
                )))
            } else {
                None
            }
        });

        Item {
            type_id: self.type_id,
            data_a: self.data_a,
            data_b: self.data_b,
            selected_sub_item_index: self.selected_sub_item_index,
            padding: self.padding,
            sub_items,
            dynamic_object,
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
            sub_items: None,
            dynamic_object: None,
        }
    }
}

#[pymethods]
impl ItemPy {
    #[new]
    #[pyo3(signature = (item_type, sub_items=None, dynamic_object=None))]
    fn new(
        item_type: ItemTypePy,
        sub_items: Option<Py<SlotsPy>>,
        dynamic_object: Option<Py<PyAny>>,
    ) -> Self {
        let mut s = Self::from(Item::new(item_type.into()));
        s.sub_items = sub_items;
        s.dynamic_object = dynamic_object;
        s
    }

    #[getter]
    fn get_item_type(&self) -> PyResult<ItemTypePy> {
        ItemTypePy::try_from(self.type_id)
            .map_err(|_| PyValueError::new_err(format!("Invalid item type id: {}", self.type_id)))
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
        let colors = Item::decode_colors(self.data_b).context(ItemSnafu)?;
        Ok(colors.into_iter().map(Into::into).collect())
    }

    #[setter]
    fn set_colors(&mut self, colors: Vec<PigmentColorPy>) -> PyResult<()> {
        if colors.len() > 3 {
            return Err(PyValueError::new_err(
                "colors must not have more than 3 elements",
            ));
        }
        let mut array = [PigmentColor::Transparent; 3];
        for (i, color) in colors.into_iter().enumerate() {
            array[i] = color.into();
        }
        self.data_b = Item::encode_colors(array);
        Ok(())
    }

    #[getter]
    fn get_sub_items(&self, py: Python<'_>) -> Option<Py<SlotsPy>> {
        self.sub_items.as_ref().map(|e| e.clone_ref(py))
    }

    #[setter]
    fn set_sub_items(&mut self, sub_items: Option<Py<SlotsPy>>) {
        self.sub_items = sub_items;
    }

    #[getter]
    fn get_dynamic_object(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.dynamic_object.as_ref().map(|e| e.clone_ref(py))
    }

    #[setter]
    fn set_dynamic_object(&mut self, dynamic_object: Option<Py<PyAny>>) {
        self.dynamic_object = dynamic_object;
    }

    fn __repr__(&self, py: Python<'_>) -> String {
        let sub_items_repr = match &self.sub_items {
            Some(b) => b
                .bind(py)
                .repr()
                .map(|s| s.to_string())
                .unwrap_or_else(|_| "<repr error>".to_string()),
            None => "None".to_string(),
        };
        let dyn_repr = match &self.dynamic_object {
            Some(c) => c
                .bind(py)
                .repr()
                .map(|s| s.to_string())
                .unwrap_or_else(|_| "<repr error>".to_string()),
            None => "None".to_string(),
        };
        format!(
            "Item(type_id={}, data_a={}, data_b={}, sub_index={}, padding={}, sub_items={}, dynamic_object={})",
            self.type_id,
            self.data_a,
            self.data_b,
            self.selected_sub_item_index,
            self.padding,
            sub_items_repr,
            dyn_repr
        )
    }
}

#[pyclass(name = "Slot")]
#[derive(Default)]
pub struct SlotPy {
    #[pyo3(get, set)]
    pub items: Vec<Py<ItemPy>>,
}

impl std::fmt::Debug for SlotPy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SlotPy").field("items", &"<items>").finish()
    }
}

impl SlotPy {
    pub fn inflate(py: Python<'_>, slot: Slot) -> PyResult<Py<Self>> {
        let mut items = Vec::with_capacity(slot.0.len());
        for item in slot.0 {
            items.push(ItemPy::inflate(py, item)?);
        }
        Py::new(py, Self { items })
    }

    pub fn deflate(&self, py: Python<'_>) -> Slot {
        Slot(
            self.items
                .iter()
                .map(|item_py| item_py.bind(py).borrow().deflate(py))
                .collect(),
        )
    }
}

#[pyclass]
pub struct SlotIterator {
    slot: Py<SlotPy>,
    index: usize,
}

#[pymethods]
impl SlotIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>, py: Python<'_>) -> Option<Py<ItemPy>> {
        let slot_obj = slf.slot.clone_ref(py);
        let slot = slot_obj.borrow(py);
        if slf.index < slot.items.len() {
            let item = slot.items[slf.index].clone_ref(py);
            slf.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

#[pymethods]
impl SlotPy {
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

    fn __delitem__(&mut self, mut index: isize) -> PyResult<()> {
        if index < 0 {
            index += self.items.len() as isize;
        }
        if index < 0 || index >= self.items.len() as isize {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "index out of range",
            ));
        }
        self.items.remove(index as usize);
        Ok(())
    }

    fn append(&mut self, item: Py<ItemPy>) {
        self.items.push(item);
    }

    fn extend(&mut self, items: Vec<Py<ItemPy>>) {
        self.items.extend(items);
    }

    fn insert(&mut self, index: isize, item: Py<ItemPy>) {
        let len = self.items.len() as isize;
        let idx = if index < 0 {
            let i = index + len;
            if i < 0 {
                0
            } else {
                i
            }
        } else if index > len {
            len
        } else {
            index
        };
        self.items.insert(idx as usize, item);
    }

    #[pyo3(signature = (index=None))]
    fn pop(&mut self, index: Option<isize>) -> PyResult<Py<ItemPy>> {
        let idx = index.unwrap_or(-1);
        let len = self.items.len() as isize;
        let idx = if idx < 0 { idx + len } else { idx };
        if idx < 0 || idx >= len {
            return Err(PyIndexError::new_err("pop index out of range"));
        }
        Ok(self.items.remove(idx as usize))
    }

    fn clear(&mut self) {
        self.items.clear();
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<SlotIterator> {
        Ok(SlotIterator {
            slot: slf.into(),
            index: 0,
        })
    }

    fn __repr__(&self, py: Python<'_>) -> String {
        let items_repr: Vec<String> = self
            .items
            .iter()
            .map(|item| format!("{:?}", item.bind(py).borrow()))
            .collect();
        format!("Slot(items=[{}])", items_repr.join(", "))
    }
}

#[pyclass(name = "Inventory")]
#[derive(Debug)]
pub struct InventoryPy {
    slots: Vec<Py<SlotPy>>,
}

impl InventoryPy {
    pub fn inflate(py: Python<'_>, inventory: Inventory) -> PyResult<Py<Self>> {
        let mut slots = Vec::with_capacity(Inventory::NUM_SLOTS);
        for slot in inventory {
            slots.push(SlotPy::inflate(py, slot)?);
        }
        Py::new(py, Self { slots })
    }

    pub fn deflate(&self, py: Python<'_>) -> Inventory {
        let mut slots = [const { Slot(vec![]) }; Inventory::NUM_SLOTS];
        for (i, slot_py) in self.slots.iter().enumerate() {
            if i < Inventory::NUM_SLOTS {
                slots[i] = slot_py.bind(py).borrow().deflate(py);
            }
        }
        Inventory::new(Slots::new(slots))
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
    fn new(py: Python<'_>, slots: Option<Vec<Py<SlotPy>>>) -> PyResult<Py<Self>> {
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
                    s.push(Py::new(py, SlotPy::default())?);
                }
                s
            }
        };
        Py::new(py, Self { slots })
    }

    fn __len__(&self) -> usize {
        self.slots.len()
    }

    fn __getitem__(&self, index: isize, py: Python<'_>) -> PyResult<Py<SlotPy>> {
        let len = self.slots.len() as isize;
        let idx = if index < 0 { index + len } else { index };
        if idx < 0 || idx >= len {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "index out of range",
            ));
        }
        Ok(self.slots[idx as usize].clone_ref(py))
    }

    fn __setitem__(&mut self, index: isize, item: Py<SlotPy>) -> PyResult<()> {
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

    fn __str__(&self, py: Python<'_>) -> String {
        let slots_str: Vec<String> = self
            .slots
            .iter()
            .map(|slot| {
                slot.bind(py)
                    .str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| "<str error>".to_string())
            })
            .collect();
        format!("[{}]", slots_str.join(", "))
    }
}
