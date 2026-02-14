use super::{into_py_err, lib};
use lib::game::coord::{BlockCoord, ChunkBlockCoord, ChunkCoord};
use pyo3::prelude::*;
use std::hash::Hash;

#[derive(Clone, Hash, PartialEq, Eq)]
#[pyclass(frozen, eq, hash, name = "ChunkCoord")]
pub struct ChunkCoordPy {
    pub(crate) inner: ChunkCoord,
}

#[pymethods]
impl ChunkCoordPy {
    #[new]
    fn new(x: u32, y: u8) -> PyResult<Self> {
        Ok(Self {
            inner: ChunkCoord::new(x, y).map_err(into_py_err)?,
        })
    }

    #[getter]
    fn get_x(&self) -> u32 {
        self.inner.x()
    }

    #[getter]
    fn get_y(&self) -> u8 {
        self.inner.y()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("ChunkCoord({}, {})", self.inner.x(), self.inner.y())
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
#[pyclass(frozen, eq, hash, name = "ChunkBlockCoord")]
pub struct ChunkBlockCoordPy {
    pub(crate) inner: ChunkBlockCoord,
}

#[pymethods]
impl ChunkBlockCoordPy {
    #[new]
    fn new(x: u8, y: u8) -> PyResult<Self> {
        Ok(Self {
            inner: ChunkBlockCoord::new(x, y).map_err(into_py_err)?,
        })
    }

    #[getter]
    fn get_x(&self) -> u8 {
        self.inner.x()
    }

    #[getter]
    fn get_y(&self) -> u8 {
        self.inner.y()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
#[pyclass(frozen, eq, hash, name = "BlockCoord")]
pub struct BlockCoordPy {
    pub(crate) inner: BlockCoord,
}

#[pymethods]
impl BlockCoordPy {
    #[new]
    fn new(x: u32, y: u16) -> PyResult<Self> {
        Ok(Self {
            inner: BlockCoord::new(x, y).map_err(into_py_err)?,
        })
    }

    #[getter]
    fn get_x(&self) -> u32 {
        self.inner.x()
    }

    #[getter]
    fn get_y(&self) -> u16 {
        self.inner.y()
    }

    fn decompose(&self) -> (ChunkCoordPy, ChunkBlockCoordPy) {
        let (chunk_coord, chunk_block_coord) = self.inner.decompose();
        (
            ChunkCoordPy { inner: chunk_coord },
            ChunkBlockCoordPy {
                inner: chunk_block_coord,
            },
        )
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}
