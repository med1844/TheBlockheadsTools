use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use typed_floats::NonNaNFinite;

// Rust doesn't have inheritance, yet the game was build on that.
// Thus we have to emulate that, and thankfully it's not too hard.
macro_rules! inherit {
    ($child:ident -> $parent:ty, $field:ident) => {
        impl Deref for $child {
            type Target = $parent;
            fn deref(&self) -> &Self::Target {
                &self.$field
            }
        }

        impl DerefMut for $child {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$field
            }
        }
    };

    ($child:ident -> $parent:ty) => {
        impl Deref for $child {
            type Target = $parent;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $child {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

// We need a root struct to match the plist's top-level dictionary
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DynamicObjectList<T> {
    #[serde(rename = "dynamicObjects")]
    dynamic_objects: Vec<T>,
}

impl<T> Deref for DynamicObjectList<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.dynamic_objects
    }
}

impl<T> DerefMut for DynamicObjectList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.dynamic_objects
    }
}

impl<T> Default for DynamicObjectList<T> {
    fn default() -> Self {
        Self {
            dynamic_objects: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(transparent)]
pub struct UniqueID(u64);
inherit!(UniqueID -> u64);

impl UniqueID {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn inner(&self) -> &u64 {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicObject {
    #[serde(rename = "floatPos")]
    pub float_pos: [NonNaNFinite<f32>; 2],
    pub pos_x: u64,
    pub pos_y: u16,
    #[serde(rename = "uniqueID")]
    pub unique_id: UniqueID,
}

// Corresponds to Plant
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plant {
    #[serde(flatten)]
    pub obj: DynamicObject,
    pub save_time: f64,
    pub season_offset: i32,
    pub gather_progress: i32,
    pub has_flowered_this_season: bool,
    pub flowering: bool,
    pub frozen: bool,
    pub age: f32,
    pub max_age: f32,
    pub max_age_gene: u16,
    pub growth_rate: f32,
    pub growth_rate_gene: u16,
}

inherit!(Plant -> DynamicObject, obj);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtificialLight {
    pub max_red: u32,
    pub max_green: u32,
    pub max_blue: u32,
    pub max_heat: u32,
    pub radius: u32,
    #[serde(rename = "contributionGridOrigin.x")]
    pub contribution_grid_origin_x: i32,
    #[serde(rename = "contributionGridOrigin.y")]
    pub contribution_grid_origin_y: i32,
    pub light_direction: LightDirection,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalPlant {
    #[serde(flatten)]
    pub plant: Plant,
    pub available_food: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light: Option<ArtificialLight>,
}

inherit!(NormalPlant -> Plant, plant);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CornPlant(pub NormalPlant);
inherit!(CornPlant -> NormalPlant);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CarrotPlant(pub NormalPlant);
inherit!(CarrotPlant -> NormalPlant);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TomatoPlant(pub NormalPlant);
inherit!(TomatoPlant -> NormalPlant);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum LightDirection {
    All = 0,
    Down = 1,
    Up = 2,
}

// NOTE: final_goal_square_x/y, load_requires_recalculation are optional and needs serde(default)
// which doesn't work together with serde(flatten), which is needed for DynamicObject.
// Either manually flatten DynamicObject, or remove these fields. For now we go latter.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Blockhead {
    #[serde(flatten)]
    pub obj: DynamicObject,
    pub actions: plist::Value,
    pub clothing_increment_timer: u64,
    pub double_time_unlocked: bool,
    pub interaction_item_index: i64, // could be -1... my god
    pub interaction_item_sub_index: i64,
    pub name: String,
    pub selected_tool_index: u64,
    pub skin_options: plist::Data,
    pub state: plist::Data,
}

#[cfg(test)]
mod tests {
    use super::{DynamicObjectList, TomatoPlant};

    #[test]
    fn test_round_trip() {
        let test_xml_tomato_plant = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>dynamicObjects</key>
    <array>
        <dict>
            <key>age</key>
            <real>12673.24</real>
            <key>availableFood</key>
            <real>1246.764</real>
            <key>floatPos</key>
            <array>
                <real>10729.5</real>
                <real>520</real>
            </array>
            <key>flowering</key>
            <false/>
            <key>frozen</key>
            <false/>
            <key>gatherProgress</key>
            <integer>0</integer>
            <key>growthRate</key>
            <real>0</real>
            <key>growthRateGene</key>
            <integer>183</integer>
            <key>hasFloweredThisSeason</key>
            <false/>
            <key>maxAge</key>
            <real>14916.71</real>
            <key>maxAgeGene</key>
            <integer>166</integer>
            <key>pos_x</key>
            <integer>10729</integer>
            <key>pos_y</key>
            <integer>520</integer>
            <key>saveTime</key>
            <real>6269.400037363171</real>
            <key>seasonOffset</key>
            <integer>5</integer>
            <key>uniqueID</key>
            <integer>165</integer>
        </dict>
        <dict>
            <key>age</key>
            <real>15333.81</real>
            <key>availableFood</key>
            <real>1749.358</real>
            <key>floatPos</key>
            <array>
                <real>10730.5</real>
                <real>520</real>
            </array>
            <key>flowering</key>
            <false/>
            <key>frozen</key>
            <false/>
            <key>gatherProgress</key>
            <integer>0</integer>
            <key>growthRate</key>
            <real>0</real>
            <key>growthRateGene</key>
            <integer>192</integer>
            <key>hasFloweredThisSeason</key>
            <false/>
            <key>maxAge</key>
            <real>15120</real>
            <key>maxAgeGene</key>
            <integer>170</integer>
            <key>pos_x</key>
            <integer>10730</integer>
            <key>pos_y</key>
            <integer>520</integer>
            <key>saveTime</key>
            <real>6269.400037363171</real>
            <key>seasonOffset</key>
            <integer>24</integer>
            <key>uniqueID</key>
            <integer>166</integer>
        </dict>
        <dict>
            <key>age</key>
            <real>9545.338</real>
            <key>availableFood</key>
            <real>1377.515</real>
            <key>floatPos</key>
            <array>
                <real>10735.5</real>
                <real>520</real>
            </array>
            <key>flowering</key>
            <true/>
            <key>frozen</key>
            <false/>
            <key>gatherProgress</key>
            <integer>0</integer>
            <key>growthRate</key>
            <real>0</real>
            <key>growthRateGene</key>
            <integer>190</integer>
            <key>hasFloweredThisSeason</key>
            <true/>
            <key>maxAge</key>
            <real>14459.29</real>
            <key>maxAgeGene</key>
            <integer>157</integer>
            <key>pos_x</key>
            <integer>10735</integer>
            <key>pos_y</key>
            <integer>520</integer>
            <key>saveTime</key>
            <real>6269.400037363171</real>
            <key>seasonOffset</key>
            <integer>-3</integer>
            <key>uniqueID</key>
            <integer>167</integer>
        </dict>
    </array>
</dict>
</plist>"#;
        let tomatos =
            plist::from_bytes::<DynamicObjectList<TomatoPlant>>(test_xml_tomato_plant.as_bytes())
                .unwrap();
        let mut serialized = Vec::with_capacity(test_xml_tomato_plant.as_bytes().len());
        plist::to_writer_xml(&mut serialized, &tomatos).unwrap();
        let tomatos_round_triped =
            plist::from_bytes::<DynamicObjectList<TomatoPlant>>(&serialized).unwrap();
        assert_eq!(tomatos, tomatos_round_triped);
    }
}
