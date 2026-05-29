use super::{
    DynamicObject, InteractionObject, InteractionObjectType, UniqueID,
    chest::{Chest, ChestError, ChestSaveDictXml},
    deserialize_some, serialize_some,
};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HandCar(TrainCar);
inherit!(HandCar -> TrainCar);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
pub(crate) struct FreightCarDwXml(TrainCar);
inherit!(FreightCarDwXml -> TrainCar);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FreightCarSaveDictXml {
    #[serde(flatten)]
    train_car: TrainCar,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    chest: Option<ChestSaveDictXml>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FreightCar {
    train_car: TrainCar,
    pub chest: Option<Chest>,
}
inherit!(FreightCar -> TrainCar, train_car);

impl FreightCar {
    fn new(train_car: TrainCar, chest_xml: Option<ChestSaveDictXml>) -> Result<Self, ChestError> {
        let chest = chest_xml.map(Chest::from_chest_save_dict_xml).transpose()?;
        Ok(Self { train_car, chest })
    }

    pub(crate) fn from_save_dict_xml(
        save_dict_xml: FreightCarSaveDictXml,
    ) -> Result<Self, ChestError> {
        Self::new(save_dict_xml.train_car, save_dict_xml.chest)
    }

    pub(crate) fn from_xml_and_chest(
        xml: FreightCarDwXml,
        chest_xml: Option<ChestSaveDictXml>,
    ) -> Result<Self, ChestError> {
        Self::new(xml.0, chest_xml)
    }

    fn to_train_car_and_chest(&self) -> Result<(TrainCar, Option<ChestSaveDictXml>), ChestError> {
        Ok((
            self.train_car.clone(),
            self.chest
                .as_ref()
                .map(|chest| chest.to_chest_save_dict_xml())
                .transpose()?,
        ))
    }

    pub(crate) fn to_save_dict_xml(&self) -> Result<FreightCarSaveDictXml, ChestError> {
        let (train_car, chest) = self.to_train_car_and_chest()?;
        Ok(FreightCarSaveDictXml { train_car, chest })
    }

    pub(crate) fn to_xml_and_chest(
        &self,
    ) -> Result<(FreightCarDwXml, Option<ChestSaveDictXml>), ChestError> {
        let (train_car, chest) = self.to_train_car_and_chest()?;
        Ok((FreightCarDwXml(train_car), chest))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
