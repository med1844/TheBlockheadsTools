use super::{ArtificialLight, InteractionObject};
use crate::util::serde::{deserialize_some, serialize_some};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::ops::{Deref, DerefMut};
use strum_macros::IntoStaticStr;
use typed_floats::NonNaNFinite;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize_repr,
    Deserialize_repr,
    IntoStaticStr,
    Default,
    TryFromPrimitive,
)]
#[repr(u8)]
pub enum WorkbenchType {
    #[default]
    Undefined = 0,
    BasicPortal = 1,
    Workbench = 2,
    Campfire = 3,
    Weave = 4,
    Wood = 5,
    Tool = 6,
    Press = 7,
    Kiln = 8,
    Furnace = 9,
    Craft = 10,
    Mix = 11,
    Dye = 12,
    PlacedPortal = 13,
    Metalwork = 14,
    SteamGenerator = 15,
    ElectricKiln = 16,
    ElectricFurnace = 17,
    ElectricMetalworkBench = 18,
    ElectricStove = 19,
    SolarPanel = 20,
    Flywheel = 21,
    ArmorBench = 22,
    TrainYard = 23,
    Easel = 24,
    Build = 25,
    Refinery = 26,
    ElectricPress = 27,
    CompostBin = 28,
    Sluice = 29,
    EggExtractor = 30,
    PizzaOven = 31,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workbench {
    #[serde(flatten)]
    obj: InteractionObject,
    pub available_electricity: u64,
    pub craft_progress_count: NonNaNFinite<f32>,
    pub fire_spread_timer: NonNaNFinite<f32>,
    pub fuel_fraction: NonNaNFinite<f32>,
    pub has_fuel: bool,
    pub hurry_cost: u64,
    pub hurry_seconds: NonNaNFinite<f32>,
    pub hurry_timer: NonNaNFinite<f32>,
    pub hurrying: bool,
    pub last_world_time: NonNaNFinite<f32>,
    pub level: u8,
    pub save_time: NonNaNFinite<f32>,
    pub selected_index: u8,
    pub workbench_type: WorkbenchType,
    pub x_scroll: NonNaNFinite<f32>,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub light_dict: Option<ArtificialLight>,
}
inherit!(Workbench -> InteractionObject, obj);

impl Workbench {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        obj: InteractionObject,
        available_electricity: u64,
        craft_progress_count: NonNaNFinite<f32>,
        fire_spread_timer: NonNaNFinite<f32>,
        fuel_fraction: NonNaNFinite<f32>,
        has_fuel: bool,
        hurry_cost: u64,
        hurry_seconds: NonNaNFinite<f32>,
        hurry_timer: NonNaNFinite<f32>,
        hurrying: bool,
        last_world_time: NonNaNFinite<f32>,
        level: u8,
        save_time: NonNaNFinite<f32>,
        selected_index: u8,
        workbench_type: WorkbenchType,
        x_scroll: NonNaNFinite<f32>,
        light_dict: Option<ArtificialLight>,
    ) -> Self {
        Self {
            obj,
            available_electricity,
            craft_progress_count,
            fire_spread_timer,
            fuel_fraction,
            has_fuel,
            hurry_cost,
            hurry_seconds,
            hurry_timer,
            hurrying,
            last_world_time,
            level,
            save_time,
            selected_index,
            workbench_type,
            x_scroll,
            light_dict,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            super::item::{Extra, Item, ItemType},
            DynamicObject, InteractionObject, InteractionObjectType, UniqueID,
        },
        Workbench, WorkbenchType,
    };

    #[test]
    fn test_extra_workbench_isolation() {
        let wb_data = Workbench {
            obj: InteractionObject {
                obj: DynamicObject {
                    float_pos: [0.0f32.try_into().unwrap(), 0.0f32.try_into().unwrap()],
                    pos_x: 5,
                    pos_y: 5,
                    unique_id: UniqueID::new(456),
                    owner_id: Some("wb_owner".to_string()),
                },
                interaction_object_type: InteractionObjectType::Workbench,
                is_in_use: false,
                flipped: false,
                paint_color: 0,
            },
            available_electricity: 0,
            craft_progress_count: 0.0f32.try_into().unwrap(),
            fire_spread_timer: 0.0f32.try_into().unwrap(),
            fuel_fraction: 0.0f32.try_into().unwrap(),
            has_fuel: false,
            hurry_cost: 0,
            hurry_seconds: 0.0f32.try_into().unwrap(),
            hurry_timer: 0.0f32.try_into().unwrap(),
            hurrying: false,
            last_world_time: 0.0f32.try_into().unwrap(),
            level: 1,
            save_time: 100.0f32.try_into().unwrap(),
            selected_index: 0,
            workbench_type: WorkbenchType::Workbench,
            x_scroll: 0.0f32.try_into().unwrap(),
            light_dict: None,
        };

        let item = Item {
            type_id: ItemType::WorkBench as u16,
            data_a: 0,
            data_b: 0,
            selected_sub_item_index: 0,
            padding: 0,
            extra: Some(Extra::Workbench(Box::new(wb_data))),
        };

        let serialized = plist::to_value(&item).unwrap();
        let deserialized: Item = plist::from_value(&serialized).unwrap();
        assert_eq!(item, deserialized);
    }
}
