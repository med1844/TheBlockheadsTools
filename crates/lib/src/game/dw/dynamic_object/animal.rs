use super::{ArtificialLight, DynamicObject};
use crate::util::serde::{deserialize_some, serialize_some};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::ops::{Deref, DerefMut};

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Animal {
    #[serde(flatten)]
    obj: DynamicObject,
    pub save_time: f64,
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
}
inherit!(Animal -> DynamicObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dodo(Animal);
inherit!(Dodo -> Animal);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Donkey(Animal);
inherit!(Donkey -> Animal);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClownFish(Animal);
inherit!(ClownFish -> Animal);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shark(Animal);
inherit!(Shark -> Animal);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Yak {
    #[serde(flatten)]
    animal: Animal,
    pub hair: f32,
    pub milk: f32,
}
inherit!(Yak -> Animal, animal);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
