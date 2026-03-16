from the_blockheads_tools_py import (
    WorldDb,
    Slot,
    BasketSlots,
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
    Slot(
        [
            Item(
                ItemType.Basket,
                BasketSlots(
                    [
                        Slot(
                            [
                                Item(
                                    ItemType.Chest,
                                    StandardChest(
                                        [
                                            Slot([Item(ItemType.Campfire)] * 99),
                                            Slot([Item(ItemType.ToolBench)]),
                                            Slot([Item(ItemType.CraftBench)]),
                                            Slot([Item(ItemType.WoodworkBench)]),
                                            Slot([Item(ItemType.TaylorsBench)]),
                                            Slot([Item(ItemType.Kiln)]),
                                            Slot([Item(ItemType.Furnace)]),
                                            Slot([Item(ItemType.Press)]),
                                            Slot([Item(ItemType.CompostBin)]),
                                            Slot([Item(ItemType.Easel)]),
                                            Slot([Item(ItemType.BuildersBench)]),
                                            Slot([Item(ItemType.Portal)]),
                                            Slot([Item(ItemType.Shop)]),
                                            Slot([Item(ItemType.ArmorBench)]),
                                            Slot([Item(ItemType.PizzaOven)]),
                                            Slot([Item(ItemType.MetalworkBench)]),
                                        ]
                                    ),
                                )
                            ]
                        ),
                        Slot(
                            [
                                Item(
                                    ItemType.Chest,
                                    StandardChest(
                                        [
                                            Slot([Item(ItemType.TrainYard)]),
                                            Slot([Item(ItemType.MixingBench)]),
                                            Slot([Item(ItemType.DyeBench)]),
                                            Slot([Item(ItemType.SteamGenerator)]),
                                            Slot([Item(ItemType.Flywheel)]),
                                            Slot([Item(ItemType.ElectricStove)]),
                                            Slot([Item(ItemType.ElectricKiln)]),
                                            Slot([Item(ItemType.ElectricFurnace)]),
                                            Slot([Item(ItemType.ElectricPress)]),
                                            Slot(
                                                [Item(ItemType.ElectricMetalworkBench)]
                                            ),
                                            Slot([Item(ItemType.ElectricSluice)]),
                                            Slot([Item(ItemType.EggExtractor)]),
                                            Slot([Item(ItemType.SolarPanel)] * 99),
                                            Slot([Item(ItemType.Refinery)]),
                                            Slot(),
                                            Slot(),
                                        ]
                                    ),
                                )
                            ]
                        ),
                        Slot(
                            [
                                Item(
                                    ItemType.Chest,
                                    StandardChest(
                                        [
                                            Slot([Item(ItemType.Chest)]),
                                            Slot([Item(ItemType.Safe)]),
                                            Slot([Item(ItemType.Shelf)]),
                                            Slot([Item(ItemType.GoldenChest)]),
                                            Slot([Item(ItemType.PortalChest)]),
                                            Slot([Item(ItemType.DisplayCabinet)]),
                                            Slot([Item(ItemType.FeederChest)]),
                                            Slot(),
                                            Slot(),
                                            Slot(),
                                            Slot(),
                                            Slot(),
                                            Slot(),
                                            Slot(),
                                            Slot(),
                                            Slot(),
                                        ]
                                    ),
                                )
                            ]
                        ),
                        Slot([Item(ItemType.Pizza)] * 99),
                    ]
                ),
            )
        ]
    ),
    Slot(
        [
            Item(
                ItemType.Basket,
                BasketSlots(
                    [
                        Slot([Item(ItemType.TitaniumPickaxe)]),
                        Slot([Item(ItemType.IronAxe)]),
                        Slot([Item(ItemType.GoldenBed)]),
                        Slot([Item(ItemType.SteelBlock)] * 1000),
                    ]
                ),
            )
        ]
    ),
    Slot(
        [
            Item(
                ItemType.Basket,
                BasketSlots(
                    [
                        Slot([Item(ItemType.SteelLantern)] * 99),
                        Slot([Item(ItemType.IronTrapdoor)] * 99),
                        Slot([Item(ItemType.IronDoor)] * 99),
                        Slot([Item(ItemType.Jetpack)]),
                    ]
                ),
            )
        ]
    ),
    Slot(
        [
            Item(
                ItemType.Basket,
                BasketSlots(
                    [
                        Slot([Item(ItemType.Fuel)] * 99),
                        Slot([Item(ItemType.Window)] * 2),
                        Slot([Item(ItemType.BlackWindow)] * 2),
                        Slot([Item(ItemType.Boat)]),
                    ]
                ),
            )
        ]
    ),
    Slot(
        [
            Item(
                ItemType.Basket,
                BasketSlots(
                    [
                        Slot([Item(ItemType.CopperWire)] * 99),
                        Slot([Item(ItemType.Rail)] * 99),
                        Slot([Item(ItemType.RailHandcar)]),
                        Slot([Item(ItemType.TrainStation)]),
                    ]
                ),
            )
        ]
    ),
    Slot(
        [
            Item(
                ItemType.Basket,
                BasketSlots(
                    [
                        Slot([Item(ItemType.SteamLocomotive)]),
                        Slot([Item(ItemType.FreightCar)]),
                        Slot([Item(ItemType.PassengerCar)]),
                        Slot([Item(ItemType.Sign)]),
                    ]
                ),
            )
        ]
    ),
    Slot(
        [
            Item(
                ItemType.Basket,
                BasketSlots(
                    [
                        Slot(
                            [
                                Item(
                                    ItemType.Chest,
                                    StandardChest(
                                        [
                                            Slot([Item(ItemType.TradePortal)]),
                                            Slot([Item(ItemType.BrickColumn)] * 10),
                                            Slot([Item(ItemType.SteelStairs)] * 99),
                                            Slot(
                                                [Item(ItemType.ElectricElevatorMotor)]
                                            ),
                                            Slot([Item(ItemType.ElevatorShaft)] * 1000),
                                            Slot([Item(ItemType.BucketOfWater)] * 99),
                                            Slot(),
                                            Slot(),
                                            Slot(),
                                            Slot(),
                                            Slot(),
                                            Slot(),
                                            Slot(),
                                            Slot(),
                                            Slot(),
                                            Slot(),
                                        ]
                                    ),
                                )
                            ]
                        ),
                        Slot(),
                        Slot(),
                        Slot(),
                    ]
                ),
            )
        ]
    ),
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
