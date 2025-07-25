use super::{
    block::{Block, BlockMut},
    coord::{BlockCoord, ChunkCoord, ChunkOffset},
};
use crate::util::{
    error::BhResult,
    gzip::{FromGzip, Gzip},
};
use flate2::read::GzDecoder;
use heed::{Database, RoTxn, RwTxn, types::*};
use std::{collections::HashMap, io::Read};

#[derive(Debug)]
pub struct Chunk(
    [u8; Self::NUM_BLOCK_PER_ROW * Self::NUM_BLOCK_PER_COL * Self::NUM_BYTES_PER_BLOCK + 5],
); // 5 unknown bytes

impl Chunk {
    pub const NUM_BLOCK_PER_ROW: usize = 32;
    pub const NUM_BLOCK_PER_COL: usize = 32;
    pub const NUM_BYTES_PER_BLOCK: usize = 64;

    fn new_empty() -> Self {
        Self([0; 32 * 32 * 64 + 5])
    }

    pub fn inner(&self) -> &[u8] {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }

    pub fn block_at<O: ChunkOffset>(&self, coord: O) -> Block {
        let offset = coord.to_offset();
        let slice =
            <&[u8; 64]>::try_from(&self.inner()[offset..offset + Self::NUM_BYTES_PER_BLOCK])
                .expect(
                    "Return value of `ChunkBlockCoord.to_offset()` is guaranteed \
                to be smaller than chunk bytes len - 32",
                );
        Block::new(slice)
    }

    pub fn block_at_mut<O: ChunkOffset>(&mut self, coord: O) -> BlockMut {
        let offset = coord.to_offset();
        let slice = <&mut [u8; 64]>::try_from(
            &mut self.inner_mut()[offset..offset + Self::NUM_BYTES_PER_BLOCK],
        )
        .expect(
            "Return value of `ChunkBlockCoord.to_offset()` is guaranteed \
                to be smaller than chunk bytes len - 32",
        );
        BlockMut::new(slice)
    }
}

impl AsRef<[u8]> for Chunk {
    fn as_ref(&self) -> &[u8] {
        self.inner()
    }
}

impl FromGzip for Chunk {
    fn from_compressed_gzip(bytes: &[u8]) -> Result<Self, std::io::Error> {
        let mut decoder = GzDecoder::new(bytes);
        let mut chunk = Self::new_empty();
        decoder.read_exact(chunk.inner_mut())?;
        Ok(chunk)
    }
}

#[derive(Debug)]
pub struct Chunks(HashMap<ChunkCoord, Gzip<Chunk>>);

impl Chunks {
    pub fn from_db(db: &Database<Str, Bytes>, rtxn: &RoTxn) -> Result<Self, heed::Error> {
        Ok(Self(
            db.iter(rtxn)?
                .filter_map(|v| v.ok())
                .filter_map(|(k, v)| {
                    ChunkCoord::try_from_str(k)
                        .ok()
                        .map(|k| (k, Gzip::from_compressed(v.to_owned())))
                })
                .collect(),
        ))
    }

    pub fn to_db(&self, db: &Database<Str, Bytes>, wtxn: &mut RwTxn) -> BhResult<()> {
        for (key, value) in self.0.iter() {
            let data = value.into_compressed()?;
            db.put(wtxn, key.to_string().as_str(), &data)?;
        }
        Ok(())
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, ChunkCoord, Gzip<Chunk>> {
        self.0.keys()
    }

    pub fn contains_key(&self, key: &ChunkCoord) -> bool {
        self.0.contains_key(key)
    }

    pub fn chunk_at<I: Into<ChunkCoord>>(&mut self, coord: I) -> Option<std::io::Result<&Chunk>> {
        self.0.get_mut(&coord.into()).map(|v| v.as_uncompressed())
    }

    pub fn chunk_at_mut<I: Into<ChunkCoord>>(
        &mut self,
        coord: I,
    ) -> Option<std::io::Result<&mut Chunk>> {
        self.0
            .get_mut(&coord.into())
            .map(|v| v.as_uncompressed_mut())
    }

    pub fn block_at<I: Into<BlockCoord>>(&mut self, coord: I) -> Option<std::io::Result<Block>> {
        let block_coord = coord.into();
        let (chunk_coord, chunk_block_coord) = block_coord.decompose();
        self.0.get_mut(&chunk_coord).map(|v| {
            v.as_uncompressed_mut()
                .map(|chunk| chunk.block_at(chunk_block_coord))
        })
    }

    pub fn block_at_mut<I: Into<BlockCoord>>(
        &mut self,
        coord: I,
    ) -> Option<std::io::Result<BlockMut>> {
        let block_coord = coord.into();
        let (chunk_coord, chunk_block_coord) = block_coord.decompose();
        self.0.get_mut(&chunk_coord).map(|v| {
            v.as_uncompressed_mut()
                .map(|chunk| chunk.block_at_mut(&chunk_block_coord))
        })
    }
}
