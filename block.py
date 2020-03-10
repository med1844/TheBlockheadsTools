# encoding: utf-8
from blockType import BlockType, id_to_block_name


class Block:
    """
    This is the support class for block objects in the blockheads, offering 
    encapsulated `get` and `set` block attribute methods. Each block takes 
    64 bytes, yet the effect of most bytes are still uncertain and need to be 
    tested. Thus, structures that supports future changes should be applied.

    这是the blockheads中方块的支持类。用于提供对单个方块属性的设置与提取。
    目前已知每个方块占据64字节。可惜的是，还有很多字节的意义没有被解析出来。
    因此，这一模块会采用支持未来改动的设计。

    ### Currently supported attributes 目前可用的属性

    - "first_layer_id", 1 byte
    - "third_layer_id", 1 byte
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
        return id_to_block_name(self.get("first_layer_id")[0])[:5]
    
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

    def get(self, attr_name):
        """
        Return a list of bytes corresponding to the attribute name.
        根据输入的属性名，返回对应的bytes列表。

        ### Arguments
        - `attr_name`
            the attribute name that want to read
            想要读取的属性名
        
        ### Return
        A list of several integers.
        一个包含几个整数的列表
        """
        res = [self[pos] for pos in self.pos_map[attr_name]]
        return res
    
    def set(self, attr_name, *values):
        """
        Set values according to the attribute name and input values.
        根据输入的属性名和值，设置方块对应属性。

        ### Arguments
        - `attr_name`
            the attribute name for setting values
            要设置的属性
        - `*values`
            The values to be set on each position.
            要在各个位置上设置的值。
        
        ### Return
        Nothing.
        无。
        """
        positions = self.pos_map[attr_name]
        if len(values) != len(positions):
            raise TypeError("%s requires %d argument, while only offering %d"
                            % (attr_name, len(positions), len(values)))

        for i, pos in enumerate(positions):
            if isinstance(values[i], BlockType):
                self[pos] = values[i].value
            else:
                self[pos] = values[i]
    
    def to_hex(self):
        return ' '.join(['%02x' % self[i] for i in range(64)])