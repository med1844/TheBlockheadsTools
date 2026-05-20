use super::{super::item::Inventory, DynamicObject};
use crate::util::serde::{deserialize_some, serialize_some};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlockheadXml {
    #[serde(flatten)]
    obj: DynamicObject,
    actions: plist::Value,
    clothing_increment_timer: u64,
    double_time_unlocked: bool,
    interaction_item_index: i32, // could be -1... my god
    interaction_item_sub_index: i32,
    name: String,
    selected_tool_index: u64,
    skin_options: plist::Data,
    state: plist::Data,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none",
        rename = "finalGoalSquare.x"
    )]
    final_goal_square_x: Option<u64>,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none",
        rename = "finalGoalSquare.y"
    )]
    final_goal_square_y: Option<u64>,
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        serialize_with = "serialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    load_requires_recalculation: Option<bool>,
}
inherit!(BlockheadXml -> DynamicObject, obj);

#[derive(Debug, Clone, PartialEq)]
pub struct Blockhead {
    obj: DynamicObject,
    pub actions: plist::Value,
    pub clothing_increment_timer: u64,
    pub double_time_unlocked: bool,
    pub interaction_item_index: i32, // could be -1... my god
    pub interaction_item_sub_index: i32,
    pub name: String,
    pub selected_tool_index: u64,
    pub skin_options: plist::Data,
    pub state: plist::Data,
    pub final_goal_square_x: Option<u64>,
    pub final_goal_square_y: Option<u64>,
    pub load_requires_recalculation: Option<bool>,
    pub inventory: Inventory,
}
inherit!(Blockhead -> DynamicObject, obj);

impl Blockhead {
    pub const INVENTORY_NUM_SLOTS: usize = 8;

    pub(crate) fn from_xml_and_inventory(xml: BlockheadXml, inventory: Inventory) -> Self {
        Self {
            obj: xml.obj,
            actions: xml.actions,
            clothing_increment_timer: xml.clothing_increment_timer,
            double_time_unlocked: xml.double_time_unlocked,
            interaction_item_index: xml.interaction_item_index,
            interaction_item_sub_index: xml.interaction_item_sub_index,
            name: xml.name,
            selected_tool_index: xml.selected_tool_index,
            skin_options: xml.skin_options,
            state: xml.state,
            final_goal_square_x: xml.final_goal_square_x,
            final_goal_square_y: xml.final_goal_square_y,
            load_requires_recalculation: xml.load_requires_recalculation,
            inventory,
        }
    }
}

impl From<&Blockhead> for BlockheadXml {
    fn from(value: &Blockhead) -> Self {
        Self {
            obj: value.obj.clone(),
            actions: value.actions.clone(),
            clothing_increment_timer: value.clothing_increment_timer,
            double_time_unlocked: value.double_time_unlocked,
            interaction_item_index: value.interaction_item_index,
            interaction_item_sub_index: value.interaction_item_sub_index,
            name: value.name.clone(),
            selected_tool_index: value.selected_tool_index,
            skin_options: value.skin_options.clone(),
            state: value.state.clone(),
            final_goal_square_x: value.final_goal_square_x,
            final_goal_square_y: value.final_goal_square_y,
            load_requires_recalculation: value.load_requires_recalculation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::DynamicObjectList, BlockheadXml};
    use crate::util::plist::{diff_plist_keys, to_xml_plist};

    #[test]
    fn blockhead_xml_round_trip_test() {
        let blockheads_data = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
        <key>dynamicObjects</key>
        <array>
                <dict>
                        <key>actions</key>
                        <array>
                                <dict>
                                        <key>craftCountOrExtraData</key>
                                        <integer>0</integer>
                                        <key>goalInteraction</key>
                                        <integer>0</integer>
                                        <key>goalTilePos.x</key>
                                        <integer>11192</integer>
                                        <key>goalTilePos.y</key>
                                        <integer>642</integer>
                                        <key>inProgress</key>
                                        <true/>
                                        <key>interactionItemIndex</key>
                                        <integer>0</integer>
                                        <key>interactionItemSubIndex</key>
                                        <integer>-1</integer>
                                        <key>interactionItemType</key>
                                        <integer>0</integer>
                                        <key>interactionObjectID</key>
                                        <integer>0</integer>
                                        <key>interactionTestResult</key>
                                        <data>
                                        AAAAAAAAAAAAAAAA
                                        </data>
                                        <key>inventoryChange</key>
                                        <dict/>
                                        <key>isAI</key>
                                        <false/>
                                        <key>pathType</key>
                                        <integer>0</integer>
                                </dict>
                        </array>
                        <key>clothingIncrementTimer</key>
                        <integer>0</integer>
                        <key>doubleTimeUnlocked</key>
                        <false/>
                        <key>finalGoalSquare.x</key>
                        <integer>11192</integer>
                        <key>finalGoalSquare.y</key>
                        <integer>642</integer>
                        <key>floatPos</key>
                        <array>
                                <real>11183.259765625</real>
                                <real>638.39410400390625</real>
                        </array>
                        <key>interactionItemIndex</key>
                        <integer>0</integer>
                        <key>interactionItemSubIndex</key>
                        <integer>-1</integer>
                        <key>loadRequiresRecalculation</key>
                        <true/>
                        <key>name</key>
                        <string>JESS</string>
                        <key>pos_x</key>
                        <integer>11182</integer>
                        <key>pos_y</key>
                        <integer>637</integer>
                        <key>selectedToolIndex</key>
                        <integer>0</integer>
                        <key>skinOptions</key>
                        <data>
                        AAADANGIZ/9GJRj/1br7/4dJ6/8=
                        </data>
                        <key>state</key>
                        <data>
                        AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIA/AACA
                        P3lOmT/anpg/AAAAAAAAAAA9JH8/AACAP5qZXD8AAIA/AAAAAAEA
                        AAAAAAAAJjmEwCY5hMAAAAAAAAAAAAAAAAAAAAAA
                        </data>
                        <key>uniqueID</key>
                        <integer>339</integer>
                </dict>
        </array>
</dict>
</plist>";
        let blockheads: DynamicObjectList<BlockheadXml> =
            plist::from_reader_xml(blockheads_data.as_bytes()).expect("should deserialize");
        let round_trip_blockheads_data = to_xml_plist(&blockheads).expect("should serialize");
        let round_trip_blockheads: DynamicObjectList<BlockheadXml> =
            plist::from_reader_xml(round_trip_blockheads_data.as_slice())
                .expect("should deserialize");
        assert_eq!(blockheads, round_trip_blockheads);

        let blockheads_value: plist::Value =
            plist::from_reader_xml(blockheads_data.as_bytes()).expect("should deserialize");
        let round_trip_blockheads_value: plist::Value =
            plist::from_reader_xml(round_trip_blockheads_data.as_slice())
                .expect("should deserialize");
        let mut diffs = Vec::new();
        diff_plist_keys(
            "",
            &blockheads_value,
            &round_trip_blockheads_value,
            &mut diffs,
        );
        assert!(
            diffs.is_empty(),
            "structural fidelity violations:\n{}",
            diffs.join("\n")
        );
    }
}
