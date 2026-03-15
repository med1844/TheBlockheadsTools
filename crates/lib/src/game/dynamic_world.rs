use super::{
    coord::{ChunkCoord, CoordError},
    dynamic_object::{
        DynamicObjectList, DynamicObjectType,
        animal::{CaveTroll, ClownFish, Dodo, Donkey, DropBear, Scorpion, Shark, Yak},
        chest::{Chest, ChestError, ChestMeta},
        craft::{
            Bed, Boat, Column, Door, ElevatorMotor, ElevatorShaft, Ladder, Rail, Sign, Stairs,
            TradePortal, TradingPost, Window, Wire,
        },
        dropped_item::DroppedItem,
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
use lmdb_rs::{
    codec::types::{Bytes, Str},
    database::Database,
    txn::{RoTxn, RwTxn},
};
use serde::Serialize;
use snafu::prelude::*;
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
            + self.dropped_item.num_obj()
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

#[derive(Debug, Snafu)]
pub enum DynamicWorldError {
    #[snafu(display("Failed to iterate over database: {source}"))]
    IterateDatabase {
        source: lmdb_rs::error::DatabaseError,
    },
    #[snafu(display("Failed to deserialize {object_type:?}: {source}"))]
    DeserializeObject {
        object_type: DynamicObjectType,
        source: plist::Error,
    },
    #[snafu(display("Failed to serialize {object_type:?}: {source}"))]
    SerializeObject {
        object_type: DynamicObjectType,
        source: plist::Error,
    },
    #[snafu(display("Failed to get entry {key} from database: {source}"))]
    GetEntry {
        source: lmdb_rs::error::DatabaseError,
        key: String,
    },
    #[snafu(display("Failed to put entry with key {key} in database: {source}"))]
    PutEntry {
        key: String,
        source: lmdb_rs::error::DatabaseError,
    },
    #[snafu(display("Failed to load chest {id} in chunk {coord}: {source}"))]
    LoadChest {
        id: u64,
        coord: ChunkCoord,
        source: ChestError,
    },
    #[snafu(display("Failed to save chest {id} in chunk {coord}: {source}"))]
    SaveChest {
        id: u64,
        coord: ChunkCoord,
        source: ChestError,
    },
    #[snafu(display("Failed to parse chunk coord {coord}: {source}"))]
    ParseChunkCoordFromStr { coord: String, source: CoordError },
}

type Result<T> = std::result::Result<T, DynamicWorldError>;

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
            let Ok(obj_type) = DynamicObjectType::try_from_str(type_id_str) else {
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
            ) -> Result<T> {
                plist::from_bytes(bytes).context(DeserializeObjectSnafu {
                    object_type: dyn_obj_type,
                })
            }

            match obj_type {
                DynamicObjectType::AppleTree => entry.apple_tree = load(v, obj_type)?,
                DynamicObjectType::MapleTree => entry.maple_tree = load(v, obj_type)?,
                DynamicObjectType::MangoTree => entry.mango_tree = load(v, obj_type)?,
                DynamicObjectType::PineTree => entry.pine_tree = load(v, obj_type)?,
                DynamicObjectType::CactusTree => entry.cactus_tree = load(v, obj_type)?,
                DynamicObjectType::CoconutTree => entry.coconut_tree = load(v, obj_type)?,
                DynamicObjectType::OrangeTree => entry.orange_tree = load(v, obj_type)?,
                DynamicObjectType::CherryTree => entry.cherry_tree = load(v, obj_type)?,
                DynamicObjectType::CoffeeTree => entry.coffee_tree = load(v, obj_type)?,
                DynamicObjectType::FlaxPlant => entry.flax_plant = load(v, obj_type)?,
                DynamicObjectType::SunflowerPlant => entry.sunflower_plant = load(v, obj_type)?,
                DynamicObjectType::CornPlant => entry.corn_plant = load(v, obj_type)?,
                DynamicObjectType::Dodo => entry.dodo = load(v, obj_type)?,
                DynamicObjectType::DroppedItem => entry.dropped_item = load(v, obj_type)?,
                DynamicObjectType::Fire => entry.fire = v.to_vec(),
                DynamicObjectType::Torch => entry.torch = v.to_vec(),
                DynamicObjectType::GlowBlock => entry.glow_block = v.to_vec(),
                DynamicObjectType::Ladder => entry.ladder = load(v, obj_type)?,
                DynamicObjectType::Door => entry.door = load(v, obj_type)?,
                DynamicObjectType::ArtificialLight => entry.artificial_light = v.to_vec(),
                DynamicObjectType::Bed => entry.bed = load(v, obj_type)?,
                DynamicObjectType::DropBear => entry.dropbear = load(v, obj_type)?,
                DynamicObjectType::GatherBlock => entry.gather_block = v.to_vec(),
                DynamicObjectType::CarrotPlant => entry.carrot_plant = load(v, obj_type)?,
                DynamicObjectType::Donkey => entry.donkey = load(v, obj_type)?,
                DynamicObjectType::Egg => entry.egg = v.to_vec(),
                DynamicObjectType::Window => entry.window = load(v, obj_type)?,
                DynamicObjectType::Boat => entry.boat = load(v, obj_type)?,
                DynamicObjectType::ChilliPlant => entry.chilli_plant = load(v, obj_type)?,
                DynamicObjectType::KelpPlant => entry.kelp_plant = load(v, obj_type)?,
                DynamicObjectType::ClownFish => entry.clown_fish = load(v, obj_type)?,
                DynamicObjectType::Shark => entry.shark = load(v, obj_type)?,
                DynamicObjectType::LimeTree => entry.lime_tree = load(v, obj_type)?,
                DynamicObjectType::Wire => entry.wire = load(v, obj_type)?,
                DynamicObjectType::CaveTroll => entry.cave_troll = load(v, obj_type)?,
                DynamicObjectType::Rail => entry.rail = load(v, obj_type)?,
                DynamicObjectType::HandCar => entry.hand_car = load(v, obj_type)?,
                DynamicObjectType::SteamLocomotive => entry.steam_locomotive = load(v, obj_type)?,
                DynamicObjectType::FreightCar => entry.freight_car = load(v, obj_type)?,
                DynamicObjectType::PassengerCar => entry.passenger_car = load(v, obj_type)?,
                DynamicObjectType::Workbench => entry.workbench = load(v, obj_type)?,
                DynamicObjectType::Chest => {
                    let chest_metas: DynamicObjectList<ChestMeta> =
                        plist::from_bytes(v).context(DeserializeObjectSnafu {
                            object_type: obj_type,
                        })?;
                    entry.chest = chest_metas
                        .into_inner()
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
                DynamicObjectType::Sign => entry.sign = load(v, obj_type)?,
                DynamicObjectType::TradingPost => entry.trading_post = load(v, obj_type)?,
                DynamicObjectType::TrainStation => entry.train_station = load(v, obj_type)?,
                DynamicObjectType::TradePortal => entry.trade_portal = load(v, obj_type)?,
                DynamicObjectType::Scorpion => entry.scorpion = load(v, obj_type)?,
                DynamicObjectType::Painting => entry.painting = v.to_vec(),
                DynamicObjectType::Column => entry.column = load(v, obj_type)?,
                DynamicObjectType::Stairs => entry.stairs = load(v, obj_type)?,
                DynamicObjectType::ElevatorMotor => entry.elevator_motor = load(v, obj_type)?,
                DynamicObjectType::ElevatorShaft => entry.elevator_shaft = load(v, obj_type)?,
                DynamicObjectType::GemTree => entry.gem_tree = load(v, obj_type)?,
                DynamicObjectType::VinePlant => entry.vine_plant = load(v, obj_type)?,
                DynamicObjectType::TulipPlant => entry.tulip_plant = load(v, obj_type)?,
                DynamicObjectType::OwnershipSign => entry.ownership_sign = v.to_vec(),
                DynamicObjectType::WheatPlant => entry.wheat_plant = load(v, obj_type)?,
                DynamicObjectType::TomatoPlant => entry.tomato_plant = load(v, obj_type)?,
                DynamicObjectType::Yak => entry.yak = load(v, obj_type)?,
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
                    &value.to_plist().context(SerializeObjectSnafu {
                        object_type: obj_type,
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
            put(db, wtxn, &coord_str, DroppedItem, &obj.dropped_item)?;
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
    use super::{ChunkDynamicObjects, DynamicObjectType, DynamicWorld};
    use crate::game::{
        coord::ChunkCoord,
        dynamic_object::{
            DynamicObjectList,
            chest::{Chest, ChestMeta},
        },
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
        monster.dropped_item = read_test_xml(DynamicObjectType::DroppedItem);
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
