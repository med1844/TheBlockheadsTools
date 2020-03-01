# encoding: utf-8
import biplist
import io
from exportable import Exportable


class BPList(Exportable):
    """
    This is the support class for bplist, making export easier.
    bplist的支持类。封装只是为了让导出变得更方便。
    """

    def __init__(self, data):
        self._data = data

    def __repr__(self):
        return "<BPList>" + repr(self._data)
    
    def __eq__(self, other):
        if isinstance(other, BPList):
            return self._data == other._data
    
    def __getitem__(self, key):
        return self._data[key]
    
    def __setitem__(self, key, value):
        self._data[key] = value

    @classmethod
    def from_bytes(cls, src_bytes):
        """
        Create and return a `BPList` object from given bytes.
        根据给定数据创建并返回一个新`BPList`对象。

        ### Arguments
        - `src_bytes`
            A sequence of bytes in binary representation that is a bplist
            一串bplist二进制形式的字符序列。

        ### Return
        A new `BPList` object.
        一个新的`BPList`对象。
        """
        result = BPList(biplist.readPlistFromString(src_bytes))
        for k, v in result._data.items():
            if isinstance(v, biplist.Data) and v.startswith(b"bplist00"):
                result._data[k] = BPList.from_bytes(v)
        return result

    def export(self):
        """
        Export `self` as a binary bplist.
        将自己导出成二进制的bplist
        """
        result_dict = dict(self._data)
        for k, v in result_dict.items():
            if isinstance(v, BPList):
                result_dict[k] = biplist.Data(v.export())
        return biplist.writePlistToString(result_dict)


if __name__ == "__main__":
    from pprint import pprint
    with open("./test_data/bplists/worldv2", "rb") as f:
        l = BPList.from_bytes(f.read())
        pprint(l._data)