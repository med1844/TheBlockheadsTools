use super::super::item::{Item, ItemXml};
use super::DynamicObject;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

/// Canonical in-memory form of a dropped item
#[derive(Debug, Clone, PartialEq)]
pub struct DroppedItem {
    obj: DynamicObject,
    pub item: Item,
    pub bounce_timer: f64,
    pub creation_time: f64,
    pub fall_speed: f64,
    pub hovers: bool,
    pub float_pos_vx: f64,
    pub float_pos_vy: f64,
}
inherit!(DroppedItem -> DynamicObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DroppedItemXml {
    #[serde(flatten)]
    obj: DynamicObject,
    #[serde(flatten)]
    item: ItemXml,

    bounce_timer: f64,
    creation_time: f64,
    fall_speed: f64,
    hovers: bool,

    #[serde(rename = "floatPos[VX]")]
    float_pos_vx: f64,
    #[serde(rename = "floatPos[VY]")]
    float_pos_vy: f64,
}

impl Serialize for DroppedItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        DroppedItemXml {
            obj: self.obj.clone(),
            item: self.item.to_xml().map_err(serde::ser::Error::custom)?,
            bounce_timer: self.bounce_timer,
            creation_time: self.creation_time,
            fall_speed: self.fall_speed,
            hovers: self.hovers,
            float_pos_vx: self.float_pos_vx,
            float_pos_vy: self.float_pos_vy,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DroppedItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let xml = DroppedItemXml::deserialize(deserializer)?;
        Ok(Self {
            obj: xml.obj,
            item: Item::from_xml(xml.item).map_err(serde::de::Error::custom)?,
            bounce_timer: xml.bounce_timer,
            creation_time: xml.creation_time,
            fall_speed: xml.fall_speed,
            hovers: xml.hovers,
            float_pos_vx: xml.float_pos_vx,
            float_pos_vy: xml.float_pos_vy,
        })
    }
}
