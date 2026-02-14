use super::{
    block::BlockPy,
    coord::{BlockCoordPy, ChunkBlockCoordPy, ChunkCoordPy},
    into_py_err, lib, SharedWorldDb,
};
use lib::game::{chunk::Chunk, coord::ChunkCoord};
use pyo3::prelude::*;
use std::{borrow::Cow, collections::HashSet};

#[pyclass(name = "Chunk")]
pub struct ChunkPy {
    inner: Chunk,
}

impl ChunkPy {
    pub(crate) fn inner(&self) -> &Chunk {
        &self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> &mut Chunk {
        &mut self.inner
    }
}

#[pymethods]
impl ChunkPy {
    #[classattr]
    const WIDTH: i32 = 32;

    #[classattr]
    const HEIGHT: i32 = 32;

    #[new]
    fn new() -> Self {
        Self {
            inner: Chunk::new_empty(),
        }
    }

    fn as_bytes(&'_ self) -> PyResult<Cow<'_, [u8]>> {
        Ok(Cow::Borrowed(self.inner.as_bytes()))
    }

    fn block_at(slf: Py<Self>, coord: ChunkBlockCoordPy, py: Python<'_>) -> BlockPy {
        BlockPy {
            chunk: slf.clone_ref(py),
            coord: coord.inner,
        }
    }
}

#[derive(FromPyObject)]
enum IntoChunkCoord {
    BlockCoord(BlockCoordPy),
    ChunkCoord(ChunkCoordPy),
}

impl From<IntoChunkCoord> for ChunkCoord {
    fn from(val: IntoChunkCoord) -> Self {
        match val {
            IntoChunkCoord::BlockCoord(block_coord_py) => {
                let (chunk_coord, _) = block_coord_py.inner.decompose();
                chunk_coord
            }
            IntoChunkCoord::ChunkCoord(chunk_coord_py) => chunk_coord_py.inner,
        }
    }
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

    fn chunk_at(&self, coord: IntoChunkCoord) -> PyResult<Option<ChunkPy>> {
        let world_db = self.world_db.read().unwrap();
        world_db
            .chunks
            .chunk_at(coord)
            .map(|compressed_chunk| {
                compressed_chunk
                    .decompress()
                    .map(|chunk| ChunkPy { inner: chunk })
            })
            .transpose()
            .map_err(into_py_err)
    }

    fn set_chunk_at(&self, coord: IntoChunkCoord, chunk: &ChunkPy) -> PyResult<()> {
        self.world_db
            .write()
            .unwrap()
            .chunks
            .set_chunk_at(coord, chunk.inner.compress().map_err(into_py_err)?);
        Ok(())
    }
}
