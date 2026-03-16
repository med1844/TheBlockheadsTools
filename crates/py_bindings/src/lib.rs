use pyo3::{
    create_exception,
    exceptions::{PyException, PyValueError},
    prelude::*,
};
use snafu::prelude::*;
use std::sync::{Arc, RwLock};
use the_blockheads_tools_lib::{self as lib};

use lib::game::{
    block::BlockError, chunk::ChunkError, coord::CoordError, db::world_db::WorldDbError,
    item::ItemError,
};

pub type SharedWorldDb = Arc<RwLock<lib::game::db::world_db::WorldDb>>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum BindingError {
    CoordError { source: CoordError },
    BlockError { source: BlockError },
    ChunkError { source: ChunkError },
    WorldDbError { source: WorldDbError },
    ItemError { source: ItemError },
}

impl From<BindingError> for PyErr {
    fn from(value: BindingError) -> Self {
        let error_message = value.to_string();
        match value {
            BindingError::CoordError { .. }
            | BindingError::BlockError { .. }
            | BindingError::ItemError { .. } => PyValueError::new_err(error_message),
            BindingError::ChunkError { .. } | BindingError::WorldDbError { .. } => {
                PyException::new_err(error_message)
            }
        }
    }
}

// For accessors, they might point to no-longer valid resources.
// ```py
// world_db = WorldDb.open("save_file")
// chunk = world_db.chunks.chunk_at((432, 10))
// world_db.chunks.remove((432, 10))
// chunk.block_at((7, 13))  # InvalidAccessorError!
// ```
create_exception!(
    the_blockheads_tools_py,
    InvalidAccessorError,
    pyo3::exceptions::PyException
);

mod block;
mod chunk;
mod coord;
mod item;
mod world_db;

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn the_blockheads_tools_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<coord::BlockCoordPy>()?;
    m.add_class::<coord::ChunkBlockCoordPy>()?;
    m.add_class::<coord::ChunkCoordPy>()?;

    m.add_class::<block::BlockTypePy>()?;
    m.add_class::<block::BlockContentTypePy>()?;
    m.add_class::<block::BlockPy>()?;

    m.add_class::<chunk::ChunkPy>()?;
    m.add_class::<chunk::ChunksPy>()?;

    m.add_class::<item::ItemTypePy>()?;
    m.add_class::<item::ChestTypePy>()?;
    m.add_class::<item::WorkbenchTypePy>()?;
    m.add_class::<item::PigmentColorPy>()?;
    m.add_class::<item::ItemPy>()?;
    m.add_class::<item::SlotPy>()?;
    m.add_class::<item::BasketSlotsPy>()?;
    m.add_class::<item::InventoryPy>()?;
    m.add_class::<item::ChestPy>()?;
    m.add_class::<item::StandardChestPy>()?;
    m.add_class::<item::SafeChestPy>()?;
    m.add_class::<item::GoldChestPy>()?;
    m.add_class::<item::FeederChestPy>()?;
    m.add_class::<item::ShelfChestPy>()?;
    m.add_class::<item::CabinetPy>()?;
    m.add_class::<item::PortalChestPy>()?;
    m.add_class::<item::WorkbenchPy>()?;

    m.add_class::<world_db::WorldDbMainPy>()?;
    m.add_class::<world_db::WorldV2Py>()?;
    m.add_class::<world_db::DynamicWorldV2Py>()?;
    m.add_class::<world_db::BlockheadPy>()?;

    m.add_class::<world_db::ArchPy>()?;
    m.add_class::<world_db::WorldDbPy>()?;

    Ok(())
}
