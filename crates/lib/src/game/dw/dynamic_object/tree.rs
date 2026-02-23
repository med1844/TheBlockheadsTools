use super::DynamicObject;
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub obj: DynamicObject,
    pub age: f32,
    pub dead: bool,
    pub time_died: f32,
    pub remove_check_count: f32,
    #[serde(default)]
    pub growth_counter: f32,
    #[serde(default)]
    pub growth_rate: f32,
    #[serde(default)]
    pub growth_rate_gene: u32,
    pub height: u32,
    #[serde(default)]
    pub max_age: f32,
    #[serde(default)]
    pub max_height: u32,
    #[serde(default)]
    pub max_height_gene: u32,
    #[serde(default)]
    pub max_height_reached: u32,
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
    pub tree: Tree,
    pub available_food: f32,
}
inherit!(AppleTree -> Tree, tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapleTree(pub Tree);
inherit!(MapleTree -> Tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MangoTree(pub Tree);
inherit!(MangoTree -> Tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PineTree {
    #[serde(flatten)]
    pub tree: Tree,
    pub available_food: f32,
}
inherit!(PineTree -> Tree, tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CactusTree {
    #[serde(flatten)]
    pub tree: Tree,
    pub available_food: f32,
    pub split_direction: bool,
    pub split_height_a: u32,
    pub split_height_b: u32,
}
inherit!(CactusTree -> Tree, tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoconutTree(pub Tree);
inherit!(CoconutTree -> Tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrangeTree(pub Tree);
inherit!(OrangeTree -> Tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CherryTree(pub Tree);
inherit!(CherryTree -> Tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoffeeTree(pub Tree);
inherit!(CoffeeTree -> Tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LimeTree(pub Tree);
inherit!(LimeTree -> Tree);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GemTree {
    #[serde(flatten)]
    pub tree: Tree,
    pub gem_tree_type: TreeType,
    pub fruit_year: i32,
}
inherit!(GemTree -> Tree, tree);

#[cfg(test)]
mod tests {
    use super::{
        super::{super::dynamic_world::DynamicObjectType, DynamicObjectList},
        AppleTree, CactusTree, CherryTree, CoconutTree, CoffeeTree, GemTree, LimeTree, MangoTree,
        MapleTree, OrangeTree, PineTree,
    };
    use serde::Serialize;

    fn check_round_trip<T>(obj_type: DynamicObjectType) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let test_xml = std::fs::read(format!("resources/type_{}.xml", obj_type as u16))?;
        let trees = plist::from_bytes::<DynamicObjectList<T>>(&test_xml)?;
        let mut serialized = Vec::with_capacity(test_xml.len());
        plist::to_writer_xml(&mut serialized, &trees)?;
        let trees_round_trip = plist::from_bytes::<DynamicObjectList<T>>(&serialized)?;
        assert_eq!(trees, trees_round_trip);
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
    fn test_lime_tree_round_trip() {
        check_round_trip::<LimeTree>(DynamicObjectType::LimeTree).unwrap();
    }

    #[test]
    fn test_gem_tree_round_trip() {
        check_round_trip::<GemTree>(DynamicObjectType::GemTree).unwrap();
    }
}
