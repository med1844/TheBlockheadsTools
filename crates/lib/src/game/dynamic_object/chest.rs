use super::{
    super::{
        super::util::{
            gzip::{compress, decompress},
            plist::to_xml_plist,
            serde::{deserialize_some, serialize_some},
        },
        item::{ItemError, ItemType, Slot, Slots},
    },
    InteractionObject, InteractionObjectType,
};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use snafu::prelude::*;
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
pub enum ChestType {
    #[default]
    Standard = 0,
    Safe = 1,
    Shelf = 2,
    Gold = 3,
    Portal = 4,
    Cabinet = 5,
    Feeder = 6,
}

impl From<ChestType> for u8 {
    fn from(value: ChestType) -> Self {
        value as u8
    }
}

pub const NUM_STANDARD_SLOTS: usize = 4 * 4;
pub const NUM_SHELF_SLOTS: usize = 2 * 2;
type StandardSlots = Slots<NUM_STANDARD_SLOTS>;
type ShelfSlots = Slots<NUM_SHELF_SLOTS>;

#[derive(Debug, Snafu)]
pub enum ChestError {
    #[snafu(display("Get saveItemSlot when portal chest shouldn't have one"))]
    PortalChestHaveSlots,
    #[snafu(display("No saveItemSlot when chest type {chest_type:?} should have one"))]
    NoSaveItemSlot { chest_type: ChestType },
    #[snafu(display("Failed to compress chest slots"))]
    CompressSlots { source: std::io::Error },
    #[snafu(display("Failed to decompress chest slots"))]
    DecompressSlots { source: std::io::Error },
    #[snafu(display("Failed to deserialize chest slots"))]
    DeserializeSlots { source: plist::Error },
    #[snafu(display("Failed to serialize chest slots"))]
    SerializeSlots { source: plist::Error },
    #[snafu(display("Failed to load chest slots"))]
    LoadSlots { source: Box<ItemError> },
    #[snafu(display("Failed to save chest slots"))]
    SaveSlots { source: Box<ItemError> },
}

type Result<T> = std::result::Result<T, ChestError>;

#[derive(Debug, Clone, PartialEq)]
pub enum ChestSlots {
    Standard(StandardSlots),
    Safe(StandardSlots),
    Shelf {
        render_items: [Option<ItemType>; NUM_SHELF_SLOTS],
        item_data_bs: [Option<u16>; NUM_SHELF_SLOTS],
        slots: ShelfSlots,
    },
    Gold(StandardSlots),
    Portal,
    Cabinet {
        render_items: [Option<ItemType>; NUM_SHELF_SLOTS],
        item_data_bs: [Option<u16>; NUM_SHELF_SLOTS],
        slots: ShelfSlots,
    },
    Feeder(StandardSlots),
}

impl Default for ChestSlots {
    fn default() -> Self {
        Self::Standard(Slots::default())
    }
}

impl ChestSlots {
    pub fn chest_type(&self) -> ChestType {
        match self {
            Self::Standard(_) => ChestType::Standard,
            Self::Safe(_) => ChestType::Safe,
            Self::Shelf { .. } => ChestType::Shelf,
            Self::Gold(_) => ChestType::Gold,
            Self::Portal => ChestType::Portal,
            Self::Cabinet { .. } => ChestType::Cabinet,
            Self::Feeder(_) => ChestType::Feeder,
        }
    }

    pub fn from_chest_type_and_slots(
        chest_type: ChestType,
        slot_values: Option<Vec<plist::Value>>,
        render_items: Option<[Option<ItemType>; NUM_SHELF_SLOTS]>,
        item_data_bs: Option<[Option<u16>; NUM_SHELF_SLOTS]>,
    ) -> Result<Self> {
        match (slot_values, chest_type) {
            (
                Some(slot_values),
                ChestType::Standard | ChestType::Safe | ChestType::Gold | ChestType::Feeder,
            ) => {
                let slots = Slots::from_values(slot_values)
                    .map_err(Box::new)
                    .context(LoadSlotsSnafu)?;
                Ok(match chest_type {
                    ChestType::Standard => Self::Standard(slots),
                    ChestType::Safe => Self::Safe(slots),
                    ChestType::Gold => Self::Gold(slots),
                    ChestType::Feeder => Self::Feeder(slots),
                    _ => unreachable!(), // bad design
                })
            }
            (Some(slot_values), ChestType::Shelf | ChestType::Cabinet) => {
                let slots = Slots::from_values(slot_values)
                    .map_err(Box::new)
                    .context(LoadSlotsSnafu)?;
                let render_items = render_items.unwrap_or([None; NUM_SHELF_SLOTS]);
                let item_data_bs = item_data_bs.unwrap_or([None; NUM_SHELF_SLOTS]);
                Ok(match chest_type {
                    ChestType::Shelf => Self::Shelf {
                        render_items,
                        item_data_bs,
                        slots,
                    },
                    ChestType::Cabinet => Self::Cabinet {
                        render_items,
                        item_data_bs,
                        slots,
                    },
                    _ => unreachable!(), // bad design
                })
            }
            (slots, ChestType::Portal) => match slots {
                Some(_) => PortalChestHaveSlotsSnafu.fail(),
                None => Ok(Self::Portal),
            },
            (
                None,
                ChestType::Standard
                | ChestType::Safe
                | ChestType::Shelf
                | ChestType::Gold
                | ChestType::Cabinet
                | ChestType::Feeder,
            ) => NoSaveItemSlotSnafu { chest_type }.fail(),
        }
    }

    #[allow(clippy::type_complexity)]
    fn to_chest_type_and_slots(
        &self,
    ) -> Result<(
        ChestType,
        Option<Vec<plist::Value>>,
        Option<[Option<ItemType>; NUM_SHELF_SLOTS]>,
        Option<[Option<u16>; NUM_SHELF_SLOTS]>,
    )> {
        let chest_type = self.chest_type();
        let slot_values = match self {
            Self::Standard(s) | Self::Safe(s) | Self::Gold(s) | Self::Feeder(s) => {
                Some(s.to_values())
            }
            Self::Shelf { slots, .. } | Self::Cabinet { slots, .. } => Some(slots.to_values()),
            Self::Portal => None,
        }
        .transpose()
        .map_err(Box::new)
        .context(SaveSlotsSnafu)?;
        let (render_items, item_data_bs) = match self {
            Self::Standard(_) | Self::Safe(_) | Self::Gold(_) | Self::Portal | Self::Feeder(_) => {
                (None, None)
            }
            Self::Shelf {
                render_items,
                item_data_bs,
                ..
            }
            | Self::Cabinet {
                render_items,
                item_data_bs,
                ..
            } => (Some(*render_items), Some(*item_data_bs)),
        };
        Ok((chest_type, slot_values, render_items, item_data_bs))
    }

    pub fn as_slots(&self) -> Option<&[Slot]> {
        match self {
            Self::Standard(s) | Self::Safe(s) | Self::Gold(s) | ChestSlots::Feeder(s) => {
                Some(s.as_slice())
            }
            ChestSlots::Shelf { slots, .. } | ChestSlots::Cabinet { slots, .. } => {
                Some(slots.as_slice())
            }
            ChestSlots::Portal => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chest {
    obj: InteractionObject,
    pub save_time: f64,
    pub slots: ChestSlots,
}
inherit!(Chest -> InteractionObject, obj);

impl Default for Chest {
    fn default() -> Self {
        Self {
            obj: InteractionObject {
                interaction_object_type: InteractionObjectType::Chest,
                ..Default::default()
            },
            save_time: 0.0,
            slots: ChestSlots::default(),
        }
    }
}

impl Chest {
    pub fn new(obj: InteractionObject, save_time: f64, slots: ChestSlots) -> Self {
        Self {
            obj,
            save_time,
            slots,
        }
    }
}

// Chest data from binary item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChestItem {
    #[serde(flatten)]
    obj: InteractionObject,
    pub chest_type: ChestType,
    pub save_time: f64,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub save_item_slots: Option<Vec<plist::Value>>,
}
inherit!(ChestItem -> InteractionObject, obj);

// Contains metadata of a chest stored in dynamic world sub-db
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChestMeta {
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
inherit!(ChestMeta -> InteractionObject, obj);

impl Chest {
    pub(crate) fn parse_slot_bytes(bytes: &[u8]) -> Result<Vec<plist::Value>> {
        let decompressed = decompress(bytes).context(DecompressSlotsSnafu)?;
        plist::from_bytes(&decompressed).context(DeserializeSlotsSnafu)
    }

    pub(crate) fn from_meta_and_slots(meta: ChestMeta, slot_bytes: Option<&[u8]>) -> Result<Self> {
        let slot_values = slot_bytes
            .map(|bytes| -> Result<Vec<plist::Value>> { Self::parse_slot_bytes(bytes) })
            .transpose()?;
        let shelf_render_items = [
            meta.shelf_render_items_0,
            meta.shelf_render_items_1,
            meta.shelf_render_items_2,
            meta.shelf_render_items_3,
        ];

        let shelf_item_data_bs = [
            meta.shelf_item_data_bs_0,
            meta.shelf_item_data_bs_1,
            meta.shelf_item_data_bs_2,
            meta.shelf_item_data_bs_3,
        ];

        let slots = ChestSlots::from_chest_type_and_slots(
            meta.chest_type,
            slot_values,
            Some(shelf_render_items),
            Some(shelf_item_data_bs),
        )?;

        Ok(Self {
            obj: meta.obj,
            save_time: meta.save_time,
            slots,
        })
    }

    pub(crate) fn to_meta_and_slots(&self) -> Result<(ChestMeta, Option<Vec<u8>>)> {
        let (chest_type, save_item_slots, shelf_render_items, shelf_item_data_bs) =
            self.slots.to_chest_type_and_slots()?;
        let [r0, r1, r2, r3] = shelf_render_items.unwrap_or_default();
        let [d0, d1, d2, d3] = shelf_item_data_bs.unwrap_or_default();
        let slot_bytes = match save_item_slots {
            Some(slot_values) => {
                let compressed = compress(
                    &to_xml_plist(&plist::Value::Array(slot_values))
                        .context(SerializeSlotsSnafu)?,
                )
                .context(CompressSlotsSnafu)?;
                Some(compressed)
            }
            None => None,
        };
        Ok((
            ChestMeta {
                obj: self.obj.clone(), // should be cheap if there's no owner id
                chest_type,
                save_time: self.save_time,
                shelf_render_items_0: r0,
                shelf_render_items_1: r1,
                shelf_render_items_2: r2,
                shelf_render_items_3: r3,
                shelf_item_data_bs_0: d0,
                shelf_item_data_bs_1: d1,
                shelf_item_data_bs_2: d2,
                shelf_item_data_bs_3: d3,
            },
            slot_bytes,
        ))
    }

    pub(crate) fn from_chest_item(chest_item: ChestItem) -> Result<Self> {
        Ok(Self {
            obj: chest_item.obj,
            save_time: chest_item.save_time,
            slots: ChestSlots::from_chest_type_and_slots(
                chest_item.chest_type,
                chest_item.save_item_slots,
                None,
                None,
            )?,
        })
    }

    pub(crate) fn to_chest_item(&self) -> Result<ChestItem> {
        let (chest_type, save_item_slots, _, _) = self.slots.to_chest_type_and_slots()?;
        Ok(ChestItem {
            obj: self.obj.clone(),
            chest_type,
            save_time: self.save_time,
            save_item_slots,
        })
    }
}
