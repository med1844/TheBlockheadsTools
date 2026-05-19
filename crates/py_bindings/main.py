from the_blockheads_tools_py import (
    WorldDb,
    Slot,
    Slots,
    Item,
    ItemType,
    StandardChest,
    Arch,
    BlockCoord,
    ChunkCoord,
    ChunkBlockCoord,
    Chunk,
    BlockType,
    BlockContentType,
)
import sys
import pathlib
import shutil

assert len(sys.argv) == 3
db = WorldDb.open_path(sys.argv[1])
bh = db.main.blockheads[0]
inv = bh.inventory
assert inv is not None

slots = [
    Slot([Item(ItemType.Jetpack)]),
    Slot([Item(ItemType.Fuel)] * 99),
    Slot([Item(ItemType.Pizza)] * 99),
    Slot([Item(ItemType.Basket, sub_items=Slots([Slot([Item(ItemType.GoldenBed)]), Slot([Item(ItemType.NorthPoleHatOfWarmth)]), Slot(), Slot()]))]),
    Slot(),
    Slot(),
    Slot(),
]

for i, slot in enumerate(slots):
    inv[i + 1] = slot

bh.inventory = inv

spawn_x = db.main.world_v2.start_portal_pos_x
spawn_y = db.main.world_v2.start_portal_pos_y
chunk_coord, _ = BlockCoord(spawn_x, spawn_y).decompose()
chunk_coord_above = ChunkCoord(chunk_coord.x, chunk_coord.y + 1)
chunk = db.chunks.chunk_at(chunk_coord_above)
assert chunk is not None
for y in range(Chunk.HEIGHT):
    for x in range(Chunk.WIDTH):
        block = chunk.block_at(ChunkBlockCoord(x, y))
        if (x == 0 or x == Chunk.WIDTH - 1 or y == 0 or y == Chunk.HEIGHT - 1) and not (
            x == 15 and y == 0
        ):
            block.set_fg(BlockType.SteelBlock)
        else:
            block.set_fg(BlockType.Air)
        block.set_bg(BlockType.SteelBlock)
        block.set_visibility(255)
        block.set_content(BlockContentType.Nothing)
db.chunks.set_chunk_at(chunk_coord_above, chunk)

path = pathlib.Path(sys.argv[2])
if path.exists():
    print(f"deleting {path}")
    shutil.rmtree(path)
path.mkdir(parents=True, exist_ok=True)
db.save_path(sys.argv[2], Arch.Arch32)
