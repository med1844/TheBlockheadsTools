use super::{DynamicObject, deserialize_some, serialize_some};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum AnimalType {
    Nothing = 0,
    Dodo = 1,
    DropBear = 2,
    Donkey = 3,
    ClownFish = 4,
    Shark = 5,
    CaveTroll = 6,
    Scorpion = 7,
    Yak = 8,
}

// In blockheads source code this is called NPC
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Animal {
    #[serde(flatten)]
    obj: DynamicObject,
    pub save_time: f32,
    pub age: f32,
    pub breed: u16,
    pub damage: u16,
    pub fullness: f32,
    pub has_been_fed_by_blockhead_or_chest: bool,
    pub has_bred: bool,
    pub lay_cooldown_timer: f32,
    pub lay_timer: f32,
    pub mate_breed: u16,
    pub mate_cooldown_timer: f32,
    pub tame_cooldown_timer: f32,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none",
        rename = "tameCountsByClientID"
    )]
    pub tame_counts_by_client_id: Option<HashMap<String, i32>>, // may be negative if you hit too many times
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none",
        rename = "tamedClientID"
    )]
    pub tamed_client_id: Option<String>,
}
inherit!(Animal -> DynamicObject, obj);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Dodo(Animal);
inherit!(Dodo -> Animal);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Donkey(Animal);
inherit!(Donkey -> Animal);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ClownFish(Animal);
inherit!(ClownFish -> Animal);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Shark(Animal);
inherit!(Shark -> Animal);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Scorpion(Animal);
inherit!(Scorpion -> Animal);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaveTroll {
    #[serde(flatten)]
    animal: Animal,
    pub dead: bool,
    #[serde(rename = "defendSquare.x")]
    pub defend_square_x: u32,
    #[serde(rename = "defendSquare.y")]
    pub defend_square_y: u32,
    pub state: plist::Data,
}
inherit!(CaveTroll -> Animal, animal);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Yak {
    #[serde(flatten)]
    animal: Animal,
    pub hair: f32,
    pub milk: f32,
}
inherit!(Yak -> Animal, animal);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropBear {
    #[serde(flatten)]
    animal: Animal,
    pub courage_meter: f32,
    #[serde(rename = "dropPos.x")]
    pub drop_pos_x: bool,
    #[serde(rename = "dropPos.y")]
    pub drop_pos_y: bool,
    pub drop_speed: f32,
    pub dropping: bool,
    pub goal_tree_direction: bool,
    pub on_ground: bool,
    pub provoke_meter: f32,
}
inherit!(DropBear -> Animal, animal);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum DodoBreed {
    #[default]
    Standard = 0,
    Stone = 1,
    Limestone = 2,
    Sandstone = 3,
    Marble = 4,
    RedMarble = 5,
    Lapis = 6,
    Dirt = 7,
    Compost = 8,
    Wood = 9,
    Gravel = 10,
    Sand = 11,
    BlackSand = 12,
    Glass = 13,
    BlackGlass = 14,
    Clay = 15,
    RedBrick = 16,
    Flint = 17,
    Coal = 18,
    Oil = 19,
    Fuel = 20,
    Copper = 21,
    Tin = 22,
    Iron = 23,
    Gold = 24,
    Titanium = 25,
    Platinum = 26,
    Amethyst = 27,
    Sapphire = 28,
    Emerald = 29,
    Ruby = 30,
    Diamond = 31,
    Rainbow = 32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DodoGenes {
    pub breed: DodoBreed,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Egg {
    #[serde(flatten)]
    obj: DynamicObject,
    pub genes_dict: DodoGenes,
    pub hatch_timer: f32,
    pub save_time: f32,
}
inherit!(Egg -> DynamicObject, obj);
