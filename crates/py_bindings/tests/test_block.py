from the_blockheads_tools_py import BlockType, Chunk, Block, ChunkBlockCoord, WorldDb, ChunkCoord, BlockContentType
from constants import WORLD_DB_PATH


def test_block_type():
    assert int(BlockType.Air) == 2
    assert BlockType(16) == BlockType.TimeCrystal


def test_block():
    world_db = WorldDb.open_path(WORLD_DB_PATH)
    spawn_chunk = world_db.chunks.chunk_at(ChunkCoord(335, 16))
    assert spawn_chunk is not None

    spawn_block = spawn_chunk.block_at(ChunkBlockCoord(20, 7))
    assert spawn_block.fg() == BlockType.SpawnPortalBase
    spawn_block.set_fg(BlockType.TimeCrystal)
    assert spawn_block.bg() == BlockType.Stone
    spawn_block.set_bg(BlockType.Air)
    assert spawn_block.content() == BlockContentType.Nothing
    spawn_block.set_content(BlockContentType.Flint)
    _ = spawn_block.height()
    spawn_block.set_height(123)
    _ = spawn_block.damage()
    spawn_block.set_damage(255)
    _ = spawn_block.visibility()
    spawn_block.set_visibility(5)
    _ = spawn_block.brightness()
    spawn_block.set_brightness(0)

    assert spawn_block.fg() == BlockType.TimeCrystal
    assert spawn_block.bg() == BlockType.Air
    assert spawn_block.content() == BlockContentType.Flint
    assert spawn_block.height() == 123
    assert spawn_block.damage() == 255
    assert spawn_block.visibility() == 5
    assert spawn_block.brightness() == 0

