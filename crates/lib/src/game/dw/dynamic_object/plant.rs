use super::{ArtificialLight, DynamicObject};
use crate::util::serde::{deserialize_some, serialize_some};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plant {
    #[serde(flatten)]
    obj: DynamicObject,
    pub save_time: f64,
    pub season_offset: i32,
    pub gather_progress: i32,
    pub has_flowered_this_season: bool,
    pub flowering: bool,
    pub frozen: bool,
    pub age: f32,
    pub max_age: f32,
    pub max_age_gene: u16,
    pub growth_rate: f32,
    pub growth_rate_gene: u16,
}
inherit!(Plant -> DynamicObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalPlant {
    #[serde(flatten)]
    plant: Plant,
    pub available_food: f32,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub light_dict: Option<ArtificialLight>,
}
inherit!(NormalPlant -> Plant, plant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CornPlant(NormalPlant);
inherit!(CornPlant -> NormalPlant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarrotPlant(NormalPlant);
inherit!(CarrotPlant -> NormalPlant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TomatoPlant(NormalPlant);
inherit!(TomatoPlant -> NormalPlant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WheatPlant(NormalPlant);
inherit!(WheatPlant -> NormalPlant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChilliPlant(NormalPlant);
inherit!(ChilliPlant -> NormalPlant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SunflowerPlant(NormalPlant);
inherit!(SunflowerPlant -> NormalPlant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlaxPlant(NormalPlant);
inherit!(FlaxPlant -> NormalPlant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KelpPlant {
    #[serde(flatten)]
    normal_plant: NormalPlant,
    pub growth_timer: f32,
    pub number_of_occupied_tiles_above: i32,
}
inherit!(KelpPlant -> NormalPlant, normal_plant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TulipPlant {
    #[serde(flatten)]
    normal_plant: NormalPlant,
    pub color_genes: u16,
    pub mate_color_genes: u16,
    pub mix_genes: u16,
}
inherit!(TulipPlant -> NormalPlant, normal_plant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VinePlant {
    #[serde(flatten)]
    normal_plant: NormalPlant,
    pub growth_timer: f32,
    pub number_of_occupied_tiles_below: i32,
}
inherit!(VinePlant -> NormalPlant, normal_plant);
