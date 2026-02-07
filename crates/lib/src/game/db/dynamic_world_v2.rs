use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicWorldV2 {
    pub active_blockhead_index: u64,
    #[serde(rename = "dynamicObjectIDCount")]
    pub dynamic_object_id_count: u64,
    pub save_version: u8,
    pub saved_glow_indices: plist::Data, // bplist
    pub workbench_has_been_crafted: bool,
}
