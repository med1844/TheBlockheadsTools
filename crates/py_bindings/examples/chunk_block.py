from the_blockheads_tools_py import WorldDb
from util import Snippet


def chunk_block(world_db: WorldDb):
    with Snippet("read_chunk_output"):
        # --8<-- [start:read_chunk]
        from the_blockheads_tools_py import Chunk, Block, BlockCoord

        # read your world
        # world_db = ...

        chunks = world_db.chunks
        print("num chunks:", len(chunks.keys()))

        # Get spawn point coords
        world_v2 = world_db.main.world_v2
        spawn_x = world_v2.start_portal_pos_x
        spawn_y = world_v2.start_portal_pos_y

        # Break down the block coord to 1) chunk coord and 2) block coord in that chunk
        block_coord = BlockCoord(spawn_x, spawn_y)
        chunk_coord, chunk_block_coord = block_coord.decompose()
        print("coords:", block_coord, chunk_coord, chunk_block_coord)

        # Get chunk
        spawn_chunk = chunks.chunk_at(chunk_coord)
        assert spawn_chunk is not None

        # Get block in chunk
        spawn_block = spawn_chunk.block_at(chunk_block_coord)
        print("Foreground block type:", spawn_block.fg())
        print("Content block type:", spawn_block.content())
        # --8<-- [end:read_chunk]

    # --8<-- [start:write_chunk_basic]
    from the_blockheads_tools_py import BlockType

    # Change spawn portal to time crystal
    spawn_block.set_fg(BlockType.TimeCrystal)

    # Update chunks to apply change
    chunks.set_chunk_at(chunk_coord, spawn_chunk)
    # --8<-- [end:write_chunk_basic]

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
    # --8<-- [end:chunk_ownership]

    # --8<-- [start:write_chunk_util]
    from the_blockheads_tools_py import BlockCoord, ChunkCoord
    from typing import Optional

    class ClassModifier:
        def __init__(self, world_db: WorldDb):
            self.world_db = world_db
            self.modified_chunks: dict[ChunkCoord, Chunk] = {}

        def __enter__(self):
            pass

        def chunk_at(self, coord: ChunkCoord) -> Optional[Chunk]:
            if coord not in self.modified_chunks.keys():
                maybe_chunk = self.world_db.chunks.chunk_at(coord)
                if maybe_chunk is not None:
                    self.modified_chunks[coord] = maybe_chunk
            if coord in self.modified_chunks:
                return self.modified_chunks[coord]
            return None

        def block_at(self, coord: BlockCoord) -> Optional[Block]:
            chunk_coord, chunk_block_coord = coord.decompose()
            chunk = self.chunk_at(chunk_coord)
            if chunk is not None:
                return chunk.block_at(chunk_block_coord)
            return None

        def __exit__(self, exc_type, exc_value, traceback):
            for coord, chunk in self.modified_chunks.items():
                self.world_db.chunks.set_chunk_at(coord, chunk)

    # --8<-- [end:write_chunk_util]

    spawn_chunk_coord = chunk_coord

    # --8<-- [start:write_chunk_visibility]
    from the_blockheads_tools_py import ChunkBlockCoord

    chunk_coord = ChunkCoord(spawn_chunk_coord.x - 1, spawn_chunk_coord.y + 1)
    chunk = chunks.chunk_at(chunk_coord)
    assert chunk is not None
    for x in range(Chunk.WIDTH // 2):
        for y in range(Chunk.HEIGHT // 2):
            block = chunk.block_at(ChunkBlockCoord(x, y))
            block.set_visibility(x * 16 + y)
            block = chunk.block_at(ChunkBlockCoord(x + 16, y + 16))
            block.set_visibility(255)
    chunks.set_chunk_at(chunk_coord, chunk)
    # --8<-- [end:write_chunk_visibility]

    # --8<-- [start:write_chunk_brightness]
    chunk_coord = ChunkCoord(spawn_chunk_coord.x, spawn_chunk_coord.y)
    chunk = chunks.chunk_at(chunk_coord)
    assert chunk is not None
    for x in range(Chunk.WIDTH // 2):
        for y in range(Chunk.HEIGHT // 2):
            block = chunk.block_at(ChunkBlockCoord(x + 16, y))
            block.set_visibility(255)
            block.set_brightness(x * 16 + y)
    chunks.set_chunk_at(chunk_coord, chunk)
    # --8<-- [end:write_chunk_brightness]

    # --8<-- [start:write_chunk_fg_bg_content]
    from the_blockheads_tools_py import BlockContentType
    from random import choice, random

    chunk_coord = ChunkCoord(spawn_chunk_coord.x + 1, spawn_chunk_coord.y + 2)
    chunk = Chunk()
    for x in range(Chunk.WIDTH // 2):
        for y in range(Chunk.HEIGHT // 2):
            block = chunk.block_at(ChunkBlockCoord(x, y))
            block.set_visibility(255)
            block.set_brightness(255)
            r = random()
            block.set_bg(BlockType.Air)
            block.set_fg(BlockType.Air)
            if r > 0.2:
                block.set_bg(BlockType.Stone)
            if r > 0.4:
                block.set_fg(BlockType.Stone)
            if r > 0.6:
                block.set_content(
                    choice(
                        [
                            BlockContentType.Coal,
                            BlockContentType.CopperOre,
                            BlockContentType.TinOre,
                            BlockContentType.IronOre,
                            BlockContentType.GoldNuggets,
                            BlockContentType.PlatinumOre,
                            BlockContentType.TitaniumOre,
                        ]
                    )
                )
    chunks.set_chunk_at(chunk_coord, chunk)
    # --8<-- [end:write_chunk_fg_bg_content]

    # --8<-- [start:write_chunk_water_snow_height]
    def get_checkerboard_value(x: int, y: int) -> tuple[bool, int]:
        # 1. Define the boundary of the 30x30 interior
        is_interior = 1 <= x <= 30 and 1 <= y <= 30

        # 2. Check checkerboard parity:
        is_white = is_interior and (x + y) % 2 == 0

        if not is_white:
            return False, -1

        # 3. Calculate the index purely based on coordinates
        rows_above = y - 1
        cells_from_above = rows_above * 15
        cells_in_current_row = (x + (1 if y % 2 != 0 else 0)) // 2

        return True, cells_from_above + cells_in_current_row

    num_white = 15 * 30
    chunk_coord = ChunkCoord(spawn_chunk_coord.x - 1, spawn_chunk_coord.y + 2)
    chunk = Chunk()
    for x in range(Chunk.WIDTH):
        for y in range(Chunk.HEIGHT):
            block = chunk.block_at(ChunkBlockCoord(x, y))
            block.set_visibility(255)
            block.set_brightness(255)
            match get_checkerboard_value(x, y):
                case (True, val):
                    block.set_height(int(val / num_white * 255))
                    block.set_fg(BlockType.Water)
                    block.set_bg(BlockType.Water)
                case (False, _):
                    block.set_fg(BlockType.SteelBlock)
                    block.set_bg(BlockType.SteelBlock)
    chunks.set_chunk_at(chunk_coord, chunk)

    # --8<-- [end:write_chunk_water_snow_height]
