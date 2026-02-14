from constants import Snippet


# --8<-- [start:open_save]
from the_blockheads_tools_py import WorldDb

# The path to the save file, it should have `data.mdb` in it.
# ```
# > tree resources
# resources
# └── 3d7
#     └── world_db
#         └── data.mdb
# ```
world_db_path = "../tests/resources/3d7/world_db"

# Load a world
world_db = WorldDb.open_path(world_db_path)
# --8<-- [end:open_save]

with Snippet("world_info_output"):
    # --8<-- [start:world_info]
    # Read world information
    world_v2 = world_db.main.world_v2
    print("world name:", world_v2.world_name)
    print("seed:", world_v2.random_seed)
    print("world width:", world_v2.world_width_macro)
    print("start portal:", (world_v2.translation))
    # --8<-- [end:world_info]


def world_db_io(world_db: WorldDb):
    world_v2 = world_db.main.world_v2

    # --8<-- [start:write_save]
    from the_blockheads_tools_py import Arch
    world_v2.world_name = "edited"
    modified_world_db_bytes = world_db.save_bytes(Arch.Arch32)

    # or use save_path
    # world_db.save_path("./path/to/your/output", Arch.Arch32)
    # --8<-- [end:write_save]

    modified_world_db = world_db.open_bytes(modified_world_db_bytes)
    assert modified_world_db.main.world_v2.world_name == "edited"
