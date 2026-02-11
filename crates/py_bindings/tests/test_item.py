from the_blockheads_tools_py import (
    Item, ItemType, PigmentColor, Slot, BasketExtra, ChestExtra, ChestType,
    WorkbenchExtra, WorkbenchType
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

def test_slot():
    items = [Item(ItemType.Apple), Item(ItemType.Apple)]
    slot = Slot(items)
    assert len(slot) == 2

    # Reference identity check
    item1 = slot[0]
    item2 = slot[0]
    assert item1 is item2

    # In-place mutation check
    slot[0].item_type = ItemType.Mango
    assert slot[0].item_type == ItemType.Mango
    assert item1.item_type == ItemType.Mango

    # Test __setitem__
    new_item = Item(ItemType.Flint)
    slot[1] = new_item
    assert slot[1] is new_item

    # Test public field
    slot.items = [Item(ItemType.Flint)]
    assert len(slot) == 1
    assert slot[0].item_type == ItemType.Flint

    with pytest.raises(IndexError):
        _ = slot[1]
    with pytest.raises(IndexError):
        slot[1] = Item(ItemType.Apple)

    # Test negative indexing
    slot.items = [Item(ItemType.Apple), Item(ItemType.Mango)]
    assert slot[-1].item_type == ItemType.Mango
    last_item = slot[-1]
    assert last_item is slot[1]

def test_basket_extra():
    # Test creation
    basket = BasketExtra()
    assert len(basket) == 4
    assert isinstance(basket[0], Slot)
    assert len(basket[0]) == 0

    # Test identity and nested mutation
    item = Item(ItemType.Apple)
    slot = Slot([item])
    basket[0] = slot

    assert basket[0] is slot
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
    slot = Slot([item])

    # Set slot 0
    chest[0] = slot
    assert chest[0] is slot
    assert chest[0][0] is item

    # Nested mutation via reference
    item.damage = 10
    assert chest[0][0].damage == 10

    # Test __setitem__ replaced identity
    new_slot = Slot([Item(ItemType.Mango)])
    chest[0] = new_slot
    assert chest[0] is new_slot
    assert chest[0] is not slot

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

    new_items = [Slot() for _ in range(16)]
    chest.items = new_items
    assert len(chest.items) == 16

    chest.items = [Slot()] * 5
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
    chest[7] = Slot([item_in_chest])

    # Wrap in Item
    container = Item(ItemType.PortalChest, extra=chest)

    # We can test that the properties we set are stable in python.
    assert type(container.extra) is ChestExtra
    assert container.extra.unique_id == 0xDEADBEEFCAFEBABE
    assert container.extra[7][0].item_type == ItemType.Diamond
    assert container.extra[7][0].damage == 5

def test_workbench_basic():
    wb = WorkbenchExtra()
    assert wb.workbench_type == WorkbenchType.Workbench
    assert wb.level == 1
    assert wb.owner_id == "server"

    wb = WorkbenchExtra(WorkbenchType.Craft, 2, "crafter")
    assert wb.workbench_type == WorkbenchType.Craft
    assert wb.level == 2
    assert wb.owner_id == "crafter"

def test_workbench_properties():
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
    wb = WorkbenchExtra(WorkbenchType.Easel)
    item = Item(ItemType.Easel, extra=wb)

    assert isinstance(item.extra, WorkbenchExtra)
    assert item.extra.workbench_type == WorkbenchType.Easel

    match item.extra:
        case WorkbenchExtra(workbench_type=wt):
            assert wt == WorkbenchType.Easel
        case _:
            pytest.fail("Should have matched WorkbenchExtra")

def test_slot_sequence_protocol():
    slot = Slot()
    assert len(slot) == 0
    item = Item(ItemType.Apple)
    
    slot.append(item)
    assert len(slot) == 1
    assert slot[0].item_type == ItemType.Apple
    assert slot[0] is item

    newItem = Item(ItemType.Mango)
    slot[0] = newItem
    assert slot[0] is newItem
    assert slot[0].item_type == ItemType.Mango
    
    del slot[0]
    assert len(slot) == 0

    with pytest.raises(TypeError):
        _ = slot[0:1]
    with pytest.raises(TypeError):
        slot[0:1] = [item]
    with pytest.raises(TypeError):
        del slot[0:1]

def test_slot_mutators():
    slot = Slot()
    i1 = Item(ItemType.Apple)
    i2 = Item(ItemType.Mango)
    i3 = Item(ItemType.Flint)

    slot.append(i1)
    assert len(slot) == 1
    assert slot[-1] is i1

    slot.extend([i2, i3])
    assert len(slot) == 3
    assert slot[1] is i2
    assert slot[2] is i3

    i4 = Item(ItemType.Stick)
    slot.insert(1, i4) # [Apple, Stick, Mango, Flint]
    assert len(slot) == 4
    assert slot[0] is i1
    assert slot[1] is i4
    assert slot[2] is i2
    assert slot[3] is i3

    popped = slot.pop(1)
    assert popped is i4
    assert len(slot) == 3
    assert slot[1] is i2

    popped_last = slot.pop()
    assert popped_last is i3
    assert len(slot) == 2

    slot.clear()
    assert len(slot) == 0

def test_slot_iterator():
    slot = Slot()
    items = [Item(ItemType.Apple), Item(ItemType.Mango)]
    slot.extend(items)

    iterated = list(slot)
    assert len(iterated) == 2
    assert iterated[0] is items[0]
    assert iterated[1] is items[1]

