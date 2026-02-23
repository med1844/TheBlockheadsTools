use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::ops::{Deref, DerefMut};
use typed_floats::NonNaNFinite;

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
