use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldV2 {
    #[serde(rename = "blockheadDatasv2")]
    pub blockhead_datas_v2: plist::Value,
    pub circum_navigate_booleans_data: plist::Data, // bplist dict
    pub creation_date: plist::Date,
    pub distance_ordered_food_types: plist::Data, // suspect: Vec<ItemId>, where ItemId = u32
    pub expert_mode: bool,
    pub found_items: plist::Data, // bplist dict
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_port: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_players: Option<String>,
    #[serde(rename = "migrationComplete_1.7")]
    pub migration_complete_v1_7: bool,
    pub no_rain_timer: f64,
    pub portal_level: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_image_data: Option<plist::Data>,
    pub random_seed: u64,
    pub remote_game: bool,
    pub run_at_launch: bool,
    pub save_date: plist::Date,
    #[serde(rename = "saveID")]
    pub save_id: String,
    pub save_version: u64,
    #[serde(rename = "startPortalPos.x")]
    pub start_portal_pos_x: u64,
    #[serde(rename = "startPortalPos.y")]
    pub start_portal_pos_y: u64,
    pub translation: (f64, f64),
    pub world_name: String,
    pub world_time: f64,
    pub world_width_macro: u32,
}
