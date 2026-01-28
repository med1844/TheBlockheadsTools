use super::{
    block::{Block, BlockMut},
    coord::{BlockCoord, ChunkCoord, ChunkOffset},
};
use crate::util::{
    error::BhResult,
    gzip::{FromGzip, Gzip},
};
use flate2::read::GzDecoder;
use lmdb_rs::{
    codec::types::{Bytes, Str},
    database::Database,
    txn::{RoTxn, RwTxn},
};
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct Chunk(
    Box<[u8; Self::NUM_BLOCK_PER_ROW * Self::NUM_BLOCK_PER_COL * Self::NUM_BYTES_PER_BLOCK + 5]>,
); // 5 unknown bytes

impl Chunk {
    pub const NUM_BLOCK_PER_ROW: usize = 32;
    pub const NUM_BLOCK_PER_COL: usize = 32;
    pub const NUM_BYTES_PER_BLOCK: usize = 64;

    pub fn new_empty() -> Self {
        Self(Box::new([0; 32 * 32 * 64 + 5]))
    }

    pub fn inner(&self) -> &[u8] {
        self.0.as_ref()
    }

    pub fn inner_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }

    pub fn block_at<O: ChunkOffset>(&'_ self, coord: O) -> Block<'_> {
        let offset = coord.to_offset();
        let slice =
            <&[u8; 64]>::try_from(&self.inner()[offset..offset + Self::NUM_BYTES_PER_BLOCK])
                .expect(
                    "Return value of `ChunkBlockCoord.to_offset()` is guaranteed \
                to be smaller than chunk bytes len - 32",
                );
        Block::new(slice)
    }

    pub fn block_at_mut<O: ChunkOffset>(&'_ mut self, coord: O) -> BlockMut<'_> {
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

#[derive(Default)]
struct ChunkIndexIter {
    x: u32,
    y: u8,
}

impl Iterator for ChunkIndexIter {
    type Item = ChunkCoord;
    fn next(&mut self) -> Option<Self::Item> {
        let ret = Some(ChunkCoord::new(self.x, self.y).unwrap());
        // Assume num chunk per column is 32 chunks
        self.y += 1;
        self.x += (self.y >> 5) as u32;
        self.y &= 31;
        ret
    }
}

#[derive(Debug)]
pub struct Chunks(Vec<Option<Gzip<Chunk>>>);

impl Chunks {
    pub const NUM_CHUNK_PER_COL: usize = 32;

    fn to_index(coord: ChunkCoord) -> usize {
        coord.x() as usize * Self::NUM_CHUNK_PER_COL + coord.y() as usize
    }

    pub fn inner_mut(&mut self) -> &mut Vec<Option<Gzip<Chunk>>> {
        &mut self.0
    }

    pub fn from_db(db: &Database<Str, Bytes>, rtxn: &RoTxn, world_width: u32) -> BhResult<Self> {
        let mut chunks = vec![None; world_width as usize * Self::NUM_CHUNK_PER_COL];
        for pair in db.iter(rtxn)? {
            if let Ok((k, v)) = pair
                && let Ok(coord) = ChunkCoord::try_from_str(k)
            {
                chunks[Self::to_index(coord)] = Some(Gzip::Compressed(v.to_owned()));
            }
        }
        Ok(Self(chunks))
    }

    pub fn to_db<W: Write>(&self, db: &Database<Str, Bytes>, wtxn: &mut RwTxn<W>) -> BhResult<()> {
        for (coord, chunk) in ChunkIndexIter::default().zip(self.0.iter()) {
            if let Some(chunk) = chunk {
                let data = chunk.to_compressed()?;
                db.put(wtxn, coord.to_string().as_str(), &data)?;
            }
        }
        Ok(())
    }

    pub fn keys(&self) -> Vec<ChunkCoord> {
        ChunkIndexIter::default()
            .zip(self.0.iter())
            .filter_map(|(coord, chunk)| chunk.is_some().then_some(coord))
            .collect()
    }

    pub fn contains_key(&self, key: ChunkCoord) -> bool {
        self.0.get(Self::to_index(key)).is_some()
    }

    pub fn chunk_at<I: Into<ChunkCoord>>(&self, coord: I) -> Option<&Gzip<Chunk>> {
        self.0
            .get(Self::to_index(coord.into()))
            .and_then(|out| out.as_ref())
    }

    pub fn chunk_at_mut<I: Into<ChunkCoord>>(&mut self, coord: I) -> Option<&mut Gzip<Chunk>> {
        self.0
            .get_mut(Self::to_index(coord.into()))
            .and_then(|out| out.as_mut())
    }

    pub fn set_chunk_at<I: Into<ChunkCoord>>(&mut self, coord: I, chunk: Gzip<Chunk>) {
        self.0[Self::to_index(coord.into())] = Some(chunk);
    }

    pub fn block_at<I: Into<BlockCoord>>(
        &'_ mut self,
        coord: I,
    ) -> Option<std::io::Result<Block<'_>>> {
        let block_coord = coord.into();
        let (chunk_coord, chunk_block_coord) = block_coord.decompose();
        self.chunk_at_mut(chunk_coord).map(|v| {
            v.as_uncompressed_mut()
                .map(|chunk| chunk.block_at(chunk_block_coord))
        })
    }

    pub fn block_at_mut<I: Into<BlockCoord>>(
        &'_ mut self,
        coord: I,
    ) -> Option<std::io::Result<BlockMut<'_>>> {
        let block_coord = coord.into();
        let (chunk_coord, chunk_block_coord) = block_coord.decompose();
        self.chunk_at_mut(chunk_coord).map(|v| {
            v.as_uncompressed_mut()
                .map(|chunk| chunk.block_at_mut(chunk_block_coord))
        })
    }
}
