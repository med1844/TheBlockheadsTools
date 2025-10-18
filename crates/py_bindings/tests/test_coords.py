import pytest
from the_blockheads_tools_py import ChunkCoord, ChunkBlockCoord, BlockCoord


def test_chunk_coord():
    coord = ChunkCoord(432, 10)
    assert str(coord) == "432_10"
    assert repr(coord) == "ChunkCoord(432, 10)"
    assert coord.x == 432
    assert coord.y == 10

    with pytest.raises(AttributeError):
        coord.x = 5
    with pytest.raises(AttributeError):
        coord.y = 5
    with pytest.raises(ValueError):
        ChunkCoord(123, 45)
    with pytest.raises(OverflowError):
        ChunkCoord(1 << 32, 45)
    with pytest.raises(OverflowError):
        ChunkCoord(123, 257)
    with pytest.raises(OverflowError):
        ChunkCoord(-1, 0)


def test_chunk_block_coord():
    coord = ChunkBlockCoord(12, 10)
    assert str(coord) == "ChunkBlockCoord(12, 10)"
    assert coord.x == 12
    assert coord.y == 10

    with pytest.raises(AttributeError):
        coord.x = 5
    with pytest.raises(AttributeError):
        coord.y = 5
    with pytest.raises(ValueError):
        ChunkBlockCoord(32, 5)
    with pytest.raises(ValueError):
        ChunkBlockCoord(5, 32)
    with pytest.raises(OverflowError):
        ChunkBlockCoord(0, -1)


def test_block_coord():
    coord = BlockCoord(12345, 678)
    assert str(coord) == "BlockCoord(12345, 678)"
    assert coord.x == 12345
    assert coord.y == 678

    with pytest.raises(AttributeError):
        coord.x = 5
    with pytest.raises(AttributeError):
        coord.y = 5
    with pytest.raises(ValueError):
        BlockCoord(123, 1024)
    with pytest.raises(OverflowError):
        BlockCoord(1 << 32, 45)
    with pytest.raises(OverflowError):
        BlockCoord(123, 65536)
    with pytest.raises(OverflowError):
        BlockCoord(-1, 0)
