# encoding: utf-8
from exportable import Exportable
from blockType import id_to_block_name


class Block(Exportable):
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
        "third_layer_id": [2],
        "brightness": [6],
    }

    def __init__(self, src_bytes):
        super().__init__()
        self._data = list(src_bytes)
    
    def __repr__(self):
        return id_to_block_name(self.get_attr("first_layer_id")[0])
    
    def __str__(self):
        return "<Block: %r>"\
               % {attr: self.get_attr(attr) for attr in self.pos_map}

    def get_attr(self, attr_name) -> list:
        """
        Return a list of bytes corresponding to the attribute name.
        根据输入的属性名，返回对应的bytes列表。

        ### Arguments
        - `attr_name`
            the attribute name that want to read
            想要读取的属性名
        
        ### Return
        A list of bytes.
        一个包含bytes的列表
        """
        return [self._data[pos] for pos in self.pos_map[attr_name]]
    
    def set_attr(self, attr_name, *values) -> None:
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
            self._data[pos] = values[i]

    def export(self) -> bytes:
        """
        Export bytes representing `self` which would be later saved to files.
        导出用于保存的方块数据。
        """
        return bytes(self._data)