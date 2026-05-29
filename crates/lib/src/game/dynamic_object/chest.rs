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
            slots: ChestSlots::default(),
        }
    }
}

impl Chest {
    pub fn new(obj: InteractionObject, slots: ChestSlots) -> Self {
        Self { obj, slots }
    }
}

// Chest data from binary item or freight car chest
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChestSaveDictXml {
    #[serde(flatten)]
    obj: InteractionObject,
    pub chest_type: ChestType,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub save_item_slots: Option<Vec<plist::Value>>,
}
inherit!(ChestSaveDictXml -> InteractionObject, obj);

// Contains metadata of a chest stored in dynamic world sub-db
// Doesn't have slots
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChestDwXml {
    #[serde(flatten)]
    obj: InteractionObject,
    chest_type: ChestType,

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
inherit!(ChestDwXml -> InteractionObject, obj);

impl Chest {
    pub(crate) fn parse_slot_bytes(bytes: &[u8]) -> Result<Vec<plist::Value>> {
        let decompressed = decompress(bytes).context(DecompressSlotsSnafu)?;
        plist::from_bytes(&decompressed).context(DeserializeSlotsSnafu)
    }

    pub(crate) fn from_dw_xml_and_slots(
        xml: ChestDwXml,
        slot_bytes: Option<&[u8]>,
    ) -> Result<Self> {
        let slot_values = slot_bytes
            .map(|bytes| -> Result<Vec<plist::Value>> { Self::parse_slot_bytes(bytes) })
            .transpose()?;
        let shelf_render_items = [
            xml.shelf_render_items_0,
            xml.shelf_render_items_1,
            xml.shelf_render_items_2,
            xml.shelf_render_items_3,
        ];

        let shelf_item_data_bs = [
            xml.shelf_item_data_bs_0,
            xml.shelf_item_data_bs_1,
            xml.shelf_item_data_bs_2,
            xml.shelf_item_data_bs_3,
        ];

        let slots = ChestSlots::from_chest_type_and_slots(
            xml.chest_type,
            slot_values,
            Some(shelf_render_items),
            Some(shelf_item_data_bs),
        )?;

        Ok(Self {
            obj: xml.obj,
            slots,
        })
    }

    pub(crate) fn to_dw_xml_and_slots(&self) -> Result<(ChestDwXml, Option<Vec<u8>>)> {
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
            ChestDwXml {
                obj: self.obj.clone(), // should be cheap if there's no owner id
                chest_type,
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

    pub(crate) fn from_chest_save_dict_xml(chest_item: ChestSaveDictXml) -> Result<Self> {
        Ok(Self {
            obj: chest_item.obj,
            slots: ChestSlots::from_chest_type_and_slots(
                chest_item.chest_type,
                chest_item.save_item_slots,
                None,
                None,
            )?,
        })
    }

    pub(crate) fn to_chest_save_dict_xml(&self) -> Result<ChestSaveDictXml> {
        let (chest_type, save_item_slots, _, _) = self.slots.to_chest_type_and_slots()?;
        Ok(ChestSaveDictXml {
            obj: self.obj.clone(),
            chest_type,
            save_item_slots,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Chest, ChestSaveDictXml};
    use crate::util::plist::{diff_plist_keys, to_xml_plist};

    fn chest_save_dict_round_trip(data: &[u8]) {
        let xml: ChestSaveDictXml =
            plist::from_reader_xml(data).expect("should be able to deserialize");
        let chest = Chest::from_chest_save_dict_xml(xml).expect("should be able to load as chest");
        let round_trip_chest_xml_data = to_xml_plist(
            &(chest
                .to_chest_save_dict_xml()
                .expect("should be able to save as chest")),
        )
        .expect("should serialize");
        let round_trip_chest_xml: ChestSaveDictXml =
            plist::from_reader_xml(round_trip_chest_xml_data.as_slice())
                .expect("should be able to deserialize");
        let round_trip_chest = Chest::from_chest_save_dict_xml(round_trip_chest_xml)
            .expect("should be able to load as chest");
        assert_eq!(chest, round_trip_chest);

        let xml_value: plist::Value =
            plist::from_reader_xml(data).expect("should be able to deserialize");
        let round_trip_xml_value: plist::Value =
            plist::from_reader_xml(round_trip_chest_xml_data.as_slice())
                .expect("should be able to deserialize");
        let mut diffs = Vec::new();
        diff_plist_keys("", &xml_value, &round_trip_xml_value, &mut diffs);
        assert!(
            diffs.is_empty(),
            "structural fidelity violations:\n{}",
            diffs.join("\n")
        );
    }

    #[test]
    fn test_dyn_obj_0() {
        // Covers: Bed, Door, Ladder, Torches
        let data = b"
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
	<key>chestType</key>
	<integer>0</integer>
	<key>flipped</key>
	<false/>
	<key>floatPos</key>
	<array>
		<real>11189.5</real>
		<real>668</real>
	</array>
	<key>interactionObjectType</key>
	<integer>2</integer>
	<key>isInUse</key>
	<false/>
	<key>ownerID</key>
	<string>server</string>
	<key>paintColor</key>
	<integer>0</integer>
	<key>pos_x</key>
	<integer>11189</integer>
	<key>pos_y</key>
	<integer>668</integer>
	<key>saveTime</key>
	<real>7115.363215520978</real>
	<key>uniqueID</key>
	<integer>834</integer>
	<key>saveItemSlots</key>
    <array>
    	<array>
    		<data>
    		NAAAAAAAAAAfiwgAAAAAAAAHbZBfT4MwFMWf4VPUvsO1MyiajkXHTEiII5E9
    		+GQQ6mwGtJZOxreXP26o7Onee/q7556ULg5Fjr6Yqrgo55jYlxixMhUZL7dz
    		vIkfLRcvPJNe+Otl/BKtkMx5pVG0eQiDJcIWwL2UOQPwYx9FYfAco9YDYPWE
    		Ef7QWt4B1HVtJx1lp6LowAoiJSRTuglbM6tdsDOd4fbM4P4nTqtmPNWeadAd
    		a7yMQlfa6Ucd5LdcpDt2ejQoLzXbMuURCsf2yL7nItGRqEY4USrpO4MqluQe
    		IeSW2A6Ffhr165vZqFE4rfW2XLMibiSbZnBm0xBSVK+HM3G7y+fpZkr3ef6z
    		+5J/7lngT3HXca9+8xSGPxxr//+e+Q3BpcUmFgIAAA==
    		</data>
    	</array>
    	<array>
    		<data>
    		RQAAAAAAAAAfiwgAAAAAAAAHbZBfT4MwFMWf4VPUvsMdGtlmui46ZkJCHIns
    		wSeDUGczoLV0Mr69/HFDZU/33tPfPfekZHnMM/TFVMlFscCOPcGIFYlIebFb
    		4G30aM3wkprkytusopdwjWTGS43C7UPgrxC2AO6lzBiAF3koDPznCDUeAOsn
    		jPCH1vIOoKoqO24pOxF5C5YQKiGZ0nXQmFnNgp3qFDdnevc/cRo15YmmpkH2
    		rKYpgbY004/ay2+ZSPbs/GgQXmi2Y4pOCJzaE/ueiViHohzgWKm46wyiWJxR
    		x3Hm1/YtgW4adHd6M2gEzmudLdcsj2rJxhnc+TiEFOXrcYx2ly/T9QXjNs9/
    		9lDwzwPzvTE+c53pb55A/4dD7f6fmt/4tmb1FgIAAA==
    		</data>
    	</array>
    	<array>
    		<data>
    		pAAAAAAAAAAfiwgAAAAAAAAHbZFdT8IwFIav4VfU3rPj/EAwpUQZJiRElzgu
    		vDJzO2JDaWtXhP17u/ExFK7a8/Y5b9/TsuFmKckP2kJoNaBhcEkJqkznQs0H
    		dJY8dXp0yNvsInoZJW/xmBgpCkfi2eN0MiK0A/BgjESAKIlIPJ28JsR7AIyf
    		KaFfzpl7gPV6HaQVFWR6WYEFxFYbtK6cerOObwhyl1N/zdb9Txyv5iJzvN1i
    		Cyx5zqBafLVTt/KH1NkCD4ctJpTDOVoeMthv9+yn1KmLddHAqbVpvWsxi6nk
    		YRj2r4NbBnXV6N27q0ZjcGirbYXVKpZphiMpULlJ1PgXzvr35AVaPxqDXXlo
    		dLhMSoNnwndvTuMbXbxvzrBV5vN0eUrXk/xnV0p8r/A4+J7odcP+Mc9g+/rN
    		Wv8cb/8C7FB+GFACAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		qQAAAAAAAAAfiwgAAAAAAAAHjVJdb4IwFH3WX8F4l4qKH0vFbOISEjNJhg97
    		WipcXTegXVtl/vsVlMGmS/bU3tNzT889LZ59polxACEpy6ambXVNA7KIxTTb
    		Tc11+NAZmzO3jW+81Tx8DhYGT6hURrC+X/pzw+wgdMd5Agh5oWcES/8pNLQG
    		QotH0zBfleK3COV5bpGCZUUsLYgSBYJxEOq41GId3WDFKjb1NSf1H3Y0GtNI
    		ue0WfoejG2NULLo6oyd4A3Fhec4SJipGC9NMwQ6E28Wo2lYN24RyDnHN3ZJE
    		AmqcM6ICJmsCEYKUuxYWQBLXtu3J2HIwKqsaH456NYbRd1spW9gQJFJ6vNXm
    		DSIVHjlcGu5fGqbSz9YS/jRMFaTXxezh5FKO5RkI36vZUgkdoCtB6PgxOpcV
    		nRMt8P90OZMvn1ecFJFdZx8v2WWQv7mSHCCkaWPMMupRd2Bbg2G/13Wcvt3r
    		TxrvUvbtM/qxh+bAlfJ46Iya92B0+ln1Wv5Kt/0F+yhWKCwDAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		PwAAAAAAAAAfiwgAAAAAAAAHjVJdT8IwFH2GXzH7zrohDDBlRAGTJUSWOB58
    		MmW7YHWstS1f/95uMDcFE1/W3tNzT889KxkdNqm1A6kYz4bItR1kQRbzhGXr
    		IVpEj60+GvlNcjOZj6OXcGqJlClthYuHWTC2UAvjeyFSwHgSTaxwFjxHltHA
    		ePqELPSmtbjDeL/f2zRn2THf5ESFQ8kFSH2cGbGWabATnSBzzUn9hx2DJizW
    		frNBPuDoJwTni6nO6AleQpJbHvOUy5LRICzTsAbpOwSX27JhlTIhIKm4K5oq
    		wLVzTnXIVUWgUtJi1yASaOq7rjvw7C7BRVXhXq9dYQR/txWyuQ1JY23Gmy/f
    		IdbRUcCl4dtLw0wF2ULBn4aZhs11Me+KGt9nIINJRVZamvx8BdKkT/C5LOmC
    		GoH/hyu4ej1cMovErrOPV2znOf7mKrqDiG1qUxZJ95yOY3cGXrfnmq/bHtT+
    		QE7cZuxzC/V5S+G+1+7UryH49K6qtXiTfvMLAyz5vSoDAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		MAEAAAAAAAAfiwgAAAAAAAAHZZFRT8IwFIWf4VfUvrPLQBBNKVGGCQnRJY4H
    		n8zcrrgw1toWx/493WAM3dPtPf3u6WnLZoddSn5R6URkU+o6fUowi0ScZJsp
    		XQfPvQmd8S678V7nwbu/IDJNtCH++mm1nBPaA3iUMkUAL/CIv1q+BcR6ACxe
    		KKHfxsgHgDzPnbCknEjsSlCDr4REZYqVNevZASc2MbXHnNz/xLFqnESGdzts
    		iwWPGZTFdmf1JH+mItriZbPDkszgBhXvM6iXNfuVitD4QjdwqFRYrTpMYZhy
    		13XvR86IQdU1+vhu0GgMLmOVbWJwFxQS2xmG/dt2CpFnqJZeQ2uj7KNzjcre
    		n8G5rXEp9MehbV0lbZuXdNGmq/z/2X2W/OzxOklNTMaD4TXP4PTmTa3+i3eP
    		+RKNaEYCAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		pQAAAAAAAAAfiwgAAAAAAAAHbZFfT8IwFMWf4VPUvrPLVBBNGVGGCQnRJY4H
    		n8zcrthQ2toWcd/ebvwZCk/tPf3d03NbNvpZCfKNxnIlhzQMupSgzFXB5WJI
    		5+ljZ0BHUZtdxM/j9DWZEC24dSSZP8ymY0I7APdaCwSI05gks+lLSrwHwOSJ
    		EvrpnL4D2Gw2QVZRQa5WFWghMUqjceXMm3V8Q1C4gvprtu5/4ni14LmL2i22
    		xDIqGFSLr3bqVn4XKl/i4bDFuHS4QBN1Gey3e/ZDqMwlyjZwZkxW71rMYCai
    		MAxvr4Meg7pq9P7NVaMxOLTVttwomYgsx7HgKN00bvytM/49I4vGj8ZgVx4a
    		Ha7SUuNp+LDfO42vlX37OcNWmc/T5SldT/KfXUv+tcbj4Hti0L8Mj3kG29dv
    		1vrnovYvlnvWilACAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		qgAAAAAAAAAfiwgAAAAAAAAHjZJdb4IwFIav9Vcw7qXgF2ypmE1dYmImyfBi
    		V0uFo+uGtGvrB/9+BWWwyZJdtT085+17XorHp11iHEBIytKR6Vi2aUAasZim
    		25G5Ch87njn22/hmupyEL8HM4AmVyghWD4v5xDA7CN1zngBC03BqBIv5c2ho
    		DYRmT6ZhvinF7xA6Ho8WySkrYrsclCgQjINQ2UKLdXSDFavY1Nec1X/Y0dWY
    		Rspvt/AHZH6MUb7o06V6Lq8hzi1PWMJESbQwTRVsQfg2RuW2bNgklHOIK3ZD
    		Egmo9p0RFTBZAUQIUuxaWABJfMdxvK41wKg4VfWh261qGH23FbK5DUEipcdb
    		rt8hUmHG4dpw79owlfN0JeFPw1TBrlnMcRvmZ8cUxHxa0VIJHaAvQej4Mboc
    		S5wTLfD/dDmTr6cGJ3lkzXR2TRdB/mYlOUBId7Uxi6hdu9+3boe9ru0NHKfv
    		Om7tH+TkPqWfe6gPXCp7w4FXvwej88uq1uJV+u0veTa34ywDAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		/gAAAAAAAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		twAAAAAAAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		lgAAAAAAAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		AgEAAAAAAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		NQAAAAAAAAAfiwgAAAAAAAAHbZFRT8IwEMef2aeofWfHVHAxpUQZJiRElzge
    		fDILO7FxrLUtjn17u8EcOp969+/v/r27stlhl5Mv1EbIYkoDf0QJFhuZiWI7
    		pevkYRjSGffYRfQ0T17iBVG5MJbE6/vVck7oEOBOqRwBoiQi8Wr5nBDnAbB4
    		pIS+W6tuAcqy9NOa8jdyV4MGYi0ValutnNnQFfiZzah75uj+qx2nZmJjuTdg
    		H1jxjEF9uOykHuW3XKY2lqa9HbBU67SJBkxjmvMgCMJrf8ygyTp9cnPZaQx+
    		yhpbYXGXVAo7W1FY3KLm4ysGbdzSsixQL6MONla7PXKD2o3E4JS2uEqdwVzm
    		UvftR313Jc3roU82c/1PV326mfYvuy/E5x7PG2+JcDwJz3kGx613Z/Nj3PsG
    		azIhFEgCAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		EQAAAAAAAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		LwAAAAAAAAA=
    		</data>
    	</array>
    	<array/>
    </array>
</dict>
</plist>";
        chest_save_dict_round_trip(data.as_slice());
    }

    #[test]
    fn test_dyn_obj_1() {
        // Covers: Egg, HandCar, Locomotive, FreightCar, PassengerCar, Chest, Workbench
        let data = b"
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
    <key>chestType</key>
	<integer>0</integer>
	<key>flipped</key>
	<false/>
	<key>floatPos</key>
	<array>
		<real>11189.5</real>
		<real>669</real>
	</array>
	<key>interactionObjectType</key>
	<integer>2</integer>
	<key>isInUse</key>
	<false/>
	<key>ownerID</key>
	<string>server</string>
	<key>paintColor</key>
	<integer>0</integer>
	<key>pos_x</key>
	<integer>11189</integer>
	<key>pos_y</key>
	<integer>669</integer>
	<key>saveTime</key>
	<real>7115.363215520978</real>
	<key>uniqueID</key>
	<integer>7372</integer>
	<key>saveItemSlots</key>
    <array>
    	<array>
    		<data>
    		bwAAAAAAAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		rgAAAAAAAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		WwAAAAAAAAAfiwgAAAAAAAAHZVFdb4IwFH3WX9H13ZYiCi4Vs4lLTMxGMnzY
    		09LBnZIhsFK//v1KHeLHU+89Pfecc1s+OWwytANZpUU+xoxYGEEeF0mar8Z4
    		Gb30PDzxu/wheJtGH+EMlVlaKRQunxfzKcI9Sp/KMgNKgyhA4WL+HiGtQens
    		FSO8Vqp8pHS/3xNRs0hcbGpiRUNZlCDVcaHFenqAJCrB2uakfhVHo0kaK7/b
    		4T9w9BNO60N3/+gJ/s4KocKiam47XEgpTNXhEkTmM8Y8Rgacmq7Fh67dYpye
    		x4zsCnKoAm3U6ja2J8KXBDhH0liaK1iB9C1Om9LIXoVdCxWvo3QDspU1CVyr
    		T/oj5jDPGeia2RdxzWRZVJ+HdqixMLtdOzbs4z3bbHzLrcQO6kg3iYZ95hDb
    		Ho08yxl67m2ebZ7+bmEe3Jtornvp0jxBe5q/9rt/P8ZeF4ICAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		zAAAAAAAAAAfiwgAAAAAAAAHZZDRT4MwEMaft7+i9h1uJLqg6Vh0zISEKFH2
    		4JMh48YaWYttJ+O/t4MxNn26fl9+99312PywK8kPKs2lmFHPnVCCYi1zLooZ
    		XaXPjk/nwZjdhK+L9CNZkqrk2pBk9RRHC0IdgMeqKhEgTEOSxNF7SmwGwPKF
    		Ero1pnoAqOvazY6Uu5a7I6ghUbJCZZrYhjm2wc1NTu2YLv1qHevmfG2C8Yh9
    		YRPkDI7FqpPb2SgKLjDSb7zYmh4ZsU1WaoQztSllZhKpByBTKmtfI6YwKwPP
    		8+4n7h2DVg3+9NYfPAbntjZW1gJVFA6p2ih7v0Cjsl9hcJI9Xkn9eRhgLgwW
    		qLrRDHp5STf/6Xahv+xe8O89Xm7SE74/veIZdOcbanv6YPwLY/79ghECAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		zQAAAAAAAAAfiwgAAAAAAAAHdZLRboIwFIav9SlY76UwRdlSMZtoQmI2suHF
    		rpYGKjbDtmvL1LdfERE241V7zvn6n7+nRbPDrrB+iFSUsylwbQdYhKU8oyyf
    		gnWyHPhgFvTRXfg6Tz7ihSUKqrQVr59X0dwCAwifhCgIhGESWvEqek8sowHh
    		4gVYYKu1eIRwv9/buKLslO8qUMFYckGkPq6M2MAcsDOdAdOmVv9jx2Qzmuqg
    		30Nf5BhkCFaLic7ZOk1YThmJ1BvNt7pBemiDC0XghdoUHOuYqxbAUuLTrock
    		wUXguu7D0PaGnuuN7z0ET8m2PB759ng08fyh40x8t0sgeNGqe5WkWEqcanOT
    		tl/dpHOqKuTcTPufcy3LjvEtVkujd6vM94zIKGzLSksjGSgizSwRPIcNLrj6
    		PLQwZZrkRNZ3R7AJu/TxmjajuGaV5kKQ7JbPktHvknSNNgK+P3G6cgjWz9uu
    		p68R9H8B+1rcSLECAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		OgAAAAAAAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		zgAAAAAAAAAfiwgAAAAAAAAHvVRRb5swEH5OfgXlPTi0JIHKocqSTkKKVraS
    		hz1NHlwTbwR7tts0/36GQHBUqDZNGi/47j6f7z5/Z3z3us+tFxCSsmJuu87Y
    		tqBIWUaL7dzeJB9Hvn0XDvHV6mGZfI3vLZ5Tqax482EdLS17hNCC8xwQWiUr
    		K15Hj4mlcyB0/8m27J1S/Bahw+HgkBLlpGxfAiWKBeMg1HGtk430BidTma2P
    		OWW/KEd7M5qqcDjAP+EYZhiVP23V3pM73YFUTaiNGcHkyOEMGGBaKNiCCMcY
    		Ncsz/imnnENmoJ9ILgGZCEZUzKQBIUKQ03KABZA8dF03cJ0JRpVlBKaebzgx
    		andWuctyBEmVZuDh+w9I+yq/7qicyqjYSHinck70piXLmfhDMjiT3147sFV7
    		PfhjB77q+g1akheIFOwfc6b62DTW+maJIvVafF4037yOojZs8lrnQP9qdBWx
    		+N9F/JXxRl0l4QndmxI5ifLGDxwvmPqBNxm7rhfMTI2W2OeC/nqGaNVxt74/
    		m15crubAnE4otrSASH6h250xpa02u4fKIL1vpM4T5bjXN2Nv6s+MqNF8dQA7
    		FCCMBrBUQr9zoQShnxyMarOBXyr/HeF36L5X9t1UmkxOTHxDZPuvnshw+BtS
    		NstPuQUAAA==
    		</data>
    	</array>
    	<array>
    		<data>
    		0AAAAAAAAAAfiwgAAAAAAAAHZZFRT8IwFIWf4VfUvrPLBHSYMqIMkyVEFx0P
    		PpmFXUbjaGdbnPx7y8bYDE/tOf167+ktm//uc/KDSnMpZtR1hpSg2MiUi2xG
    		1/HzwKNzv89ugtdF/BEtSZFzbUi0flqFC0IHAI9FkSNAEAckWoXvMbE1AJYv
    		lNCdMcUDQFmWTnKinI3cn0ANkZIFKnNc2WIDe8FJTUptm7r6vzjWTfnG+P0e
    		+8KjnzI4LVad3dpGkXGBoX7j2c40SI9tk1wjXKhtLhMTSd0CiVJJtesxhUnu
    		u647vXUmDCrV+ndjzxlNvNHUG4/uO8cMLhWqDrIUqMKgbaCNsqP0NSr7KgZn
    		2eCF1J+/LcyFwQxVnYJBI7v08Zq22a7Zg+DfB+wmaQhv6g67PIN6ku1a/YLf
    		/wNxHKieHAIAAA==
    		</data>
    	</array>
    	<array>
    		<data>
    		sgAAAAAAAAA=
    		</data>
    		<data>
    		sgAAAAAAAAA=
    		</data>
    		<data>
    		sgAAAAAAAAA=
    		</data>
    		<data>
    		sgAAAAAAAAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		MAQAAAAAAAAfiwgAAAAAAAAHtVTfU+IwEH6Gv6LXd7otAoIT6iDgTOcY7Yzl
    		waeb2K4YLU0uiVb+e9NCbaX4cHN3fcn++Ha/zWa35PJ9m1pvKBXj2dT2HNe2
    		MIt5wrLN1F5H172xfel3yY/F7Ty6D5eWSJnSVri+WgVzy+4BzIRIEWARLaxw
    		FdxFlskBsLyxLftJa3EBkOe5QwuUE/NtAVQQSi5Q6t3KJOuZACfRiW1o9tm/
    		lGOsCYu13+2QF9z5CYHiMNrBujfHT6h0tBNYuTuEZRo3KP0zApVYoR9TJgQm
    		NfaRpgqh4edUh1zVAColLaUOkUhT3/O8iesMCZRabR8NJrWNwGdYmbYoQ9JY
    		m7vdPjxj/E3B/XbBTAXZWuG3BfM8Qxksar/S0rygr1CaZhI4qBVcUEMw5ymX
    		bXa3zS64+vXeRpY9OI3etdFlZ46xir5hoHF7l3J9utu1aF6caroX55tZ9U33
    		Pvh0Nrp+CIe/kFvUP/Plf6Ru0V3P/sVNW2n5nzfweJqLt4vYtjGUhw3wxo53
    		5vb77nA4OB8196EAvmbs9ys2Z7UaivFkMGqOiCmpXPH6LH8PfvcDoJ8sPbUE
    		AAA=
    		</data>
    	</array>
    	<array>
    		<data>
    		GgQAAAAAAAAfiwgAAAAAAAAHjZRbj9owEIWfl1+R5p042VxKqmxWLRcJCXWR
    		CFr1qTLJAO6a2LXN7d/XCaRhSVj2KfHwzfHJYezo+bChxg6EJCx/Mh3LNg3I
    		U5aRfPVkzpNRt2c+x53oy+Cln/yaDg1OiVTGdP5jMu4bZheh75xTQGiQDIzp
    		ZDxLDK2B0PCnaZhrpfg3hPb7vYULykrZpgAlmgrGQajjRIt1dYOVqczU25zU
    		39nR1YykKu48RG9wjLMIFQ+9OldPZbzDhOIFhSGFVAmSEnWsyIeI5ApWIOLA
    		910/QtWyak4FXiptaSVAyj7b5qpuFYBpbFt2hMq3qmVJBMy4LmUJ2YC4z1PC
    		OWQ1t8RUArr4nWE1ZbIGsBC4fDtrOo4TupZf657rgRde7IX+t51kt0BHAqdK
    		x3nX4xrLkeZvelxvhTj2mVTNYO1mqCU9g5Tlmby/dQF/LskS1dN502Zh5PzJ
    		L4s/ehqSI4emZadpmchxPpdwU5liqV6ZoOV/fmU0cMPQCsOg5/m+HTiuE1z5
    		prC7zPaD5Ng+BzEe1KzUA52vYgliVwR0XlY4x1qgzygTn1LnTP4+tMRRjFc7
    		3XaQiqG7ZiXeQVsynu1Zrms/2oE+fV/94DoZCcWZhWycZ9DirOUbtjn5u4XL
    		iCqiF3pek98z8bbQ19q6fRQemx2HWSoYpR9MY4ROF1D9LC+vuPMPsQ6KD1MF
    		AAA=
    		</data>
    	</array>
    	<array/>
    	<array/>
    	<array/>
    	<array/>
    	<array/>
    </array>
</dict>
</plist>";
        chest_save_dict_round_trip(data.as_slice());
    }

    #[test]
    fn test_dyn_obj_2() {
        // Covers: Mirror, ElevatorShaft, Stairs, Column, Painting, TradingPost,
        //         TradePortal, TrainStation, Sign, OwnerSign
        let data = b"
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
	<key>chestType</key>
	<integer>0</integer>
	<key>flipped</key>
	<false/>
	<key>floatPos</key>
	<array>
		<real>11189.5</real>
		<real>670</real>
	</array>
	<key>interactionObjectType</key>
	<integer>2</integer>
	<key>isInUse</key>
	<false/>
	<key>ownerID</key>
	<string>server</string>
	<key>paintColor</key>
	<integer>0</integer>
	<key>pos_x</key>
	<integer>11189</integer>
	<key>pos_y</key>
	<integer>670</integer>
	<key>saveTime</key>
	<real>7115.363215520978</real>
	<key>uniqueID</key>
	<integer>7373</integer>
	<key>saveItemSlots</key>
	<array>
		<array>
			<data>
			SwEAAAAAAAAfiwgAAAAAAAAHdVLBcoIwED3rV1DuEsGq2Ik4rdgZZpzKTPHQ
			UyeF1dIiSZMo8vcNoEBLe0r27dt9uy/Bi/Mh0U7ARUzTuW4aQ12DNKRRnO7n
			+jZ4HNj6wunjG3ezDF78lcaSWEjN3z6svaWmDxC6ZywBhNzA1fy19xxoqgdC
			qydd09+lZHcIZVlmkIJlhPRQEAXyOWXAZb5WzQaqwIhkpCuZqvuPcRQaxaF0
			+j38CbkTYVQcKrqgFbxLYsagTvbwjiQCUCtPifSpaAiEc1LeepgDSRzTNKcT
			Y4xRGTX4ZGo1GEZ1Wdk2TiVwEko17ebtA0IZ5AwajSK9B+7MMLpe60rhpVsB
			/w5MsxS45zZ5Ibl6E0cAV/ZgdAmvdEaUwJImlHfVh111RsXrucssPfibnXfZ
			pTO/uYKcIIgPrcUqF23LNEbWzJ7Z4+nodmy3TS2YxzT+OkJ74do8azhq62BU
			vXxzlr/G6X8DOmzBCcwCAAA=
			</data>
		</array>
		<array>
			<data>
			PwQAAAAAAAAfiwgAAAAAAAAHjVLRTsIwFH2Gr6h9Z3eDyNCUEWWYEFGXOB58
			Mgur2Dja2hbH/t5uY04yHnhq7+k5p+f2lswOuwz9UKWZ4FPsOS5GlG9Eyvh2
			itfxw2CCZ0GfXIUv8/gtWiCZMW1QtL5fLecIDwDupMwoQBiHKFotX2NkPQAW
			zxjhT2PkLUCe505SspyN2JVEDZESkipTrKzZwAqc1KTYXlO7n8SxaMo2Juj3
			yBctgpRAudjqiNbwRyYSEwndnPZIolRS7XpE0SQLPM/zfeeaQFW1+NgftRiB
			P1llywzdxYWkrS3jhm6pCjx34hNoqoafJdo8cpHzJ2GEsnmcwxlpmeQSbdHV
			jv1hV2k1VC3Dlq2NsvMLNFX2KQkcy4YuE2swF5lQXX+36y6Ffr+8i5J9Nveo
			y91z9r2n/4M3jJuhe9IngXra7Vr9lKD/C+rbpbvAAgAA
			</data>
		</array>
		<array>
			<data>
			QAQAAAAAAAA=
			</data>
		</array>
		<array>
			<data>
			9QAAAAAAAAAfiwgAAAAAAAAHbZHfT8IwEMef4a+ofWfHiPzQlBJlmJAQXeJ4
			8Mk0W5mNY61tcey/txvMiePp7r753Ld3V7I47jP0zbURMp9j3xtixPNYJiJP
			53gbPQ1meEH75CZ4WUZv4QqpTBiLwu3jZr1EeADwoFTGAYIoQOFm/Roh5wGw
			esYIf1ir7gGKovBYRXmx3FeggVBLxbUtN85s4Bq8xCbYPXNyvxjHqYmILe33
			yCcvaUKgCq46qyc5lvlOpAfNrGtskB4RueUp19Qn0KRNxy6TzIbStDDTmtVZ
			j2jOMur7/nTmjQnUVatPpqNWI/DbVtsKy/dRqXh3htHtuDuFLHKu10FLG6vd
			6anh2l2BwLlscMWcwVJmUnf9h113Jc378co1qsWu02WXrtf9zx5y8XXgfwdv
			iDt/cuFN4PRRbaw/mfZ/AIgneT17AgAA
			</data>
			<data>
			9QAAAAAAAAAfiwgAAAAAAAAHbZFRT8IwEMef4VPUvrNjREBMKVGGCQnRJY4H
			n0yzldk41toWx7693WAOHU9398/v/r27ksVxn6Fvro2Q+Rz73hAjnscyEXk6
			x9voaXCHF7RPboKXZfQWrpDKhLEo3D5u1kuEBwAPSmUcIIgCFG7WrxFyHgCr
			Z4zwh7XqHqAoCo9VlBfLfQUaCLVUXNty48wGrsFLbILdMyf3P+M4NRGxpf0e
			+eQlTQhUwVVn9STHMt+J9KCZdY0N0iMitzzlmvoEmrTp2GWS2VCaFmZaszrr
			Ec1ZRn3fn868MYG6avXJdNRqBH7balth+T4qFe/OMLodd6eQRc71OmhpY7U7
			PTVcuysQOJcNrpgzWMpM6q7/sOuupHk/XrlGtdh1uuzS9br/2UMuvg78cvCG
			mPmTySVP4PRRbaw/mfZ/AP9SDch7AgAA
			</data>
		</array>
		<array>
			<data>
			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
			HSv4cpA/71/u8/PX/wBZ5gpSFAsAAA==
			</data>
		</array>
		<array>
			<data>
			qAAAAAAAAAAfiwgAAAAAAAAHjVLLbsIwEDzDV6S5ExMehVQmqAUqIaESiXDo
			qXKThbo1tmub19/XBNKkTStxsnd2dnc8azw8bJizA6Wp4APX95quAzwRKeXr
			gbuMHxt9dxjW8c14Poqfo4kjGdXGiZYPs+nIcRsI3UvJAKFxPHai2XQRO7YH
			QpMn13HfjJF3CO33e4+cWF4iNieiRpESEpQ5zmyzhi3wUpO6dsy5+w85Fk1p
			YsJ6DX/AMUwxOh02uqBnOBGUj8SWmzxdw5QbWIMKmxjl15y9YlRKSAvuijAN
			qJQXxERCFwSiFMluNayAsND3/cD3uhhlUYHf9joFhtF3Wdb2JEORxNi3zV/f
			ITHxUUJVcLcqmOopX2r4V7DYc1DTcZHXRtkNhhqUNROjS5jTJbEDRoIJdZVd
			UuiXQ5WZefA3+1hlZ85UuIomEFO4TocmO0velGw4e95v+14ruO12et1m0GsF
			QWkFWR0wtmDC/FpnYd+W088tlP3LRwd+q10WgtH52xVn9mXD+hfYPOqCSQMA
			AA==
			</data>
		</array>
		<array>
			<data>
			0gAAAAAAAAAfiwgAAAAAAAAHjVRNc9owED2HX+H6jmUD/uo4zjSQpswwxdOa
			Q08dxV6IWmG5kvj695UhYHkQmZwkrZ7ePu1bKXnYr6m1BS4Iq+5tz3FtC6qC
			laRa3duL/Gs/sh/SXvJpMh/nv7Inq6ZESCtbPM6mY8vuI/SlrikgNMknVjab
			/swtxYHQ03fbsl+lrD8jtNvtHNygnIKtG6BAGWc1cHmYKbK+OuCUsrRVmhN7
			R46KlqSQae8u+QuHtExQM6jVW/QUXlJS13DZvEuWmApA2j7DMmOiBWDO8XF2
			l3DANPU8L/IcP0HHVRsPwmEbS9Dl2JGWVBI4LqRSO3/5A4XMDzW0OZrtFfA0
			TNB5ejkpptVCwE3BFLZAr5ncayZKVq9yoorRos+lOQEKVklOXjaNymdOyjkn
			K1I5+wu+pVdFCOJOincpDgaKIHANBFf11w246cDFgoFuC9Ks0wrA4eiDQZJJ
			0BrvH+kGTBcYmeHPHMDEPhgF5gPfAMuPq/mhda9G7vsGeM3E7xv2Rd4NvNGr
			prBXaI5LshEmehN6U5F/G5hODPh4GA27/Yo6j5ayAtOMkwLmy6UAKW51sOeO
			Yo3/1C2OG0au6wfBIAw9bxRG+jPt5GG7CrimMBGqmatVKoCrryZBb8szvMZK
			8phRxj/0/rpmvOOFwQrNieE1VuAt5GStfRKn5xD7A2c0iKPA9/1hHPqednOj
			JbojHQ/PhWrH4w+c9v4DxwcfzRgGAAA=
			</data>
		</array>
		<array>
			<data>
			7AAAAAAAAAAfiwgAAAAAAAAHbZFRT8IwEMef4VPUvrNjEBFNKVGGCQmRJY4H
			n0yzFWwcbW2LY9/eMphDx9Pd/fO7f++uZHrY5eibGyuUnOAw6GPEZaoyIbcT
			vE6ee2M8pV1yE61myVs8RzoX1qF4/bRczBDuATxqnXOAKIlQvFy8Jsh7AMxf
			MMIfzukHgKIoAnakglTtjqCF2CjNjSuX3qznG4LMZdg/c3L/M45XM5E62u2Q
			T17SjMAx+OqsnuRUyY3Y7g1zvrFGOkRIx7fc0AGBOq07NrliLla2gZkxrMo6
			xHCW0zAMx/3glkBVNfrobtBoBH7bKlvh+C4pNb8yw3DUnkIVkptF1NDWGX96
			arnxVyBwLmtcM28wU7kybf9+210r+35ok9Vi1+myTVfr/mf3Unzt+eXgNXEf
			joaXPIHTRzWx+mTa/QFcfO9bewIAAA==
			</data>
			<data>
			7AAAAAAAAAAfiwgAAAAAAAAHbZFRT8IwEMef4VPUvrNjooimlCjDhITgEseD
			T6bZCjaOtrbFsW9vGcyh4+nu/vndv3dXMtlvc/TNjRVKjnEY9DHiMlWZkJsx
			XiXPvRGe0C65il6myVs8QzoX1qF49bSYTxHuATxqnXOAKIlQvJi/Jsh7AMyW
			GOEP5/QDQFEUATtQQaq2B9BCbJTmxpULb9bzDUHmMuyfObr/GcermUgd7XbI
			Jy9pRuAQfHVSj3Kq5FpsdoY531gjHSKk4xtu6DWBOq071rliLla2gZkxrMo6
			xHCW0zAMR/3glkBVNfrwbtBoBH7bKlvh+DYpNb8ww2DYnkIVkpt51NDWGX96
			arnxVyBwKmtcM28wVbkybf9+210r+75vk9Vil+myTVfr/md3Unzt+PngNXEf
			Dm/OeQLHj2pi9cm0+wP0hamFewIAAA==
			</data>
			<data>
			7AAAAAAAAAAfiwgAAAAAAAAHbZFRT8IwEMef4VPUvrNjCoimlCjDhITgEseD
			T6bZCjaOtrbFsW9vGcyh8+nu/vndv3dXMj3scvTFjRVKTnAY9DHiMlWZkNsJ
			XidPvTGe0i65ip5nyWs8RzoX1qF4/bhczBDuATxonXOAKIlQvFy8JMh7AMxX
			GOF35/Q9QFEUATtSQap2R9BCbJTmxpVLb9bzDUHmMuyfObn/GsermUgd7XbI
			By9pRuAYfHVWT3Kq5EZs94Y531gjHSKk41tu6IBAndYdm1wxFyvbwMwYVmUd
			YjjLaRiG434wJFBVjT66HTQagZ+2ylY4vktKzdszXN+M2lOoQnKziBraOuNP
			Ty03/goEzmWNa+YNZipXpu3fb7trZd8ObbJa7H+6bNPVun/ZvRSfe345eE3c
			haPhJU/g9FFNrD6Zdr8BoUi1JXsCAAA=
			</data>
		</array>
		<array>
			<data>
			yAAAAAAAAAAfiwgAAAAAAAAHdVLBcoIwED3rV1DuEgG12ok4Vu0MM44yLR56
			6qSwWlokaRKL/H0DqGCxwyHZt2/37T6CJ8d9rP0AFxFNxrppdHUNkoCGUbIb
			6xv/qTPUJ04b383XM//VW2gsjoTUvM3j0p1pegehKWMxIDT355q3dF98TfVA
			aLHSNf1DSvaAUJqmBslZRkD3OVEgj1MGXGZL1ayjCoxQhrqSKbtfjaPQMAqk
			027hL8icEKP8UNEJLeFtHDEGl2QLb0ksANXylEiPiopAOCfFrYU5kNgxTXNk
			G32MiqjCB/e9CsPoUla0jRIJnARSTbt+/4RA+hmDSiNP74A7A4zO10ulcJON
			gH8HpmkC3J1XeSG5+ieOAK7swegUnumMKIEZjSlvqneb6oyKt2OTWXhwm53d
			2Cp35i9XkB/wo31tsdLFoW0ZvYFt9dVndfumXTM1Z0o4ysay/vPUXTV2PSTR
			9wHq3pyHGJnW1UgYlY+kOosH5rR/ASXK84X3AgAA
			</data>
		</array>
		<array>
			<data>
			owAAAAAAAAAfiwgAAAAAAAAHjZJdT8IwFIav4VfM3bNujK+ZMqKACQmRJY4L
			r0zdDlgdbW2LsH9vGcxNp4lX7Tl9ztv3nBZPjrvM+gCpKGdj23Nc2wKW8JSy
			7dhex3edkT0J2/hqtprGj9HcEhlV2orWt8vF1LI7CN0IkQFCs3hmRcvFQ2wZ
			DYTm97Zlv2gtrhE6HA4OOVFOwncnUKFIcgFS50sj1jEFTqpT21xzVv9mx2RT
			muiw3cJvkIcpRqfFRJfsOZ1wxiDRpirOBZRMC1OmYQsy7GFUbsuSTUaFgLRi
			tdwDqh1zoiOuqnMiJSl2LSyBZKHneYHr9DEqoio/GPpVDqOvskL25EKSwunq
			+dVY/rdfqhZsrWrshmSqZphvNgr+kOs25fiBgVzMKlZpaR49VCDN/DG6hCUu
			iBGY8ozLprrbVBdcPR2bZDGy3+m8SReD/Mkq8gEx3dWaPA995PtONxj0/WEv
			GHi9oPYuRZ2Go24022hzz+j7HupjKe8P3FFQd4PR+QNWa/F5w/YnX2plSFMD
			AAA=
			</data>
		</array>
		<array>
			<data>
			LQEAAAAAAAAfiwgAAAAAAAAHhVNNU4MwED23vyJyLzH91ok4teBMZzqWUXrw
			5ETYKkqTmERp/72BilTB8ZTs5u3Le7sJvdxtM/QBSqeCXzjEPXUQ8FgkKX+6
			cNbRdW/qXHpdeuKv5tF9GCCZpdqgcH21XMyR08N4JmUGGPuRj8Ll4i5ClgPj
			4MZBzrMx8hzjPM9dVqDcWGwLoMahEhKU2S8tWc8WuIlJHHvNgf2HHJtN0th4
			3Q59hb2XUFwsNvrKHtKx4BxiY6uivYQK06EpN/AEyhtSXG2rkk2WSglJjd2w
			TAM+OhfMhELXAKYUK3cdqoBlHiHkbOSOKC6jOj+eDOocxd9lJe1zUxxpiiti
			xUpDq8cX66zd1rSlUi/4WsOftsRmo+EPuhYhIuegFn6N1UbZt+FpUHZMFH+F
			FVwySzAXmVBN9tMmuxT6Ydeio2hsO3rfRJft/o3V7AOidHtk8jCa6WDoDseD
			PhmNxhMyIeOjSRVIAzvTMNud3QYzFN6uomAeBX7D9TtP397huEuVnDPS7zfF
			5f+0nuLD467X8mN43U+LZpjwrwMAAA==
			</data>
		</array>
		<array/>
		<array/>
		<array/>
		<array/>
		<array/>
    </array>
</dict>
</plist>";
        chest_save_dict_round_trip(data.as_slice());
    }
}
