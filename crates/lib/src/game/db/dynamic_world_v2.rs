use super::super::dynamic_object::UniqueID;
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

impl DynamicWorldV2 {
    pub fn new_unique_id(&mut self) -> UniqueID {
        let unique_id = UniqueID::new(self.dynamic_object_id_count);
        self.dynamic_object_id_count += 1;
        unique_id
    }
}
