use super::{chunk::ChunksPy, into_py_err, item::InventoryPy, lib, SharedWorldDb};
use lib::game::db::world_db::WorldDb;
use pyo3::prelude::*;
use std::{
    borrow::Cow,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

#[pyclass(name = "WorldV2")]
pub struct WorldV2Py {
    inner: SharedWorldDb,
}

#[pymethods]
impl WorldV2Py {
    #[getter]
    fn get_blockhead_datas_v2(&self) -> String {
        let world_db = self.inner.read().unwrap();
        format!("{:?}", world_db.main.world_v2.blockhead_datas_v2)
    }

    #[getter]
    fn get_circum_navigate_booleans_data(&'_ self) -> Cow<'_, [u8]> {
        let world_db = self.inner.read().unwrap();
        Cow::Owned(
            world_db
                .main
                .world_v2
                .circum_navigate_booleans_data
                .as_ref()
                .to_vec(),
        )
    }

    #[setter]
    fn set_circum_navigate_booleans_data(&self, value: Vec<u8>) {
        let mut world_db = self.inner.write().unwrap();
        world_db.main.world_v2.circum_navigate_booleans_data = value.into();
    }

    #[getter]
    fn get_creation_date(&self) -> String {
        let world_db = self.inner.read().unwrap();
        format!("{:?}", world_db.main.world_v2.creation_date)
    }

    #[getter]
    fn get_distance_ordered_food_types(&self) -> Cow<'_, [u8]> {
        let world_db = self.inner.read().unwrap();
        Cow::Owned(
            world_db
                .main
                .world_v2
                .distance_ordered_food_types
                .as_ref()
                .to_vec(),
        )
    }

    #[setter]
    fn set_distance_ordered_food_types(&self, value: Vec<u8>) {
        let mut world_db = self.inner.write().unwrap();
        world_db.main.world_v2.distance_ordered_food_types = value.into();
    }

    #[getter]
    fn get_expert_mode(&self) -> bool {
        self.inner.read().unwrap().main.world_v2.expert_mode
    }

    #[setter]
    fn set_expert_mode(&self, value: bool) {
        self.inner.write().unwrap().main.world_v2.expert_mode = value;
    }

    #[getter]
    fn get_found_items(&'_ self) -> Cow<'_, [u8]> {
        let world_db = self.inner.read().unwrap();
        Cow::Owned(world_db.main.world_v2.found_items.as_ref().to_vec())
    }

    #[setter]
    fn set_found_items(&self, value: Vec<u8>) {
        self.inner.write().unwrap().main.world_v2.found_items = value.into();
    }

    #[getter]
    fn get_host_port(&self) -> Option<String> {
        self.inner.read().unwrap().main.world_v2.host_port.clone()
    }

    #[setter]
    fn set_host_port(&self, value: Option<&str>) {
        self.inner.write().unwrap().main.world_v2.host_port = value.map(ToString::to_string);
    }

    #[getter]
    fn get_max_players(&self) -> Option<String> {
        self.inner.read().unwrap().main.world_v2.max_players.clone()
    }

    #[setter]
    fn set_max_players(&self, value: Option<&str>) {
        self.inner.write().unwrap().main.world_v2.max_players = value.map(ToString::to_string);
    }

    #[getter]
    fn get_migration_complete_v1_7(&self) -> bool {
        self.inner
            .read()
            .unwrap()
            .main
            .world_v2
            .migration_complete_v1_7
    }

    #[setter]
    fn set_migration_complete_v1_7(&self, value: bool) {
        self.inner
            .write()
            .unwrap()
            .main
            .world_v2
            .migration_complete_v1_7 = value;
    }

    #[getter]
    fn get_no_rain_timer(&self) -> f64 {
        self.inner.read().unwrap().main.world_v2.no_rain_timer
    }

    #[setter]
    fn set_no_rain_timer(&self, value: f64) {
        self.inner.write().unwrap().main.world_v2.no_rain_timer = value;
    }

    #[getter]
    fn get_portal_level(&self) -> u64 {
        self.inner.read().unwrap().main.world_v2.portal_level
    }

    #[setter]
    fn set_portal_level(&self, value: u64) {
        self.inner.write().unwrap().main.world_v2.portal_level = value;
    }

    #[getter]
    fn get_random_seed(&self) -> u64 {
        self.inner.read().unwrap().main.world_v2.random_seed
    }

    #[setter]
    fn set_random_seed(&self, value: u64) {
        self.inner.write().unwrap().main.world_v2.random_seed = value;
    }

    #[getter]
    fn get_remote_game(&self) -> bool {
        self.inner.read().unwrap().main.world_v2.remote_game
    }

    #[setter]
    fn set_remote_game(&self, value: bool) {
        self.inner.write().unwrap().main.world_v2.remote_game = value;
    }

    #[getter]
    fn get_run_at_launch(&self) -> bool {
        self.inner.read().unwrap().main.world_v2.run_at_launch
    }

    #[setter]
    fn set_run_at_launch(&self, value: bool) {
        self.inner.write().unwrap().main.world_v2.run_at_launch = value;
    }

    #[getter]
    fn get_save_date(&self) -> String {
        let world_db = self.inner.read().unwrap();
        format!("{:?}", world_db.main.world_v2.save_date)
    }

    #[getter]
    fn get_save_id(&self) -> String {
        self.inner.read().unwrap().main.world_v2.save_id.clone()
    }

    #[setter]
    fn set_save_id(&self, value: &str) {
        self.inner.write().unwrap().main.world_v2.save_id = value.to_string();
    }

    #[getter]
    fn get_save_version(&self) -> u64 {
        self.inner.read().unwrap().main.world_v2.save_version
    }

    #[setter]
    fn set_save_version(&self, value: u64) {
        self.inner.write().unwrap().main.world_v2.save_version = value;
    }

    #[getter]
    fn get_start_portal_pos_x(&self) -> u64 {
        self.inner.read().unwrap().main.world_v2.start_portal_pos_x
    }

    #[setter]
    fn set_start_portal_pos_x(&self, value: u64) {
        self.inner.write().unwrap().main.world_v2.start_portal_pos_x = value;
    }

    #[getter]
    fn get_start_portal_pos_y(&self) -> u64 {
        self.inner.read().unwrap().main.world_v2.start_portal_pos_y
    }

    #[setter]
    fn set_start_portal_pos_y(&self, value: u64) {
        self.inner.write().unwrap().main.world_v2.start_portal_pos_y = value;
    }

    #[getter]
    fn get_translation(&self) -> (f64, f64) {
        self.inner.read().unwrap().main.world_v2.translation
    }

    #[setter]
    fn set_translation(&self, value: (f64, f64)) {
        self.inner.write().unwrap().main.world_v2.translation = value;
    }

    #[getter]
    fn get_world_name(&self) -> String {
        self.inner.read().unwrap().main.world_v2.world_name.clone()
    }

    #[setter]
    fn set_world_name(&self, value: &str) {
        self.inner.write().unwrap().main.world_v2.world_name = value.to_string();
    }

    #[getter]
    fn get_world_time(&self) -> f64 {
        self.inner.read().unwrap().main.world_v2.world_time
    }

    #[setter]
    fn set_world_time(&self, value: f64) {
        self.inner.write().unwrap().main.world_v2.world_time = value;
    }

    #[getter]
    fn get_world_width_macro(&self) -> u32 {
        self.inner.read().unwrap().main.world_v2.world_width_macro
    }

    #[setter]
    fn set_world_width_macro(&self, value: u32) {
        self.inner.write().unwrap().main.world_v2.world_width_macro = value;
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.inner.read().unwrap().main.world_v2)
    }
}

#[pyclass(name = "DynamicWorldV2")]
pub struct DynamicWorldV2Py {
    inner: SharedWorldDb,
}

impl DynamicWorldV2Py {
    fn read(&self) -> RwLockReadGuard<'_, WorldDb> {
        self.inner.read().unwrap()
    }

    fn write(&self) -> RwLockWriteGuard<'_, WorldDb> {
        self.inner.write().unwrap()
    }
}

#[pymethods]
impl DynamicWorldV2Py {
    #[getter]
    fn get_active_blockhead_index(&self) -> u64 {
        self.read().main.dynamic_world_v2.active_blockhead_index
    }

    #[setter]
    fn set_active_blockhead_index(&self, value: u64) {
        self.write().main.dynamic_world_v2.active_blockhead_index = value;
    }

    #[getter]
    fn get_dynamic_object_id_count(&self) -> u64 {
        self.read().main.dynamic_world_v2.dynamic_object_id_count
    }

    #[setter]
    fn set_dynamic_object_id_count(&self, value: u64) {
        self.write().main.dynamic_world_v2.dynamic_object_id_count = value;
    }

    #[getter]
    fn get_save_version(&self) -> u8 {
        self.read().main.dynamic_world_v2.save_version
    }

    #[setter]
    fn set_save_version(&self, value: u8) {
        self.write().main.dynamic_world_v2.save_version = value;
    }

    #[getter]
    fn get_saved_glow_indices(&'_ self) -> Cow<'_, [u8]> {
        Cow::Owned(
            self.read()
                .main
                .dynamic_world_v2
                .saved_glow_indices
                .as_ref()
                .to_vec(),
        )
    }

    #[setter]
    fn set_saved_glow_indices(&self, value: Vec<u8>) {
        self.write().main.dynamic_world_v2.saved_glow_indices = value.into();
    }

    #[getter]
    fn get_workbench_has_been_crafted(&self) -> bool {
        self.read().main.dynamic_world_v2.workbench_has_been_crafted
    }

    #[setter]
    fn set_workbench_has_been_crafted(&self, value: bool) {
        self.write()
            .main
            .dynamic_world_v2
            .workbench_has_been_crafted = value;
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.inner.read().unwrap().main.dynamic_world_v2)
    }
}

#[pyclass(name = "Blockhead")]
pub struct BlockheadPy {
    inner: SharedWorldDb,
    index: usize,
}

impl BlockheadPy {
    fn read(&self) -> RwLockReadGuard<'_, WorldDb> {
        self.inner.read().unwrap()
    }

    fn write(&self) -> RwLockWriteGuard<'_, WorldDb> {
        self.inner.write().unwrap()
    }
}

#[pymethods]
impl BlockheadPy {
    #[getter]
    fn get_name(&self) -> String {
        self.read().main.blockheads[self.index].name.clone()
    }

    #[setter]
    fn set_name(&self, value: String) {
        self.write().main.blockheads[self.index].name = value;
    }

    #[getter]
    fn get_clothing_increment_timer(&self) -> u64 {
        self.read().main.blockheads[self.index].clothing_increment_timer
    }

    #[setter]
    fn set_clothing_increment_timer(&self, value: u64) {
        self.write().main.blockheads[self.index].clothing_increment_timer = value;
    }

    #[getter]
    fn get_double_time_unlocked(&self) -> bool {
        self.read().main.blockheads[self.index].double_time_unlocked
    }

    #[setter]
    fn set_double_time_unlocked(&self, value: bool) {
        self.write().main.blockheads[self.index].double_time_unlocked = value;
    }

    #[getter]
    fn get_skin_options(&self) -> Cow<'_, [u8]> {
        Cow::Owned(
            self.read().main.blockheads[self.index]
                .skin_options
                .as_ref()
                .to_vec(),
        )
    }

    #[setter]
    fn set_skin_options(&self, value: Vec<u8>) {
        self.write().main.blockheads[self.index].skin_options = value.into();
    }

    #[getter]
    fn get_state(&self) -> Cow<'_, [u8]> {
        Cow::Owned(
            self.read().main.blockheads[self.index]
                .state
                .as_ref()
                .to_vec(),
        )
    }

    #[setter]
    fn set_state(&self, value: Vec<u8>) {
        self.write().main.blockheads[self.index].state = value.into();
    }

    #[getter]
    fn get_inventory(&self, py: Python<'_>) -> PyResult<Option<Py<InventoryPy>>> {
        let world_db = self.read();
        let unique_id = world_db.main.blockheads[self.index].obj.unique_id.clone();
        if let Some(inv) = world_db.main.blockhead_inventories.get(&unique_id) {
            Ok(Some(InventoryPy::inflate(py, inv.clone())?))
        } else {
            Ok(None)
        }
    }

    #[setter]
    fn set_inventory(&self, py: Python<'_>, inventory: Option<Py<InventoryPy>>) -> PyResult<()> {
        let mut world_db = self.write();
        let unique_id = world_db.main.blockheads[self.index].obj.unique_id.clone();
        if let Some(inv_py) = inventory {
            let inv = inv_py.bind(py).borrow().deflate(py);
            world_db.main.blockhead_inventories.insert(unique_id, inv);
        } else {
            world_db.main.blockhead_inventories.remove(&unique_id);
        }
        Ok(())
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.read().main.blockheads[self.index])
    }
}

#[pyclass(name = "WorldDbMain")]
pub struct WorldDbMainPy {
    inner: SharedWorldDb,
}

#[pymethods]
impl WorldDbMainPy {
    #[getter]
    fn get_blockheads(&'_ self) -> Vec<BlockheadPy> {
        self.inner
            .read()
            .unwrap()
            .main
            .blockheads
            .iter()
            .enumerate()
            .map(|(index, _)| BlockheadPy {
                inner: self.inner.clone(),
                index,
            })
            .collect()
    }

    #[getter]
    fn get_dynamic_world_v2(&'_ self) -> DynamicWorldV2Py {
        DynamicWorldV2Py {
            inner: self.inner.clone(),
        }
    }

    #[getter]
    fn get_world_v2(&self) -> WorldV2Py {
        WorldV2Py {
            inner: self.inner.clone(),
        }
    }

    #[getter]
    fn blockhead_inventory_keys(&self) -> std::collections::HashSet<u64> {
        self.inner
            .read()
            .unwrap()
            .main
            .blockhead_inventories
            .keys()
            .map(|id| *id.inner())
            .collect()
    }

    fn get_blockhead_inventory(
        &self,
        py: Python<'_>,
        id: u64,
    ) -> PyResult<Option<Py<InventoryPy>>> {
        let world_db = self.inner.read().unwrap();
        if let Some(inv) = world_db
            .main
            .blockhead_inventories
            .get(&lib::game::dw::dynamic_object::UniqueID::new(id))
        {
            Ok(Some(InventoryPy::inflate(py, inv.clone())?))
        } else {
            Ok(None)
        }
    }

    fn set_blockhead_inventory(
        &self,
        py: Python<'_>,
        id: u64,
        inventory: Option<Py<InventoryPy>>,
    ) -> PyResult<()> {
        let mut world_db = self.inner.write().unwrap();
        let unique_id = lib::game::dw::dynamic_object::UniqueID::new(id);
        if let Some(inv_py) = inventory {
            let inv = inv_py.bind(py).borrow().deflate(py);
            world_db.main.blockhead_inventories.insert(unique_id, inv);
        } else {
            world_db.main.blockhead_inventories.remove(&unique_id);
        }
        Ok(())
    }
}

#[pyclass(name = "WorldDb")]
pub struct WorldDbPy {
    // Python doesn't care about lifetimes. Thus we model the save file in the pythonic way.
    // This imposes severe runtime expense - each time some downstream accessor accesses some data in world_db,
    // we need to get the mutex lock, which is slow as hell.
    // Every object other than trivial ones like coords will hold a shared reference.
    inner: SharedWorldDb,
}

#[pymethods]
impl WorldDbPy {
    #[staticmethod]
    fn open(path: &str) -> PyResult<Self> {
        let world_db = WorldDb::from_path(path).map_err(into_py_err)?;
        Ok(Self {
            inner: Arc::new(RwLock::new(world_db)),
        })
    }

    fn save(&self, path: &str) -> PyResult<()> {
        self.inner
            .read()
            .unwrap()
            .to_path(path)
            .map_err(into_py_err)?;
        Ok(())
    }

    #[getter]
    fn get_chunks(&self) -> ChunksPy {
        ChunksPy {
            world_db: self.inner.clone(),
        }
    }

    #[getter]
    fn get_main(&self) -> WorldDbMainPy {
        WorldDbMainPy {
            inner: self.inner.clone(),
        }
    }
}
