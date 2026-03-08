use super::{
    super::item::{ItemType, Slot},
    InteractionObject,
};
use crate::util::serde::{deserialize_some, serialize_some};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::ops::{Deref, DerefMut};
use strum_macros::IntoStaticStr;

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
pub enum ChestType {
    #[default]
    Standard = 0,
    Safe = 1,
    Shelf = 2,
    Gold = 3,
    Portal = 4,
    DisplayCabinet = 5,
    Feeder = 6,
}

impl From<ChestType> for u8 {
    fn from(value: ChestType) -> Self {
        value as u8
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChestData {
    #[serde(flatten)]
    obj: InteractionObject,
    pub chest_type: ChestType,
    pub save_item_slots: [Slot; Self::NUM_SLOTS],
}

impl ChestData {
    pub const NUM_SLOTS: usize = 16;
}
inherit!(ChestData -> InteractionObject, obj);

impl ChestData {
    pub fn new(
        interaction_object: InteractionObject,
        chest_type: ChestType,
        save_item_slots: [Slot; Self::NUM_SLOTS],
    ) -> Self {
        Self {
            obj: interaction_object,
            chest_type,
            save_item_slots,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chest {
    obj: InteractionObject,
    pub chest_type: ChestType,
    pub shelf_render_items: Option<[ItemType; 4]>,
    pub shelf_item_data_bs: Option<[u16; 4]>,
    pub save_time: f64,
}
inherit!(Chest -> InteractionObject, obj);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChestRaw {
    #[serde(flatten)]
    obj: InteractionObject,
    chest_type: ChestType,
    save_time: f64,

    #[serde(
        rename = "shelfRenderItems_0",
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    shelf_render_items_0: Option<ItemType>,
    #[serde(
        rename = "shelfRenderItems_1",
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    shelf_render_items_1: Option<ItemType>,
    #[serde(
        rename = "shelfRenderItems_2",
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    shelf_render_items_2: Option<ItemType>,
    #[serde(
        rename = "shelfRenderItems_3",
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    shelf_render_items_3: Option<ItemType>,

    #[serde(
        rename = "shelfItemDataBs_0",
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    shelf_item_data_bs_0: Option<u16>,
    #[serde(
        rename = "shelfItemDataBs_1",
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    shelf_item_data_bs_1: Option<u16>,
    #[serde(
        rename = "shelfItemDataBs_2",
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    shelf_item_data_bs_2: Option<u16>,
    #[serde(
        rename = "shelfItemDataBs_3",
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    shelf_item_data_bs_3: Option<u16>,
}

impl TryFrom<ChestRaw> for Chest {
    type Error = String;

    fn try_from(raw: ChestRaw) -> Result<Self, Self::Error> {
        let render_items = match (
            raw.shelf_render_items_0,
            raw.shelf_render_items_1,
            raw.shelf_render_items_2,
            raw.shelf_render_items_3,
        ) {
            (Some(r0), Some(r1), Some(r2), Some(r3)) => Some([r0, r1, r2, r3]),
            (None, None, None, None) => None,
            _ => return Err("Incomplete shelf_render_items array".to_string()),
        };

        let data_bs = match (
            raw.shelf_item_data_bs_0,
            raw.shelf_item_data_bs_1,
            raw.shelf_item_data_bs_2,
            raw.shelf_item_data_bs_3,
        ) {
            (Some(d0), Some(d1), Some(d2), Some(d3)) => Some([d0, d1, d2, d3]),
            (None, None, None, None) => None,
            _ => return Err("Incomplete shelf_item_data_bs array".to_string()),
        };

        if render_items.is_some() != data_bs.is_some() {
            return Err(
                "Mismatched presence of shelf_render_items and shelf_item_data_bs".to_string(),
            );
        }

        Ok(Self {
            obj: raw.obj,
            chest_type: raw.chest_type,
            save_time: raw.save_time,
            shelf_render_items: render_items,
            shelf_item_data_bs: data_bs,
        })
    }
}

impl From<Chest> for ChestRaw {
    fn from(chest: Chest) -> Self {
        let (r0, r1, r2, r3) = match chest.shelf_render_items {
            Some([a, b, c, d]) => (Some(a), Some(b), Some(c), Some(d)),
            None => (None, None, None, None),
        };
        let (d0, d1, d2, d3) = match chest.shelf_item_data_bs {
            Some([a, b, c, d]) => (Some(a), Some(b), Some(c), Some(d)),
            None => (None, None, None, None),
        };

        Self {
            obj: chest.obj,
            chest_type: chest.chest_type,
            save_time: chest.save_time,
            shelf_render_items_0: r0,
            shelf_render_items_1: r1,
            shelf_render_items_2: r2,
            shelf_render_items_3: r3,
            shelf_item_data_bs_0: d0,
            shelf_item_data_bs_1: d1,
            shelf_item_data_bs_2: d2,
            shelf_item_data_bs_3: d3,
        }
    }
}

impl Serialize for Chest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ChestRaw::from(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Chest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = ChestRaw::deserialize(deserializer)?;
        Chest::try_from(raw).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            super::item::{Extra, Item, ItemType},
            DynamicObject, InteractionObject, InteractionObjectType, UniqueID,
        },
        ChestData, ChestType, Slot,
    };

    #[test]
    fn test_extra_chest_isolation() {
        let chest_data = ChestData {
            obj: InteractionObject {
                obj: DynamicObject {
                    float_pos: [0.0f32.try_into().unwrap(), 0.0f32.try_into().unwrap()],
                    pos_x: 10,
                    pos_y: 20,
                    unique_id: UniqueID::new(123),
                    owner_id: Some("test_owner".to_string()),
                },
                interaction_object_type: InteractionObjectType::Chest,
                is_in_use: false,
                flipped: false,
                paint_color: 0,
            },
            chest_type: ChestType::Standard,
            save_item_slots: [const { Slot(vec![]) }; 16],
        };

        let item = Item {
            type_id: ItemType::Chest as u16,
            data_a: 0,
            data_b: 0,
            selected_sub_item_index: 0,
            padding: 0,
            extra: Some(Extra::Chest(Box::new(chest_data))),
        };

        let serialized = plist::to_value(&item).unwrap();
        let deserialized: Item = plist::from_value(&serialized).unwrap();
        assert_eq!(item, deserialized);
    }
}
