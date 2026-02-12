from the_blockheads_tools_py import (
    BlockCoord,
    BlockType,
    ChunkBlockCoord,
    WorldDb,
    ChunkCoord,
    Chunk,
    Chunks,
    Block,
)
from constants import WORLD_DB_PATH


def test_chunks():
    world_db = WorldDb.open_path(WORLD_DB_PATH)
    chunks = world_db.chunks
    assert isinstance(chunks, Chunks)

    keys = chunks.keys()
    assert ChunkCoord(335, 16) in keys
    assert ChunkCoord(0, 0) not in keys

    spawn_chunk = chunks.chunk_at(ChunkCoord(335, 16))
    assert isinstance(spawn_chunk, Chunk)

    block = spawn_chunk.block_at(ChunkBlockCoord(20, 8))
    assert isinstance(block, Block)
    assert block.fg() == BlockType.Air

    spawn_block = chunks.block_at(BlockCoord(10740, 519))
    assert spawn_block is not None
    assert spawn_block.fg() == BlockType.SpawnPortalBase
