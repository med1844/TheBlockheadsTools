import pytest
from the_blockheads_tools_py import WorldDb
from constants import WORLD_DB_PATH


def test_world_v2_read():
    world_db = WorldDb.open(WORLD_DB_PATH)
    world_v2 = world_db.main.world_v2

    assert world_v2.expert_mode == False
    assert world_v2.host_port == "15151"
    assert world_v2.max_players == "32"
    assert world_v2.migration_complete_v1_7 == True
    assert world_v2.no_rain_timer == 220.7978
    assert world_v2.portal_level == 0
    assert world_v2.random_seed == 1711316399
    assert world_v2.remote_game == False
    assert world_v2.run_at_launch == True
    assert world_v2.save_id == "3d716d9bbf89c77ef5001e9cd227ec29"
    assert world_v2.save_version == 1100
    assert world_v2.start_portal_pos_x == 10740
    assert world_v2.start_portal_pos_y == 520
    assert world_v2.translation == (10740.0, 520.0)
    assert world_v2.world_name == "TEST"
    assert world_v2.world_time == 3370.800003290176
    assert world_v2.world_width_macro == 512

    assert world_v2.circum_navigate_booleans_data.startswith(b"bplist00")
    assert world_v2.found_items.startswith(b"bplist00")


def test_world_v2_write():
    world_db = WorldDb.open(WORLD_DB_PATH)
    world_v2 = world_db.main.world_v2
    with pytest.raises(TypeError):
        world_v2.host_port = 12345  # ty: ignore[invalid-assignment]
    world_v2.host_port = "51515"
    assert world_v2.host_port == "51515"

    with pytest.raises(TypeError):
        world_v2.translation = "15151"  # ty: ignore[invalid-assignment]
    world_v2.translation = (520, 520)
    assert world_v2.translation == (520.0, 520.0)

    with pytest.raises(TypeError):
        world_v2.translation = "15151"  # ty: ignore[invalid-assignment]
    world_v2.circum_navigate_booleans_data = b"12345"
    assert world_v2.circum_navigate_booleans_data == b"12345"
