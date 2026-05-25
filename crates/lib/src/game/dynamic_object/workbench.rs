use super::{ArtificialLight, InteractionObject, InteractionObjectType};
use crate::util::serde::{deserialize_some, serialize_some};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::ops::{Deref, DerefMut};
use strum::IntoStaticStr;

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
    pub craft_progress_count: f32,
    pub fire_spread_timer: f32,
    pub fuel_fraction: f32,
    pub has_fuel: bool,
    pub hurry_cost: u64,
    pub hurry_seconds: f32,
    pub hurry_timer: f32,
    pub hurrying: bool,
    pub last_world_time: f32,
    pub level: u8,
    pub selected_index: u8,
    pub workbench_type: WorkbenchType,
    pub x_scroll: f32,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub light_dict: Option<ArtificialLight>,
}
inherit!(Workbench -> InteractionObject, obj);

impl Default for Workbench {
    fn default() -> Self {
        Self {
            obj: InteractionObject {
                interaction_object_type: InteractionObjectType::Workbench,
                ..Default::default()
            },
            available_electricity: 0,
            craft_progress_count: 0.0,
            fire_spread_timer: 0.0,
            fuel_fraction: 0.0,
            has_fuel: false,
            hurry_cost: 0,
            hurry_seconds: 0.0,
            hurry_timer: 0.0,
            hurrying: false,
            last_world_time: 0.0,
            level: 0,
            selected_index: 0,
            workbench_type: WorkbenchType::default(),
            x_scroll: 0.0,
            light_dict: Option::default(),
        }
    }
}

impl Workbench {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        obj: InteractionObject,
        available_electricity: u64,
        craft_progress_count: f32,
        fire_spread_timer: f32,
        fuel_fraction: f32,
        has_fuel: bool,
        hurry_cost: u64,
        hurry_seconds: f32,
        hurry_timer: f32,
        hurrying: bool,
        last_world_time: f32,
        level: u8,
        selected_index: u8,
        workbench_type: WorkbenchType,
        x_scroll: f32,
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
            selected_index,
            workbench_type,
            x_scroll,
            light_dict,
        }
    }
}
