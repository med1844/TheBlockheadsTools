from the_blockheads_tools_py import WorldDb
from constants import WORLD_DB_PATH


def test_world_db_load():
    # we can open it
    world_db = WorldDb.open(WORLD_DB_PATH)

    # and we can access it's properties
    world_db.chunks
