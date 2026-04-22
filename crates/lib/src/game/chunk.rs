use super::{
    super::util::gzip::{compress_into, decompress_exact_into, decompress_into},
    block::{BlockView, BlockViewMut},
    coord::{BlockCoord, ChunkCoord, ChunkOffset},
};
use lmdb_rs::{
    codec::types::{Bytes, Str},
    database::Database,
    txn::{RoTxn, RwTxn},
};
use snafu::prelude::*;
use std::io::Write;

type ChunkBytes = [u8; Chunk::NUM_BYTES];

#[derive(Debug, Clone, Copy)]
pub struct ChunkView<'a>(&'a ChunkBytes);

impl<'a> ChunkView<'a> {
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
pub struct ChunkViewMut<'a>(&'a mut ChunkBytes);

impl<'a> ChunkViewMut<'a> {
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

#[derive(Debug, Snafu)]
pub enum ChunkError {
    #[snafu(display("Failed to compress chunk: {source}"))]
    CompressChunk { source: std::io::Error },
    #[snafu(display("Failed to decompress chunk: {source}"))]
    DecompressChunk { source: std::io::Error },
    #[snafu(display("Invalid chunk size, expect {expect}, got {got}: {source}"))]
    InvalidChunkSize {
        expect: usize,
        got: usize,
        source: std::array::TryFromSliceError,
    },
}

type ChunkResult<T> = std::result::Result<T, ChunkError>;

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

    pub fn view(&'_ self) -> ChunkView<'_> {
        ChunkView(&self.0)
    }

    pub fn view_mut(&'_ mut self) -> ChunkViewMut<'_> {
        ChunkViewMut(&mut self.0)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.0.as_mut_slice()
    }

    pub fn compress(&self) -> ChunkResult<CompressedChunk> {
        let mut compressed_bytes = Vec::new();
        compress_into(self.as_bytes(), &mut compressed_bytes).context(CompressChunkSnafu)?;
        Ok(CompressedChunk::new(compressed_bytes))
    }
}

#[derive(Debug, Clone)]
pub struct CompressedChunk(Vec<u8>);

impl CompressedChunk {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn decompress_view<'a>(&self, buffer: &'a mut Vec<u8>) -> ChunkResult<ChunkView<'a>> {
        buffer.clear();
        decompress_into(&self.0, buffer).context(DecompressChunkSnafu)?;
        Ok(ChunkView(buffer.as_slice().try_into().context(
            InvalidChunkSizeSnafu {
                expect: Chunk::NUM_BYTES,
                got: buffer.len(),
            },
        )?))
    }

    pub fn decompress(&self) -> ChunkResult<Chunk> {
        let mut chunk = Chunk::new_empty();
        decompress_exact_into(&self.0, chunk.as_bytes_mut()).context(DecompressChunkSnafu)?;
        Ok(chunk)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    // decompress, modify, then re-compress
    pub fn apply<'a, O, F: FnOnce(ChunkViewMut<'a>) -> O>(&mut self, _: F) -> O {
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

#[derive(Debug, Snafu)]
pub enum ChunksError {
    #[snafu(display("Failed to iterate over database"))]
    IterateDatabase {
        source: lmdb_rs::error::DatabaseError,
    },
    #[snafu(display("Failed to put entry with key {key} in database"))]
    PutEntry {
        key: String,
        source: lmdb_rs::error::DatabaseError,
    },
}

type ChunksResult<T> = std::result::Result<T, ChunksError>;

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

    pub fn from_db(
        db: &Database<Str, Bytes>,
        rtxn: &RoTxn,
        world_width: u32,
    ) -> ChunksResult<Self> {
        let mut chunks = vec![None; world_width as usize * Self::NUM_CHUNK_PER_COL];
        for pair in db.iter(rtxn).context(IterateDatabaseSnafu)? {
            if let Ok((k, v)) = pair
                && let Ok(coord) = ChunkCoord::try_from_str(k)
            {
                chunks[Self::to_index(coord)] = Some(CompressedChunk(v.to_vec()));
            }
        }
        Ok(Self(chunks))
    }

    pub fn to_db<W: Write>(
        &self,
        db: &Database<Str, Bytes>,
        wtxn: &mut RwTxn<W>,
    ) -> ChunksResult<()> {
        for (coord, chunk) in ChunkIndexIter::default().zip(self.0.iter()) {
            if let Some(chunk) = chunk {
                let coord_str = coord.to_string();
                db.put(wtxn, coord_str.as_str(), chunk.as_bytes())
                    .context(PutEntrySnafu { key: coord_str })?;
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
    ) -> Option<ChunkResult<BlockView<'a>>> {
        let block_coord = coord.into();
        let (chunk_coord, chunk_block_coord) = block_coord.decompose();
        self.chunk_at(chunk_coord).map(|chunk| {
            chunk
                .decompress_view(chunk_buffer)
                .map(|chunk_slice| chunk_slice.block_at(chunk_block_coord))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Chunk, ChunkCoord, ChunkIndexIter, Chunks, CompressedChunk};
    use crate::game::block::{Block, BlockMut};
    use crate::game::coord::BlockCoord;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    #[test]
    fn test_chunk_size() {
        assert_eq!(Chunk::NUM_BYTES, 32 * 32 * 64 + 5);
        let chunk = Chunk::new_empty();
        assert_eq!(chunk.as_bytes().len(), Chunk::NUM_BYTES);
    }

    #[test]
    fn test_chunk_index_iter() {
        let mut iter = ChunkIndexIter::default();
        assert_eq!(iter.next(), Some(ChunkCoord::new(0, 0).unwrap()));
        for _ in 0..31 {
            iter.next();
        }
        assert_eq!(iter.next(), Some(ChunkCoord::new(1, 0).unwrap()));
    }

    #[test]
    fn test_chunks_to_index() {
        assert_eq!(Chunks::to_index(ChunkCoord::new(0, 0).unwrap()), 0);
        assert_eq!(Chunks::to_index(ChunkCoord::new(0, 31).unwrap()), 31);
        assert_eq!(Chunks::to_index(ChunkCoord::new(1, 0).unwrap()), 32);
        assert_eq!(
            Chunks::to_index(ChunkCoord::new(10, 5).unwrap()),
            10 * 32 + 5
        );
    }

    #[test]
    fn test_compressed_chunk_decompression() {
        let chunk = Chunk::new_empty();
        let mut raw_bytes = chunk.as_bytes().to_vec();
        raw_bytes[100] = 0xAA;
        raw_bytes[chunk.as_bytes().len() - 1] = 0xBB;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&raw_bytes).unwrap();
        let compressed_bytes = encoder.finish().unwrap();

        let compressed = CompressedChunk::new(compressed_bytes);
        let decompressed = compressed.decompress().unwrap();
        assert_eq!(decompressed.as_bytes(), raw_bytes.as_slice());

        let mut buffer = Vec::new();
        let _ = compressed.decompress_view(&mut buffer).unwrap();
        assert_eq!(buffer, raw_bytes);
    }

    #[test]
    fn test_chunks_block_at() {
        let mut chunks = Chunks(vec![None; 32 * 32]);
        let coord = ChunkCoord::new(1, 1).unwrap();

        let mut chunk = Chunk::new_empty();
        let block_coord = BlockCoord::new(32, 32).unwrap(); // (1, 1) in chunks, (0, 0) in block
        let (_, local_coord) = block_coord.decompose();

        {
            let mut slice = chunk.view_mut();
            let mut block = slice.block_at_mut(local_coord);
            block.set_fg(0xF0);
        }

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(chunk.as_bytes()).unwrap();
        let compressed_bytes = encoder.finish().unwrap();

        chunks.set_chunk_at(coord, CompressedChunk::new(compressed_bytes));

        let mut buffer = Vec::new();
        let block = chunks.block_at(block_coord, &mut buffer).unwrap().unwrap();
        assert_eq!(block.fg_raw(), 0xF0);
    }
}
