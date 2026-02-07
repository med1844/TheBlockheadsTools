from the_blockheads_tools_py import WorldDb
import sys

db = WorldDb.open(sys.argv[1])
print(db.main.blockheads[0].skin_options)
print(db.main.dynamic_world_v2.active_blockhead_index)
