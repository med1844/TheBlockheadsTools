from the_blockheads_tools_py import WorldDb
from constants import WORLD_DB_PATH, Snippet


def chunk_block(world_db: WorldDb):
    with Snippet("read_chunk_output"):
        # --8<-- [start:read_chunk]
        from the_blockheads_tools_py import Chunk, Block, BlockCoord

        # read your world
        # world_db = ...

        chunks = world_db.chunks
        print("num chunks:", len(chunks.keys()))

        world_v2 = world_db.main.world_v2
        spawn_x = world_v2.start_portal_pos_x
        spawn_y = world_v2.start_portal_pos_y

        block_coord = BlockCoord(spawn_x, spawn_y)
        chunk_coord, chunk_block_coord = block_coord.decompose()

        # Get chunk
        spawn_chunk = chunks.chunk_at(chunk_coord)
        assert spawn_chunk is not None

        # Get block in chunk
        spawn_block = spawn_chunk.block_at(chunk_block_coord)
        print("Foreground block type:", spawn_block.fg())
        print("Content block type:", spawn_block.content())
        # --8<-- [end:read_chunk]


    # --8<-- [start:write_chunk]
    from the_blockheads_tools_py import BlockType

    # Change spawn portal to time crystal
    spawn_block.set_fg(BlockType.TimeCrystal)

    # Update chunks to apply change
    chunks.set_chunk_at(chunk_coord, spawn_chunk)
    # --8<-- [end:write_chunk]

    # --8<-- [start:chunk_ownership]
    spawn_chunk = chunks.chunk_at(chunk_coord)
    assert spawn_chunk is not None

    copied_spawn_chunk = chunks.chunk_at(chunk_coord)
    assert copied_spawn_chunk is not None
    assert copied_spawn_chunk is not spawn_chunk  # Different instance!

    updated_spawn_chunk = chunks.chunk_at(chunk_coord)
    assert updated_spawn_chunk is not None

    # Each chunk instance holds their own data,
    # thus they are all different instances.
    assert updated_spawn_chunk is not spawn_chunk
    assert updated_spawn_chunk is not copied_spawn_chunk

    updated_spawn_block = updated_spawn_chunk.block_at(chunk_block_coord)
    assert updated_spawn_block.fg() == BlockType.TimeCrystal
    # --8<-- [end:chunk_ownership]

