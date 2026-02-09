use super::{
    super::coord::ChunkCoord,
    dynamic_object::{CarrotPlant, CornPlant, DynamicObjectList, TomatoPlant},
};
use crate::{BhError, BhResult};
use lmdb_rs::{
    codec::types::{Bytes, Str},
    database::Database,
    txn::{RoTxn, RwTxn},
};
use num_enum::TryFromPrimitive;
use serde::Serialize;
use std::{collections::HashMap, io::Write, ops::Deref};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum DynamicObjectType {
    AppleTree = 1,
    MapleTree = 2,
    MangoTree = 3,
    PineTree = 4,
    CactusTree = 5,
    CoconutTree = 6,
    OrangeTree = 7,
    CherryTree = 8,
    CoffeeTree = 9,
    FlaxPlant = 10,
    SunflowerPlant = 11,
    CornPlant = 12,
    Dodo = 13,
    Item = 14,
    Fire = 16,
    Torch = 17,
    GlowBlock = 18,
    Ladder = 19,
    Door = 20,
    ArtificialLight = 21,
    Bed = 23,
    Dropbear = 25,
    GatherBlock = 26,
    CarrotPlant = 27,
    Donkey = 28,
    Egg = 30,
    Window = 31,
    Boat = 32,
    ChilliPlant = 33,
    KelpPlant = 34,
    ClownFish = 35,
    Shark = 36,
    LimeTree = 37,
    Wire = 38,
    CaveTroll = 39,
    Rail = 40,
    Workbench = 45,
    Chest = 46,
    Sign = 47,
    TradingPost = 48,
    TradePortal = 50,
    Scorpion = 51,
    Column = 53,
    Stairs = 54,
    ElevatorMotor = 55,
    ElevatorShaft = 56,
    GemTree = 57,
    VinePlant = 58,
    TulipPlant = 59,
    WheatPlant = 61,
    TomatoPlant = 62,
    Yak = 63,
}

impl DynamicObjectType {
    fn try_from_str(s: &str) -> BhResult<Self> {
        let value: u16 = s
            .parse()
            .map_err(|_| BhError::ParseError(format!("Dynamic object type {} is invalid", s)))?;
        Self::try_from(value).map_err(|e| BhError::InvalidDynamicOjectId(e.number))
    }
}

trait IsEmpty {
    fn is_empty(&self) -> bool;
}

impl IsEmpty for Vec<u8> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<T> IsEmpty for DynamicObjectList<T> {
    fn is_empty(&self) -> bool {
        self.deref().is_empty()
    }
}

trait ToXmlPlist {
    fn to_plist(&self) -> Vec<u8>;
}

impl ToXmlPlist for Vec<u8> {
    fn to_plist(&self) -> Vec<u8> {
        self.clone()
    }
}

impl<T: Serialize> ToXmlPlist for DynamicObjectList<T> {
    fn to_plist(&self) -> Vec<u8> {
        let mut serialized = Vec::new();
        plist::to_writer_xml(&mut serialized, self).unwrap();
        serialized
    }
}

/// Contains all different types of dynamic objects that one chunk might have.
#[derive(Debug, Default, PartialEq)]
pub struct ChunkDynamicObjects {
    pub apple_tree: Vec<u8>,
    pub maple_tree: Vec<u8>,
    pub mango_tree: Vec<u8>,
    pub pine_tree: Vec<u8>,
    pub cactus_tree: Vec<u8>,
    pub coconut_tree: Vec<u8>,
    pub orange_tree: Vec<u8>,
    pub cherry_tree: Vec<u8>,
    pub coffee_tree: Vec<u8>,
    pub flax_plant: Vec<u8>,
    pub sunflower_plant: Vec<u8>,
    pub corn_plant: DynamicObjectList<CornPlant>,
    pub dodo: Vec<u8>,
    pub item: Vec<u8>,
    pub fire: Vec<u8>,
    pub torch: Vec<u8>,
    pub glow_block: Vec<u8>,
    pub ladder: Vec<u8>,
    pub door: Vec<u8>,
    pub artificial_light: Vec<u8>,
    pub bed: Vec<u8>,
    pub dropbear: Vec<u8>,
    pub gather_block: Vec<u8>,
    pub carrot_plant: DynamicObjectList<CarrotPlant>,
    pub donkey: Vec<u8>,
    pub egg: Vec<u8>,
    pub window: Vec<u8>,
    pub boat: Vec<u8>,
    pub chilli_plant: Vec<u8>,
    pub kelp_plant: Vec<u8>,
    pub clown_fish: Vec<u8>,
    pub shark: Vec<u8>,
    pub lime_tree: Vec<u8>,
    pub wire: Vec<u8>,
    pub cave_troll: Vec<u8>,
    pub rail: Vec<u8>,
    pub workbench: Vec<u8>,
    pub chest: Vec<u8>,
    pub sign: Vec<u8>,
    pub trading_post: Vec<u8>,
    pub trade_portal: Vec<u8>,
    pub scorpion: Vec<u8>,
    pub column: Vec<u8>,
    pub stairs: Vec<u8>,
    pub elevator_motor: Vec<u8>,
    pub elevator_shaft: Vec<u8>,
    pub gem_tree: Vec<u8>,
    pub vine_plant: Vec<u8>,
    pub tulip_plant: Vec<u8>,
    pub wheat_plant: Vec<u8>,
    pub tomato_plant: DynamicObjectList<TomatoPlant>,
    pub yak: Vec<u8>,
}

impl ChunkDynamicObjects {
    pub fn num_objects(&self) -> usize {
        self.tomato_plant.len() + self.corn_plant.len() + self.carrot_plant.len()
    }
}

#[derive(Debug)]
pub struct DynamicWorld(HashMap<ChunkCoord, ChunkDynamicObjects>);

impl DynamicWorld {
    pub fn chunk_at<I: Into<ChunkCoord>>(&self, coord: I) -> Option<&ChunkDynamicObjects> {
        self.0.get(&coord.into())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ChunkCoord, &ChunkDynamicObjects)> {
        self.0.iter()
    }

    pub fn from_db(db: &Database<Str, Bytes>, rtxn: &RoTxn) -> BhResult<Self> {
        let mut map = HashMap::new();
        for (k, v) in db.iter(rtxn)?.filter_map(|v| v.ok()) {
            let Some((coord_str, type_id_str)) = k.split_once("/") else {
                continue;
            };
            let coord = ChunkCoord::try_from_str(coord_str)?;
            let Ok(dyn_obj_type) = DynamicObjectType::try_from_str(type_id_str) else {
                println!(
                    "Found object type {} we don't understand in chunk {}",
                    type_id_str, coord_str
                );
                continue;
            };
            let entry = map
                .entry(coord)
                .or_insert_with(ChunkDynamicObjects::default);
            match dyn_obj_type {
                DynamicObjectType::AppleTree => entry.apple_tree = v.to_vec(),
                DynamicObjectType::MapleTree => entry.maple_tree = v.to_vec(),
                DynamicObjectType::MangoTree => entry.mango_tree = v.to_vec(),
                DynamicObjectType::PineTree => entry.pine_tree = v.to_vec(),
                DynamicObjectType::CactusTree => entry.cactus_tree = v.to_vec(),
                DynamicObjectType::CoconutTree => entry.coconut_tree = v.to_vec(),
                DynamicObjectType::OrangeTree => entry.orange_tree = v.to_vec(),
                DynamicObjectType::CherryTree => entry.cherry_tree = v.to_vec(),
                DynamicObjectType::CoffeeTree => entry.coffee_tree = v.to_vec(),
                DynamicObjectType::FlaxPlant => entry.flax_plant = v.to_vec(),
                DynamicObjectType::SunflowerPlant => entry.sunflower_plant = v.to_vec(),
                DynamicObjectType::CornPlant => entry.corn_plant = plist::from_bytes(v)?,
                DynamicObjectType::Dodo => entry.dodo = v.to_vec(),
                DynamicObjectType::Item => entry.item = v.to_vec(),
                DynamicObjectType::Fire => entry.fire = v.to_vec(),
                DynamicObjectType::Torch => entry.torch = v.to_vec(),
                DynamicObjectType::GlowBlock => entry.glow_block = v.to_vec(),
                DynamicObjectType::Ladder => entry.ladder = v.to_vec(),
                DynamicObjectType::Door => entry.door = v.to_vec(),
                DynamicObjectType::ArtificialLight => entry.artificial_light = v.to_vec(),
                DynamicObjectType::Bed => entry.bed = v.to_vec(),
                DynamicObjectType::Dropbear => entry.dropbear = v.to_vec(),
                DynamicObjectType::GatherBlock => entry.gather_block = v.to_vec(),
                DynamicObjectType::CarrotPlant => entry.carrot_plant = plist::from_bytes(v)?,
                DynamicObjectType::Donkey => entry.donkey = v.to_vec(),
                DynamicObjectType::Egg => entry.egg = v.to_vec(),
                DynamicObjectType::Window => entry.window = v.to_vec(),
                DynamicObjectType::Boat => entry.boat = v.to_vec(),
                DynamicObjectType::ChilliPlant => entry.chilli_plant = v.to_vec(),
                DynamicObjectType::KelpPlant => entry.kelp_plant = v.to_vec(),
                DynamicObjectType::ClownFish => entry.clown_fish = v.to_vec(),
                DynamicObjectType::Shark => entry.shark = v.to_vec(),
                DynamicObjectType::LimeTree => entry.lime_tree = v.to_vec(),
                DynamicObjectType::Wire => entry.wire = v.to_vec(),
                DynamicObjectType::CaveTroll => entry.cave_troll = v.to_vec(),
                DynamicObjectType::Rail => entry.rail = v.to_vec(),
                DynamicObjectType::Workbench => entry.workbench = v.to_vec(),
                DynamicObjectType::Chest => entry.chest = v.to_vec(),
                DynamicObjectType::Sign => entry.sign = v.to_vec(),
                DynamicObjectType::TradingPost => entry.trading_post = v.to_vec(),
                DynamicObjectType::TradePortal => entry.trade_portal = v.to_vec(),
                DynamicObjectType::Scorpion => entry.scorpion = v.to_vec(),
                DynamicObjectType::Column => entry.column = v.to_vec(),
                DynamicObjectType::Stairs => entry.stairs = v.to_vec(),
                DynamicObjectType::ElevatorMotor => entry.elevator_motor = v.to_vec(),
                DynamicObjectType::ElevatorShaft => entry.elevator_shaft = v.to_vec(),
                DynamicObjectType::GemTree => entry.gem_tree = v.to_vec(),
                DynamicObjectType::VinePlant => entry.vine_plant = v.to_vec(),
                DynamicObjectType::TulipPlant => entry.tulip_plant = v.to_vec(),
                DynamicObjectType::WheatPlant => entry.wheat_plant = v.to_vec(),
                DynamicObjectType::TomatoPlant => entry.tomato_plant = plist::from_bytes(v)?,
                DynamicObjectType::Yak => entry.yak = v.to_vec(),
            }
        }
        Ok(Self(map))
    }

    pub fn to_db<W: Write>(&self, db: &Database<Str, Bytes>, wtxn: &mut RwTxn<W>) -> BhResult<()> {
        #[inline(always)]
        fn put<W: Write, T: ToXmlPlist + IsEmpty>(
            db: &Database<Str, Bytes>,
            wtxn: &mut RwTxn<W>,
            coord_str: &str,
            obj_type: DynamicObjectType,
            value: &T,
        ) -> BhResult<()> {
            if !value.is_empty() {
                db.put(
                    wtxn,
                    &format!("{}/{}", coord_str, obj_type as u16),
                    &value.to_plist(),
                )?;
            }
            Ok(())
        }

        for (coord, obj) in self.0.iter() {
            let coord = coord.to_string();
            use DynamicObjectType::*;
            put(db, wtxn, &coord, AppleTree, &obj.apple_tree)?;
            put(db, wtxn, &coord, MapleTree, &obj.maple_tree)?;
            put(db, wtxn, &coord, MangoTree, &obj.mango_tree)?;
            put(db, wtxn, &coord, PineTree, &obj.pine_tree)?;
            put(db, wtxn, &coord, CactusTree, &obj.cactus_tree)?;
            put(db, wtxn, &coord, CoconutTree, &obj.coconut_tree)?;
            put(db, wtxn, &coord, OrangeTree, &obj.orange_tree)?;
            put(db, wtxn, &coord, CherryTree, &obj.cherry_tree)?;
            put(db, wtxn, &coord, CoffeeTree, &obj.coffee_tree)?;
            put(db, wtxn, &coord, FlaxPlant, &obj.flax_plant)?;
            put(db, wtxn, &coord, SunflowerPlant, &obj.sunflower_plant)?;
            put(db, wtxn, &coord, CornPlant, &obj.corn_plant)?;
            put(db, wtxn, &coord, Dodo, &obj.dodo)?;
            put(db, wtxn, &coord, Item, &obj.item)?;
            put(db, wtxn, &coord, Fire, &obj.fire)?;
            put(db, wtxn, &coord, Torch, &obj.torch)?;
            put(db, wtxn, &coord, GlowBlock, &obj.glow_block)?;
            put(db, wtxn, &coord, Ladder, &obj.ladder)?;
            put(db, wtxn, &coord, Door, &obj.door)?;
            put(db, wtxn, &coord, ArtificialLight, &obj.artificial_light)?;
            put(db, wtxn, &coord, Bed, &obj.bed)?;
            put(db, wtxn, &coord, Dropbear, &obj.dropbear)?;
            put(db, wtxn, &coord, GatherBlock, &obj.gather_block)?;
            put(db, wtxn, &coord, CarrotPlant, &obj.carrot_plant)?;
            put(db, wtxn, &coord, Donkey, &obj.donkey)?;
            put(db, wtxn, &coord, Egg, &obj.egg)?;
            put(db, wtxn, &coord, Window, &obj.window)?;
            put(db, wtxn, &coord, Boat, &obj.boat)?;
            put(db, wtxn, &coord, ChilliPlant, &obj.chilli_plant)?;
            put(db, wtxn, &coord, KelpPlant, &obj.kelp_plant)?;
            put(db, wtxn, &coord, ClownFish, &obj.clown_fish)?;
            put(db, wtxn, &coord, Shark, &obj.shark)?;
            put(db, wtxn, &coord, LimeTree, &obj.lime_tree)?;
            put(db, wtxn, &coord, Wire, &obj.wire)?;
            put(db, wtxn, &coord, CaveTroll, &obj.cave_troll)?;
            put(db, wtxn, &coord, Rail, &obj.rail)?;
            put(db, wtxn, &coord, Workbench, &obj.workbench)?;
            put(db, wtxn, &coord, Chest, &obj.chest)?;
            put(db, wtxn, &coord, Sign, &obj.sign)?;
            put(db, wtxn, &coord, TradingPost, &obj.trading_post)?;
            put(db, wtxn, &coord, TradePortal, &obj.trade_portal)?;
            put(db, wtxn, &coord, Scorpion, &obj.scorpion)?;
            put(db, wtxn, &coord, Column, &obj.column)?;
            put(db, wtxn, &coord, Stairs, &obj.stairs)?;
            put(db, wtxn, &coord, ElevatorMotor, &obj.elevator_motor)?;
            put(db, wtxn, &coord, ElevatorShaft, &obj.elevator_shaft)?;
            put(db, wtxn, &coord, GemTree, &obj.gem_tree)?;
            put(db, wtxn, &coord, VinePlant, &obj.vine_plant)?;
            put(db, wtxn, &coord, TulipPlant, &obj.tulip_plant)?;
            put(db, wtxn, &coord, WheatPlant, &obj.wheat_plant)?;
            put(db, wtxn, &coord, TomatoPlant, &obj.tomato_plant)?;
            put(db, wtxn, &coord, Yak, &obj.yak)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ChunkDynamicObjects, DynamicObjectType, DynamicWorld};
    use crate::game::coord::ChunkCoord;
    use lmdb_rs::arch::DynArch;
    use lmdb_rs::env::{Env, EnvWrite};
    use std::collections::HashMap;

    #[test]
    fn test_dynamic_object_type_parsing() {
        assert_eq!(
            DynamicObjectType::try_from_str("1").unwrap(),
            DynamicObjectType::AppleTree
        );
        assert_eq!(
            DynamicObjectType::try_from_str("12").unwrap(),
            DynamicObjectType::CornPlant
        );
        assert_eq!(
            DynamicObjectType::try_from_str("45").unwrap(),
            DynamicObjectType::Workbench
        );
        assert_eq!(
            DynamicObjectType::try_from_str("46").unwrap(),
            DynamicObjectType::Chest
        );
        assert_eq!(
            DynamicObjectType::try_from_str("63").unwrap(),
            DynamicObjectType::Yak
        );

        assert!(DynamicObjectType::try_from_str("999").is_err());
        assert!(DynamicObjectType::try_from_str("abc").is_err());
    }

    #[test]
    fn test_dynamic_world_round_trip() {
        let tomato_plist_source: &[u8] = b"
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
    <key>dynamicObjects</key>
    <array>
        <dict>
            <key>floatPos</key>
            <array>
                <real>4431.5</real>
                <real>522</real>
            </array>
            <key>pos_x</key>
            <integer>4431</integer>
            <key>pos_y</key>
            <integer>522</integer>
            <key>uniqueID</key>
            <integer>106576</integer>
            <key>saveTime</key>
            <real>5581.80003093183</real>
            <key>seasonOffset</key>
            <integer>-1</integer>
            <key>gatherProgress</key>
            <integer>0</integer>
            <key>hasFloweredThisSeason</key>
            <false/>
            <key>flowering</key>
            <false/>
            <key>frozen</key>
            <false/>
            <key>age</key>
            <real>6353.9091796875</real>
            <key>maxAge</key>
            <real>16644.7109375</real>
            <key>maxAgeGene</key>
            <integer>200</integer>
            <key>growthRate</key>
            <real>0.9686275124549866</real>
            <key>growthRateGene</key>
            <integer>196</integer>
            <key>availableFood</key>
            <real>1046.7919921875</real>
        </dict>
    </array>
</dict>
</plist>";
        let corn_plist_source: &[u8] = b"
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
    <key>dynamicObjects</key>
    <array>
        <dict>
            <key>age</key>
            <real>6684.742</real>
            <key>availableFood</key>
            <real>1436.561</real>
            <key>floatPos</key>
            <array>
                <real>11562.5</real>
                <real>659</real>
            </array>
            <key>flowering</key>
            <false/>
            <key>frozen</key>
            <false/>
            <key>gatherProgress</key>
            <integer>0</integer>
            <key>growthRate</key>
            <real>0.9333333</real>
            <key>growthRateGene</key>
            <integer>187</integer>
            <key>hasFloweredThisSeason</key>
            <false/>
            <key>maxAge</key>
            <real>15882.35</real>
            <key>maxAgeGene</key>
            <integer>185</integer>
            <key>pos_x</key>
            <integer>11562</integer>
            <key>pos_y</key>
            <integer>659</integer>
            <key>saveTime</key>
            <real>5493.800029620528</real>
            <key>seasonOffset</key>
            <integer>6</integer>
            <key>uniqueID</key>
            <integer>129771</integer>
        </dict>
    </array>
</dict>
</plist>";
        let carrot_plist_source: &[u8] = b"
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
    <key>dynamicObjects</key>
    <array>
        <dict>
            <key>age</key>
            <real>16878.66</real>
            <key>availableFood</key>
            <real>88.2</real>
            <key>floatPos</key>
            <array>
                <real>11002.5</real>
                <real>517</real>
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
            <integer>207</integer>
            <key>hasFloweredThisSeason</key>
            <false/>
            <key>maxAge</key>
            <real>16797.18</real>
            <key>maxAgeGene</key>
            <integer>203</integer>
            <key>pos_x</key>
            <integer>11002</integer>
            <key>pos_y</key>
            <integer>517</integer>
            <key>saveTime</key>
            <real>5890.200031712651</real>
            <key>seasonOffset</key>
            <integer>-7</integer>
            <key>uniqueID</key>
            <integer>4804</integer>
        </dict>
    </array>
</dict>
</plist>";

        let mut monster = ChunkDynamicObjects::default();

        if !tomato_plist_source.is_empty() {
            monster.tomato_plant = plist::from_bytes(tomato_plist_source).unwrap();
        }
        if !corn_plist_source.is_empty() {
            monster.corn_plant = plist::from_bytes(corn_plist_source).unwrap();
        }
        if !carrot_plist_source.is_empty() {
            monster.carrot_plant = plist::from_bytes(carrot_plist_source).unwrap();
        }

        monster.apple_tree = vec![1, 0xAA, 0xBB];
        monster.maple_tree = vec![2, 0xAA, 0xBB];
        monster.mango_tree = vec![3, 0xAA, 0xBB];
        monster.pine_tree = vec![4, 0xAA, 0xBB];
        monster.cactus_tree = vec![5, 0xAA, 0xBB];
        monster.coconut_tree = vec![6, 0xAA, 0xBB];
        monster.orange_tree = vec![7, 0xAA, 0xBB];
        monster.cherry_tree = vec![8, 0xAA, 0xBB];
        monster.coffee_tree = vec![9, 0xAA, 0xBB];
        monster.flax_plant = vec![10, 0xAA, 0xBB];
        monster.sunflower_plant = vec![11, 0xAA, 0xBB];
        monster.dodo = vec![13, 0xAA, 0xBB];
        monster.item = vec![14, 0xAA, 0xBB];
        monster.fire = vec![16, 0xAA, 0xBB];
        monster.torch = vec![17, 0xAA, 0xBB];
        monster.glow_block = vec![18, 0xAA, 0xBB];
        monster.ladder = vec![19, 0xAA, 0xBB];
        monster.door = vec![20, 0xAA, 0xBB];
        monster.artificial_light = vec![21, 0xAA, 0xBB];
        monster.bed = vec![23, 0xAA, 0xBB];
        monster.dropbear = vec![25, 0xAA, 0xBB];
        monster.gather_block = vec![26, 0xAA, 0xBB];
        monster.donkey = vec![28, 0xAA, 0xBB];
        monster.egg = vec![30, 0xAA, 0xBB];
        monster.window = vec![31, 0xAA, 0xBB];
        monster.boat = vec![32, 0xAA, 0xBB];
        monster.chilli_plant = vec![33, 0xAA, 0xBB];
        monster.kelp_plant = vec![34, 0xAA, 0xBB];
        monster.clown_fish = vec![35, 0xAA, 0xBB];
        monster.shark = vec![36, 0xAA, 0xBB];
        monster.lime_tree = vec![37, 0xAA, 0xBB];
        monster.wire = vec![38, 0xAA, 0xBB];
        monster.cave_troll = vec![39, 0xAA, 0xBB];
        monster.rail = vec![40, 0xAA, 0xBB];
        monster.workbench = vec![45, 0xAA, 0xBB];
        monster.chest = vec![46, 0xAA, 0xBB];
        monster.sign = vec![47, 0xAA, 0xBB];
        monster.trading_post = vec![48, 0xAA, 0xBB];
        monster.trade_portal = vec![50, 0xAA, 0xBB];
        monster.scorpion = vec![51, 0xAA, 0xBB];
        monster.column = vec![53, 0xAA, 0xBB];
        monster.stairs = vec![54, 0xAA, 0xBB];
        monster.elevator_motor = vec![55, 0xAA, 0xBB];
        monster.elevator_shaft = vec![56, 0xAA, 0xBB];
        monster.gem_tree = vec![57, 0xAA, 0xBB];
        monster.vine_plant = vec![58, 0xAA, 0xBB];
        monster.tulip_plant = vec![59, 0xAA, 0xBB];
        monster.wheat_plant = vec![61, 0xAA, 0xBB];
        monster.yak = vec![63, 0xAA, 0xBB];

        let coord = ChunkCoord::new(10, 20).unwrap();
        let mut map = HashMap::new();
        map.insert(coord, monster);
        let dw = DynamicWorld(map);

        // write to buffer
        let mut buffer = Vec::new();
        {
            let mut env_write = EnvWrite::new(&mut buffer, DynArch::Arch64);
            let mut wtxn = env_write.write_txn().unwrap();
            let db = wtxn.create_database(Some("dw")).unwrap();
            dw.to_db(&db, &mut wtxn).unwrap();
            wtxn.commit().unwrap();
        }

        // read from buffer
        {
            let env = Env::new(&buffer).unwrap();
            let rtxn = env.read_txn().unwrap();
            let db = env
                .open_database::<lmdb_rs::codec::types::Str, lmdb_rs::codec::types::Bytes>(
                    &rtxn,
                    Some("dw"),
                )
                .unwrap()
                .unwrap();

            let coord_str = coord.to_string();
            let check_key = |id: DynamicObjectType| {
                let key = format!("{}/{}", coord_str, id as u16);
                assert!(
                    db.get(&rtxn, &key).unwrap().is_some(),
                    "Key {} missing",
                    key
                );
            };

            check_key(DynamicObjectType::AppleTree);
            check_key(DynamicObjectType::MapleTree);
            check_key(DynamicObjectType::MangoTree);
            check_key(DynamicObjectType::PineTree);
            check_key(DynamicObjectType::CactusTree);
            check_key(DynamicObjectType::CoconutTree);
            check_key(DynamicObjectType::OrangeTree);
            check_key(DynamicObjectType::CherryTree);
            check_key(DynamicObjectType::CoffeeTree);
            check_key(DynamicObjectType::FlaxPlant);
            check_key(DynamicObjectType::SunflowerPlant);
            check_key(DynamicObjectType::CornPlant);
            check_key(DynamicObjectType::Dodo);
            check_key(DynamicObjectType::Item);
            check_key(DynamicObjectType::Fire);
            check_key(DynamicObjectType::Torch);
            check_key(DynamicObjectType::GlowBlock);
            check_key(DynamicObjectType::Ladder);
            check_key(DynamicObjectType::Door);
            check_key(DynamicObjectType::ArtificialLight);
            check_key(DynamicObjectType::Bed);
            check_key(DynamicObjectType::Dropbear);
            check_key(DynamicObjectType::GatherBlock);
            check_key(DynamicObjectType::CarrotPlant);
            check_key(DynamicObjectType::Donkey);
            check_key(DynamicObjectType::Egg);
            check_key(DynamicObjectType::Window);
            check_key(DynamicObjectType::Boat);
            check_key(DynamicObjectType::ChilliPlant);
            check_key(DynamicObjectType::KelpPlant);
            check_key(DynamicObjectType::ClownFish);
            check_key(DynamicObjectType::Shark);
            check_key(DynamicObjectType::LimeTree);
            check_key(DynamicObjectType::Wire);
            check_key(DynamicObjectType::CaveTroll);
            check_key(DynamicObjectType::Rail);
            check_key(DynamicObjectType::Workbench);
            check_key(DynamicObjectType::Chest);
            check_key(DynamicObjectType::Sign);
            check_key(DynamicObjectType::TradingPost);
            check_key(DynamicObjectType::TradePortal);
            check_key(DynamicObjectType::Scorpion);
            check_key(DynamicObjectType::Column);
            check_key(DynamicObjectType::Stairs);
            check_key(DynamicObjectType::ElevatorMotor);
            check_key(DynamicObjectType::ElevatorShaft);
            check_key(DynamicObjectType::GemTree);
            check_key(DynamicObjectType::VinePlant);
            check_key(DynamicObjectType::TulipPlant);
            check_key(DynamicObjectType::WheatPlant);
            check_key(DynamicObjectType::TomatoPlant);
            check_key(DynamicObjectType::Yak);

            let round_tripped_dw = DynamicWorld::from_db(&db, &rtxn).unwrap();
            let round_tripped_dw_chunk = round_tripped_dw.chunk_at(coord).unwrap();

            assert_eq!(round_tripped_dw_chunk, dw.chunk_at(coord).unwrap());
        }
    }
}
