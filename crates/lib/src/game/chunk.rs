use super::{
    block::{BlockView, BlockViewMut},
    coord::{BlockCoord, ChunkCoord, ChunkOffset},
};
use crate::util::{
    error::BhResult,
    gzip::{decompress_exact_into, decompress_into},
};
use lmdb_rs::{
    codec::types::{Bytes, Str},
    database::Database,
    txn::{RoTxn, RwTxn},
};
use std::io::Write;

type ChunkBytes = [u8; Chunk::NUM_BYTES];

#[derive(Debug, Clone, Copy)]
pub struct ChunkSlice<'a>(&'a ChunkBytes);

impl<'a> ChunkSlice<'a> {
    pub fn block_at<O: ChunkOffset>(&self, coord: O) -> BlockView<'a> {
        let offset = coord.to_offset();
        let slice =
            <&[u8; 64]>::try_from(&self.0.as_slice()[offset..offset + Chunk::NUM_BYTES_PER_BLOCK])
                .expect(
                    "Return value of `ChunkBlockCoord.to_offset()` is guaranteed \
                to be smaller than chunk bytes len - 32",
                );
        BlockView::new(slice)
    }
}

#[derive(Debug)]
pub struct ChunkSliceMut<'a>(&'a mut ChunkBytes);

impl<'a> ChunkSliceMut<'a> {
    pub fn block_at_mut<O: ChunkOffset>(&'_ mut self, coord: O) -> BlockViewMut<'_> {
        let offset = coord.to_offset();
        let slice = <&mut [u8; 64]>::try_from(
            &mut self.0.as_mut_slice()[offset..offset + Chunk::NUM_BYTES_PER_BLOCK],
        )
        .expect(
            "Return value of `ChunkBlockCoord.to_offset()` is guaranteed \
                to be smaller than chunk bytes len - 32",
        );
        BlockViewMut::new(slice)
    }
}

#[derive(Debug, Clone)]
pub struct Chunk(Box<ChunkBytes>); // 5 unknown bytes

impl Chunk {
    pub const NUM_BLOCK_PER_ROW: usize = 32;
    pub const NUM_BLOCK_PER_COL: usize = 32;
    pub const NUM_BYTES_PER_BLOCK: usize = 64;
    pub const NUM_BYTES: usize =
        Self::NUM_BLOCK_PER_ROW * Self::NUM_BLOCK_PER_COL * Self::NUM_BYTES_PER_BLOCK + 5;

    pub fn new_empty() -> Self {
        Self(Box::new([0; Self::NUM_BYTES]))
    }

    pub fn as_slice(&'_ self) -> ChunkSlice<'_> {
        ChunkSlice(&self.0)
    }

    pub fn as_mut_slice(&'_ mut self) -> ChunkSliceMut<'_> {
        ChunkSliceMut(&mut self.0)
    }

    pub fn inner(&self) -> &[u8] {
        self.0.as_slice()
    }

    fn inner_mut(&mut self) -> &mut [u8] {
        self.0.as_mut_slice()
    }
}

#[derive(Debug, Clone)]
pub struct CompressedChunk(Vec<u8>);

impl CompressedChunk {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn decompress_view<'a>(&self, buffer: &'a mut Vec<u8>) -> BhResult<ChunkSlice<'a>> {
        buffer.clear();
        decompress_into(&self.0, buffer)?;
        Ok(ChunkSlice(buffer.as_slice().try_into()?))
    }

    pub fn decompress(&self) -> BhResult<Chunk> {
        let mut chunk = Chunk::new_empty();
        decompress_exact_into(&self.0, chunk.inner_mut())?;
        Ok(chunk)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    // decompress, modify, then re-compress
    pub fn apply<'a, O, F: FnOnce(ChunkSliceMut<'a>) -> O>(&mut self, _: F) -> O {
        todo!()
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
pub struct Chunks(Vec<Option<CompressedChunk>>);

impl Chunks {
    pub const NUM_CHUNK_PER_COL: usize = 32;

    fn to_index(coord: ChunkCoord) -> usize {
        coord.x() as usize * Self::NUM_CHUNK_PER_COL + coord.y() as usize
    }

    pub fn inner(&self) -> &Vec<Option<CompressedChunk>> {
        &self.0
    }

    pub fn from_db(db: &Database<Str, Bytes>, rtxn: &RoTxn, world_width: u32) -> BhResult<Self> {
        let mut chunks = vec![None; world_width as usize * Self::NUM_CHUNK_PER_COL];
        for pair in db.iter(rtxn)? {
            if let Ok((k, v)) = pair
                && let Ok(coord) = ChunkCoord::try_from_str(k)
            {
                chunks[Self::to_index(coord)] = Some(CompressedChunk(v.to_vec()));
            }
        }
        Ok(Self(chunks))
    }

    pub fn to_db<W: Write>(&self, db: &Database<Str, Bytes>, wtxn: &mut RwTxn<W>) -> BhResult<()> {
        for (coord, chunk) in ChunkIndexIter::default().zip(self.0.iter()) {
            if let Some(chunk) = chunk {
                db.put(wtxn, coord.to_string().as_str(), chunk.as_bytes())?;
            }
        }
        Ok(())
    }

    pub fn keys(&self) -> impl Iterator<Item = ChunkCoord> {
        ChunkIndexIter::default()
            .zip(self.0.iter())
            .filter_map(|(coord, chunk)| chunk.is_some().then_some(coord))
    }

    pub fn contains_key(&self, key: ChunkCoord) -> bool {
        self.0.get(Self::to_index(key)).is_some()
    }

    pub fn chunk_at<I: Into<ChunkCoord>>(&self, coord: I) -> Option<&CompressedChunk> {
        self.0
            .get(Self::to_index(coord.into()))
            .and_then(|out| out.as_ref())
    }

    pub fn set_chunk_at<I: Into<ChunkCoord>>(&mut self, coord: I, chunk: CompressedChunk) {
        self.0[Self::to_index(coord.into())] = Some(chunk);
    }

    pub fn block_at<'a, I: Into<BlockCoord>>(
        &mut self,
        coord: I,
        chunk_buffer: &'a mut Vec<u8>,
    ) -> Option<BhResult<BlockView<'a>>> {
        let block_coord = coord.into();
        let (chunk_coord, chunk_block_coord) = block_coord.decompose();
        self.chunk_at(chunk_coord).map(|chunk| {
            chunk
                .decompress_view(chunk_buffer)
                .map(|chunk_slice| chunk_slice.block_at(chunk_block_coord))
        })
    }
}
