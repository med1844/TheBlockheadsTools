use super::{
    block::BlockPy,
    coord::{BlockCoordPy, ChunkBlockCoordPy, ChunkCoordPy},
    into_py_err, lib, InvalidAccessorError, SharedWorldDb,
};
use lib::game::coord::{BlockCoord, ChunkCoord};
use pyo3::prelude::*;
use std::{borrow::Cow, collections::HashSet};

#[pyclass(name = "Chunk")]
pub struct ChunkPy {
    world_db: SharedWorldDb,
    coord: ChunkCoord,
}

#[pymethods]
impl ChunkPy {
    fn as_bytes(&'_ self) -> PyResult<Cow<'_, [u8]>> {
        let world_db = self.world_db.read().unwrap();
        let chunk = world_db.chunks.chunk_at(self.coord);
        match chunk {
            Some(chunk) => Ok(Cow::Owned(
                chunk.decompress().map_err(into_py_err)?.inner().to_vec(),
            )),
            None => Err(InvalidAccessorError::new_err(format!(
                "The chunk at {} doesn't exist.",
                self.coord
            ))),
        }
    }

    fn block_at(&self, coord: ChunkBlockCoordPy) -> BlockPy {
        BlockPy {
            world_db: self.world_db.clone(),
            block_coord: BlockCoord::from_decomposed(self.coord, coord.inner),
        }
    }
}

#[derive(FromPyObject)]
enum IntoChunkCoord {
    BlockCoord(BlockCoordPy),
    ChunkCoord(ChunkCoordPy),
}

#[pyclass(name = "Chunks")]
pub struct ChunksPy {
    pub(crate) world_db: SharedWorldDb,
}

#[pymethods]
impl ChunksPy {
    fn __contains__(&self, coord: &ChunkCoordPy) -> bool {
        let world_db = self.world_db.read().unwrap();
        world_db.chunks.contains_key(coord.inner)
    }

    fn keys(&self) -> HashSet<ChunkCoordPy> {
        // Ton's of allocation, but I guess python users will be ok with that.
        let world_db = self.world_db.read().unwrap();
        HashSet::from_iter(
            world_db
                .chunks
                .keys()
                .map(|value| ChunkCoordPy { inner: value }),
        )
    }

    fn chunk_at(&self, coord: IntoChunkCoord) -> Option<ChunkPy> {
        let chunk_coord = match coord {
            IntoChunkCoord::BlockCoord(block_coord_py) => {
                let (chunk_coord, _) = block_coord_py.inner.decompose();
                chunk_coord
            }
            IntoChunkCoord::ChunkCoord(chunk_coord_py) => chunk_coord_py.inner,
        };
        let world_db = self.world_db.read().unwrap();
        world_db.chunks.contains_key(chunk_coord).then(|| ChunkPy {
            world_db: self.world_db.clone(),
            coord: chunk_coord,
        })
    }

    fn block_at(&self, coord: BlockCoordPy) -> Option<BlockPy> {
        let (chunk_coord, _) = coord.inner.decompose();
        let world_db = self.world_db.read().unwrap();
        world_db
            .chunks
            .contains_key(chunk_coord)
            .then_some(BlockPy {
                world_db: self.world_db.clone(),
                block_coord: coord.inner,
            })
    }
}
