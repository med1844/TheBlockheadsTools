use super::{
    super::item::ItemType, ArtificialLight, DynamicObject, InteractionObject,
    InteractionObjectType, deserialize_some, serialize_some,
};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(i8)]
// Where does the torch connects to
pub enum TorchConnectionType {
    Bg = -2,
    Left = -1,
    Ground = 0,
    Right = 1,
    Mg = 2,
    // source code in torchConnectionTypeForPos suggests existence of 3 and 5
    // 3 likely means column, 5 likely means top for chandeliers
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Torch {
    #[serde(flatten)]
    obj: DynamicObject,
    pub light_dict: ArtificialLight,
    pub connection_type: TorchConnectionType,
    pub item_type: ItemType,
    pub data_a: u16,
    pub data_b: u16,
}
inherit!(Torch -> DynamicObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ladder {
    #[serde(flatten)]
    obj: DynamicObject,
    pub paint_color: u16,
    pub item_type: ItemType,
}
inherit!(Ladder -> DynamicObject, obj);

impl Default for Ladder {
    fn default() -> Self {
        Self {
            obj: DynamicObject::default(),
            paint_color: Default::default(),
            item_type: ItemType::Ladder,
        }
    }
}

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

impl Default for Door {
    fn default() -> Self {
        Self {
            obj: DynamicObject::default(),
            item_type: ItemType::Door,
            blocked: Default::default(),
            iron_place_client_id: Option::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bed {
    #[serde(flatten)]
    obj: InteractionObject,
    pub item_type: ItemType,
    pub bedding_color: u16,
}
inherit!(Bed -> InteractionObject, obj);

impl Default for Bed {
    fn default() -> Self {
        Self {
            obj: InteractionObject {
                interaction_object_type: InteractionObjectType::Bed,
                ..Default::default()
            },
            item_type: ItemType::Bed,
            bedding_color: Default::default(),
        }
    }
}

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
// How does the sign connects
pub enum SignConnectionType {
    None = 0,         // standalone item
    GroundDouble = 1, // connects to sign or block below, standing on both side
    GroundSingle = 2, // connects to sign or block below, standing on single leg
    #[default]
    Front = 3, // connects to front face of some block
    Side = 4,         // connects to left or right face of some block
    Up = 5,           // connects to sign or block above
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sign {
    #[serde(flatten)]
    obj: InteractionObject,
    pub text: String,
    pub connection_type: SignConnectionType,
    pub offset_type: u64,
}
inherit!(Sign -> InteractionObject, obj);

impl Default for Sign {
    fn default() -> Self {
        Self {
            obj: InteractionObject {
                interaction_object_type: InteractionObjectType::Sign,
                ..Default::default()
            },
            text: String::default(),
            connection_type: SignConnectionType::default(),
            offset_type: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingPost {
    #[serde(flatten)]
    obj: InteractionObject,
    pub coin_count: u32,
    pub price_tier: u32,
    pub sell_slot: plist::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seller_client_name: Option<String>,
}
inherit!(TradingPost -> InteractionObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradePortal {
    #[serde(flatten)]
    obj: InteractionObject,
    pub level: u32,
    pub local_price_offsets: plist::Dictionary,
    pub light_dict: ArtificialLight,
}
inherit!(TradePortal -> InteractionObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Painting {
    #[serde(flatten)]
    obj: DynamicObject,
    pub has_verified_image_data: bool,
    pub output_image_data: plist::Data,
    pub item_type: ItemType,
}
inherit!(Painting -> DynamicObject, obj);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ColumnConfiguration {
    Undefined = 0,
    NoPlinth = 1,
    PlinthBelow = 2,
    PlinthAbove = 3,
    PlinthAboveAndBelow = 4,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    #[serde(flatten)]
    obj: DynamicObject,
    pub item_type: ItemType,
    pub paint_color: u16,
    pub configuration: ColumnConfiguration,
}
inherit!(Column -> DynamicObject, obj);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum StairsConfiguration {
    Undefined = 0,
    HighRightSolid = 1,
    HighLeftSolid = 2,
    HighRightFloating = 3,
    HighLeftFloating = 4,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stairs {
    #[serde(flatten)]
    obj: DynamicObject,
    pub item_type: ItemType,
    pub paint_color: u16,
    pub configuration: StairsConfiguration,
}
inherit!(Stairs -> DynamicObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElevatorMotor {
    #[serde(flatten)]
    obj: DynamicObject,
    pub item_type: ItemType,
    pub available_electricity: u16,
    pub min_y: u16,
    pub max_y: u16,
}
inherit!(ElevatorMotor -> DynamicObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElevatorShaft {
    #[serde(flatten)]
    obj: DynamicObject,
    pub item_type: ItemType,
    #[serde(rename = "lastKnownMotorPos.x")]
    pub last_known_motor_pos_x: i32,
    #[serde(rename = "lastKnownMotorPos.y")]
    pub last_known_motor_pos_y: u16,
    pub paint_color: u16,
}
inherit!(ElevatorShaft -> DynamicObject, obj);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnershipSign {
    #[serde(flatten)]
    sign: Sign,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none",
        rename = "landOwnerID"
    )]
    pub land_owner_id: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub land_owner_name: Option<String>,
    pub w: u8,
    pub h: u8,
}
inherit!(OwnershipSign -> Sign, sign);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mirror {
    #[serde(flatten)]
    obj: InteractionObject,
}
inherit!(Mirror -> InteractionObject, obj);
