use super::item::ItemType;
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
    pub paint_color: u16,
}
inherit!(InteractionObject -> DynamicObject, obj);

impl InteractionObject {
    pub fn new(
        obj: DynamicObject,
        interaction_object_type: InteractionObjectType,
        is_in_use: bool,
        flipped: bool,
        paint_color: u16,
    ) -> Self {
        Self {
            obj,
            interaction_object_type,
            is_in_use,
            flipped,
            paint_color,
        }
    }
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum LightDirection {
    #[default]
    All = 0,
    Down = 1,
    Up = 2,
}

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
    Ladder(Box<craft::Ladder>),             // ID = 19
    Door(Box<craft::Door>),                 // ID = 20
    Bed(Box<craft::Bed>),                   // ID = 23
    Egg(Box<animal::Egg>),                  // ID = 30
    Workbench(Box<workbench::Workbench>),   // ID = 45
    Chest(Box<chest::Chest>),               // ID = 46
    Sign(Box<craft::Sign>),                 // ID = 47
    TrainStation(Box<train::TrainStation>), // ID = 49
}

impl Default for AnyDynamicObject {
    fn default() -> Self {
        AnyDynamicObject::Ladder(Box::default())
    }
}

impl AnyDynamicObject {
    pub fn try_from_save_dict(item_type: ItemType, value: plist::Value) -> Result<Self> {
        match item_type {
            ItemType::Chest
            | ItemType::Safe
            | ItemType::Shelf
            | ItemType::GoldenChest
            | ItemType::PortalChest
            | ItemType::DisplayCabinet
            | ItemType::FeederChest => {
                let chest_item = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "ChestItem",
                    dict: value,
                })?;
                Ok(Self::Chest(Box::new(
                    chest::Chest::from_chest_item(chest_item).context(LoadChestSnafu)?,
                )))
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
                let workbench = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Workbench",
                    dict: value,
                })?;
                Ok(Self::Workbench(Box::new(workbench)))
            }
            ItemType::Bed
            | ItemType::SoftBed
            | ItemType::GoldenBed
            | ItemType::RainbowSoftBed
            | ItemType::RainbowGoldenBed => {
                let bed = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Bed",
                    dict: value,
                })?;
                Ok(Self::Bed(Box::new(bed)))
            }
            ItemType::Ladder => {
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
                let door = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Door",
                    dict: value,
                })?;
                Ok(Self::Door(door))
            }
            ItemType::DodoEgg => {
                let egg = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Egg",
                    dict: value,
                })?;
                Ok(Self::Egg(egg))
            }
            ItemType::Sign => {
                let sign = plist::from_value(&value).context(DeserializeDictionarySnafu {
                    target_type: "Sign",
                    dict: value,
                })?;
                Ok(Self::Sign(sign))
            }
            ItemType::TrainStation => {
                let train_station =
                    plist::from_value(&value).context(DeserializeDictionarySnafu {
                        target_type: "TrainStation",
                        dict: value,
                    })?;
                Ok(Self::TrainStation(train_station))
            }
            _ => UnsupportedItemTypeForDynObjSnafu { item_type, value }.fail(),
        }
    }

    pub fn to_save_dict(&self) -> Result<plist::Value> {
        Ok(match self {
            Self::Ladder(ladder) => plist::to_value(ladder).context(SerializeDictionarySnafu {
                source_type: "Ladder",
            })?,
            Self::Door(door) => plist::to_value(door).context(SerializeDictionarySnafu {
                source_type: "Door",
            })?,
            Self::Bed(bed) => {
                plist::to_value(bed).context(SerializeDictionarySnafu { source_type: "Bed" })?
            }
            Self::Egg(egg) => {
                plist::to_value(egg).context(SerializeDictionarySnafu { source_type: "Egg" })?
            }
            Self::Workbench(workbench) => {
                plist::to_value(workbench).context(SerializeDictionarySnafu {
                    source_type: "Workbench",
                })?
            }
            Self::Chest(chest) => {
                let chest_item = chest.to_chest_item().context(SaveChestSnafu)?;

                plist::to_value(&chest_item).context(SerializeDictionarySnafu {
                    source_type: "ChestItem",
                })?
            }
            Self::Sign(sign) => plist::to_value(sign).context(SerializeDictionarySnafu {
                source_type: "Sign",
            })?,
            Self::TrainStation(train_station) => {
                plist::to_value(train_station).context(SerializeDictionarySnafu {
                    source_type: "TrainStation",
                })?
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DynamicObjectList, DynamicObjectType,
        animal::{CaveTroll, ClownFish, Dodo, Donkey, DropBear, Egg, Scorpion, Shark, Yak},
        craft::{
            Bed, Boat, Column, Door, ElevatorMotor, ElevatorShaft, Ladder, Rail, Sign, Stairs,
            Torch, TradePortal, TradingPost, Window, Wire,
        },
        plant::{
            CarrotPlant, ChilliPlant, CornPlant, FlaxPlant, KelpPlant, SunflowerPlant, TomatoPlant,
            TulipPlant, VinePlant, WheatPlant,
        },
        train::{FreightCar, HandCar, PassengerCar, SteamLocomotive, TrainStation},
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
        // check_round_trip::<DroppedItem>(DynamicObjectType::DroppedItem).unwrap();
        // check_round_trip::<Fire>(DynamicObjectType::Fire).unwrap();
        check_round_trip::<Torch>(DynamicObjectType::Torch).unwrap();
        // check_round_trip::<GlowBlock>(DynamicObjectType::GlowBlock).unwrap();
        check_round_trip::<Ladder>(DynamicObjectType::Ladder).unwrap();
        check_round_trip::<Door>(DynamicObjectType::Door).unwrap();
        // check_round_trip::<ArtificialLight>(DynamicObjectType::ArtificialLight).unwrap();
        check_round_trip::<Bed>(DynamicObjectType::Bed).unwrap();
        check_round_trip::<DropBear>(DynamicObjectType::DropBear).unwrap();
        // check_round_trip::<GatherBlock>(DynamicObjectType::GatherBlock).unwrap();
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
        check_round_trip::<FreightCar>(DynamicObjectType::FreightCar).unwrap();
        check_round_trip::<PassengerCar>(DynamicObjectType::PassengerCar).unwrap();
        check_round_trip::<Workbench>(DynamicObjectType::Workbench).unwrap();
        // check_round_trip::<Chest>(DynamicObjectType::Chest).unwrap();
        check_round_trip::<Sign>(DynamicObjectType::Sign).unwrap();
        check_round_trip::<TradingPost>(DynamicObjectType::TradingPost).unwrap();
        check_round_trip::<TrainStation>(DynamicObjectType::TrainStation).unwrap();
        check_round_trip::<TradePortal>(DynamicObjectType::TradePortal).unwrap();
        check_round_trip::<Scorpion>(DynamicObjectType::Scorpion).unwrap();
        // check_round_trip::<Painting>(DynamicObjectType::Painting).unwrap();
        check_round_trip::<Column>(DynamicObjectType::Column).unwrap();
        check_round_trip::<Stairs>(DynamicObjectType::Stairs).unwrap();
        check_round_trip::<ElevatorMotor>(DynamicObjectType::ElevatorMotor).unwrap();
        check_round_trip::<ElevatorShaft>(DynamicObjectType::ElevatorShaft).unwrap();
        check_round_trip::<GemTree>(DynamicObjectType::GemTree).unwrap();
        check_round_trip::<VinePlant>(DynamicObjectType::VinePlant).unwrap();
        check_round_trip::<TulipPlant>(DynamicObjectType::TulipPlant).unwrap();
        // check_round_trip::<OwnershipSign>(DynamicObjectType::OwnershipSign).unwrap();
        check_round_trip::<WheatPlant>(DynamicObjectType::WheatPlant).unwrap();
        check_round_trip::<TomatoPlant>(DynamicObjectType::TomatoPlant).unwrap();
        check_round_trip::<Yak>(DynamicObjectType::Yak).unwrap();
        // check_round_trip::<Mirror>(DynamicObjectType::Mirror).unwrap();
    }
}
