import io
import pytest
import blockType
from chunk import Chunk


class TestBlockType:

    def test_block_name_to_id(self):
        assert blockType.block_name_to_id("STEEL_BLOCK") == 57
        assert blockType.block_name_to_id("BLACK_GLASS") == 59
    
    def test_block_name_to_id_keyerror(self):
        with pytest.raises(KeyError):
            blockType.block_name_to_id("NOT_EXIST")
        with pytest.raises(KeyError):
            blockType.block_name_to_id("STEAL_BLOCK")
    
    def test_id_to_block_name(self):
        assert blockType.id_to_block_name(1) == "STONE"
        assert blockType.id_to_block_name(9) == "STONE_"
        assert blockType.id_to_block_name(57) == "STEEL_BLOCK"
    
    def test_id_to_block_name_keyerror(self):
        for id_ in [-1, 21, 22, 23, 66, 114514]:
            with pytest.raises(KeyError):
                blockType.id_to_block_name(id_)


class TestChunk:

    def test_load_from_gzip_file(self):
        with open("./test_data/blocks/25_19_compressedBlock", "rb") as f:
            Chunk.from_gzip_file(f)
    
    def test_load_from_raw(self):
        with open("./test_data/blocks/blocks_35_1", "rb") as f:
            Chunk(f.read())

    def test_read_and_export(self):
        with open("./test_data/blocks/blocks_9_8", "rb") as f:
            data = f.read()
        c = Chunk(data)
        exported = c.export()
        f = io.BytesIO(exported)
        c2 = Chunk.from_gzip_file(f)
        assert repr(c) == repr(c2)
    
    def test_get_block(self):
        with open("./test_data/blocks/blocks_9_8", "rb") as f:
            data = f.read()
        c = Chunk(data)
        for x in range(32):
            for y in range(32):
                c.get_block(x, y)
    
    def test_get_block_assert_fail(self):
        with open("./test_data/blocks/blocks_9_8", "rb") as f:
            data = f.read()
        c = Chunk(data)
        for x in [-2, -1, 32, 33]:
            for y in [-2, -1, 32, 33]:
                with pytest.raises(AssertionError):
                    c.get_block(x, y)
            for y in range(32):
                with pytest.raises(AssertionError):
                    c.get_block(x, y)
        for x in range(32):
            for y in [-2, -1, 32, 33]:
                with pytest.raises(AssertionError):
                    c.get_block(x, y)
    
    def test_set_block_attr(self):
        with open("./test_data/blocks/blocks_9_8", "rb") as f:
            data = f.read()
        c = Chunk(data)
        b = c.get_block(5, 7)
        b.set_attr("first_layer_id", 57)
