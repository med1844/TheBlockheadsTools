import io
from exportable import Exportable


class Chunk(Exportable):
    """
    This is the support class for chunks in the blockheads, offering load, 
    modify, and exporting methods.

    这是对the blockheads中的区块的支持类，用于封装读取，修改，导出等方法。
    """

    def __init__(self, src_bytes: bytes):
        with io.BytesIO(src_bytes) as f:


    def export(self) -> bytes:
        """

        """