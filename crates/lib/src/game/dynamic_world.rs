use super::{
    coord::{ChunkCoord, CoordError},
    dynamic_object::{
        AnyDynamicObject, AnyDynamicObjectRef, DynamicObjectList, DynamicObjectType,
        animal::{CaveTroll, ClownFish, Dodo, Donkey, DropBear, Egg, Scorpion, Shark, Yak},
        chest::{Chest, ChestError, ChestMeta},
        craft::{
            Bed, Boat, Column, Door, ElevatorMotor, ElevatorShaft, Ladder, Rail, Sign, Stairs,
            Torch, TradePortal, TradingPost, Window, Wire,
        },
        dropped_item::{DroppedItem, DroppedItemXml},
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
    item::ItemError,
};
use lmdb_rs::{
    codec::types::{Bytes, Str},
    database::Database,
    txn::{RoTxn, RwTxn},
};
use serde::Serialize;
use snafu::prelude::*;
use std::{
    collections::{HashMap, hash_map},
    io::Write,
    ops::Deref,
};

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
    fn to_plist(&self) -> std::result::Result<Vec<u8>, plist::Error>;
}

impl ToXmlPlist for Vec<u8> {
    fn to_plist(&self) -> std::result::Result<Vec<u8>, plist::Error> {
        Ok(self.clone())
    }
}

impl<T: Serialize> ToXmlPlist for DynamicObjectList<T> {
    fn to_plist(&self) -> std::result::Result<Vec<u8>, plist::Error> {
        let mut serialized = Vec::new();
        plist::to_writer_xml(&mut serialized, self).unwrap(); // TODO must be safe
        Ok(serialized)
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
    pub dropped_item: DynamicObjectList<DroppedItem>,
    pub fire: Vec<u8>,
    pub torch: DynamicObjectList<Torch>,
    pub glow_block: Vec<u8>,
    pub ladder: DynamicObjectList<Ladder>,
    pub door: DynamicObjectList<Door>,
    pub artificial_light: Vec<u8>,
    pub bed: DynamicObjectList<Bed>,
    pub dropbear: DynamicObjectList<DropBear>,
    pub gather_block: Vec<u8>,
    pub carrot_plant: DynamicObjectList<CarrotPlant>,
    pub donkey: DynamicObjectList<Donkey>,
    pub egg: DynamicObjectList<Egg>,
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
            + self.dropped_item.num_obj()
            // + self.fire.num_obj()
            + self.torch.num_obj()
            // + self.glow_block.num_obj()
            + self.ladder.num_obj()
            + self.door.num_obj()
            // + self.artificial_light.num_obj()
            + self.bed.num_obj()
            + self.dropbear.num_obj()
            // + self.gather_block.num_obj()
            + self.carrot_plant.num_obj()
            + self.donkey.num_obj()
            + self.egg.num_obj()
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
            + self.workbench.num_obj()
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

    fn move_element(
        &mut self,
        dst: &mut Self,
        obj_type: DynamicObjectType,
        i: usize,
    ) -> Option<usize> {
        fn mv<T>(
            src: &mut DynamicObjectList<T>,
            dst: &mut DynamicObjectList<T>,
            i: usize,
        ) -> Option<usize> {
            let new_i = dst.len();
            dst.push(src.remove(i));
            Some(new_i)
        }
        use DynamicObjectType::*;
        match obj_type {
            AppleTree => mv(&mut self.apple_tree, &mut dst.apple_tree, i),
            MapleTree => mv(&mut self.maple_tree, &mut dst.maple_tree, i),
            MangoTree => mv(&mut self.mango_tree, &mut dst.mango_tree, i),
            PineTree => mv(&mut self.pine_tree, &mut dst.pine_tree, i),
            CactusTree => mv(&mut self.cactus_tree, &mut dst.cactus_tree, i),
            CoconutTree => mv(&mut self.coconut_tree, &mut dst.coconut_tree, i),
            OrangeTree => mv(&mut self.orange_tree, &mut dst.orange_tree, i),
            CherryTree => mv(&mut self.cherry_tree, &mut dst.cherry_tree, i),
            CoffeeTree => mv(&mut self.coffee_tree, &mut dst.coffee_tree, i),
            FlaxPlant => mv(&mut self.flax_plant, &mut dst.flax_plant, i),
            SunflowerPlant => mv(&mut self.sunflower_plant, &mut dst.sunflower_plant, i),
            CornPlant => mv(&mut self.corn_plant, &mut dst.corn_plant, i),
            Dodo => mv(&mut self.dodo, &mut dst.dodo, i),
            DroppedItem => mv(&mut self.dropped_item, &mut dst.dropped_item, i),
            Fire => None, // helper(&mut self.fire, &mut dst.fire, index),
            Torch => mv(&mut self.torch, &mut dst.torch, i),
            GlowBlock => None, // helper(&mut self.glow_block, &mut dst.glow_block, index),
            Ladder => mv(&mut self.ladder, &mut dst.ladder, i),
            Door => mv(&mut self.door, &mut dst.door, i),
            ArtificialLight => None, // helper(&mut self.artificial_light, &mut dst.artificial_light, index),
            Bed => mv(&mut self.bed, &mut dst.bed, i),
            DropBear => mv(&mut self.dropbear, &mut dst.dropbear, i),
            GatherBlock => None, // helper(&mut self.gather_block, &mut dst.gather_block, index),
            CarrotPlant => mv(&mut self.carrot_plant, &mut dst.carrot_plant, i),
            Donkey => mv(&mut self.donkey, &mut dst.donkey, i),
            Egg => mv(&mut self.egg, &mut dst.egg, i),
            Window => mv(&mut self.window, &mut dst.window, i),
            Boat => mv(&mut self.boat, &mut dst.boat, i),
            ChilliPlant => mv(&mut self.chilli_plant, &mut dst.chilli_plant, i),
            KelpPlant => mv(&mut self.kelp_plant, &mut dst.kelp_plant, i),
            ClownFish => mv(&mut self.clown_fish, &mut dst.clown_fish, i),
            Shark => mv(&mut self.shark, &mut dst.shark, i),
            LimeTree => mv(&mut self.lime_tree, &mut dst.lime_tree, i),
            Wire => mv(&mut self.wire, &mut dst.wire, i),
            CaveTroll => mv(&mut self.cave_troll, &mut dst.cave_troll, i),
            Rail => mv(&mut self.rail, &mut dst.rail, i),
            HandCar => mv(&mut self.hand_car, &mut dst.hand_car, i),
            SteamLocomotive => mv(&mut self.steam_locomotive, &mut dst.steam_locomotive, i),
            FreightCar => mv(&mut self.freight_car, &mut dst.freight_car, i),
            PassengerCar => mv(&mut self.passenger_car, &mut dst.passenger_car, i),
            Workbench => mv(&mut self.workbench, &mut dst.workbench, i),
            Chest => mv(&mut self.chest, &mut dst.chest, i),
            Sign => mv(&mut self.sign, &mut dst.sign, i),
            TradingPost => mv(&mut self.trading_post, &mut dst.trading_post, i),
            TrainStation => mv(&mut self.train_station, &mut dst.train_station, i),
            TradePortal => mv(&mut self.trade_portal, &mut dst.trade_portal, i),
            Scorpion => mv(&mut self.scorpion, &mut dst.scorpion, i),
            Painting => None, // helper(&mut self.painting, &mut dst.painting, index),
            Column => mv(&mut self.column, &mut dst.column, i),
            Stairs => mv(&mut self.stairs, &mut dst.stairs, i),
            ElevatorMotor => mv(&mut self.elevator_motor, &mut dst.elevator_motor, i),
            ElevatorShaft => mv(&mut self.elevator_shaft, &mut dst.elevator_shaft, i),
            GemTree => mv(&mut self.gem_tree, &mut dst.gem_tree, i),
            VinePlant => mv(&mut self.vine_plant, &mut dst.vine_plant, i),
            TulipPlant => mv(&mut self.tulip_plant, &mut dst.tulip_plant, i),
            OwnershipSign => None, // helper(&mut self.ownership_sign, &mut dst.ownership_sign, index)
            WheatPlant => mv(&mut self.wheat_plant, &mut dst.wheat_plant, i),
            TomatoPlant => mv(&mut self.tomato_plant, &mut dst.tomato_plant, i),
            Yak => mv(&mut self.yak, &mut dst.yak, i),
            Mirror => None, // helper(&mut self.mirror, &mut dst.mirror, index),
        }
    }

    pub fn insert(&mut self, any_dyn_obj: AnyDynamicObject) {
        match any_dyn_obj {
            AnyDynamicObject::Ladder(v) => self.ladder.push(*v),
            AnyDynamicObject::Door(v) => self.door.push(*v),
            AnyDynamicObject::Bed(v) => self.bed.push(*v),
            AnyDynamicObject::Egg(v) => self.egg.push(*v),
            AnyDynamicObject::Workbench(v) => self.workbench.push(*v),
            AnyDynamicObject::Chest(v) => self.chest.push(*v),
            AnyDynamicObject::Sign(v) => self.sign.push(*v),
            AnyDynamicObject::TrainStation(v) => self.train_station.push(*v),
        }
    }

    pub fn remove(&mut self, dyn_obj_ty: DynamicObjectType, index: usize) {
        use DynamicObjectType::*;
        fn remove<T>(dst: &mut DynamicObjectList<T>, index: usize) {
            dst.remove(index);
        }
        match dyn_obj_ty {
            AppleTree => remove(&mut self.apple_tree, index),
            MapleTree => remove(&mut self.maple_tree, index),
            MangoTree => remove(&mut self.mango_tree, index),
            PineTree => remove(&mut self.pine_tree, index),
            CactusTree => remove(&mut self.cactus_tree, index),
            CoconutTree => remove(&mut self.coconut_tree, index),
            OrangeTree => remove(&mut self.orange_tree, index),
            CherryTree => remove(&mut self.cherry_tree, index),
            CoffeeTree => remove(&mut self.coffee_tree, index),
            FlaxPlant => remove(&mut self.flax_plant, index),
            SunflowerPlant => remove(&mut self.sunflower_plant, index),
            CornPlant => remove(&mut self.corn_plant, index),
            Dodo => remove(&mut self.dodo, index),
            DroppedItem => remove(&mut self.dropped_item, index),
            Fire => {} // remove(&mut self.fire, index),
            Torch => remove(&mut self.torch, index),
            GlowBlock => {} // remove(&mut self.glow_block, index),
            Ladder => remove(&mut self.ladder, index),
            Door => remove(&mut self.door, index),
            ArtificialLight => {} // remove(&mut self.artificial_light, index),
            Bed => remove(&mut self.bed, index),
            DropBear => remove(&mut self.dropbear, index),
            GatherBlock => {} // remove(&mut self.gather_block, index),
            CarrotPlant => remove(&mut self.carrot_plant, index),
            Donkey => remove(&mut self.donkey, index),
            Egg => remove(&mut self.egg, index),
            Window => remove(&mut self.window, index),
            Boat => remove(&mut self.boat, index),
            ChilliPlant => remove(&mut self.chilli_plant, index),
            KelpPlant => remove(&mut self.kelp_plant, index),
            ClownFish => remove(&mut self.clown_fish, index),
            Shark => remove(&mut self.shark, index),
            LimeTree => remove(&mut self.lime_tree, index),
            Wire => remove(&mut self.wire, index),
            CaveTroll => remove(&mut self.cave_troll, index),
            Rail => remove(&mut self.rail, index),
            HandCar => remove(&mut self.hand_car, index),
            SteamLocomotive => remove(&mut self.steam_locomotive, index),
            FreightCar => remove(&mut self.freight_car, index),
            PassengerCar => remove(&mut self.passenger_car, index),
            Workbench => remove(&mut self.workbench, index),
            Chest => remove(&mut self.chest, index),
            Sign => remove(&mut self.sign, index),
            TradingPost => remove(&mut self.trading_post, index),
            TrainStation => remove(&mut self.train_station, index),
            TradePortal => remove(&mut self.trade_portal, index),
            Scorpion => remove(&mut self.scorpion, index),
            Painting => {} // remove(&mut self.painting, index),
            Column => remove(&mut self.column, index),
            Stairs => remove(&mut self.stairs, index),
            ElevatorMotor => remove(&mut self.elevator_motor, index),
            ElevatorShaft => remove(&mut self.elevator_shaft, index),
            GemTree => remove(&mut self.gem_tree, index),
            VinePlant => remove(&mut self.vine_plant, index),
            TulipPlant => remove(&mut self.tulip_plant, index),
            OwnershipSign => {} // remove(&mut self.ownership_sign, index),
            WheatPlant => remove(&mut self.wheat_plant, index),
            TomatoPlant => remove(&mut self.tomato_plant, index),
            Yak => remove(&mut self.yak, index),
            Mirror => {} // remove(&mut self.mirror, index),
        }
    }

    pub fn get(
        &self,
        dyn_obj_ty: DynamicObjectType,
        index: usize,
    ) -> Option<AnyDynamicObjectRef<'_>> {
        use DynamicObjectType::*;
        Some(match dyn_obj_ty {
            // AppleTree => self.apple_tree.get(index),
            // MapleTree => self.maple_tree.get(index),
            // MangoTree => self.mango_tree.get(index),
            // PineTree => self.pine_tree.get(index),
            // CactusTree => self.cactus_tree.get(index),
            // CoconutTree => self.coconut_tree.get(index),
            // OrangeTree => self.orange_tree.get(index),
            // CherryTree => self.cherry_tree.get(index),
            // CoffeeTree => self.coffee_tree.get(index),
            // FlaxPlant => self.flax_plant.get(index),
            // SunflowerPlant => self.sunflower_plant.get(index),
            // CornPlant => self.corn_plant.get(index),
            // Dodo => self.dodo.get(index),
            // DroppedItem => self.dropped_item.get(index),
            // Fire => self.fire.get(index),
            // Torch => self.torch.get(index),
            // GlowBlock => self.glow_block.get(index),
            Ladder => AnyDynamicObjectRef::Ladder(self.ladder.get(index)?),
            Door => AnyDynamicObjectRef::Door(self.door.get(index)?),
            // ArtificialLight => self.artificial_light.get(index),
            Bed => AnyDynamicObjectRef::Bed(self.bed.get(index)?),
            // DropBear => self.dropbear.get(index),
            // GatherBlock => self.gather_block.get(index),
            // CarrotPlant => self.carrot_plant.get(index),
            // Donkey => self.donkey.get(index),
            Egg => AnyDynamicObjectRef::Egg(self.egg.get(index)?),
            // Window => self.window.get(index),
            // Boat => self.boat.get(index),
            // ChilliPlant => self.chilli_plant.get(index),
            // KelpPlant => self.kelp_plant.get(index),
            // ClownFish => self.clown_fish.get(index),
            // Shark => self.shark.get(index),
            // LimeTree => self.lime_tree.get(index),
            // Wire => self.wire.get(index),
            // CaveTroll => self.cave_troll.get(index),
            // Rail => self.rail.get(index),
            // HandCar => self.hand_car.get(index),
            // SteamLocomotive => self.steam_locomotive.get(index),
            // FreightCar => self.freight_car.get(index),
            // PassengerCar => self.passenger_car.get(index),
            Workbench => AnyDynamicObjectRef::Workbench(self.workbench.get(index)?),
            Chest => AnyDynamicObjectRef::Chest(self.chest.get(index)?),
            Sign => AnyDynamicObjectRef::Sign(self.sign.get(index)?),
            // TradingPost => self.trading_post.get(index),
            TrainStation => AnyDynamicObjectRef::TrainStation(self.train_station.get(index)?),
            // TradePortal => self.trade_portal.get(index),
            // Scorpion => self.scorpion.get(index),
            // Painting => self.painting.get(index),
            // Column => self.column.get(index),
            // Stairs => self.stairs.get(index),
            // ElevatorMotor => self.elevator_motor.get(index),
            // ElevatorShaft => self.elevator_shaft.get(index),
            // GemTree => self.gem_tree.get(index),
            // VinePlant => self.vine_plant.get(index),
            // TulipPlant => self.tulip_plant.get(index),
            // OwnershipSign => self.ownership_sign.get(index),
            // WheatPlant => self.wheat_plant.get(index),
            // TomatoPlant => self.tomato_plant.get(index),
            // Yak => self.yak.get(index),
            // Mirror => self.mirror.get(index),
            _ => None?,
        })
    }
}

#[derive(Debug, Snafu)]
pub enum DynamicWorldError {
    #[snafu(display("Failed to iterate over database"))]
    IterateDatabase {
        source: lmdb_rs::error::DatabaseError,
    },
    #[snafu(display("Failed to deserialize {object_type:?} in {coord}"))]
    DeserializeObject {
        object_type: DynamicObjectType,
        coord: ChunkCoord,
        source: plist::Error,
    },
    #[snafu(display("Failed to serialize {object_type:?} in {coord}"))]
    SerializeObject {
        object_type: DynamicObjectType,
        coord: String,
        source: plist::Error,
    },
    #[snafu(display("Failed to get entry {key} from database"))]
    GetEntry {
        source: lmdb_rs::error::DatabaseError,
        key: String,
    },
    #[snafu(display("Failed to put entry with key {key} in database"))]
    PutEntry {
        key: String,
        source: lmdb_rs::error::DatabaseError,
    },
    #[snafu(display("Failed to load chest {id} in chunk {coord}"))]
    LoadChest {
        id: u64,
        coord: ChunkCoord,
        source: ChestError,
    },
    #[snafu(display("Failed to save chest {id} in chunk {coord}"))]
    SaveChest {
        id: u64,
        coord: ChunkCoord,
        source: ChestError,
    },
    #[snafu(display("Failed to parse chunk coord {coord}"))]
    ParseChunkCoordFromStr { coord: String, source: CoordError },
    #[snafu(display("Failed to load dropped item in chunk {coord}: {xml}"))]
    LoadDroppedItem {
        coord: ChunkCoord,
        xml: String,
        source: ItemError,
    },
    #[snafu(display("Failed to save dropped item in chunk {coord}"))]
    SaveDroppedItem {
        coord: ChunkCoord,
        source: ItemError,
    },
}

type Result<T> = std::result::Result<T, DynamicWorldError>;

// TODO: handle chest data like `349_20/chest_834` and `trainchest_1342`
#[derive(Debug)]
pub struct DynamicWorld(HashMap<ChunkCoord, ChunkDynamicObjects>);

impl DynamicWorld {
    pub fn chunk_at<I: Into<ChunkCoord>>(&self, coord: I) -> Option<&ChunkDynamicObjects> {
        self.0.get(&coord.into())
    }

    pub fn chunk_at_mut<I: Into<ChunkCoord>>(
        &mut self,
        coord: I,
    ) -> Option<&mut ChunkDynamicObjects> {
        self.0.get_mut(&coord.into())
    }

    pub fn entry<I: Into<ChunkCoord>>(
        &'_ mut self,
        coord: I,
    ) -> hash_map::Entry<'_, ChunkCoord, ChunkDynamicObjects> {
        self.0.entry(coord.into())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ChunkCoord, &ChunkDynamicObjects)> {
        self.0.iter()
    }

    /// Moves `i`-th obj_type in `src_coord` chunk to `dst_coord`, returns new index in dst chunk
    pub fn move_element(
        &mut self,
        src_coord: ChunkCoord,
        dst_coord: ChunkCoord,
        obj_type: DynamicObjectType,
        index: usize,
    ) -> Option<usize> {
        if src_coord == dst_coord {
            return None;
        }
        // ensure dst chunk always exists
        let _ = self.entry(dst_coord).or_default();
        if let [Some(src), Some(dst)] = self.0.get_disjoint_mut([&src_coord, &dst_coord]) {
            src.move_element(dst, obj_type, index)
        } else {
            None
        }
    }

    pub fn from_db(db: &Database<Str, Bytes>, rtxn: &RoTxn) -> Result<Self> {
        let mut map = HashMap::new();
        for (k, v) in db
            .iter(rtxn)
            .context(IterateDatabaseSnafu)?
            .filter_map(|v| v.ok())
        {
            let Some((coord_str, type_id_str)) = k.split_once("/") else {
                println!("Found key {} we don't understand in dynamic world", k);
                continue;
            };
            let coord = ChunkCoord::try_from_str(coord_str).with_context(|_| {
                ParseChunkCoordFromStrSnafu {
                    coord: coord_str.to_owned(),
                }
            })?;
            let Ok(obj_ty) = DynamicObjectType::try_from_str(type_id_str) else {
                if !type_id_str.starts_with("chest_") {
                    println!(
                        "Found object type {} we don't understand in chunk in dynamic world {}",
                        type_id_str, coord_str
                    );
                }
                continue;
            };
            let entry = map
                .entry(coord)
                .or_insert_with(ChunkDynamicObjects::default);

            fn load<T: serde::de::DeserializeOwned>(
                bytes: &[u8],
                dyn_obj_type: DynamicObjectType,
                coord: ChunkCoord,
            ) -> Result<T> {
                plist::from_reader_xml(bytes).context(DeserializeObjectSnafu {
                    object_type: dyn_obj_type,
                    coord,
                })
            }

            match obj_ty {
                DynamicObjectType::AppleTree => entry.apple_tree = load(v, obj_ty, coord)?,
                DynamicObjectType::MapleTree => entry.maple_tree = load(v, obj_ty, coord)?,
                DynamicObjectType::MangoTree => entry.mango_tree = load(v, obj_ty, coord)?,
                DynamicObjectType::PineTree => entry.pine_tree = load(v, obj_ty, coord)?,
                DynamicObjectType::CactusTree => entry.cactus_tree = load(v, obj_ty, coord)?,
                DynamicObjectType::CoconutTree => entry.coconut_tree = load(v, obj_ty, coord)?,
                DynamicObjectType::OrangeTree => entry.orange_tree = load(v, obj_ty, coord)?,
                DynamicObjectType::CherryTree => entry.cherry_tree = load(v, obj_ty, coord)?,
                DynamicObjectType::CoffeeTree => entry.coffee_tree = load(v, obj_ty, coord)?,
                DynamicObjectType::FlaxPlant => entry.flax_plant = load(v, obj_ty, coord)?,
                DynamicObjectType::SunflowerPlant => {
                    entry.sunflower_plant = load(v, obj_ty, coord)?
                }
                DynamicObjectType::CornPlant => entry.corn_plant = load(v, obj_ty, coord)?,
                DynamicObjectType::Dodo => entry.dodo = load(v, obj_ty, coord)?,
                DynamicObjectType::DroppedItem => {
                    let dropped_item_xml: DynamicObjectList<DroppedItemXml> =
                        plist::from_reader_xml(v).context(DeserializeObjectSnafu {
                            object_type: obj_ty,
                            coord,
                        })?;
                    entry.dropped_item = dropped_item_xml
                        .into_iter()
                        .map(|xml| {
                            DroppedItem::try_from_xml(xml).context(LoadDroppedItemSnafu {
                                coord,
                                // SAFETY: can be deserialized to DynamicObjectList means it's valid xml and thus &str
                                xml: str::from_utf8(v).unwrap().to_string(),
                            })
                        })
                        .collect::<Result<DynamicObjectList<DroppedItem>>>()?;
                }
                DynamicObjectType::Fire => entry.fire = v.to_vec(),
                DynamicObjectType::Torch => entry.torch = load(v, obj_ty, coord)?,
                DynamicObjectType::GlowBlock => entry.glow_block = v.to_vec(),
                DynamicObjectType::Ladder => entry.ladder = load(v, obj_ty, coord)?,
                DynamicObjectType::Door => entry.door = load(v, obj_ty, coord)?,
                DynamicObjectType::ArtificialLight => entry.artificial_light = v.to_vec(),
                DynamicObjectType::Bed => entry.bed = load(v, obj_ty, coord)?,
                DynamicObjectType::DropBear => entry.dropbear = load(v, obj_ty, coord)?,
                DynamicObjectType::GatherBlock => entry.gather_block = v.to_vec(),
                DynamicObjectType::CarrotPlant => entry.carrot_plant = load(v, obj_ty, coord)?,
                DynamicObjectType::Donkey => entry.donkey = load(v, obj_ty, coord)?,
                DynamicObjectType::Egg => entry.egg = load(v, obj_ty, coord)?,
                DynamicObjectType::Window => entry.window = load(v, obj_ty, coord)?,
                DynamicObjectType::Boat => entry.boat = load(v, obj_ty, coord)?,
                DynamicObjectType::ChilliPlant => entry.chilli_plant = load(v, obj_ty, coord)?,
                DynamicObjectType::KelpPlant => entry.kelp_plant = load(v, obj_ty, coord)?,
                DynamicObjectType::ClownFish => entry.clown_fish = load(v, obj_ty, coord)?,
                DynamicObjectType::Shark => entry.shark = load(v, obj_ty, coord)?,
                DynamicObjectType::LimeTree => entry.lime_tree = load(v, obj_ty, coord)?,
                DynamicObjectType::Wire => entry.wire = load(v, obj_ty, coord)?,
                DynamicObjectType::CaveTroll => entry.cave_troll = load(v, obj_ty, coord)?,
                DynamicObjectType::Rail => entry.rail = load(v, obj_ty, coord)?,
                DynamicObjectType::HandCar => entry.hand_car = load(v, obj_ty, coord)?,
                DynamicObjectType::SteamLocomotive => {
                    entry.steam_locomotive = load(v, obj_ty, coord)?
                }
                DynamicObjectType::FreightCar => entry.freight_car = load(v, obj_ty, coord)?,
                DynamicObjectType::PassengerCar => entry.passenger_car = load(v, obj_ty, coord)?,
                DynamicObjectType::Workbench => entry.workbench = load(v, obj_ty, coord)?,
                DynamicObjectType::Chest => {
                    let chest_metas: DynamicObjectList<ChestMeta> =
                        plist::from_bytes(v).context(DeserializeObjectSnafu {
                            object_type: obj_ty,
                            coord,
                        })?;
                    entry.chest = chest_metas
                        .into_iter()
                        .map(|chest_meta| {
                            let id = *chest_meta.unique_id.inner();
                            let key = format!("{}/chest_{}", coord_str, id);
                            let slot_bytes = db.get(rtxn, &key).context(GetEntrySnafu { key })?;
                            Chest::from_meta_and_slots(chest_meta, slot_bytes)
                                .context(LoadChestSnafu { coord, id })
                        })
                        .collect::<Result<DynamicObjectList<Chest>>>()?
                }
                DynamicObjectType::Sign => entry.sign = load(v, obj_ty, coord)?,
                DynamicObjectType::TradingPost => entry.trading_post = load(v, obj_ty, coord)?,
                DynamicObjectType::TrainStation => entry.train_station = load(v, obj_ty, coord)?,
                DynamicObjectType::TradePortal => entry.trade_portal = load(v, obj_ty, coord)?,
                DynamicObjectType::Scorpion => entry.scorpion = load(v, obj_ty, coord)?,
                DynamicObjectType::Painting => entry.painting = v.to_vec(),
                DynamicObjectType::Column => entry.column = load(v, obj_ty, coord)?,
                DynamicObjectType::Stairs => entry.stairs = load(v, obj_ty, coord)?,
                DynamicObjectType::ElevatorMotor => entry.elevator_motor = load(v, obj_ty, coord)?,
                DynamicObjectType::ElevatorShaft => entry.elevator_shaft = load(v, obj_ty, coord)?,
                DynamicObjectType::GemTree => entry.gem_tree = load(v, obj_ty, coord)?,
                DynamicObjectType::VinePlant => entry.vine_plant = load(v, obj_ty, coord)?,
                DynamicObjectType::TulipPlant => entry.tulip_plant = load(v, obj_ty, coord)?,
                DynamicObjectType::OwnershipSign => entry.ownership_sign = v.to_vec(),
                DynamicObjectType::WheatPlant => entry.wheat_plant = load(v, obj_ty, coord)?,
                DynamicObjectType::TomatoPlant => entry.tomato_plant = load(v, obj_ty, coord)?,
                DynamicObjectType::Yak => entry.yak = load(v, obj_ty, coord)?,
                DynamicObjectType::Mirror => entry.mirror = v.to_vec(),
            };
        }
        Ok(Self(map))
    }

    pub fn to_db<W: Write>(&self, db: &Database<Str, Bytes>, wtxn: &mut RwTxn<W>) -> Result<()> {
        #[inline(always)]
        fn put<W: Write, T: ToXmlPlist + IsEmpty>(
            db: &Database<Str, Bytes>,
            wtxn: &mut RwTxn<W>,
            coord_str: &str,
            obj_type: DynamicObjectType,
            value: &T,
        ) -> Result<()> {
            let key = format!("{}/{}", coord_str, obj_type as u16);
            if !value.is_empty() {
                db.put(
                    wtxn,
                    &key,
                    &value.to_plist().with_context(|_| SerializeObjectSnafu {
                        object_type: obj_type,
                        coord: coord_str.to_owned(),
                    })?,
                )
                .context(PutEntrySnafu { key })?;
            }
            Ok(())
        }

        for (coord, obj) in self.0.iter() {
            let coord_str = coord.to_string();
            use DynamicObjectType::*;
            put(db, wtxn, &coord_str, AppleTree, &obj.apple_tree)?;
            put(db, wtxn, &coord_str, MapleTree, &obj.maple_tree)?;
            put(db, wtxn, &coord_str, MangoTree, &obj.mango_tree)?;
            put(db, wtxn, &coord_str, PineTree, &obj.pine_tree)?;
            put(db, wtxn, &coord_str, CactusTree, &obj.cactus_tree)?;
            put(db, wtxn, &coord_str, CoconutTree, &obj.coconut_tree)?;
            put(db, wtxn, &coord_str, OrangeTree, &obj.orange_tree)?;
            put(db, wtxn, &coord_str, CherryTree, &obj.cherry_tree)?;
            put(db, wtxn, &coord_str, CoffeeTree, &obj.coffee_tree)?;
            put(db, wtxn, &coord_str, FlaxPlant, &obj.flax_plant)?;
            put(db, wtxn, &coord_str, SunflowerPlant, &obj.sunflower_plant)?;
            put(db, wtxn, &coord_str, CornPlant, &obj.corn_plant)?;
            put(db, wtxn, &coord_str, Dodo, &obj.dodo)?;
            let dropped_item_xml = obj
                .dropped_item
                .iter()
                .map(|dropped_item| -> Result<DroppedItemXml> {
                    dropped_item
                        .to_xml()
                        .context(SaveDroppedItemSnafu { coord: *coord })
                })
                .collect::<Result<DynamicObjectList<DroppedItemXml>>>()?;
            put(db, wtxn, &coord_str, DroppedItem, &dropped_item_xml)?;
            put(db, wtxn, &coord_str, Fire, &obj.fire)?;
            put(db, wtxn, &coord_str, Torch, &obj.torch)?;
            put(db, wtxn, &coord_str, GlowBlock, &obj.glow_block)?;
            put(db, wtxn, &coord_str, Ladder, &obj.ladder)?;
            put(db, wtxn, &coord_str, Door, &obj.door)?;
            put(db, wtxn, &coord_str, ArtificialLight, &obj.artificial_light)?;
            put(db, wtxn, &coord_str, Bed, &obj.bed)?;
            put(db, wtxn, &coord_str, DropBear, &obj.dropbear)?;
            put(db, wtxn, &coord_str, GatherBlock, &obj.gather_block)?;
            put(db, wtxn, &coord_str, CarrotPlant, &obj.carrot_plant)?;
            put(db, wtxn, &coord_str, Donkey, &obj.donkey)?;
            put(db, wtxn, &coord_str, Egg, &obj.egg)?;
            put(db, wtxn, &coord_str, Window, &obj.window)?;
            put(db, wtxn, &coord_str, Boat, &obj.boat)?;
            put(db, wtxn, &coord_str, ChilliPlant, &obj.chilli_plant)?;
            put(db, wtxn, &coord_str, KelpPlant, &obj.kelp_plant)?;
            put(db, wtxn, &coord_str, ClownFish, &obj.clown_fish)?;
            put(db, wtxn, &coord_str, Shark, &obj.shark)?;
            put(db, wtxn, &coord_str, LimeTree, &obj.lime_tree)?;
            put(db, wtxn, &coord_str, Wire, &obj.wire)?;
            put(db, wtxn, &coord_str, CaveTroll, &obj.cave_troll)?;
            put(db, wtxn, &coord_str, Rail, &obj.rail)?;
            put(db, wtxn, &coord_str, HandCar, &obj.hand_car)?;
            put(db, wtxn, &coord_str, SteamLocomotive, &obj.steam_locomotive)?;
            put(db, wtxn, &coord_str, FreightCar, &obj.freight_car)?;
            put(db, wtxn, &coord_str, PassengerCar, &obj.passenger_car)?;
            put(db, wtxn, &coord_str, Workbench, &obj.workbench)?;
            let chest_metas = obj
                .chest
                .iter()
                .map(|c| -> Result<ChestMeta> {
                    let id = *c.unique_id.inner();
                    let (chest_meta, slot_bytes) =
                        c.to_meta_and_slots().context(SaveChestSnafu {
                            id,
                            coord: coord.to_owned(),
                        })?;
                    let key = format!("{}/chest_{}", &coord_str, id);
                    if let Some(slot_bytes) = slot_bytes {
                        db.put(wtxn, &key, &slot_bytes)
                            .context(PutEntrySnafu { key })?;
                    }
                    Ok(chest_meta)
                })
                .collect::<Result<DynamicObjectList<ChestMeta>>>()?;
            put(db, wtxn, &coord_str, Chest, &chest_metas)?;
            put(db, wtxn, &coord_str, Sign, &obj.sign)?;
            put(db, wtxn, &coord_str, TradingPost, &obj.trading_post)?;
            put(db, wtxn, &coord_str, TrainStation, &obj.train_station)?;
            put(db, wtxn, &coord_str, TradePortal, &obj.trade_portal)?;
            put(db, wtxn, &coord_str, Scorpion, &obj.scorpion)?;
            put(db, wtxn, &coord_str, Painting, &obj.painting)?;
            put(db, wtxn, &coord_str, Column, &obj.column)?;
            put(db, wtxn, &coord_str, Stairs, &obj.stairs)?;
            put(db, wtxn, &coord_str, ElevatorMotor, &obj.elevator_motor)?;
            put(db, wtxn, &coord_str, ElevatorShaft, &obj.elevator_shaft)?;
            put(db, wtxn, &coord_str, GemTree, &obj.gem_tree)?;
            put(db, wtxn, &coord_str, VinePlant, &obj.vine_plant)?;
            put(db, wtxn, &coord_str, TulipPlant, &obj.tulip_plant)?;
            put(db, wtxn, &coord_str, OwnershipSign, &obj.ownership_sign)?;
            put(db, wtxn, &coord_str, WheatPlant, &obj.wheat_plant)?;
            put(db, wtxn, &coord_str, TomatoPlant, &obj.tomato_plant)?;
            put(db, wtxn, &coord_str, Yak, &obj.yak)?;
            put(db, wtxn, &coord_str, Mirror, &obj.mirror)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            coord::ChunkCoord,
            dynamic_object::{
                DynamicObjectList,
                chest::{Chest, ChestMeta},
                dropped_item::{DroppedItem, DroppedItemXml},
            },
        },
        ChunkDynamicObjects, DynamicObjectType, DynamicWorld,
    };
    use lmdb_rs::{
        arch::DynArch,
        codec::types::{Bytes, Str},
        env::{Env, EnvWrite},
    };
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
        let dropped_item_xml: DynamicObjectList<DroppedItemXml> =
            read_test_xml(DynamicObjectType::DroppedItem);
        monster.dropped_item = dropped_item_xml
            .into_iter()
            .map(DroppedItem::try_from_xml)
            .collect::<Result<DynamicObjectList<_>, _>>()
            .unwrap();
        monster.fire = vec![16, 0xAA, 0xBB];
        monster.torch = read_test_xml(DynamicObjectType::Torch);
        monster.glow_block = vec![18, 0xAA, 0xBB];
        monster.ladder = read_test_xml(DynamicObjectType::Ladder);
        monster.door = read_test_xml(DynamicObjectType::Door);
        monster.artificial_light = vec![21, 0xAA, 0xBB];
        monster.bed = read_test_xml(DynamicObjectType::Bed);
        monster.dropbear = read_test_xml(DynamicObjectType::DropBear);
        monster.gather_block = vec![26, 0xAA, 0xBB];
        monster.carrot_plant = read_test_xml(DynamicObjectType::CarrotPlant);
        monster.donkey = read_test_xml(DynamicObjectType::Donkey);
        monster.egg = read_test_xml(DynamicObjectType::Egg);
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
        let chest_metas: DynamicObjectList<ChestMeta> = read_test_xml(DynamicObjectType::Chest);
        monster.chest = chest_metas
            .into_inner()
            .into_iter()
            .map(|meta| {
                let id = *meta.unique_id.inner();
                let slot_path = format!("resources/chest_{}.xml.gz", id);
                let slot_bytes = std::fs::read(&slot_path).ok();
                Chest::from_meta_and_slots(meta, slot_bytes.as_deref())
                    .unwrap_or_else(|e| panic!("Failed to construct test chest {}: {:?}", id, e))
            })
            .collect();
        let chest_ids: Vec<_> = monster
            .chest
            .iter()
            .filter_map(|c| c.slots.as_slots().map(|_| c.unique_id.clone()))
            .collect();
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
                .open_database::<Str, Bytes>(&rtxn, Some("dw"))
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
            for chest_id in chest_ids {
                let key = format!("{}/chest_{}", coord_str, chest_id.inner());
                assert!(
                    db.get(&rtxn, &key).unwrap().is_some(),
                    "Key {} missing",
                    key
                );
            }

            let round_tripped_dw = DynamicWorld::from_db(&db, &rtxn).unwrap();
            let round_tripped_dw_chunk = round_tripped_dw.chunk_at(coord).unwrap();

            assert_eq!(round_tripped_dw_chunk, dw.chunk_at(coord).unwrap());
        }
    }
}
