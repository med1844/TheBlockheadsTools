from the_blockheads_tools_py import (
    Item,
    ItemType,
    PigmentColor,
    Slot,
    Slots,
    Chest,
    StandardChest,
    SafeChest,
    GoldChest,
    FeederChest,
    ShelfChest,
    Cabinet,
    PortalChest,
    Workbench,
    WorkbenchType,
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
    item.data_b = (3 | (6 << 4)) << 8
    assert item.colors[0] == PigmentColor.EmeraldGreen
    assert item.colors[1] == PigmentColor.RedOchre
    assert item.colors[2] == PigmentColor.Transparent


def test_item_colors():
    item = Item(ItemType.Paint)
    assert item.colors == [PigmentColor.Transparent] * 3
    assert item.data_b == 0

    colors = [
        PigmentColor.RedOchre,
        PigmentColor.EmeraldGreen,
        PigmentColor.Transparent,
    ]
    item.colors = colors
    assert item.colors == colors
    # (3 << 12) | (6 << 8) | (0 << 4) | 0 = 13824
    assert item.data_b == 13824


def test_item_edge_cases():
    item = Item(ItemType.Flint)

    item.type_id = 9999
    with pytest.raises(ValueError, match="Invalid item type id"):
        _ = item.item_type

    with pytest.raises(ValueError, match="colors must not have more than 3 elements"):
        item.colors = [
            PigmentColor.RedOchre,
            PigmentColor.CarbonBlack,
            PigmentColor.CopperBlue,
            PigmentColor.EmeraldGreen,
        ]

    item.data_a = 0xFFFF
    assert item.damage == 65535

    item.data_b = 0x7FFF
    with pytest.raises(ValueError, match="ItemError"):
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


def test_basket_slots():
    # Test creation
    basket = Slots()
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
    container_item = Item(ItemType.Basket, sub_items=basket)
    assert container_item.sub_items is basket

    # Match dispatch test
    match container_item.sub_items:
        case Slots() as basket:
            assert len(basket) == 4
        case _:
            pytest.fail("Should have matched Slots")


def test_item_repr():
    item = Item(ItemType.Flint)
    r = repr(item)
    assert "type_id=3" in r
    assert "sub_items=None" in r
    assert "dynamic_object=None" in r

    basket = Slots()
    item_with_basket = Item(ItemType.Basket, sub_items=basket)
    assert "Slots" in repr(item_with_basket)


def test_chest_basic():
    # Test initialization and defaults
    chest = SafeChest(owner_id="player1")
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
    chest = StandardChest()
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
    chest = GoldChest()
    item = Item(ItemType.GoldenChest, dynamic_object=chest)

    assert isinstance(item.dynamic_object, Chest)
    assert isinstance(item.dynamic_object, GoldChest)
    assert item.dynamic_object is chest

    # Match dispatch
    match item.dynamic_object:
        case GoldChest(owner_id=owner):
            assert owner is None
        case _:
            pytest.fail("Should have matched GoldChest")


def test_chest_validation():
    chest = StandardChest()
    assert len(chest) == 16

    with pytest.raises(ValueError):
        StandardChest(slots=[Slot()])

def test_shelf_chest():
    shelf = ShelfChest()
    assert len(shelf) == 4
    assert shelf.render_items is None

    shelf.render_items = [ItemType.Apple, ItemType.Mango, ItemType.Flint, ItemType.Stick]
    assert len(shelf.render_items) == 4

    with pytest.raises(ValueError):
        ShelfChest(slots=[Slot()] * 5)


def test_chest_roundtrip():
    # 1. StandardChest
    std_chest = StandardChest(owner_id="std_owner")
    std_chest.paint_color = 1
    std_chest.float_pos = [1.0, 2.0]
    std_chest[0] = Slot([Item(ItemType.Apple)])

    container1 = Item(ItemType.Chest, dynamic_object=std_chest)
    assert type(container1.dynamic_object) is StandardChest
    assert container1.dynamic_object.owner_id == "std_owner"
    assert container1.dynamic_object.paint_color == 1
    assert container1.dynamic_object[0][0].item_type == ItemType.Apple

    # 2. SafeChest
    safe_chest = SafeChest(owner_id="safe_owner")
    safe_chest.paint_color = 2
    safe_chest[15] = Slot([Item(ItemType.Mango)])

    container2 = Item(ItemType.Safe, dynamic_object=safe_chest)
    assert type(container2.dynamic_object) is SafeChest
    assert container2.dynamic_object.owner_id == "safe_owner"
    assert container2.dynamic_object.paint_color == 2
    assert container2.dynamic_object[15][0].item_type == ItemType.Mango

    # 3. GoldChest
    gold_chest = GoldChest(owner_id="gold_owner")
    gold_chest[5] = Slot([Item(ItemType.Diamond)])

    container3 = Item(ItemType.GoldenChest, dynamic_object=gold_chest)
    assert type(container3.dynamic_object) is GoldChest
    assert container3.dynamic_object.owner_id == "gold_owner"
    assert container3.dynamic_object[5][0].item_type == ItemType.Diamond

    # 4. FeederChest
    feeder_chest = FeederChest(owner_id="feeder_owner")
    feeder_chest[8] = Slot([Item(ItemType.DodoEgg)])

    container4 = Item(ItemType.FeederChest, dynamic_object=feeder_chest)
    assert type(container4.dynamic_object) is FeederChest
    assert container4.dynamic_object.owner_id == "feeder_owner"
    assert container4.dynamic_object[8][0].item_type == ItemType.DodoEgg

    # 5. ShelfChest
    shelf_chest = ShelfChest()
    shelf_chest.render_items = [ItemType.Apple, ItemType.Unknown, ItemType.Unknown, ItemType.Unknown]
    shelf_chest.item_data_bs = [1, 2, 3, 4]
    shelf_chest[0] = Slot([Item(ItemType.Apple)])

    container5 = Item(ItemType.Shelf, dynamic_object=shelf_chest)
    assert type(container5.dynamic_object) is ShelfChest
    assert container5.dynamic_object.render_items is not None
    assert container5.dynamic_object.render_items[0] == ItemType.Apple
    assert container5.dynamic_object.item_data_bs == [1, 2, 3, 4]
    assert container5.dynamic_object[0][0].item_type == ItemType.Apple

    # 6. Cabinet
    cabinet = Cabinet()
    cabinet.render_items = [ItemType.Mango, ItemType.Unknown, ItemType.Unknown, ItemType.Unknown]
    cabinet.item_data_bs = [10, 20, 30, 40]
    cabinet[3] = Slot([Item(ItemType.Mango)])

    container6 = Item(ItemType.DisplayCabinet, dynamic_object=cabinet)
    assert type(container6.dynamic_object) is Cabinet
    assert container6.dynamic_object.render_items is not None
    assert container6.dynamic_object.render_items[0] == ItemType.Mango
    assert container6.dynamic_object.item_data_bs == [10, 20, 30, 40]
    assert container6.dynamic_object[3][0].item_type == ItemType.Mango

    # 7. PortalChest
    portal = PortalChest(owner_id="portal_master")
    portal.flipped = True
    portal.paint_color = 123
    portal.pos_x = 987654
    portal.pos_y = 512
    portal.float_pos = [10.5, 20.5]
    portal.unique_id = 0xDEADBEEFCAFEBABE

    container7 = Item(ItemType.PortalChest, dynamic_object=portal)

    assert type(container7.dynamic_object) is PortalChest
    assert container7.dynamic_object.unique_id == 0xDEADBEEFCAFEBABE
    assert container7.dynamic_object.owner_id == "portal_master"


def test_workbench_basic():
    wb = Workbench()
    assert wb.workbench_type == WorkbenchType.Workbench
    assert wb.level == 1
    assert wb.owner_id is None

    wb = Workbench(WorkbenchType.Craft, 2, "crafter")
    assert wb.workbench_type == WorkbenchType.Craft
    assert wb.level == 2
    assert wb.owner_id == "crafter"


def test_workbench_properties():
    wb = Workbench()

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
    wb = Workbench(WorkbenchType.Easel)
    item = Item(ItemType.Easel, dynamic_object=wb)

    assert isinstance(item.dynamic_object, Workbench)
    assert item.dynamic_object.workbench_type == WorkbenchType.Easel

    match item.dynamic_object:
        case Workbench(workbench_type=wt):
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
    slot.insert(1, i4)  # [Apple, Stick, Mango, Flint]
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
