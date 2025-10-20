import pytest
from the_blockheads_tools_py import WorldDb
from constants import WORLD_DB_PATH


def test_world_db_main():
    main = WorldDb.open(WORLD_DB_PATH).main
    assert main.blockheads.startswith(b"<?xml")

    with pytest.raises(AttributeError):
        main.blockheads = b"12345"
    with pytest.raises(AttributeError):
        main.dynamic_world_v2 = b"12345"
