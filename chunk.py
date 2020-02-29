import io
import gzip
from exportable import Exportable
from block import Block


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

    def __init__(self, src_bytes: bytes):
        self._blocks = [[None] * self.CHUNK_WIDTH
                        for _ in range(self.CHUNK_HEIGHT)]
        with io.BytesIO(src_bytes) as f:
            for row in range(self.CHUNK_HEIGHT):
                for col in range(self.CHUNK_WIDTH):
                    self._blocks[row][col] = Block(f.read(self.BLOCK_SIZE))
    
    def __repr__(self):
        return '\n'.join([
            ''.join(['%10s' % repr(self._blocks[row][col])
                     for col in range(self.CHUNK_WIDTH)])
            for row in range(self.CHUNK_HEIGHT - 1, -1, -1)
        ])
    
    @classmethod
    def from_gzip_file(cls, gzip_file):
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
        with gzip.open(gzip_file, "rb") as f:
            return Chunk(f.read())

    def export(self) -> bytes:
        """
        Export a bytes object. First get all 
        返回将chunk数据经由gzip压缩后所得的二进制字符串。
        """
        with io.BytesIO() as f:
            with gzip.GzipFile(fileobj=f, mode="wb") as g:
                for row in range(self.CHUNK_HEIGHT):
                    for b in self._blocks[row]:
                        g.write(b.export())
                g.write(bytes((0,) * 5))
            result = f.getvalue()
        return result
    
    def get_block(self, x: int, y: int) -> Block:
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
        assert self._blocks[x][y] is not None
        return self._blocks[x][y]


if __name__ == "__main__":
    with open("./test_data/blocks/blocks_46_14", "rb") as f:
        chunk = Chunk(f.read())
    print(chunk.export())