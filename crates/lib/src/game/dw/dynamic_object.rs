use crate::{BhError, BhResult};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::ops::{Deref, DerefMut};
use typed_floats::NonNaNFinite;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
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
    Item = 14,
    Fire = 16,
    Torch = 17,
    GlowBlock = 18,
    Ladder = 19,
    Door = 20,
    ArtificialLight = 21,
    Bed = 23,
    Dropbear = 25,
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
    Workbench = 45,
    Chest = 46,
    Sign = 47,
    TradingPost = 48,
    TradePortal = 50,
    Scorpion = 51,
    Column = 53,
    Stairs = 54,
    ElevatorMotor = 55,
    ElevatorShaft = 56,
    GemTree = 57,
    VinePlant = 58,
    TulipPlant = 59,
    WheatPlant = 61,
    TomatoPlant = 62,
    Yak = 63,
}

impl DynamicObjectType {
    pub fn try_from_str(s: &str) -> BhResult<Self> {
        let value: u16 = s
            .parse()
            .map_err(|_| BhError::ParseError(format!("Dynamic object type {} is invalid", s)))?;
        Self::try_from(value).map_err(|e| BhError::InvalidDynamicOjectId(e.number))
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(transparent)]
pub struct UniqueID(u64);
inherit!(UniqueID -> u64);

impl UniqueID {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn inner(&self) -> &u64 {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicObject {
    #[serde(rename = "floatPos")]
    pub float_pos: [NonNaNFinite<f32>; 2],
    pub pos_x: u64,
    pub pos_y: u16,
    #[serde(rename = "uniqueID")]
    pub unique_id: UniqueID,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtificialLight {
    #[serde(flatten)]
    pub obj: DynamicObject,
    pub max_red: u32,
    pub max_green: u32,
    pub max_blue: u32,
    pub max_heat: u32,
    pub radius: u32,
    #[serde(rename = "contributionGridOrigin.x")]
    pub contribution_grid_origin_x: i32,
    #[serde(rename = "contributionGridOrigin.y")]
    pub contribution_grid_origin_y: i32,
    pub light_direction: LightDirection,
}
inherit!(ArtificialLight -> DynamicObject, obj);

#[derive(Debug, Clone, Copy, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum LightDirection {
    All = 0,
    Down = 1,
    Up = 2,
}

// NOTE: final_goal_square_x/y, load_requires_recalculation are optional and needs serde(default)
// which doesn't work together with serde(flatten), which is needed for DynamicObject.
// Either manually flatten DynamicObject, or remove these fields. For now we go latter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Blockhead {
    #[serde(flatten)]
    pub obj: DynamicObject,
    pub actions: plist::Value,
    pub clothing_increment_timer: u64,
    pub double_time_unlocked: bool,
    pub interaction_item_index: i64, // could be -1... my god
    pub interaction_item_sub_index: i64,
    pub name: String,
    pub selected_tool_index: u64,
    pub skin_options: plist::Data,
    pub state: plist::Data,
}

pub mod plant;
pub mod tree;

#[cfg(test)]
mod tests {
    use super::{
        DynamicObjectList, DynamicObjectType,
        plant::{
            ChilliPlant, CornPlant, FlaxPlant, KelpPlant, SunflowerPlant, TomatoPlant, TulipPlant,
            VinePlant, WheatPlant,
        },
        tree::{
            AppleTree, CactusTree, CherryTree, CoconutTree, CoffeeTree, GemTree, LimeTree,
            MangoTree, MapleTree, OrangeTree, PineTree,
        },
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
    fn test_apple_tree_round_trip() {
        check_round_trip::<AppleTree>(DynamicObjectType::AppleTree).unwrap();
    }

    #[test]
    fn test_maple_tree_round_trip() {
        check_round_trip::<MapleTree>(DynamicObjectType::MapleTree).unwrap();
    }

    #[test]
    fn test_mango_tree_round_trip() {
        check_round_trip::<MangoTree>(DynamicObjectType::MangoTree).unwrap();
    }

    #[test]
    fn test_pine_tree_round_trip() {
        check_round_trip::<PineTree>(DynamicObjectType::PineTree).unwrap();
    }

    #[test]
    fn test_cactus_tree_round_trip() {
        check_round_trip::<CactusTree>(DynamicObjectType::CactusTree).unwrap();
    }

    #[test]
    fn test_coconut_tree_round_trip() {
        check_round_trip::<CoconutTree>(DynamicObjectType::CoconutTree).unwrap();
    }

    #[test]
    fn test_orange_tree_round_trip() {
        check_round_trip::<OrangeTree>(DynamicObjectType::OrangeTree).unwrap();
    }

    #[test]
    fn test_cherry_tree_round_trip() {
        check_round_trip::<CherryTree>(DynamicObjectType::CherryTree).unwrap();
    }

    #[test]
    fn test_coffee_tree_round_trip() {
        check_round_trip::<CoffeeTree>(DynamicObjectType::CoffeeTree).unwrap();
    }

    #[test]
    fn test_flax_round_trip() {
        check_round_trip::<FlaxPlant>(DynamicObjectType::FlaxPlant).unwrap();
    }

    #[test]
    fn test_sunflower_round_trip() {
        check_round_trip::<SunflowerPlant>(DynamicObjectType::SunflowerPlant).unwrap();
    }

    #[test]
    fn test_corn_round_trip() {
        check_round_trip::<CornPlant>(DynamicObjectType::CornPlant).unwrap();
    }

    #[test]
    fn test_chilli_round_trip() {
        check_round_trip::<ChilliPlant>(DynamicObjectType::ChilliPlant).unwrap();
    }

    #[test]
    fn test_kelp_round_trip() {
        check_round_trip::<KelpPlant>(DynamicObjectType::KelpPlant).unwrap();
    }

    #[test]
    fn test_lime_tree_round_trip() {
        check_round_trip::<LimeTree>(DynamicObjectType::LimeTree).unwrap();
    }

    #[test]
    fn test_gem_tree_round_trip() {
        check_round_trip::<GemTree>(DynamicObjectType::GemTree).unwrap();
    }

    #[test]
    fn test_vine_round_trip() {
        check_round_trip::<VinePlant>(DynamicObjectType::VinePlant).unwrap();
    }

    #[test]
    fn test_tulip_round_trip() {
        check_round_trip::<TulipPlant>(DynamicObjectType::TulipPlant).unwrap();
    }

    #[test]
    fn test_wheat_round_trip() {
        check_round_trip::<WheatPlant>(DynamicObjectType::WheatPlant).unwrap();
    }

    #[test]
    fn test_tomato_round_trip() {
        check_round_trip::<TomatoPlant>(DynamicObjectType::TomatoPlant).unwrap();
    }
}
