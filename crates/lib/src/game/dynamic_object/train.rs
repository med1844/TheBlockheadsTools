use super::{
    DynamicObject, InteractionObject, InteractionObjectType, UniqueID,
    chest::{Chest, ChestError, ChestSaveDictXml},
    deserialize_some, serialize_some,
};
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
    train_car: TrainCar,
    pub fuel_fraction: f32,
    pub going_right: bool,
    pub has_fuel: bool,
    pub stopped: bool,
}
inherit!(SteamLocomotive -> TrainCar, train_car);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FreightCarXml(TrainCar);
inherit!(FreightCarXml -> TrainCar);

#[derive(Debug, Clone, PartialEq)]
pub struct FreightCar {
    train_car: TrainCar,
    pub chest: Option<Chest>,
}
inherit!(FreightCar -> TrainCar, train_car);

impl FreightCar {
    pub(crate) fn from_xml_and_chest(
        xml: FreightCarXml,
        chest_xml: Option<ChestSaveDictXml>,
    ) -> Result<Self, ChestError> {
        let chest = chest_xml.map(Chest::from_chest_save_dict_xml).transpose()?;
        Ok(Self {
            train_car: xml.0,
            chest,
        })
    }

    pub(crate) fn to_xml_and_chest(
        &self,
    ) -> Result<(FreightCarXml, Option<ChestSaveDictXml>), ChestError> {
        Ok((
            FreightCarXml(self.train_car.clone()),
            self.chest
                .as_ref()
                .map(|chest| chest.to_chest_save_dict_xml())
                .transpose()?,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassengerCar(TrainCar);
inherit!(PassengerCar -> TrainCar);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainStation {
    #[serde(flatten)]
    obj: InteractionObject,
    pub text: String,
}
inherit!(TrainStation -> InteractionObject, obj);

impl Default for TrainStation {
    fn default() -> Self {
        Self {
            obj: InteractionObject {
                interaction_object_type: InteractionObjectType::TrainStation,
                ..Default::default()
            },
            text: String::default(),
        }
    }
}
