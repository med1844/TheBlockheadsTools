use super::{DynamicObject, InteractionObject, UniqueID, deserialize_some, serialize_some};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainCar {
    #[serde(flatten)]
    obj: DynamicObject,
    pub engine_is_right: bool,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none",
        rename = "leftCarID"
    )]
    pub left_car_id: Option<UniqueID>,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none",
        rename = "rightCarID"
    )]
    pub right_car_id: Option<UniqueID>,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none",
        rename = "engineCarID"
    )]
    pub engine_car_id: Option<UniqueID>,
}
inherit!(TrainCar -> DynamicObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HandCar(TrainCar);
inherit!(HandCar -> TrainCar);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamLocomotive {
    #[serde(flatten)]
    obj: TrainCar,
    pub fuel_fraction: f32,
    pub going_right: bool,
    pub has_fuel: bool,
    pub stopped: bool,
}
inherit!(SteamLocomotive -> TrainCar, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FreightCar(TrainCar);
inherit!(FreightCar -> TrainCar);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassengerCar(TrainCar);
inherit!(PassengerCar -> TrainCar);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainStation {
    #[serde(flatten)]
    obj: InteractionObject,
    pub text: String,
    pub save_time: f64,
}
inherit!(TrainStation -> InteractionObject, obj);
