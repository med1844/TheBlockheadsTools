use std::ops::Deref;

use serde::{Deserialize, Serialize};

// We need a root struct to match the plist's top-level dictionary
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DynamicObjectList<T> {
    #[serde(rename = "dynamicObjects")]
    dynamic_objects: Vec<T>,
}

impl<T> DynamicObjectList<T> {
    pub fn new() -> Self {
        Self {
            dynamic_objects: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.dynamic_objects.is_empty()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DynamicObject {
    #[serde(rename = "floatPos")]
    float_pos: [f32; 2],
    pos_x: i32,
    pos_y: i32,
    #[serde(rename = "uniqueID")]
    unique_id: u64,
}

// Corresponds to Plant
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Plant {
    #[serde(flatten)]
    obj: DynamicObject,

    #[serde(rename = "saveTime")]
    save_time: f64,
    #[serde(rename = "seasonOffset")]
    season_offset: i32,
    #[serde(rename = "gatherProgress")]
    gather_progress: i32,
    #[serde(rename = "hasFloweredThisSeason")]
    has_flowered_this_season: bool,
    flowering: bool,
    frozen: bool,
    age: f32,
    #[serde(rename = "maxAge")]
    max_age: f32,
    #[serde(rename = "maxAgeGene")]
    max_age_gene: u16,
    #[serde(rename = "growthRate")]
    growth_rate: f32,
    #[serde(rename = "growthRateGene")]
    growth_rate_gene: u16,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NormalPlant {
    #[serde(flatten)]
    plant: Plant,

    #[serde(rename = "availableFood")]
    available_food: f32,

    #[serde(rename = "lightDict")]
    light: Option<ArtificialLight>,
}

impl Deref for NormalPlant {
    type Target = Plant;

    fn deref(&self) -> &Self::Target {
        &self.plant
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TomatoPlant(NormalPlant);

impl Deref for TomatoPlant {
    type Target = NormalPlant;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum LightDirection {
    All = 0,
    Down = 1,
    Up = 2,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ArtificialLight {
    #[serde(rename = "maxRed")]
    max_red: u32,
    #[serde(rename = "maxGreen")]
    max_green: u32,
    #[serde(rename = "maxBlue")]
    max_blue: u32,
    #[serde(rename = "maxHeat")]
    max_heat: u32,
    radius: u32,
    #[serde(rename = "contributionGridOrigin.x")]
    contribution_grid_origin_x: i32,
    #[serde(rename = "contributionGridOrigin.y")]
    contribution_grid_origin_y: i32,
    #[serde(rename = "lightDirection")]
    light_direction: LightDirection,
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
