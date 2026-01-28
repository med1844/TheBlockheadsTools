use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WorldV2 {
    #[serde(rename = "blockheadDatasv2")]
    pub blockhead_datas_v2: plist::Value,
    #[serde(rename = "circumNavigateBooleansData")]
    pub circum_navigate_booleans_data: plist::Data, // bplist dict
    #[serde(rename = "creationDate")]
    pub creation_date: plist::Date,
    #[serde(rename = "distanceOrderedFoodTypes")]
    pub distance_ordered_food_types: plist::Data, // suspect: Vec<ItemId>, where ItemId = u32
    #[serde(rename = "expertMode")]
    pub expert_mode: bool,
    #[serde(rename = "foundItems")]
    pub found_items: plist::Data, // bplist dict
    #[serde(rename = "hostPort")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_port: Option<String>,
    #[serde(rename = "maxPlayers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_players: Option<String>,
    #[serde(rename = "migrationComplete_1.7")]
    pub migration_complete_v1_7: bool,
    #[serde(rename = "noRainTimer")]
    pub no_rain_timer: f64,
    #[serde(rename = "portalLevel")]
    pub portal_level: u64,
    #[serde(rename = "randomSeed")]
    pub random_seed: u64,
    #[serde(rename = "remoteGame")]
    pub remote_game: bool,
    #[serde(rename = "runAtLaunch")]
    pub run_at_launch: bool,
    #[serde(rename = "saveDate")]
    pub save_date: plist::Date,
    #[serde(rename = "saveID")]
    pub save_id: String,
    #[serde(rename = "saveVersion")]
    pub save_version: u64,
    #[serde(rename = "startPortalPos.x")]
    pub start_portal_pos_x: u64,
    #[serde(rename = "startPortalPos.y")]
    pub start_portal_pos_y: u64,
    #[serde(rename = "translation")]
    pub translation: (f64, f64),
    #[serde(rename = "worldName")]
    pub world_name: String,
    #[serde(rename = "worldTime")]
    pub world_time: f64,
    #[serde(rename = "worldWidthMacro")]
    pub world_width_macro: u32,
}
