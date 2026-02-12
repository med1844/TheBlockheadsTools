from the_blockheads_tools_py import WorldDb
from constants import WORLD_DB_PATH
from pathlib import Path


def test_world_db_load():
    # we can open it
    world_db = WorldDb.open_path(WORLD_DB_PATH)

    # and we can access it's properties
    world_db.chunks


def test_world_db_load_pathlib():
    # we can open it with pathlib
    world_db = WorldDb.open_path(Path(WORLD_DB_PATH))

    # and we can access it's properties
    world_db.chunks


def test_world_db_bytes():
    with open(Path(WORLD_DB_PATH) / "data.mdb", "rb") as f:
        data = f.read()

    # we can open it from bytes
    world_db = WorldDb.open_bytes(data)

    # and we can access it's properties
    world_db.chunks

    # and we can save it to bytes
    saved_data = world_db.save_bytes()

    # and the saved data should be valid
    world_db2 = WorldDb.open_bytes(saved_data)
    world_db2.chunks


def test_world_db_bytearray():
    with open(Path(WORLD_DB_PATH) / "data.mdb", "rb") as f:
        data = bytearray(f.read())

    # we can open it from bytearray
    world_db = WorldDb.open_bytes(data)
    world_db.chunks
