# encoding: utf-8
import io
import gzip
from exportable import Exportable
from block import Block
import array


class Chunk(Exportable):
    """
    This is the support class for chunks in the blockheads, offering load, 
    modify, and exporting methods.

    这是对the blockheads中的区块的支持类，用于封装读取，修改，导出等方法。

    Chunk format:
     Y
    31| 992| 993| 994|     1023|
    30| 960| 961| 962|      991|
    ...
     2|  64|  65|  66|       95|
     1|  32|  33|  34|       63|
     0|   0|   1|   2|       31|
          0    1    2   ...  31   X
    """

    CHUNK_WIDTH = 32
    CHUNK_HEIGHT = 32
    BLOCK_SIZE = 64

    def __init__(self, src_bytes):
        self._blocks = array.array("B")
        self._blocks.fromstring(src_bytes)
    
    def __repr__(self):
        return '\n'.join([
            ''.join(['%7s' % repr(self.get_block(x, y))
                     for x in range(self.CHUNK_WIDTH)])
            for y in range(self.CHUNK_HEIGHT - 1, -1, -1)
        ])
    
    @classmethod
    def from_compressed_file(cls, compressed_file):
        """
        Read chunk data from the input file object and return a new `Chunk` 
        object.
        从输入的文件中读取并返回一个`Chunk`对象。

        ### Arguments
        - `gzip_file`
            A `file` object, whose content makes up a gzip file.
            一个内容是gzip压缩包数据的`file`对象。
        
        ### Return
        A new `Chunk` object
        一个新`Chunk`对象。
        """
        with gzip.GzipFile(fileobj=compressed_file, mode="rb") as f:
            return Chunk(f.read())
        
    @classmethod
    def create(cls):
        return cls("\0" \
            * (cls.CHUNK_WIDTH * cls.CHUNK_HEIGHT * cls.BLOCK_SIZE + 5))

    def export(self):
        """
        Export a string object, whose content is the compressed chunk data.
        返回将chunk数据经由gzip压缩后所得的二进制字符串。
        """
        with io.BytesIO() as f:
            with gzip.GzipFile(fileobj=f, mode="wb") as g:
                g.write(self._blocks.tostring())
            result = f.getvalue()
        return result
    
    def get_block(self, x, y):
        """
        Get block at position (x, y).
        获取位于(x, y)的方块。

        ### Arguments
        - `x`
            An integer in [0, 31]
            一个区间在[0, 31]的整数。
        - `y`
            An integer in [0, 31]
            一个区间在[0, 31]的整数。

        ### Return
        Reference of the `Block` object required.
        请求的`Block`对象的引用。
        """
        assert 0 <= x < 32 and 0 <= y < 32
        start_addr = (y << 5 | x) << 6
        return Block(self._blocks, start_addr)


if __name__ == "__main__":
    with open("./test_data/blocks/blocks_9_8", "rb") as f:
        chunk = Chunk(f.read())
    print(repr(chunk))