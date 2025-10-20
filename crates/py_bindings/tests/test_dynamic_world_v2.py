import pytest
from the_blockheads_tools_py import WorldDb
from constants import WORLD_DB_PATH


def test_dynamic_world_v2_read():
    world_db = WorldDb.open(WORLD_DB_PATH)
    dynamic_world_v2 = world_db.main.dynamic_world_v2
    assert dynamic_world_v2.active_blockhead_index == 0
    assert dynamic_world_v2.dynamic_object_id_count == 515
    assert dynamic_world_v2.save_version == 8
    assert dynamic_world_v2.saved_glow_indices.startswith(b"bplist00")
    assert dynamic_world_v2.workbench_has_been_crafted == False


def test_dynamic_world_v2_write():
    world_db = WorldDb.open(WORLD_DB_PATH)
    dynamic_world_v2 = world_db.main.dynamic_world_v2

    dynamic_world_v2.active_blockhead_index = 100
    assert dynamic_world_v2.active_blockhead_index == 100
    with pytest.raises(OverflowError):
        dynamic_world_v2.active_blockhead_index = -1  # u64 internally
    with pytest.raises(TypeError):
        dynamic_world_v2.active_blockhead_index = "123"

    dynamic_world_v2.dynamic_object_id_count = 12345
    assert dynamic_world_v2.dynamic_object_id_count == 12345
    with pytest.raises(OverflowError):
        dynamic_world_v2.dynamic_object_id_count = -1  # u64 internally
    with pytest.raises(TypeError):
        dynamic_world_v2.dynamic_object_id_count = "123"

    dynamic_world_v2.save_version = 0
    assert dynamic_world_v2.save_version == 0
    with pytest.raises(OverflowError):
        dynamic_world_v2.save_version = -1  # u8 internally
    with pytest.raises(TypeError):
        dynamic_world_v2.save_version = "123"

    dynamic_world_v2.saved_glow_indices = b""
    assert dynamic_world_v2.saved_glow_indices == b""
    with pytest.raises(TypeError):
        dynamic_world_v2.saved_glow_indices = "123"

    dynamic_world_v2.workbench_has_been_crafted = True
    assert dynamic_world_v2.workbench_has_been_crafted == True
    with pytest.raises(TypeError):
        dynamic_world_v2.saved_glow_indices = "123"
