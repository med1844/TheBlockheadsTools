use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicWorldV2 {
    #[serde(rename = "activeBlockheadIndex")]
    pub active_blockhead_index: u64,
    #[serde(rename = "dynamicObjectIDCount")]
    pub dynamic_object_id_count: u64,
    #[serde(rename = "saveVersion")]
    pub save_version: u8,
    #[serde(rename = "savedGlowIndices")]
    pub saved_glow_indices: plist::Data, // bplist
    #[serde(rename = "workbenchHasBeenCrafted")]
    pub workbench_has_been_crafted: bool,
}
