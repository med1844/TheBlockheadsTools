# encoding: utf-8
import biplist
import io
from exportable import Exportable


class BPList(Exportable):
    """
    This is the support class for bplist, making export easier.
    bplist的支持类。封装只是为了让导出变得更方便。
    """

    def __init__(self, data, src_type):
        self._data = data
        self.src_type = src_type

    def __repr__(self):
        return "<BPList>" + repr(self._data)
    
    def __eq__(self, other):
        if isinstance(other, BPList):
            return self._data == other._data
    
    def __getitem__(self, key):
        return self._data[key]
    
    def __setitem__(self, key, value):
        self._data[key] = value
    
    def items(self):
        assert isinstance(self._data, dict)
        return self._data.items()

    def export(self):
        """
        Export `self` as a binary bplist.
        将自己导出成二进制的bplist
        """
        if isinstance(self._data, dict):
            result = dict(self._data)
            for k, v in result.items():
                if isinstance(v, BPList):
                    result[k] = biplist.Data(v.export())
        elif isinstance(self._data, list):
            result = list(self._data)
            for i, v in enumerate(result):
                if isinstance(v, BPList):
                    result[i] = biplist.Data(v.export())
        if self.src_type == "bp":
            return biplist.writePlistToString(result)
        elif self.src_type == "xml":
            return biplist.writePlistToString(result, False)