use super::DynamicObject;
use crate::util::serde::{deserialize_some, serialize_some};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum TreeType {
    Nothing = 0,
    Apple = 1,
    Mango = 2,
    Maple = 3,
    Pine = 4,
    Cactus = 5,
    Coconut = 6,
    Orange = 7,
    Cherry = 8,
    Coffee = 9,
    Lime = 10,
    Amethyst = 11,
    Sapphire = 12,
    Emerald = 13,
    Ruby = 14,
    Diamond = 15,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeFruit {
    pub has_created_free_block_this_season: bool,
    #[serde(rename = "pos.x")]
    pub pos_x: i32,
    #[serde(rename = "pos.y")]
    pub pos_y: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tree {
    #[serde(flatten)]
    obj: DynamicObject,
    pub age: f32,
    pub dead: bool,
    pub time_died: f32,
    pub remove_check_count: f32,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub growth_counter: Option<f32>,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub growth_rate: Option<f32>,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub growth_rate_gene: Option<u32>,
    pub height: u32,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_age: Option<f32>,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_height: Option<u32>,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_height_gene: Option<u32>,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_height_reached: Option<u32>,
    pub save_time: f32,
    pub tree_season_offset: i32,
    #[serde(rename = "treeFruit")]
    pub tree_fruits: Vec<TreeFruit>,
}
inherit!(Tree -> DynamicObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppleTree {
    #[serde(flatten)]
    tree: Tree,
    pub available_food: f32,
}
inherit!(AppleTree -> Tree, tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapleTree(Tree);
inherit!(MapleTree -> Tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MangoTree(Tree);
inherit!(MangoTree -> Tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PineTree {
    #[serde(flatten)]
    tree: Tree,
    pub available_food: f32,
}
inherit!(PineTree -> Tree, tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CactusTree {
    #[serde(flatten)]
    tree: Tree,
    pub available_food: f32,
    pub split_direction: bool,
    pub split_height_a: u32,
    pub split_height_b: u32,
}
inherit!(CactusTree -> Tree, tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoconutTree(Tree);
inherit!(CoconutTree -> Tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrangeTree(Tree);
inherit!(OrangeTree -> Tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CherryTree(Tree);
inherit!(CherryTree -> Tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoffeeTree(Tree);
inherit!(CoffeeTree -> Tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LimeTree(Tree);
inherit!(LimeTree -> Tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GemTree {
    #[serde(flatten)]
    tree: Tree,
    pub gem_tree_type: TreeType,
    pub fruit_year: i32,
}
inherit!(GemTree -> Tree, tree);
