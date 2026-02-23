use super::DynamicObject;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

// Corresponds to Plant
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plant {
    #[serde(flatten)]
    pub obj: DynamicObject,
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
    pub plant: Plant,
    pub available_food: f32,
    // #[serde(default)]
    // pub light_dict: Option<ArtificialLight>,
}
inherit!(NormalPlant -> Plant, plant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CornPlant(pub NormalPlant);
inherit!(CornPlant -> NormalPlant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarrotPlant(pub NormalPlant);
inherit!(CarrotPlant -> NormalPlant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TomatoPlant(pub NormalPlant);
inherit!(TomatoPlant -> NormalPlant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WheatPlant(pub NormalPlant);
inherit!(WheatPlant -> NormalPlant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChilliPlant(pub NormalPlant);
inherit!(ChilliPlant -> NormalPlant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SunflowerPlant(pub NormalPlant);
inherit!(SunflowerPlant -> NormalPlant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlaxPlant(pub NormalPlant);
inherit!(FlaxPlant -> NormalPlant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KelpPlant {
    #[serde(flatten)]
    pub normal_plant: NormalPlant,
    pub growth_timer: f32,
    pub number_of_occupied_tiles_above: i32,
}
inherit!(KelpPlant -> NormalPlant, normal_plant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TulipPlant {
    #[serde(flatten)]
    pub normal_plant: NormalPlant,
    pub color_genes: u16,
    pub mate_color_genes: u16,
    pub mix_genes: u16,
}
inherit!(TulipPlant -> NormalPlant, normal_plant);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VinePlant {
    #[serde(flatten)]
    pub normal_plant: NormalPlant,
    pub growth_timer: f32,
    pub number_of_occupied_tiles_below: i32,
}
inherit!(VinePlant -> NormalPlant, normal_plant);

#[cfg(test)]
mod tests {
    use super::{
        super::{super::dynamic_world::DynamicObjectType, DynamicObjectList},
        ChilliPlant, FlaxPlant, KelpPlant, SunflowerPlant, TomatoPlant, TulipPlant, VinePlant,
        WheatPlant,
    };

    #[test]
    fn test_tomato_round_trip() {
        let test_xml_tomato_plant = std::fs::read(format!(
            "resources/type_{}.xml",
            DynamicObjectType::TomatoPlant as u16
        ))
        .unwrap();
        let tomatos =
            plist::from_bytes::<DynamicObjectList<TomatoPlant>>(&test_xml_tomato_plant).unwrap();
        let mut serialized = Vec::with_capacity(test_xml_tomato_plant.len());
        plist::to_writer_xml(&mut serialized, &tomatos).unwrap();
        let tomatos_round_triped =
            plist::from_bytes::<DynamicObjectList<TomatoPlant>>(&serialized).unwrap();
        assert_eq!(tomatos, tomatos_round_triped);
    }

    #[test]
    fn test_kelp_round_trip() {
        let test_xml_kelp_plant = std::fs::read(format!(
            "resources/type_{}.xml",
            DynamicObjectType::KelpPlant as u16
        ))
        .unwrap();
        let kelps =
            plist::from_bytes::<DynamicObjectList<KelpPlant>>(&test_xml_kelp_plant).unwrap();
        let mut serialized = Vec::with_capacity(test_xml_kelp_plant.len());
        plist::to_writer_xml(&mut serialized, &kelps).unwrap();
        let kelps_round_triped =
            plist::from_bytes::<DynamicObjectList<KelpPlant>>(&serialized).unwrap();
        assert_eq!(kelps, kelps_round_triped);
    }

    #[test]
    fn test_tulip_round_trip() {
        let test_xml_tulip_plant = std::fs::read(format!(
            "resources/type_{}.xml",
            DynamicObjectType::TulipPlant as u16
        ))
        .unwrap();
        let tulips =
            plist::from_bytes::<DynamicObjectList<TulipPlant>>(&test_xml_tulip_plant).unwrap();
        let mut serialized = Vec::with_capacity(test_xml_tulip_plant.len());
        plist::to_writer_xml(&mut serialized, &tulips).unwrap();
        let tulips_round_triped =
            plist::from_bytes::<DynamicObjectList<TulipPlant>>(&serialized).unwrap();
        assert_eq!(tulips, tulips_round_triped);
    }

    #[test]
    fn test_vine_round_trip() {
        let test_xml_vine_plant = std::fs::read(format!(
            "resources/type_{}.xml",
            DynamicObjectType::VinePlant as u16
        ))
        .unwrap();
        let vines =
            plist::from_bytes::<DynamicObjectList<VinePlant>>(&test_xml_vine_plant).unwrap();
        let mut serialized = Vec::with_capacity(test_xml_vine_plant.len());
        plist::to_writer_xml(&mut serialized, &vines).unwrap();
        let vines_round_triped =
            plist::from_bytes::<DynamicObjectList<VinePlant>>(&serialized).unwrap();
        assert_eq!(vines, vines_round_triped);
    }

    #[test]
    fn test_wheat_round_trip() {
        let test_xml_wheat_plant = std::fs::read(format!(
            "resources/type_{}.xml",
            DynamicObjectType::WheatPlant as u16
        ))
        .unwrap();
        let wheats =
            plist::from_bytes::<DynamicObjectList<WheatPlant>>(&test_xml_wheat_plant).unwrap();
        let mut serialized = Vec::with_capacity(test_xml_wheat_plant.len());
        plist::to_writer_xml(&mut serialized, &wheats).unwrap();
        let wheats_round_triped =
            plist::from_bytes::<DynamicObjectList<WheatPlant>>(&serialized).unwrap();
        assert_eq!(wheats, wheats_round_triped);
    }

    #[test]
    fn test_chilli_round_trip() {
        let test_xml_chilli_plant = std::fs::read(format!(
            "resources/type_{}.xml",
            DynamicObjectType::ChilliPlant as u16
        ))
        .unwrap();
        let chillis =
            plist::from_bytes::<DynamicObjectList<ChilliPlant>>(&test_xml_chilli_plant).unwrap();
        let mut serialized = Vec::with_capacity(test_xml_chilli_plant.len());
        plist::to_writer_xml(&mut serialized, &chillis).unwrap();
        let chillis_round_triped =
            plist::from_bytes::<DynamicObjectList<ChilliPlant>>(&serialized).unwrap();
        assert_eq!(chillis, chillis_round_triped);
    }

    #[test]
    fn test_sunflower_round_trip() {
        let test_xml_sunflower_plant = std::fs::read(format!(
            "resources/type_{}.xml",
            DynamicObjectType::SunflowerPlant as u16
        ))
        .unwrap();
        let sunflowers =
            plist::from_bytes::<DynamicObjectList<SunflowerPlant>>(&test_xml_sunflower_plant)
                .unwrap();
        let mut serialized = Vec::with_capacity(test_xml_sunflower_plant.len());
        plist::to_writer_xml(&mut serialized, &sunflowers).unwrap();
        let sunflowers_round_triped =
            plist::from_bytes::<DynamicObjectList<SunflowerPlant>>(&serialized).unwrap();
        assert_eq!(sunflowers, sunflowers_round_triped);
    }

    #[test]
    fn test_flax_round_trip() {
        let test_xml_flax_plant = std::fs::read(format!(
            "resources/type_{}.xml",
            DynamicObjectType::FlaxPlant as u16
        ))
        .unwrap();
        let flaxs =
            plist::from_bytes::<DynamicObjectList<FlaxPlant>>(&test_xml_flax_plant).unwrap();
        let mut serialized = Vec::with_capacity(test_xml_flax_plant.len());
        plist::to_writer_xml(&mut serialized, &flaxs).unwrap();
        let flaxs_round_triped =
            plist::from_bytes::<DynamicObjectList<FlaxPlant>>(&serialized).unwrap();
        assert_eq!(flaxs, flaxs_round_triped);
    }
}
