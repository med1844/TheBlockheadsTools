use super::{
    block::BlockType,
    dynamic_object::train::{FreightCar, FreightCarSaveDictXml},
    item::{ItemType, PigmentColors},
};
use crate::util::serde::{deserialize_some, serialize_some};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use snafu::prelude::*;
use std::ops::{Deref, DerefMut};
use strum::IntoStaticStr;

#[derive(Debug, Snafu)]
pub enum DynamicObjectError {
    #[snafu(display("Failed to parse {type_str} as dynamic object type"))]
    ParseObjTypeAsInt {
        type_str: String,
        source: std::num::ParseIntError,
    },
    #[snafu(display("Failed to parse {unique_id_str} as u64"))]
    ParseUniqueIdAsInt {
        unique_id_str: String,
        source: std::num::ParseIntError,
    },
    #[snafu(display("Invalid dynamic object type ID {id}"))]
    InvalidDynamicObjectTypeId {
        id: u8,
        source: num_enum::TryFromPrimitiveError<DynamicObjectType>,
    },
    #[snafu(display("Failed to deserialize plist dictionary to {target_type}, dict: {dict:?}"))]
    DeserializeDictionary {
        source: plist::Error,
        target_type: &'static str,
        dict: Box<plist::Value>,
    },
    #[snafu(display("Failed to serialize {source_type} to plist dictionary"))]
    SerializeDictionary {
        source: plist::Error,
        source_type: &'static str,
    },
    #[snafu(display("Failed to load chest"))]
    LoadChest { source: chest::ChestError },
    #[snafu(display("Failed to save chest"))]
    SaveChest { source: chest::ChestError },
    #[snafu(display("Failed to load train chest"))]
    LoadTrainChest { source: chest::ChestError },
    #[snafu(display("Failed to save train chest"))]
    SaveTrainChest { source: chest::ChestError },
    #[snafu(display(
        "Can't understand dynamicObjectSaveDict with {item_type:?} yet, value: {value:?}"
    ))]
    UnsupportedItemTypeForDynObj {
        item_type: ItemType,
        value: Box<plist::Value>,
    },
}

type Result<T> = std::result::Result<T, DynamicObjectError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoStaticStr)]
#[repr(u8)]
pub enum DynamicObjectType {
    AppleTree = 1,
    MapleTree = 2,
    MangoTree = 3,
    PineTree = 4,
    CactusTree = 5,
    CoconutTree = 6,
    OrangeTree = 7,
    CherryTree = 8,
    CoffeeTree = 9,
    FlaxPlant = 10,
    SunflowerPlant = 11,
    CornPlant = 12,
    Dodo = 13,
    DroppedItem = 14,
    Fire = 16,
    Torch = 17,
    GlowBlock = 18,
    Ladder = 19,
    Door = 20,
    ArtificialLight = 21,
    Bed = 23,
    DropBear = 25,
    GatherBlock = 26,
    CarrotPlant = 27,
    Donkey = 28,
    Egg = 30,
    Window = 31,
    Boat = 32,
    ChilliPlant = 33,
    KelpPlant = 34,
    ClownFish = 35,
    Shark = 36,
    LimeTree = 37,
    Wire = 38,
    CaveTroll = 39,
    Rail = 40,
    HandCar = 41,
    SteamLocomotive = 42,
    FreightCar = 43,
    PassengerCar = 44,
    Workbench = 45,
    Chest = 46,
    Sign = 47,
    TradingPost = 48,
    TrainStation = 49,
    TradePortal = 50,
    Scorpion = 51,
    Painting = 52,
    Column = 53,
    Stairs = 54,
    ElevatorMotor = 55,
    ElevatorShaft = 56,
    GemTree = 57,
    VinePlant = 58,
    TulipPlant = 59,
    OwnershipSign = 60,
    WheatPlant = 61,
    TomatoPlant = 62,
    Yak = 63,
    Mirror = 64,
}

impl DynamicObjectType {
    pub fn try_from_str(s: &str) -> Result<Self> {
        let value: u8 = s.parse().with_context(|_| ParseObjTypeAsIntSnafu {
            type_str: s.to_owned(),
        })?;
        Self::try_from(value).context(InvalidDynamicObjectTypeIdSnafu { id: value })
    }
}

// Rust doesn't have inheritance, yet the game was build on that.
// Thus we have to emulate that, and thankfully it's not too hard.
macro_rules! inherit {
    ($child:ident -> $parent:ty, $field:ident) => {
        impl Deref for $child {
            type Target = $parent;
            fn deref(&self) -> &Self::Target {
                &self.$field
            }
        }

        impl DerefMut for $child {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$field
            }
        }
    };

    ($child:ident -> $parent:ty) => {
        impl Deref for $child {
            type Target = $parent;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $child {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

// We need a root struct to match the plist's top-level dictionary
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DynamicObjectList<T> {
    #[serde(rename = "dynamicObjects")]
    dynamic_objects: Vec<T>,
}

impl<T> DynamicObjectList<T> {
    pub(crate) fn num_obj(&self) -> usize {
        self.len()
    }

    pub fn into_inner(self) -> Vec<T> {
        self.dynamic_objects
    }
}

impl<T> Deref for DynamicObjectList<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.dynamic_objects
    }
}

impl<T> DerefMut for DynamicObjectList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.dynamic_objects
    }
}

impl<T> Default for DynamicObjectList<T> {
    fn default() -> Self {
        Self {
            dynamic_objects: vec![],
        }
    }
}

impl<T> IntoIterator for DynamicObjectList<T> {
    type IntoIter = std::vec::IntoIter<T>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        self.dynamic_objects.into_iter()
    }
}

impl<T> FromIterator<T> for DynamicObjectList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            dynamic_objects: Vec::from_iter(iter),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(transparent)]
pub struct UniqueID(u64);
inherit!(UniqueID -> u64);

impl UniqueID {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn try_from_str(s: &str) -> Result<Self> {
        let value: u64 = s.parse().with_context(|_| ParseUniqueIdAsIntSnafu {
            unique_id_str: s.to_owned(),
        })?;
        Ok(Self(value))
    }

    pub fn inner(&self) -> &u64 {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut u64 {
        &mut self.0
    }

    pub fn into_inner(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DynamicObject {
    #[serde(rename = "floatPos")]
    pub float_pos: [f32; 2],
    pub pos_x: u32,
    pub pos_y: u16,
    #[serde(rename = "uniqueID")]
    pub unique_id: UniqueID,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none",
        rename = "ownerID"
    )]
    pub owner_id: Option<String>,
}

impl DynamicObject {
    pub fn set_float_pos(&mut self, pos: [f32; 2]) {
        self.float_pos = pos;
        let [x, y] = pos;
        self.pos_x = x.floor() as u32;
        self.pos_y = y.floor() as u16;
    }

    pub fn set_pos(&mut self, pos: (u32, u16)) {
        let (x, y) = pos;
        self.pos_x = x;
        self.pos_y = y;
        self.float_pos = [x as f32 + 0.5, y as f32];
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum InteractionObjectType {
    #[default]
    InteractionObject = 0,
    Workbench = 1,
    Chest = 2,
    Bed = 3,
    Sign = 4,
    TradingPost = 5,
    TrainStation = 6,
    TradePortal = 7,
    OwnershipSign = 8,
    Mirror = 9,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionObject {
    #[serde(flatten)]
    obj: DynamicObject,
    pub interaction_object_type: InteractionObjectType,
    pub is_in_use: bool,
    pub flipped: bool,
    pub paint_color: PigmentColors,

    // see -[InteractionObject_getSaveDict]
    // .objc_str.256 (0x944600) in decompiled server code
    pub save_time: f32,

    // see 0x914906 in decompiled server code
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub owner_name: Option<String>,
}
inherit!(InteractionObject -> DynamicObject, obj);

impl InteractionObject {
    pub fn new(
        obj: DynamicObject,
        interaction_object_type: InteractionObjectType,
        is_in_use: bool,
        flipped: bool,
        paint_color: PigmentColors,
        save_time: f32,
    ) -> Self {
        Self {
            obj,
            interaction_object_type,
            is_in_use,
            flipped,
            paint_color,
            save_time,
            owner_name: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum LightDirection {
    #[default]
    All = 0,
    Down = 1,
    Up = 2,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtificialLight {
    #[serde(flatten)]
    obj: DynamicObject,
    pub max_red: u32,
    pub max_green: u32,
    pub max_blue: u32,
    pub max_heat: i32,
    pub radius: u32,
    #[serde(rename = "contributionGridOrigin.x")]
    pub contribution_grid_origin_x: i32,
    #[serde(rename = "contributionGridOrigin.y")]
    pub contribution_grid_origin_y: i32,
    pub light_direction: LightDirection,
}
inherit!(ArtificialLight -> DynamicObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FireObjectXml {
    #[serde(flatten)]
    obj: DynamicObject,
    burn_timer: f32,
    #[serde(rename = "spreadTimer_0")]
    spread_timer_0: f32,
    #[serde(rename = "spreadTimer_1")]
    spread_timer_1: f32,
    #[serde(rename = "spreadTimer_2")]
    spread_timer_2: f32,
    #[serde(rename = "spreadTimer_3")]
    spread_timer_3: f32,
    light_dict: ArtificialLight,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FireObject {
    obj: DynamicObject,
    pub burn_timer: f32,
    pub spread_timer: [f32; 4],
    pub light_dict: ArtificialLight,
}

impl From<FireObjectXml> for FireObject {
    fn from(value: FireObjectXml) -> Self {
        Self {
            obj: value.obj,
            burn_timer: value.burn_timer,
            spread_timer: [
                value.spread_timer_0,
                value.spread_timer_1,
                value.spread_timer_2,
                value.spread_timer_3,
            ],
            light_dict: value.light_dict,
        }
    }
}

impl From<FireObject> for FireObjectXml {
    fn from(value: FireObject) -> Self {
        let [a, b, c, d] = value.spread_timer;
        Self {
            obj: value.obj,
            burn_timer: value.burn_timer,
            spread_timer_0: a,
            spread_timer_1: b,
            spread_timer_2: c,
            spread_timer_3: d,
            light_dict: value.light_dict,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlowBlock {
    #[serde(flatten)]
    obj: DynamicObject,
    pub light_dict: ArtificialLight,
    pub tile_type: BlockType,
}
inherit!(GlowBlock -> DynamicObject, obj);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatherBlock {
    #[serde(flatten)]
    obj: DynamicObject,
    pub timer: f32,
    pub last_known_gather_value: u32,
}
inherit!(GatherBlock -> DynamicObject, obj);

pub mod animal;
pub mod blockhead;
pub mod chest;
pub mod craft;
pub mod dropped_item;
pub mod plant;
pub mod train;
pub mod tree;
pub mod workbench;

#[derive(Debug, Clone, PartialEq, IntoStaticStr)]
pub enum AnyDynamicObject {
    Dodo(Box<animal::Dodo>),                      // ID = 13
    Ladder(Box<craft::Ladder>),                   // ID = 19
    Door(Box<craft::Door>),                       // ID = 20
    Bed(Box<craft::Bed>),                         // ID = 23
    DropBear(Box<animal::DropBear>),              // ID = 25
    Donkey(Box<animal::Donkey>),                  // ID = 28
    Egg(Box<animal::Egg>),                        // ID = 30
    ClownFish(Box<animal::ClownFish>),            // ID = 35
    Shark(Box<animal::Shark>),                    // ID = 36
    HandCar(Box<train::HandCar>),                 // ID = 41
    SteamLocomotive(Box<train::SteamLocomotive>), // ID = 42
    FreightCar(Box<train::FreightCar>),           // ID = 43
    PassengerCar(Box<train::PassengerCar>),       // ID = 44
    Workbench(Box<workbench::Workbench>),         // ID = 45
    Chest(Box<chest::Chest>),                     // ID = 46
    Sign(Box<craft::Sign>),                       // ID = 47
    TradingPost(Box<craft::TradingPost>),         // ID = 48
    TrainStation(Box<train::TrainStation>),       // ID = 49
    TradePortal(Box<craft::TradePortal>),         // ID = 50
    Scorpion(Box<animal::Scorpion>),              // ID = 51
    Painting(Box<craft::Painting>),               // ID = 52
    Column(Box<craft::Column>),                   // ID = 53
    Stairs(Box<craft::Stairs>),                   // ID = 54
    ElevatorMotor(Box<craft::ElevatorMotor>),     // ID = 55
    ElevatorShaft(Box<craft::ElevatorShaft>),     // ID = 56
    OwnershipSign(Box<craft::OwnershipSign>),     // ID = 60
    Yak(Box<animal::Yak>),                        // ID = 63
    Mirror(Box<craft::Mirror>),                   // ID = 64
}

impl Default for AnyDynamicObject {
    fn default() -> Self {
        AnyDynamicObject::Ladder(Box::default())
    }
}

impl AnyDynamicObject {
    pub fn set_float_pos(&mut self, pos: [f32; 2]) {
        match self {
            AnyDynamicObject::Dodo(dodo) => dodo.set_float_pos(pos),
            AnyDynamicObject::Ladder(ladder) => ladder.set_float_pos(pos),
            AnyDynamicObject::Door(door) => door.set_float_pos(pos),
            AnyDynamicObject::Bed(bed) => bed.set_float_pos(pos),
            AnyDynamicObject::DropBear(dropbear) => dropbear.set_float_pos(pos),
            AnyDynamicObject::Donkey(donkey) => donkey.set_float_pos(pos),
            AnyDynamicObject::Egg(egg) => egg.set_float_pos(pos),
            AnyDynamicObject::ClownFish(clown_fish) => clown_fish.set_float_pos(pos),
            AnyDynamicObject::Shark(shark) => shark.set_float_pos(pos),
            AnyDynamicObject::HandCar(hand_car) => hand_car.set_float_pos(pos),
            AnyDynamicObject::SteamLocomotive(locomotive) => locomotive.set_float_pos(pos),
            AnyDynamicObject::FreightCar(freight_car) => freight_car.set_float_pos(pos),
            AnyDynamicObject::PassengerCar(passenger_car) => passenger_car.set_float_pos(pos),
            AnyDynamicObject::Workbench(workbench) => workbench.set_float_pos(pos),
            AnyDynamicObject::Chest(chest) => chest.set_float_pos(pos),
            AnyDynamicObject::Sign(sign) => sign.set_float_pos(pos),
            AnyDynamicObject::TradingPost(trading_post) => trading_post.set_float_pos(pos),
            AnyDynamicObject::TrainStation(train_station) => train_station.set_float_pos(pos),
            AnyDynamicObject::TradePortal(trade_portal) => trade_portal.set_float_pos(pos),
            AnyDynamicObject::Scorpion(scorpion) => scorpion.set_float_pos(pos),
            AnyDynamicObject::Painting(painting) => painting.set_float_pos(pos),
            AnyDynamicObject::Column(column) => column.set_float_pos(pos),
            AnyDynamicObject::Stairs(stairs) => stairs.set_float_pos(pos),
            AnyDynamicObject::ElevatorMotor(elevator_motor) => elevator_motor.set_float_pos(pos),
            AnyDynamicObject::ElevatorShaft(elevator_shaft) => elevator_shaft.set_float_pos(pos),
            AnyDynamicObject::OwnershipSign(ownership_sign) => ownership_sign.set_float_pos(pos),
            AnyDynamicObject::Yak(yak) => yak.set_float_pos(pos),
            AnyDynamicObject::Mirror(mirror) => mirror.set_float_pos(pos),
        }
    }

    pub fn set_pos(&mut self, pos: (u32, u16)) {
        match self {
            AnyDynamicObject::Dodo(dodo) => dodo.set_pos(pos),
            AnyDynamicObject::Ladder(ladder) => ladder.set_pos(pos),
            AnyDynamicObject::Door(door) => door.set_pos(pos),
            AnyDynamicObject::Bed(bed) => bed.set_pos(pos),
            AnyDynamicObject::DropBear(dropbear) => dropbear.set_pos(pos),
            AnyDynamicObject::Donkey(donkey) => donkey.set_pos(pos),
            AnyDynamicObject::Egg(egg) => egg.set_pos(pos),
            AnyDynamicObject::ClownFish(clown_fish) => clown_fish.set_pos(pos),
            AnyDynamicObject::Shark(shark) => shark.set_pos(pos),
            AnyDynamicObject::HandCar(hand_car) => hand_car.set_pos(pos),
            AnyDynamicObject::SteamLocomotive(locomotive) => locomotive.set_pos(pos),
            AnyDynamicObject::FreightCar(freight_car) => freight_car.set_pos(pos),
            AnyDynamicObject::PassengerCar(passenger_car) => passenger_car.set_pos(pos),
            AnyDynamicObject::Workbench(workbench) => workbench.set_pos(pos),
            AnyDynamicObject::Chest(chest) => chest.set_pos(pos),
            AnyDynamicObject::Sign(sign) => sign.set_pos(pos),
            AnyDynamicObject::TradingPost(trading_post) => trading_post.set_pos(pos),
            AnyDynamicObject::TrainStation(train_station) => train_station.set_pos(pos),
            AnyDynamicObject::TradePortal(trade_portal) => trade_portal.set_pos(pos),
            AnyDynamicObject::Scorpion(scorpion) => scorpion.set_pos(pos),
            AnyDynamicObject::Painting(painting) => painting.set_pos(pos),
            AnyDynamicObject::Column(column) => column.set_pos(pos),
            AnyDynamicObject::Stairs(stairs) => stairs.set_pos(pos),
            AnyDynamicObject::ElevatorMotor(elevator_motor) => elevator_motor.set_pos(pos),
            AnyDynamicObject::ElevatorShaft(elevator_shaft) => elevator_shaft.set_pos(pos),
            AnyDynamicObject::OwnershipSign(ownership_sign) => ownership_sign.set_pos(pos),
            AnyDynamicObject::Yak(yak) => yak.set_pos(pos),
            AnyDynamicObject::Mirror(mirror) => mirror.set_pos(pos),
        }
    }

    pub fn set_unique_id(&mut self, unique_id: UniqueID) {
        match self {
            AnyDynamicObject::Dodo(dodo) => dodo.unique_id = unique_id,
            AnyDynamicObject::Ladder(ladder) => ladder.unique_id = unique_id,
            AnyDynamicObject::Door(door) => door.unique_id = unique_id,
            AnyDynamicObject::Bed(bed) => bed.unique_id = unique_id,
            AnyDynamicObject::DropBear(dropbear) => dropbear.unique_id = unique_id,
            AnyDynamicObject::Donkey(donkey) => donkey.unique_id = unique_id,
            AnyDynamicObject::Egg(egg) => egg.unique_id = unique_id,
            AnyDynamicObject::ClownFish(clown_fish) => clown_fish.unique_id = unique_id,
            AnyDynamicObject::Shark(shark) => shark.unique_id = unique_id,
            AnyDynamicObject::HandCar(hand_car) => hand_car.unique_id = unique_id,
            AnyDynamicObject::SteamLocomotive(locomotive) => locomotive.unique_id = unique_id,
            AnyDynamicObject::FreightCar(freight_car) => freight_car.unique_id = unique_id,
            AnyDynamicObject::PassengerCar(passenger_car) => passenger_car.unique_id = unique_id,
            AnyDynamicObject::Workbench(workbench) => workbench.unique_id = unique_id,
            AnyDynamicObject::Chest(chest) => chest.unique_id = unique_id,
            AnyDynamicObject::Sign(sign) => sign.unique_id = unique_id,
            AnyDynamicObject::TradingPost(trading_post) => trading_post.unique_id = unique_id,
            AnyDynamicObject::TrainStation(train_station) => train_station.unique_id = unique_id,
            AnyDynamicObject::TradePortal(trade_portal) => trade_portal.unique_id = unique_id,
            AnyDynamicObject::Scorpion(scorpion) => scorpion.unique_id = unique_id,
            AnyDynamicObject::Painting(painting) => painting.unique_id = unique_id,
            AnyDynamicObject::Column(column) => column.unique_id = unique_id,
            AnyDynamicObject::Stairs(stairs) => stairs.unique_id = unique_id,
            AnyDynamicObject::ElevatorMotor(elevator_motor) => elevator_motor.unique_id = unique_id,
            AnyDynamicObject::ElevatorShaft(elevator_shaft) => elevator_shaft.unique_id = unique_id,
            AnyDynamicObject::OwnershipSign(ownership_sign) => ownership_sign.unique_id = unique_id,
            AnyDynamicObject::Yak(yak) => yak.unique_id = unique_id,
            AnyDynamicObject::Mirror(mirror) => mirror.unique_id = unique_id,
        }
    }

    pub fn try_from_save_dict(item_type: ItemType, value: plist::Value) -> Result<Self> {
        match item_type {
            ItemType::CagedDodo => {
                // ID = 13
                let dodo = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Dodo",
                    dict: value,
                })?;
                Ok(Self::Dodo(dodo))
            }
            ItemType::Ladder => {
                // ID = 19
                let ladder = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Ladder",
                    dict: value,
                })?;
                Ok(Self::Ladder(ladder))
            }
            ItemType::WoodenGate
            | ItemType::Door
            | ItemType::IronDoor
            | ItemType::Trapdoor
            | ItemType::IronTrapdoor => {
                // ID = 20
                let door = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Door",
                    dict: value,
                })?;
                Ok(Self::Door(door))
            }
            ItemType::Bed
            | ItemType::SoftBed
            | ItemType::GoldenBed
            | ItemType::RainbowSoftBed
            | ItemType::RainbowGoldenBed => {
                // ID = 23
                let bed = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Bed",
                    dict: value,
                })?;
                Ok(Self::Bed(Box::new(bed)))
            }
            ItemType::CagedDropbear => {
                // ID = 25
                let dropbear = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "DropBear",
                    dict: value,
                })?;
                Ok(Self::DropBear(dropbear))
            }
            ItemType::CagedDonkey | ItemType::CagedUnicorn => {
                // ID = 28
                let donkey = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Donkey",
                    dict: value,
                })?;
                Ok(Self::Donkey(donkey))
            }
            ItemType::DodoEgg => {
                // ID = 30
                let egg = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Egg",
                    dict: value,
                })?;
                Ok(Self::Egg(egg))
            }
            ItemType::FishBucket => {
                // ID = 35
                let clown_fish = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "ClownFish",
                    dict: value,
                })?;
                Ok(Self::ClownFish(Box::new(clown_fish)))
            }
            ItemType::SharkBucket => {
                // ID = 36
                let shark = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Shark",
                    dict: value,
                })?;
                Ok(Self::Shark(shark))
            }
            ItemType::RailHandcar => {
                // ID = 41
                let hand_car = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "HandCar",
                    dict: value,
                })?;
                Ok(Self::HandCar(hand_car))
            }
            ItemType::SteamLocomotive => {
                // ID = 42
                let locomotive = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "SteamLocomotive",
                    dict: value,
                })?;
                Ok(Self::SteamLocomotive(locomotive))
            }
            ItemType::FreightCar => {
                // ID = 43
                let freight_car_save_dict_xml: FreightCarSaveDictXml = plist::from_value(&value)
                    .context(DeserializeDictionarySnafu {
                        target_type: "FreightCar",
                        dict: value,
                    })?;
                let freight_car = Box::new(
                    FreightCar::from_save_dict_xml(freight_car_save_dict_xml)
                        .context(LoadTrainChestSnafu)?,
                );
                Ok(Self::FreightCar(freight_car))
            }
            ItemType::PassengerCar => {
                // ID = 44
                let passenger_car =
                    plist::from_value(&value).context(DeserializeDictionarySnafu {
                        target_type: "PassengerCar",
                        dict: value,
                    })?;
                Ok(Self::PassengerCar(passenger_car))
            }
            ItemType::Portal
            | ItemType::AmethystPortal
            | ItemType::SapphirePortal
            | ItemType::EmeraldPortal
            | ItemType::RubyPortal
            | ItemType::DiamondPortal
            | ItemType::WorkBench
            | ItemType::Campfire
            | ItemType::TaylorsBench
            | ItemType::WoodworkBench
            | ItemType::ToolBench
            | ItemType::Press
            | ItemType::Kiln
            | ItemType::Furnace
            | ItemType::CraftBench
            | ItemType::MixingBench
            | ItemType::DyeBench
            | ItemType::MetalworkBench
            | ItemType::SteamGenerator
            | ItemType::ElectricKiln
            | ItemType::ElectricFurnace
            | ItemType::ElectricMetalworkBench
            | ItemType::ElectricStove
            | ItemType::SolarPanel
            | ItemType::Flywheel
            | ItemType::ArmorBench
            | ItemType::TrainYard
            | ItemType::Easel
            | ItemType::BuildersBench
            | ItemType::Refinery
            | ItemType::ElectricPress
            | ItemType::CompostBin
            | ItemType::ElectricSluice
            | ItemType::EggExtractor
            | ItemType::PizzaOven => {
                // ID = 45
                let workbench = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Workbench",
                    dict: value,
                })?;
                Ok(Self::Workbench(Box::new(workbench)))
            }
            ItemType::Chest
            | ItemType::Safe
            | ItemType::Shelf
            | ItemType::GoldenChest
            | ItemType::PortalChest
            | ItemType::DisplayCabinet
            | ItemType::FeederChest => {
                // ID = 46
                let chest_item = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "ChestItem",
                    dict: value,
                })?;
                Ok(Self::Chest(Box::new(
                    chest::Chest::from_chest_save_dict_xml(chest_item).context(LoadChestSnafu)?,
                )))
            }
            ItemType::Sign => {
                // ID = 47
                let sign = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Sign",
                    dict: value,
                })?;
                Ok(Self::Sign(sign))
            }
            ItemType::Shop => {
                // ID = 48
                let trading_post =
                    plist::from_value(&value).context(DeserializeDictionarySnafu {
                        target_type: "TradingPost",
                        dict: value,
                    })?;
                Ok(Self::TradingPost(Box::new(trading_post)))
            }
            ItemType::TrainStation => {
                // ID = 49
                let train_station =
                    plist::from_value(&value).context(DeserializeDictionarySnafu {
                        target_type: "TrainStation",
                        dict: value,
                    })?;
                Ok(Self::TrainStation(train_station))
            }
            ItemType::TradePortal => {
                // ID = 50
                let trade_portal =
                    plist::from_value(&value).context(DeserializeDictionarySnafu {
                        target_type: "TradePortal",
                        dict: value,
                    })?;
                Ok(Self::TradePortal(Box::new(trade_portal)))
            }
            ItemType::CagedScorpion => {
                // ID = 51
                let scorpion = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Scorpion",
                    dict: value,
                })?;
                Ok(Self::Scorpion(Box::new(scorpion)))
            }
            ItemType::LargeSquarePainting
            | ItemType::LargeLandscapePainting
            | ItemType::LargePortraitPainting
            | ItemType::MedSquarePainting
            | ItemType::MedLandscapePainting
            | ItemType::MedPortraitPainting
            | ItemType::SmallSquarePainting
            | ItemType::SmallLandscapePainting
            | ItemType::SmallPortraitPainting => {
                // ID = 52
                let painting = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Painting",
                    dict: value,
                })?;
                Ok(Self::Painting(Box::new(painting)))
            }
            ItemType::StoneColumn
            | ItemType::LimestoneColumn
            | ItemType::MarbleColumn
            | ItemType::SandstoneColumn
            | ItemType::RedMarbleColumn
            | ItemType::LapisLazuliColumn
            | ItemType::BasaltColumn
            | ItemType::CopperColumn
            | ItemType::TinColumn
            | ItemType::BronzeColumn
            | ItemType::IronColumn
            | ItemType::SteelColumn
            | ItemType::GoldColumn
            | ItemType::WoodColumn
            | ItemType::BrickColumn
            | ItemType::IceColumn
            | ItemType::PlatiumColumn
            | ItemType::GlassColumn
            | ItemType::BlackGlassColumn
            | ItemType::TitaniumColumn
            | ItemType::CarbonFiberColumn
            | ItemType::PlasterColumn
            | ItemType::AmethystColumn
            | ItemType::SapphireColumn
            | ItemType::EmeraldColumn
            | ItemType::RubyColumn
            | ItemType::DiamondColumn => {
                // ID = 53
                let column = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Column",
                    dict: value,
                })?;
                Ok(Self::Column(Box::new(column)))
            }
            ItemType::StoneStairs
            | ItemType::LimestoneStairs
            | ItemType::MarbleStairs
            | ItemType::SandstoneStairs
            | ItemType::RedMarbleStairs
            | ItemType::LapisLazuliStairs
            | ItemType::BasaltStairs
            | ItemType::CopperStairs
            | ItemType::TinStairs
            | ItemType::BronzeStairs
            | ItemType::IronStairs
            | ItemType::SteelStairs
            | ItemType::GoldStairs
            | ItemType::WoodStairs
            | ItemType::BrickStairs
            | ItemType::IceStairs
            | ItemType::PlatiumStairs
            | ItemType::GlassStairs
            | ItemType::BlackGlassStairs
            | ItemType::TitaniumStairs
            | ItemType::CarbonFiberStairs
            | ItemType::PlasterStairs
            | ItemType::AmethystStairs
            | ItemType::SapphireStairs
            | ItemType::EmeraldStairs
            | ItemType::RubyStairs
            | ItemType::DiamondStairs => {
                // ID = 54
                let stairs = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Stairs",
                    dict: value,
                })?;
                Ok(Self::Stairs(Box::new(stairs)))
            }
            ItemType::ElectricElevatorMotor => {
                // ID = 55
                let elevator_motor =
                    plist::from_value(&value).context(DeserializeDictionarySnafu {
                        target_type: "ElevatorMotor",
                        dict: value,
                    })?;
                Ok(Self::ElevatorMotor(Box::new(elevator_motor)))
            }
            ItemType::ElevatorShaft => {
                // ID = 56
                let elevator_shaft =
                    plist::from_value(&value).context(DeserializeDictionarySnafu {
                        target_type: "ElevatorShaft",
                        dict: value,
                    })?;
                Ok(Self::ElevatorShaft(Box::new(elevator_shaft)))
            }
            ItemType::OwnershipSign => {
                // ID = 60
                let ownership_sign =
                    plist::from_value(&value).context(DeserializeDictionarySnafu {
                        target_type: "OwnershipSign",
                        dict: value,
                    })?;
                Ok(Self::OwnershipSign(Box::new(ownership_sign)))
            }
            ItemType::CagedYak => {
                // ID = 63
                let yak = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Yak",
                    dict: value,
                })?;
                Ok(Self::Yak(yak))
            }
            ItemType::Mirror => {
                // ID = 64
                let mirror = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Mirror",
                    dict: value,
                })?;
                Ok(Self::Mirror(Box::new(mirror)))
            }
            _ => UnsupportedItemTypeForDynObjSnafu { item_type, value }.fail(),
        }
    }

    pub fn to_save_dict(&self) -> Result<plist::Value> {
        Ok(match self {
            Self::Dodo(dodo) => plist::to_value(dodo).context(SerializeDictionarySnafu {
                source_type: "Dodo",
            })?,
            Self::Ladder(ladder) => plist::to_value(ladder).context(SerializeDictionarySnafu {
                source_type: "Ladder",
            })?,
            Self::Door(door) => plist::to_value(door).context(SerializeDictionarySnafu {
                source_type: "Door",
            })?,
            Self::Bed(bed) => {
                plist::to_value(bed).context(SerializeDictionarySnafu { source_type: "Bed" })?
            }
            Self::DropBear(dropbear) => {
                plist::to_value(dropbear).context(SerializeDictionarySnafu {
                    source_type: "DropBear",
                })?
            }
            Self::Donkey(donkey) => plist::to_value(donkey).context(SerializeDictionarySnafu {
                source_type: "Donkey",
            })?,
            Self::Egg(egg) => {
                plist::to_value(egg).context(SerializeDictionarySnafu { source_type: "Egg" })?
            }
            Self::ClownFish(clown_fish) => {
                plist::to_value(clown_fish).context(SerializeDictionarySnafu {
                    source_type: "ClownFish",
                })?
            }
            Self::Shark(shark) => plist::to_value(shark).context(SerializeDictionarySnafu {
                source_type: "Shark",
            })?,
            Self::HandCar(hand_car) => {
                plist::to_value(hand_car).context(SerializeDictionarySnafu {
                    source_type: "HandCar",
                })?
            }
            Self::SteamLocomotive(locomotive) => {
                plist::to_value(locomotive).context(SerializeDictionarySnafu {
                    source_type: "SteamLocomotive",
                })?
            }
            Self::FreightCar(freight_car) => {
                let freight_car_save_dict_xml: FreightCarSaveDictXml = freight_car
                    .to_save_dict_xml()
                    .context(SaveTrainChestSnafu)?;
                plist::to_value(&freight_car_save_dict_xml).context(SerializeDictionarySnafu {
                    source_type: "FreightCar",
                })?
            }
            Self::PassengerCar(passenger_car) => {
                plist::to_value(passenger_car).context(SerializeDictionarySnafu {
                    source_type: "PassengerCar",
                })?
            }
            Self::Workbench(workbench) => {
                plist::to_value(workbench).context(SerializeDictionarySnafu {
                    source_type: "Workbench",
                })?
            }
            Self::Chest(chest) => {
                let chest_item = chest.to_chest_save_dict_xml().context(SaveChestSnafu)?;

                plist::to_value(&chest_item).context(SerializeDictionarySnafu {
                    source_type: "ChestItem",
                })?
            }
            Self::Sign(sign) => plist::to_value(sign).context(SerializeDictionarySnafu {
                source_type: "Sign",
            })?,
            Self::TradingPost(trading_post) => {
                plist::to_value(trading_post).context(SerializeDictionarySnafu {
                    source_type: "TradingPost",
                })?
            }
            Self::TrainStation(train_station) => {
                plist::to_value(train_station).context(SerializeDictionarySnafu {
                    source_type: "TrainStation",
                })?
            }
            Self::TradePortal(trade_portal) => {
                plist::to_value(trade_portal).context(SerializeDictionarySnafu {
                    source_type: "TradePortal",
                })?
            }
            Self::Scorpion(scorpion) => {
                plist::to_value(scorpion).context(SerializeDictionarySnafu {
                    source_type: "Scorpion",
                })?
            }
            Self::Painting(painting) => {
                plist::to_value(painting).context(SerializeDictionarySnafu {
                    source_type: "Painting",
                })?
            }
            Self::Column(column) => plist::to_value(column).context(SerializeDictionarySnafu {
                source_type: "Column",
            })?,
            Self::Stairs(stairs) => plist::to_value(stairs).context(SerializeDictionarySnafu {
                source_type: "Stairs",
            })?,
            Self::ElevatorMotor(elevator_motor) => {
                plist::to_value(elevator_motor).context(SerializeDictionarySnafu {
                    source_type: "ElevatorMotor",
                })?
            }
            Self::ElevatorShaft(elevator_shaft) => {
                plist::to_value(elevator_shaft).context(SerializeDictionarySnafu {
                    source_type: "ElevatorShaft",
                })?
            }
            Self::OwnershipSign(ownership_sign) => {
                plist::to_value(ownership_sign).context(SerializeDictionarySnafu {
                    source_type: "OwnershipSign",
                })?
            }
            Self::Yak(yak) => {
                plist::to_value(yak).context(SerializeDictionarySnafu { source_type: "Yak" })?
            }
            Self::Mirror(mirror) => plist::to_value(mirror).context(SerializeDictionarySnafu {
                source_type: "Mirror",
            })?,
        })
    }
}

#[derive(Debug, PartialEq, IntoStaticStr)]
pub enum AnyDynamicObjectRef<'a> {
    Dodo(&'a animal::Dodo),                      // ID = 13
    Ladder(&'a craft::Ladder),                   // ID = 19
    Door(&'a craft::Door),                       // ID = 20
    Bed(&'a craft::Bed),                         // ID = 23
    DropBear(&'a animal::DropBear),              // ID = 25
    Donkey(&'a animal::Donkey),                  // ID = 28
    Egg(&'a animal::Egg),                        // ID = 30
    ClownFish(&'a animal::ClownFish),            // ID = 35
    Shark(&'a animal::Shark),                    // ID = 36
    HandCar(&'a train::HandCar),                 // ID = 41
    SteamLocomotive(&'a train::SteamLocomotive), // ID = 42
    FreightCar(&'a train::FreightCar),           // ID = 43
    PassengerCar(&'a train::PassengerCar),       // ID = 44
    Workbench(&'a workbench::Workbench),         // ID = 45
    Chest(&'a chest::Chest),                     // ID = 46
    Sign(&'a craft::Sign),                       // ID = 47
    TradingPost(&'a craft::TradingPost),         // ID = 48
    TrainStation(&'a train::TrainStation),       // ID = 49
    TradePortal(&'a craft::TradePortal),         // ID = 50
    Scorpion(&'a animal::Scorpion),              // ID = 51
    Painting(&'a craft::Painting),               // ID = 52
    Column(&'a craft::Column),                   // ID = 53
    Stairs(&'a craft::Stairs),                   // ID = 54
    ElevatorMotor(&'a craft::ElevatorMotor),     // ID = 55
    ElevatorShaft(&'a craft::ElevatorShaft),     // ID = 56
    OwnershipSign(&'a craft::OwnershipSign),     // ID = 60
    Yak(&'a animal::Yak),                        // ID = 63
    Mirror(&'a craft::Mirror),                   // ID = 64
}

impl<'a> AnyDynamicObjectRef<'a> {
    pub fn to_owned(&self) -> AnyDynamicObject {
        match *self {
            AnyDynamicObjectRef::Dodo(x) => AnyDynamicObject::Dodo(Box::new(x.clone())),
            AnyDynamicObjectRef::Ladder(x) => AnyDynamicObject::Ladder(Box::new(x.clone())),
            AnyDynamicObjectRef::Door(x) => AnyDynamicObject::Door(Box::new(x.clone())),
            AnyDynamicObjectRef::Bed(x) => AnyDynamicObject::Bed(Box::new(x.clone())),
            AnyDynamicObjectRef::DropBear(x) => AnyDynamicObject::DropBear(Box::new(x.clone())),
            AnyDynamicObjectRef::Donkey(x) => AnyDynamicObject::Donkey(Box::new(x.clone())),
            AnyDynamicObjectRef::Egg(x) => AnyDynamicObject::Egg(Box::new(x.clone())),
            AnyDynamicObjectRef::ClownFish(x) => AnyDynamicObject::ClownFish(Box::new(x.clone())),
            AnyDynamicObjectRef::Shark(x) => AnyDynamicObject::Shark(Box::new(x.clone())),
            AnyDynamicObjectRef::HandCar(x) => AnyDynamicObject::HandCar(Box::new(x.clone())),
            AnyDynamicObjectRef::SteamLocomotive(x) => {
                AnyDynamicObject::SteamLocomotive(Box::new(x.clone()))
            }
            AnyDynamicObjectRef::FreightCar(x) => AnyDynamicObject::FreightCar(Box::new(x.clone())),
            AnyDynamicObjectRef::PassengerCar(x) => {
                AnyDynamicObject::PassengerCar(Box::new(x.clone()))
            }
            AnyDynamicObjectRef::Workbench(x) => AnyDynamicObject::Workbench(Box::new(x.clone())),
            AnyDynamicObjectRef::Chest(x) => AnyDynamicObject::Chest(Box::new(x.clone())),
            AnyDynamicObjectRef::Sign(x) => AnyDynamicObject::Sign(Box::new(x.clone())),
            AnyDynamicObjectRef::TradingPost(x) => {
                AnyDynamicObject::TradingPost(Box::new(x.clone()))
            }
            AnyDynamicObjectRef::TrainStation(x) => {
                AnyDynamicObject::TrainStation(Box::new(x.clone()))
            }
            AnyDynamicObjectRef::TradePortal(x) => {
                AnyDynamicObject::TradePortal(Box::new(x.clone()))
            }
            AnyDynamicObjectRef::Scorpion(x) => AnyDynamicObject::Scorpion(Box::new(x.clone())),
            AnyDynamicObjectRef::Painting(x) => AnyDynamicObject::Painting(Box::new(x.clone())),
            AnyDynamicObjectRef::Column(x) => AnyDynamicObject::Column(Box::new(x.clone())),
            AnyDynamicObjectRef::Stairs(x) => AnyDynamicObject::Stairs(Box::new(x.clone())),
            AnyDynamicObjectRef::ElevatorMotor(x) => {
                AnyDynamicObject::ElevatorMotor(Box::new(x.clone()))
            }
            AnyDynamicObjectRef::ElevatorShaft(x) => {
                AnyDynamicObject::ElevatorShaft(Box::new(x.clone()))
            }
            AnyDynamicObjectRef::OwnershipSign(x) => {
                AnyDynamicObject::OwnershipSign(Box::new(x.clone()))
            }
            AnyDynamicObjectRef::Yak(x) => AnyDynamicObject::Yak(Box::new(x.clone())),
            AnyDynamicObjectRef::Mirror(x) => AnyDynamicObject::Mirror(Box::new(x.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DynamicObjectList, DynamicObjectType, FireObjectXml, GatherBlock, GlowBlock,
        animal::{CaveTroll, ClownFish, Dodo, Donkey, DropBear, Egg, Scorpion, Shark, Yak},
        chest::ChestDwXml,
        craft::{
            Bed, Boat, Column, Door, ElevatorMotor, ElevatorShaft, Ladder, Mirror, OwnershipSign,
            Painting, Rail, Sign, Stairs, Torch, TradePortal, TradingPost, Window, Wire,
        },
        dropped_item::DroppedItemXml,
        plant::{
            CarrotPlant, ChilliPlant, CornPlant, FlaxPlant, KelpPlant, SunflowerPlant, TomatoPlant,
            TulipPlant, VinePlant, WheatPlant,
        },
        train::{FreightCarDwXml, HandCar, PassengerCar, SteamLocomotive, TrainStation},
        tree::{
            AppleTree, CactusTree, CherryTree, CoconutTree, CoffeeTree, GemTree, LimeTree,
            MangoTree, MapleTree, OrangeTree, PineTree,
        },
        workbench::Workbench,
    };
    use crate::util::plist::diff_plist_keys;

    fn check_round_trip<T>(obj_type: DynamicObjectType) -> Result<(), Box<dyn std::error::Error>>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let test_xml = std::fs::read(format!("resources/type_{}.xml", obj_type as u16))?;

        // round-trip: deser -> ser -> deser, check equality
        let parsed = plist::from_bytes::<DynamicObjectList<T>>(&test_xml)?;
        let mut serialized = Vec::with_capacity(test_xml.len());
        plist::to_writer_xml(&mut serialized, &parsed)?;
        let round_tripped = plist::from_bytes::<DynamicObjectList<T>>(&serialized)?;
        assert_eq!(parsed, round_tripped, "struct round-trip mismatch");

        // compare plist::Value trees to detect contamination (extra keys) and data loss (missing keys)
        let original_val = plist::from_bytes::<plist::Value>(&test_xml)?;
        let serialized_val = plist::from_bytes::<plist::Value>(&serialized)?;
        let mut diffs = Vec::new();
        diff_plist_keys("", &original_val, &serialized_val, &mut diffs);
        assert!(
            diffs.is_empty(),
            "structural fidelity violations:\n{}",
            diffs.join("\n")
        );

        Ok(())
    }

    #[test]
    fn test_round_trip() {
        check_round_trip::<AppleTree>(DynamicObjectType::AppleTree).unwrap();
        check_round_trip::<MapleTree>(DynamicObjectType::MapleTree).unwrap();
        check_round_trip::<MangoTree>(DynamicObjectType::MangoTree).unwrap();
        check_round_trip::<PineTree>(DynamicObjectType::PineTree).unwrap();
        check_round_trip::<CactusTree>(DynamicObjectType::CactusTree).unwrap();
        check_round_trip::<CoconutTree>(DynamicObjectType::CoconutTree).unwrap();
        check_round_trip::<OrangeTree>(DynamicObjectType::OrangeTree).unwrap();
        check_round_trip::<CherryTree>(DynamicObjectType::CherryTree).unwrap();
        check_round_trip::<CoffeeTree>(DynamicObjectType::CoffeeTree).unwrap();
        check_round_trip::<FlaxPlant>(DynamicObjectType::FlaxPlant).unwrap();
        check_round_trip::<SunflowerPlant>(DynamicObjectType::SunflowerPlant).unwrap();
        check_round_trip::<CornPlant>(DynamicObjectType::CornPlant).unwrap();
        check_round_trip::<Dodo>(DynamicObjectType::Dodo).unwrap();
        check_round_trip::<DroppedItemXml>(DynamicObjectType::DroppedItem).unwrap();
        check_round_trip::<FireObjectXml>(DynamicObjectType::Fire).unwrap();
        check_round_trip::<Torch>(DynamicObjectType::Torch).unwrap();
        check_round_trip::<GlowBlock>(DynamicObjectType::GlowBlock).unwrap();
        check_round_trip::<Ladder>(DynamicObjectType::Ladder).unwrap();
        check_round_trip::<Door>(DynamicObjectType::Door).unwrap();
        // check_round_trip::<ArtificialLight>(DynamicObjectType::ArtificialLight).unwrap();
        check_round_trip::<Bed>(DynamicObjectType::Bed).unwrap();
        check_round_trip::<DropBear>(DynamicObjectType::DropBear).unwrap();
        check_round_trip::<GatherBlock>(DynamicObjectType::GatherBlock).unwrap();
        check_round_trip::<CarrotPlant>(DynamicObjectType::CarrotPlant).unwrap();
        check_round_trip::<Donkey>(DynamicObjectType::Donkey).unwrap();
        check_round_trip::<Egg>(DynamicObjectType::Egg).unwrap();
        check_round_trip::<Window>(DynamicObjectType::Window).unwrap();
        check_round_trip::<Boat>(DynamicObjectType::Boat).unwrap();
        check_round_trip::<ChilliPlant>(DynamicObjectType::ChilliPlant).unwrap();
        check_round_trip::<KelpPlant>(DynamicObjectType::KelpPlant).unwrap();
        check_round_trip::<ClownFish>(DynamicObjectType::ClownFish).unwrap();
        check_round_trip::<Shark>(DynamicObjectType::Shark).unwrap();
        check_round_trip::<LimeTree>(DynamicObjectType::LimeTree).unwrap();
        check_round_trip::<Wire>(DynamicObjectType::Wire).unwrap();
        check_round_trip::<CaveTroll>(DynamicObjectType::CaveTroll).unwrap();
        check_round_trip::<Rail>(DynamicObjectType::Rail).unwrap();
        check_round_trip::<HandCar>(DynamicObjectType::HandCar).unwrap();
        check_round_trip::<SteamLocomotive>(DynamicObjectType::SteamLocomotive).unwrap();
        check_round_trip::<FreightCarDwXml>(DynamicObjectType::FreightCar).unwrap();
        check_round_trip::<PassengerCar>(DynamicObjectType::PassengerCar).unwrap();
        check_round_trip::<Workbench>(DynamicObjectType::Workbench).unwrap();
        check_round_trip::<ChestDwXml>(DynamicObjectType::Chest).unwrap();
        check_round_trip::<Sign>(DynamicObjectType::Sign).unwrap();
        check_round_trip::<TradingPost>(DynamicObjectType::TradingPost).unwrap();
        check_round_trip::<TrainStation>(DynamicObjectType::TrainStation).unwrap();
        check_round_trip::<TradePortal>(DynamicObjectType::TradePortal).unwrap();
        check_round_trip::<Scorpion>(DynamicObjectType::Scorpion).unwrap();
        check_round_trip::<Painting>(DynamicObjectType::Painting).unwrap();
        check_round_trip::<Column>(DynamicObjectType::Column).unwrap();
        check_round_trip::<Stairs>(DynamicObjectType::Stairs).unwrap();
        check_round_trip::<ElevatorMotor>(DynamicObjectType::ElevatorMotor).unwrap();
        check_round_trip::<ElevatorShaft>(DynamicObjectType::ElevatorShaft).unwrap();
        check_round_trip::<GemTree>(DynamicObjectType::GemTree).unwrap();
        check_round_trip::<VinePlant>(DynamicObjectType::VinePlant).unwrap();
        check_round_trip::<TulipPlant>(DynamicObjectType::TulipPlant).unwrap();
        check_round_trip::<OwnershipSign>(DynamicObjectType::OwnershipSign).unwrap();
        check_round_trip::<WheatPlant>(DynamicObjectType::WheatPlant).unwrap();
        check_round_trip::<TomatoPlant>(DynamicObjectType::TomatoPlant).unwrap();
        check_round_trip::<Yak>(DynamicObjectType::Yak).unwrap();
        check_round_trip::<Mirror>(DynamicObjectType::Mirror).unwrap();
    }
}
