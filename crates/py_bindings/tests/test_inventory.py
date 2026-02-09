from the_blockheads_tools_py import Inventory, StackedItem, Item, ItemType
import pytest

def test_inventory_creation():
    # Default creation
    inv = Inventory()
    assert len(inv) == 8
    for i in range(8):
        assert len(inv[i]) == 0
        assert isinstance(inv[i], StackedItem)

    # Creation with slots
    slots = [StackedItem([Item(ItemType.Flint)]) for _ in range(8)]
    inv = Inventory(slots)
    assert len(inv) == 8
    assert len(inv[0]) == 1
    assert inv[0][0].item_type == ItemType.Flint

def test_inventory_creation_errors():
    with pytest.raises(ValueError, match="Inventory must have exactly 8 slots"):
        Inventory([])
    
    with pytest.raises(ValueError, match="Inventory must have exactly 8 slots"):
        Inventory([StackedItem()] * 7)

def test_inventory_access():
    inv = Inventory()
    item = Item(ItemType.Apple)
    stacked = StackedItem([item])
    
    inv[0] = stacked
    assert inv[0] is stacked
    assert inv[0][0] is item
    
    # Negative indexing
    inv[-1] = stacked
    assert inv[7] is stacked
    
    # Out of bounds
    with pytest.raises(IndexError):
        _ = inv[8]
    with pytest.raises(IndexError):
        inv[-9] = stacked

def test_inventory_identity():
    inv = Inventory()
    slot0_ref1 = inv[0]
    slot0_ref2 = inv[0]
    
    assert slot0_ref1 is slot0_ref2
    
    # Modification via reference
    slot0_ref1.items = [Item(ItemType.GoldIngot)]
    assert len(slot0_ref2) == 1
    assert slot0_ref2[0].item_type == ItemType.GoldIngot

def test_inventory_repr():
    inv = Inventory()
    r = repr(inv)
    assert "Inventory(slots=[" in r
    assert "StackedItem(items=[])" in r
