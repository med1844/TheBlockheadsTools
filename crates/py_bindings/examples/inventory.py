from the_blockheads_tools_py import WorldDb
from util import Snippet


def inventory(world_db: WorldDb):
    with Snippet("read_blockhead_output"):
        # --8<-- [start:read_blockhead]
        # world_db = ...

        blockheads = world_db.main.blockheads
        assert len(blockheads) > 0

        blockhead = blockheads[0]
        print("blockhead name:", blockhead.name)

        inventory = blockhead.inventory
        assert inventory is not None
        print("blockhead inventory:", inventory)
        # --8<-- [end:read_blockhead]

    # --8<-- [start:edit_inventory_basic]
    from the_blockheads_tools_py import Slot, Item, ItemType

    inventory[4] = Slot([Item(ItemType.Diamond)] * 1234)

    # Remember to write back!
    blockhead.inventory = inventory
    # --8<-- [end:edit_inventory_basic]

    # --8<-- [start:edit_inventory_container]
    from the_blockheads_tools_py import BasketSlots, StandardChest

    # Add a basket filled with deprecated blocks
    inventory[5] = Slot(
        [
            Item(
                ItemType.Basket,
                BasketSlots(
                    [
                        Slot([Item(ItemType.DeprecatedDirtBlock)] * 111),
                        Slot([Item(ItemType.DeprecatedWoodBlock)] * 222),
                        Slot([Item(ItemType.DeprecatedWorkbench)] * 333),
                        Slot([Item(ItemType.DeprecatedStoneWorkbench)] * 444),
                    ]
                ),
            )
        ]
    )

    # Add a chest with checkerboard pattern of "Double Time"
    inventory[6] = Slot(
        [
            Item(
                ItemType.Chest,
                StandardChest(
                    [
                        Slot([Item(ItemType.DoubleTime)])
                        if (i ^ j) & 1 == 0
                        else Slot()
                        for i in range(4)
                        for j in range(4)
                    ]
                ),
            )
        ]
    )

    # Write back
    blockhead.inventory = inventory
    # --8<-- [end:edit_inventory_container]

    # --8<-- [start:edit_inventory_damage_dye]
    from the_blockheads_tools_py import PigmentColor

    chest_extra = inventory[6][0].extra
    assert isinstance(chest_extra, StandardChest)

    # Add a broken titanium pickaxe
    damaged_titanium_pickaxe = Item(ItemType.TitaniumPickaxe)
    damaged_titanium_pickaxe.damage = 65535
    chest_extra[1] = Slot([damaged_titanium_pickaxe])

    # Add a dyed golden bed
    blue_golden_bed = Item(ItemType.GoldenBed)
    blue_golden_bed.colors = [PigmentColor.CopperBlue]
    chest_extra[3] = Slot([blue_golden_bed])

    # and a dyed paint with 3 colors
    paint = Item(ItemType.Paint)
    paint.colors = [
        PigmentColor.IndianYellow,
        PigmentColor.TyrianPurple,
        PigmentColor.MarbleWhite,
    ]
    chest_extra[6] = Slot([paint])

    # Write back
    blockhead.inventory = inventory
    # --8<-- [end:edit_inventory_damage_dye]
