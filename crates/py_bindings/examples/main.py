from world_db_io import world_db_io
from chunk_block import chunk_block
from inventory import inventory
from the_blockheads_tools_py import WorldDb, Arch
import pathlib
import shutil

world_db = WorldDb.open_path("../tests/resources/9c3/world_db")
world_db_io(world_db)
chunk_block(world_db)
inventory(world_db)

folder_path = pathlib.Path("./output")
if folder_path.exists() and folder_path.is_dir():
    shutil.rmtree(folder_path)
    print(f"Deleted existing folder: {folder_path}")
folder_path.mkdir(parents=True, exist_ok=True)
world_db.save_path(folder_path, Arch.Arch32)
