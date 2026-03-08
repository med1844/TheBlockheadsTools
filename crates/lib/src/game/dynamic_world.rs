use super::{
    coord::ChunkCoord,
    dynamic_object::{
        DynamicObjectList, DynamicObjectType,
        animal::{CaveTroll, ClownFish, Dodo, Donkey, DropBear, Scorpion, Shark, Yak},
        chest::Chest,
        craft::{
            Bed, Boat, Column, Door, ElevatorMotor, ElevatorShaft, Ladder, Rail, Sign, Stairs,
            TradePortal, TradingPost, Window, Wire,
        },
        plant::{
            CarrotPlant, ChilliPlant, CornPlant, FlaxPlant, KelpPlant, SunflowerPlant, TomatoPlant,
            TulipPlant, VinePlant, WheatPlant,
        },
        train::{FreightCar, HandCar, PassengerCar, SteamLocomotive, TrainStation},
        tree::{
            AppleTree, CactusTree, CherryTree, CoconutTree, CoffeeTree, GemTree, LimeTree,
            MangoTree, MapleTree, OrangeTree, PineTree,
        },
        workbench::Workbench,
    },
};
use crate::BhResult;
use lmdb_rs::{
    codec::types::{Bytes, Str},
    database::Database,
    txn::{RoTxn, RwTxn},
};
use serde::Serialize;
use std::{collections::HashMap, io::Write, ops::Deref};

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
    pub apple_tree: DynamicObjectList<AppleTree>,
    pub maple_tree: DynamicObjectList<MapleTree>,
    pub mango_tree: DynamicObjectList<MangoTree>,
    pub pine_tree: DynamicObjectList<PineTree>,
    pub cactus_tree: DynamicObjectList<CactusTree>,
    pub coconut_tree: DynamicObjectList<CoconutTree>,
    pub orange_tree: DynamicObjectList<OrangeTree>,
    pub cherry_tree: DynamicObjectList<CherryTree>,
    pub coffee_tree: DynamicObjectList<CoffeeTree>,
    pub flax_plant: DynamicObjectList<FlaxPlant>,
    pub sunflower_plant: DynamicObjectList<SunflowerPlant>,
    pub corn_plant: DynamicObjectList<CornPlant>,
    pub dodo: DynamicObjectList<Dodo>,
    pub dropped_item: Vec<u8>,
    pub fire: Vec<u8>,
    pub torch: Vec<u8>,
    pub glow_block: Vec<u8>,
    pub ladder: DynamicObjectList<Ladder>,
    pub door: DynamicObjectList<Door>,
    pub artificial_light: Vec<u8>,
    pub bed: DynamicObjectList<Bed>,
    pub dropbear: DynamicObjectList<DropBear>,
    pub gather_block: Vec<u8>,
    pub carrot_plant: DynamicObjectList<CarrotPlant>,
    pub donkey: DynamicObjectList<Donkey>,
    pub egg: Vec<u8>,
    pub window: DynamicObjectList<Window>,
    pub boat: DynamicObjectList<Boat>,
    pub chilli_plant: DynamicObjectList<ChilliPlant>,
    pub kelp_plant: DynamicObjectList<KelpPlant>,
    pub clown_fish: DynamicObjectList<ClownFish>,
    pub shark: DynamicObjectList<Shark>,
    pub lime_tree: DynamicObjectList<LimeTree>,
    pub wire: DynamicObjectList<Wire>,
    pub cave_troll: DynamicObjectList<CaveTroll>,
    pub rail: DynamicObjectList<Rail>,
    pub hand_car: DynamicObjectList<HandCar>,
    pub steam_locomotive: DynamicObjectList<SteamLocomotive>,
    pub freight_car: DynamicObjectList<FreightCar>,
    pub passenger_car: DynamicObjectList<PassengerCar>,
    pub workbench: DynamicObjectList<Workbench>,
    pub chest: DynamicObjectList<Chest>,
    pub sign: DynamicObjectList<Sign>,
    pub trading_post: DynamicObjectList<TradingPost>,
    pub train_station: DynamicObjectList<TrainStation>,
    pub trade_portal: DynamicObjectList<TradePortal>,
    pub scorpion: DynamicObjectList<Scorpion>,
    pub painting: Vec<u8>,
    pub column: DynamicObjectList<Column>,
    pub stairs: DynamicObjectList<Stairs>,
    pub elevator_motor: DynamicObjectList<ElevatorMotor>,
    pub elevator_shaft: DynamicObjectList<ElevatorShaft>,
    pub gem_tree: DynamicObjectList<GemTree>,
    pub vine_plant: DynamicObjectList<VinePlant>,
    pub tulip_plant: DynamicObjectList<TulipPlant>,
    pub ownership_sign: Vec<u8>,
    pub wheat_plant: DynamicObjectList<WheatPlant>,
    pub tomato_plant: DynamicObjectList<TomatoPlant>,
    pub yak: DynamicObjectList<Yak>,
    pub mirror: Vec<u8>,
}

impl ChunkDynamicObjects {
    pub fn num_objects(&self) -> usize {
        self.apple_tree.num_obj()
            + self.maple_tree.num_obj()
            + self.mango_tree.num_obj()
            + self.pine_tree.num_obj()
            + self.cactus_tree.num_obj()
            + self.coconut_tree.num_obj()
            + self.orange_tree.num_obj()
            + self.cherry_tree.num_obj()
            + self.coffee_tree.num_obj()
            + self.flax_plant.num_obj()
            + self.sunflower_plant.num_obj()
            + self.corn_plant.num_obj()
            + self.dodo.num_obj()
            // + self.item.num_obj()
            // + self.fire.num_obj()
            // + self.torch.num_obj()
            // + self.glow_block.num_obj()
            + self.ladder.num_obj()
            + self.door.num_obj()
            // + self.artificial_light.num_obj()
            + self.bed.num_obj()
            + self.dropbear.num_obj()
            // + self.gather_block.num_obj()
            + self.carrot_plant.num_obj()
            + self.donkey.num_obj()
            // + self.egg.num_obj()
            + self.window.num_obj()
            + self.boat.num_obj()
            + self.chilli_plant.num_obj()
            + self.kelp_plant.num_obj()
            + self.clown_fish.num_obj()
            + self.shark.num_obj()
            + self.lime_tree.num_obj()
            + self.wire.num_obj()
            + self.cave_troll.num_obj()
            + self.rail.num_obj()
            + self.hand_car.num_obj()
            + self.steam_locomotive.num_obj()
            + self.freight_car.num_obj()
            + self.passenger_car.num_obj()
            // + self.workbench.num_obj()
            + self.chest.num_obj()
            + self.sign.num_obj()
            + self.trading_post.num_obj()
            + self.train_station.num_obj()
            + self.trade_portal.num_obj()
            + self.scorpion.num_obj()
            // + self.painting.num_obj()
            + self.column.num_obj()
            + self.stairs.num_obj()
            + self.elevator_motor.num_obj()
            + self.elevator_shaft.num_obj()
            + self.gem_tree.num_obj()
            + self.vine_plant.num_obj()
            + self.tulip_plant.num_obj()
            // + self.ownership_sign.num_obj()
            + self.wheat_plant.num_obj()
            + self.tomato_plant.num_obj()
            + self.yak.num_obj()
        // + self.mirror.num_obj()
    }
}

// TODO: handle chest data like `349_20/chest_834` and `trainchest_1342`
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
                DynamicObjectType::AppleTree => entry.apple_tree = plist::from_bytes(v)?,
                DynamicObjectType::MapleTree => entry.maple_tree = plist::from_bytes(v)?,
                DynamicObjectType::MangoTree => entry.mango_tree = plist::from_bytes(v)?,
                DynamicObjectType::PineTree => entry.pine_tree = plist::from_bytes(v)?,
                DynamicObjectType::CactusTree => entry.cactus_tree = plist::from_bytes(v)?,
                DynamicObjectType::CoconutTree => entry.coconut_tree = plist::from_bytes(v)?,
                DynamicObjectType::OrangeTree => entry.orange_tree = plist::from_bytes(v)?,
                DynamicObjectType::CherryTree => entry.cherry_tree = plist::from_bytes(v)?,
                DynamicObjectType::CoffeeTree => entry.coffee_tree = plist::from_bytes(v)?,
                DynamicObjectType::FlaxPlant => entry.flax_plant = plist::from_bytes(v)?,
                DynamicObjectType::SunflowerPlant => entry.sunflower_plant = plist::from_bytes(v)?,
                DynamicObjectType::CornPlant => entry.corn_plant = plist::from_bytes(v)?,
                DynamicObjectType::Dodo => entry.dodo = plist::from_bytes(v)?,
                DynamicObjectType::DroppedItem => entry.dropped_item = v.to_vec(),
                DynamicObjectType::Fire => entry.fire = v.to_vec(),
                DynamicObjectType::Torch => entry.torch = v.to_vec(),
                DynamicObjectType::GlowBlock => entry.glow_block = v.to_vec(),
                DynamicObjectType::Ladder => entry.ladder = plist::from_bytes(v)?,
                DynamicObjectType::Door => entry.door = plist::from_bytes(v)?,
                DynamicObjectType::ArtificialLight => entry.artificial_light = v.to_vec(),
                DynamicObjectType::Bed => entry.bed = plist::from_bytes(v)?,
                DynamicObjectType::DropBear => entry.dropbear = plist::from_bytes(v)?,
                DynamicObjectType::GatherBlock => entry.gather_block = v.to_vec(),
                DynamicObjectType::CarrotPlant => entry.carrot_plant = plist::from_bytes(v)?,
                DynamicObjectType::Donkey => entry.donkey = plist::from_bytes(v)?,
                DynamicObjectType::Egg => entry.egg = v.to_vec(),
                DynamicObjectType::Window => entry.window = plist::from_bytes(v)?,
                DynamicObjectType::Boat => entry.boat = plist::from_bytes(v)?,
                DynamicObjectType::ChilliPlant => entry.chilli_plant = plist::from_bytes(v)?,
                DynamicObjectType::KelpPlant => entry.kelp_plant = plist::from_bytes(v)?,
                DynamicObjectType::ClownFish => entry.clown_fish = plist::from_bytes(v)?,
                DynamicObjectType::Shark => entry.shark = plist::from_bytes(v)?,
                DynamicObjectType::LimeTree => entry.lime_tree = plist::from_bytes(v)?,
                DynamicObjectType::Wire => entry.wire = plist::from_bytes(v)?,
                DynamicObjectType::CaveTroll => entry.cave_troll = plist::from_bytes(v)?,
                DynamicObjectType::Rail => entry.rail = plist::from_bytes(v)?,
                DynamicObjectType::HandCar => entry.hand_car = plist::from_bytes(v)?,
                DynamicObjectType::SteamLocomotive => {
                    entry.steam_locomotive = plist::from_bytes(v)?
                }
                DynamicObjectType::FreightCar => entry.freight_car = plist::from_bytes(v)?,
                DynamicObjectType::PassengerCar => entry.passenger_car = plist::from_bytes(v)?,
                DynamicObjectType::Workbench => entry.workbench = plist::from_bytes(v)?,
                DynamicObjectType::Chest => entry.chest = plist::from_bytes(v)?,
                DynamicObjectType::Sign => entry.sign = plist::from_bytes(v)?,
                DynamicObjectType::TradingPost => entry.trading_post = plist::from_bytes(v)?,
                DynamicObjectType::TrainStation => entry.train_station = plist::from_bytes(v)?,
                DynamicObjectType::TradePortal => entry.trade_portal = plist::from_bytes(v)?,
                DynamicObjectType::Scorpion => entry.scorpion = plist::from_bytes(v)?,
                DynamicObjectType::Painting => entry.painting = v.to_vec(),
                DynamicObjectType::Column => entry.column = plist::from_bytes(v)?,
                DynamicObjectType::Stairs => entry.stairs = plist::from_bytes(v)?,
                DynamicObjectType::ElevatorMotor => entry.elevator_motor = plist::from_bytes(v)?,
                DynamicObjectType::ElevatorShaft => entry.elevator_shaft = plist::from_bytes(v)?,
                DynamicObjectType::GemTree => entry.gem_tree = plist::from_bytes(v)?,
                DynamicObjectType::VinePlant => entry.vine_plant = plist::from_bytes(v)?,
                DynamicObjectType::TulipPlant => entry.tulip_plant = plist::from_bytes(v)?,
                DynamicObjectType::OwnershipSign => entry.ownership_sign = v.to_vec(),
                DynamicObjectType::WheatPlant => entry.wheat_plant = plist::from_bytes(v)?,
                DynamicObjectType::TomatoPlant => entry.tomato_plant = plist::from_bytes(v)?,
                DynamicObjectType::Yak => entry.yak = plist::from_bytes(v)?,
                DynamicObjectType::Mirror => entry.mirror = v.to_vec(),
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
            put(db, wtxn, &coord, DroppedItem, &obj.dropped_item)?;
            put(db, wtxn, &coord, Fire, &obj.fire)?;
            put(db, wtxn, &coord, Torch, &obj.torch)?;
            put(db, wtxn, &coord, GlowBlock, &obj.glow_block)?;
            put(db, wtxn, &coord, Ladder, &obj.ladder)?;
            put(db, wtxn, &coord, Door, &obj.door)?;
            put(db, wtxn, &coord, ArtificialLight, &obj.artificial_light)?;
            put(db, wtxn, &coord, Bed, &obj.bed)?;
            put(db, wtxn, &coord, DropBear, &obj.dropbear)?;
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
            put(db, wtxn, &coord, HandCar, &obj.hand_car)?;
            put(db, wtxn, &coord, SteamLocomotive, &obj.steam_locomotive)?;
            put(db, wtxn, &coord, FreightCar, &obj.freight_car)?;
            put(db, wtxn, &coord, PassengerCar, &obj.passenger_car)?;
            put(db, wtxn, &coord, Workbench, &obj.workbench)?;
            put(db, wtxn, &coord, Chest, &obj.chest)?;
            put(db, wtxn, &coord, Sign, &obj.sign)?;
            put(db, wtxn, &coord, TradingPost, &obj.trading_post)?;
            put(db, wtxn, &coord, TrainStation, &obj.train_station)?;
            put(db, wtxn, &coord, TradePortal, &obj.trade_portal)?;
            put(db, wtxn, &coord, Scorpion, &obj.scorpion)?;
            put(db, wtxn, &coord, Painting, &obj.painting)?;
            put(db, wtxn, &coord, Column, &obj.column)?;
            put(db, wtxn, &coord, Stairs, &obj.stairs)?;
            put(db, wtxn, &coord, ElevatorMotor, &obj.elevator_motor)?;
            put(db, wtxn, &coord, ElevatorShaft, &obj.elevator_shaft)?;
            put(db, wtxn, &coord, GemTree, &obj.gem_tree)?;
            put(db, wtxn, &coord, VinePlant, &obj.vine_plant)?;
            put(db, wtxn, &coord, TulipPlant, &obj.tulip_plant)?;
            put(db, wtxn, &coord, OwnershipSign, &obj.ownership_sign)?;
            put(db, wtxn, &coord, WheatPlant, &obj.wheat_plant)?;
            put(db, wtxn, &coord, TomatoPlant, &obj.tomato_plant)?;
            put(db, wtxn, &coord, Yak, &obj.yak)?;
            put(db, wtxn, &coord, Mirror, &obj.mirror)?;
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
        fn read_test_xml<T: serde::de::DeserializeOwned>(
            obj_type: DynamicObjectType,
        ) -> super::DynamicObjectList<T> {
            let path = format!("resources/type_{}.xml", obj_type as u16);
            let bytes = std::fs::read(&path).unwrap_or_else(|_| panic!("Failed to read {}", path));
            plist::from_bytes(&bytes).unwrap()
        }

        let mut monster = ChunkDynamicObjects::default();

        monster.apple_tree = read_test_xml(DynamicObjectType::AppleTree);
        monster.maple_tree = read_test_xml(DynamicObjectType::MapleTree);
        monster.mango_tree = read_test_xml(DynamicObjectType::MangoTree);
        monster.pine_tree = read_test_xml(DynamicObjectType::PineTree);
        monster.cactus_tree = read_test_xml(DynamicObjectType::CactusTree);
        monster.coconut_tree = read_test_xml(DynamicObjectType::CoconutTree);
        monster.orange_tree = read_test_xml(DynamicObjectType::OrangeTree);
        monster.cherry_tree = read_test_xml(DynamicObjectType::CherryTree);
        monster.coffee_tree = read_test_xml(DynamicObjectType::CoffeeTree);
        monster.flax_plant = read_test_xml(DynamicObjectType::FlaxPlant);
        monster.sunflower_plant = read_test_xml(DynamicObjectType::SunflowerPlant);
        monster.corn_plant = read_test_xml(DynamicObjectType::CornPlant);
        monster.dodo = read_test_xml(DynamicObjectType::Dodo);
        monster.dropped_item = vec![14, 0xAA, 0xBB];
        monster.fire = vec![16, 0xAA, 0xBB];
        monster.torch = vec![17, 0xAA, 0xBB];
        monster.glow_block = vec![18, 0xAA, 0xBB];
        monster.ladder = read_test_xml(DynamicObjectType::Ladder);
        monster.door = read_test_xml(DynamicObjectType::Door);
        monster.artificial_light = vec![21, 0xAA, 0xBB];
        monster.bed = read_test_xml(DynamicObjectType::Bed);
        monster.dropbear = read_test_xml(DynamicObjectType::DropBear);
        monster.gather_block = vec![26, 0xAA, 0xBB];
        monster.carrot_plant = read_test_xml(DynamicObjectType::CarrotPlant);
        monster.donkey = read_test_xml(DynamicObjectType::Donkey);
        monster.egg = vec![30, 0xAA, 0xBB];
        monster.window = read_test_xml(DynamicObjectType::Window);
        monster.boat = read_test_xml(DynamicObjectType::Boat);
        monster.chilli_plant = read_test_xml(DynamicObjectType::ChilliPlant);
        monster.kelp_plant = read_test_xml(DynamicObjectType::KelpPlant);
        monster.clown_fish = read_test_xml(DynamicObjectType::ClownFish);
        monster.shark = read_test_xml(DynamicObjectType::Shark);
        monster.lime_tree = read_test_xml(DynamicObjectType::LimeTree);
        monster.wire = read_test_xml(DynamicObjectType::Wire);
        monster.cave_troll = read_test_xml(DynamicObjectType::CaveTroll);
        monster.rail = read_test_xml(DynamicObjectType::Rail);
        monster.hand_car = read_test_xml(DynamicObjectType::HandCar);
        monster.steam_locomotive = read_test_xml(DynamicObjectType::SteamLocomotive);
        monster.freight_car = read_test_xml(DynamicObjectType::FreightCar);
        monster.passenger_car = read_test_xml(DynamicObjectType::PassengerCar);
        monster.workbench = read_test_xml(DynamicObjectType::Workbench);
        monster.chest = read_test_xml(DynamicObjectType::Chest);
        monster.sign = read_test_xml(DynamicObjectType::Sign);
        monster.trading_post = read_test_xml(DynamicObjectType::TradingPost);
        monster.train_station = read_test_xml(DynamicObjectType::TrainStation);
        monster.trade_portal = read_test_xml(DynamicObjectType::TradePortal);
        monster.scorpion = read_test_xml(DynamicObjectType::Scorpion);
        monster.painting = vec![52, 0xAA, 0xBB];
        monster.column = read_test_xml(DynamicObjectType::Column);
        monster.stairs = read_test_xml(DynamicObjectType::Stairs);
        monster.elevator_motor = read_test_xml(DynamicObjectType::ElevatorMotor);
        monster.elevator_shaft = read_test_xml(DynamicObjectType::ElevatorShaft);
        monster.gem_tree = read_test_xml(DynamicObjectType::GemTree);
        monster.vine_plant = read_test_xml(DynamicObjectType::VinePlant);
        monster.tulip_plant = read_test_xml(DynamicObjectType::TulipPlant);
        monster.ownership_sign = vec![60, 0xAA, 0xBB];
        monster.wheat_plant = read_test_xml(DynamicObjectType::WheatPlant);
        monster.tomato_plant = read_test_xml(DynamicObjectType::TomatoPlant);
        monster.yak = read_test_xml(DynamicObjectType::Yak);
        monster.mirror = vec![64, 0xAA, 0xBB];

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
            check_key(DynamicObjectType::DroppedItem);
            check_key(DynamicObjectType::Fire);
            check_key(DynamicObjectType::Torch);
            check_key(DynamicObjectType::GlowBlock);
            check_key(DynamicObjectType::Ladder);
            check_key(DynamicObjectType::Door);
            check_key(DynamicObjectType::ArtificialLight);
            check_key(DynamicObjectType::Bed);
            check_key(DynamicObjectType::DropBear);
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
            check_key(DynamicObjectType::HandCar);
            check_key(DynamicObjectType::SteamLocomotive);
            check_key(DynamicObjectType::FreightCar);
            check_key(DynamicObjectType::PassengerCar);
            check_key(DynamicObjectType::Workbench);
            check_key(DynamicObjectType::Chest);
            check_key(DynamicObjectType::Sign);
            check_key(DynamicObjectType::TradingPost);
            check_key(DynamicObjectType::TrainStation);
            check_key(DynamicObjectType::TradePortal);
            check_key(DynamicObjectType::Scorpion);
            check_key(DynamicObjectType::Painting);
            check_key(DynamicObjectType::Column);
            check_key(DynamicObjectType::Stairs);
            check_key(DynamicObjectType::ElevatorMotor);
            check_key(DynamicObjectType::ElevatorShaft);
            check_key(DynamicObjectType::GemTree);
            check_key(DynamicObjectType::VinePlant);
            check_key(DynamicObjectType::TulipPlant);
            check_key(DynamicObjectType::OwnershipSign);
            check_key(DynamicObjectType::WheatPlant);
            check_key(DynamicObjectType::TomatoPlant);
            check_key(DynamicObjectType::Yak);
            check_key(DynamicObjectType::Mirror);

            let round_tripped_dw = DynamicWorld::from_db(&db, &rtxn).unwrap();
            let round_tripped_dw_chunk = round_tripped_dw.chunk_at(coord).unwrap();

            assert_eq!(round_tripped_dw_chunk, dw.chunk_at(coord).unwrap());
        }
    }
}
