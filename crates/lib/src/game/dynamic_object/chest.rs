use super::{super::item::Slot, InteractionObject};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
