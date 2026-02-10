import pytest
from the_blockheads_tools_py import WorldDb
from constants import WORLD_DB_PATH


def test_world_db_main():
    main = WorldDb.open(WORLD_DB_PATH).main
    # blockheads is now a list of BlockheadPy
    assert isinstance(main.blockheads, list)
    with pytest.raises(AttributeError):
        main.blockheads = b"12345"  # ty: ignore[invalid-assignment]
    with pytest.raises(AttributeError):
        main.dynamic_world_v2 = b"12345"  # ty: ignore[invalid-assignment]
    if main.blockheads:
        assert isinstance(main.blockheads[0].name, str)

def test_inventories():
    db = WorldDb.open(WORLD_DB_PATH)
    inv_keys = db.main.blockhead_inventory_keys
    assert isinstance(inv_keys, set)

    # If there are blockheads, try to get one
    if inv_keys:
        first_id = list(inv_keys)[0]
        inv = db.main.get_blockhead_inventory(first_id)
        assert inv is not None
        assert len(inv.slots) == 8

        # Test setting (roundtrip)
        db.main.set_blockhead_inventory(first_id, inv)
        inv2 = db.main.get_blockhead_inventory(first_id)
        assert inv2 is not None
        assert len(inv2.slots) == 8
        assert inv2 is inv
