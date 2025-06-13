# encoding: utf-8
from blockType import BlockType, id_to_block_name


class Block:
    """
    This is the support class for block objects in the blockheads, offering
    encapsulated `get` and `set` block attribute methods. Each block takes
    64 bytes, yet the effect of most bytes are still uncertain and need to be
    tested. Thus, structures that supports future changes should be applied.
    """

    # records the position of each attribute
    pos_map = {
        "first_layer_id": [0],
        "third_layer_id": [1],
        "sub_type": [3],
        "height": [4],
        "damage": [5],
        "visibility": [6],  # 6 and 9 are both visibility, yet only the change
        # of 6th byte would work
        "brightness": [7],
    }

    def __init__(self, src_array, start_pos):
        self._data = src_array
        self._st = start_pos

    def __repr__(self):
        return self.fg_type().name[:5]

    def __getitem__(self, index):
        if index < 0:
            index += 64
        assert index < 64
        return self._data[self._st + index]

    def __setitem__(self, index, value):
        if index < 0:
            index += 64
        assert index < 64
        self._data[self._st + index] = value

    def fg_type(self) -> BlockType:
        return BlockType(self[0])

    def set_fg_type(self, t: BlockType):
        self[0] = t.value

    def bg_type(self) -> BlockType:
        return BlockType(self[1])

    def set_bg_type(self, t: BlockType):
        self[1] = t.value

    def sub_type(self) -> int:
        """returns sub-type of current block (e.g. tree type, ore type)"""
        return self[3]

    def set_sub_type(self, t: int):
        self[3] = t

    def height(self) -> int:
        """returns height of current block (e.g. water, snow)"""
        return self[4]

    def set_height(self, h: int):
        self[4] = h

    def damage(self) -> int:
        """returns how damaged current block is"""
        return self[5]

    def visibility(self) -> int:
        return self[6]

    def to_hex(self) -> str:
        return " ".join(["%02x" % self[i] for i in range(64)])
