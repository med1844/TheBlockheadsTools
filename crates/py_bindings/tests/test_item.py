from the_blockheads_tools_py import (
    Item, ItemType, PigmentColor, StackedItem, BasketExtra, ChestExtra, ChestType
)
import pytest

def test_item_basic():
    item = Item(ItemType.Flint)
    assert item.item_type == ItemType.Flint
    assert item.damage == 0

    item.item_type = ItemType.Stone
    assert item.item_type == ItemType.Stone
    assert item.type_id == int(ItemType.Stone)

    item.damage = 100
    assert item.damage == 100
    assert item.data_a == 100

def test_item_fields():
    item = Item(ItemType.Flint)
    assert item.type_id == int(ItemType.Flint)
    assert item.data_a == 0
    assert item.data_b == 0
    assert item.selected_sub_item_index == 0
    assert item.padding == 0

    item.type_id = int(ItemType.Stone)
    assert item.item_type == ItemType.Stone

    item.data_a = 500
    assert item.damage == 500

    # RedOchre = 3, EmeraldGreen = 6 (4 bits per color)
    item.data_b = 3 | (6 << 4)
    assert item.colors[0] == PigmentColor.RedOchre
    assert item.colors[1] == PigmentColor.EmeraldGreen
    assert item.colors[2] == PigmentColor.Transparent

def test_item_colors():
    item = Item(ItemType.Paint)
    assert item.colors == [PigmentColor.Transparent] * 3
    assert item.data_b == 0

    colors = [PigmentColor.RedOchre, PigmentColor.EmeraldGreen, PigmentColor.Transparent]
    item.colors = colors
    assert item.colors == colors
    # 3 | (6 << 4) | (0 << 8) = 3 + 96 = 99
    assert item.data_b == 99

def test_item_edge_cases():
    item = Item(ItemType.Flint)

    item.type_id = 9999
    with pytest.raises(ValueError, match="Invalid item type id"):
        _ = item.item_type

    with pytest.raises(ValueError, match="colors must have exactly 3 elements"):
        item.colors = [PigmentColor.RedOchre]

    item.data_a = 0xFFFF
    assert item.damage == 65535

    item.data_b = 0x7FFF
    with pytest.raises(ValueError, match="Invalid color ID: 15"):
        _ = item.colors

def test_stacked_item():
    items = [Item(ItemType.Apple), Item(ItemType.Apple)]
    stacked = StackedItem(items)
    assert len(stacked) == 2

    # Reference identity check
    item1 = stacked[0]
    item2 = stacked[0]
    assert item1 is item2

    # In-place mutation check
    stacked[0].item_type = ItemType.Mango
    assert stacked[0].item_type == ItemType.Mango
    assert item1.item_type == ItemType.Mango

    # Test __setitem__
    new_item = Item(ItemType.Flint)
    stacked[1] = new_item
    assert stacked[1] is new_item

    # Test public field
    stacked.items = [Item(ItemType.Flint)]
    assert len(stacked) == 1
    assert stacked[0].item_type == ItemType.Flint

    with pytest.raises(IndexError):
        _ = stacked[1]
    with pytest.raises(IndexError):
        stacked[1] = Item(ItemType.Apple)

    # Test negative indexing
    stacked.items = [Item(ItemType.Apple), Item(ItemType.Mango)]
    assert stacked[-1].item_type == ItemType.Mango
    last_item = stacked[-1]
    assert last_item is stacked[1]

def test_basket_extra():
    # Test creation
    basket = BasketExtra()
    assert len(basket) == 4
    assert isinstance(basket[0], StackedItem)
    assert len(basket[0]) == 0

    # Test identity and nested mutation
    item = Item(ItemType.Apple)
    stacked = StackedItem([item])
    basket[0] = stacked

    assert basket[0] is stacked
    assert basket[0][0] is item

    # Modify nested item
    item.item_type = ItemType.Mango
    assert basket[0][0].item_type == ItemType.Mango

    # Test Item with Extra
    container_item = Item(ItemType.Basket, extra=basket)
    assert container_item.extra is basket

    # Match dispatch test
    match container_item.extra:
        case BasketExtra(items=items):
            assert len(items) == 4
        case _:
            pytest.fail("Should have matched BasketExtra")

def test_item_repr():
    item = Item(ItemType.Flint)
    r = repr(item)
    assert "type_id=3" in r
    assert "extra=None" in r

    basket = BasketExtra()
    item_with_basket = Item(ItemType.Basket, extra=basket)
    assert "BasketExtra" in repr(item_with_basket)

def test_chest_extra_basic():
    # Test initialization and defaults
    chest = ChestExtra(ChestType.Safe, owner_id="player1")
    assert chest.chest_type == ChestType.Safe
    assert chest.owner_id == "player1"
    assert len(chest) == 16
    assert chest.flipped == False
    assert chest.pos_x == 0

    # Test property mutation
    chest.flipped = True
    chest.pos_x = 1234
    chest.float_pos = [1.5, 2.5]
    chest.owner_id = "new_owner"

    assert chest.flipped == True
    assert chest.pos_x == 1234
    assert chest.float_pos == [1.5, 2.5]
    assert chest.owner_id == "new_owner"

def test_chest_identity_and_mutation():
    chest = ChestExtra()
    item = Item(ItemType.Apple)
    stacked = StackedItem([item])

    # Set slot 0
    chest[0] = stacked
    assert chest[0] is stacked
    assert chest[0][0] is item

    # Nested mutation via reference
    item.damage = 10
    assert chest[0][0].damage == 10

    # Test __setitem__ replaced identity
    new_stacked = StackedItem([Item(ItemType.Mango)])
    chest[0] = new_stacked
    assert chest[0] is new_stacked
    assert chest[0] is not stacked

def test_chest_dispatch():
    chest = ChestExtra(ChestType.Gold)
    item = Item(ItemType.Chest, extra=chest)

    assert isinstance(item.extra, ChestExtra)
    assert item.extra is chest
    assert item.extra.chest_type == ChestType.Gold

    # Match dispatch
    match item.extra:
        case ChestExtra(chest_type=ctype, owner_id=owner):
            assert ctype == ChestType.Gold
            assert owner == "server"
        case _:
            pytest.fail("Should have matched ChestExtra")

def test_chest_validation():
    chest = ChestExtra()
    assert len(chest.items) == 16

    new_items = [StackedItem() for _ in range(16)]
    chest.items = new_items
    assert len(chest.items) == 16

    chest.items = [StackedItem()] * 5
    assert len(chest.items) == 5

def test_chest_roundtrip():
    # Creating a complex chest setup
    chest = ChestExtra(ChestType.Portal, owner_id="portal_master")
    chest.flipped = True
    chest.paint_color = 123
    chest.pos_x = 987654
    chest.pos_y = 512
    chest.float_pos = [10.5, 20.5]
    chest.unique_id = 0xDEADBEEFCAFEBABE

    item_in_chest = Item(ItemType.Diamond)
    item_in_chest.damage = 5
    chest[7] = StackedItem([item_in_chest])

    # Wrap in Item
    container = Item(ItemType.PortalChest, extra=chest)

    # We can test that the properties we set are stable in python.
    assert container.extra.unique_id == 0xDEADBEEFCAFEBABE
    assert container.extra[7][0].item_type == ItemType.Diamond
    assert container.extra[7][0].damage == 5

def test_workbench_basic():
    from the_blockheads_tools_py import WorkbenchExtra, WorkbenchType

    wb = WorkbenchExtra()
    assert wb.workbench_type == WorkbenchType.Workbench
    assert wb.level == 1
    assert wb.owner_id == "server"

    wb = WorkbenchExtra(WorkbenchType.Craft, 2, "crafter")
    assert wb.workbench_type == WorkbenchType.Craft
    assert wb.level == 2
    assert wb.owner_id == "crafter"

def test_workbench_properties():
    from the_blockheads_tools_py import WorkbenchExtra

    wb = WorkbenchExtra()

    # Test mutability
    wb.level = 5
    assert wb.level == 5

    wb.available_electricity = 100
    assert wb.available_electricity == 100

    wb.craft_progress_count = 0.5
    assert wb.craft_progress_count == 0.5

    wb.float_pos = [10.0, 20.0]
    assert wb.float_pos == [10.0, 20.0]

    wb.unique_id = 999999
    assert wb.unique_id == 999999

def test_workbench_integration():
    from the_blockheads_tools_py import WorkbenchExtra, WorkbenchType

    wb = WorkbenchExtra(WorkbenchType.Easel)
    item = Item(ItemType.Easel, extra=wb)

    assert isinstance(item.extra, WorkbenchExtra)
    assert item.extra.workbench_type == WorkbenchType.Easel

    match item.extra:
        case WorkbenchExtra(workbench_type=wt):
            assert wt == WorkbenchType.Easel
        case _:
            pytest.fail("Should have matched WorkbenchExtra")
