use super::{
    super::super::item::ItemType, DynamicObject, InteractionObject, deserialize_some,
    serialize_some,
};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ladder {
    #[serde(flatten)]
    obj: DynamicObject,
    pub paint_color: u16,
    pub item_type: ItemType,
}
inherit!(Ladder -> DynamicObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Door {
    #[serde(flatten)]
    obj: DynamicObject,
    pub item_type: ItemType,
    pub blocked: u8,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none",
        rename = "ironPlaceClientID"
    )]
    pub iron_place_client_id: Option<String>,
}
inherit!(Door -> DynamicObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bed {
    #[serde(flatten)]
    obj: InteractionObject,
    pub item_type: ItemType,
    pub bedding_color: u16,
    pub save_time: f64,
}
inherit!(Bed -> InteractionObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Window {
    #[serde(flatten)]
    obj: DynamicObject,
    pub item_type: ItemType,
}
inherit!(Window -> DynamicObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Boat(DynamicObject);
inherit!(Boat -> DynamicObject);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum WireConfiguration {
    Undefined = 0,
    AllConnections = 1,
    NoConnections = 2,
    AboveBelowOnly = 3,
    AboveBelowLeft = 4,
    AboveBelowRight = 5,
    LeftRightOnly = 6,
    LeftRightUp = 7,
    LeftRightDown = 8,
    LeftDown = 9,
    LeftUp = 10,
    RightDown = 11,
    RightUp = 12,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum WireSolidConfiguration {
    Undefined = 0,
    NotSolid = 1,
    AllConnections = 2,
    ThisTileOnly = 3,
    AboveBelowOnly = 4,
    AboveBelowLeft = 5,
    AboveBelowRight = 6,
    LeftRightOnly = 7,
    LeftOnly = 8,
    LeftRightUp = 9,
    LeftRightDown = 10,
    LeftDown = 11,
    LeftUp = 12,
    RightDown = 13,
    RightUp = 14,
    RightOnly = 15,
    UpOnly = 16,
    DownOnly = 17,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wire {
    #[serde(flatten)]
    obj: DynamicObject,
    pub item_type: ItemType,
    pub configuration: WireConfiguration,
    pub solid_configuration: WireSolidConfiguration,
}
inherit!(Wire -> DynamicObject, obj);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum RailConfiguration {
    Undefined = 0,
    Flat = 1,
    DiagonalUpLeft = 2,
    DiagonalHalfUpLeftBot = 3,
    DiagonalHalfUpLeftTop = 4,
    DiagonalUpRight = 5,
    DiagonalHalfUpRightBot = 6,
    DiagonalHalfUpRightTop = 7,
    DiagonalHalfFlat = 8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rail {
    #[serde(flatten)]
    obj: DynamicObject,
    pub item_type: ItemType,
    pub configuration: RailConfiguration,
    pub owned_by_station: bool,
}
inherit!(Rail -> DynamicObject, obj);

