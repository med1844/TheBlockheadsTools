# encoding: utf-8
import biplist
import io
from exportable import Exportable
from copy import deepcopy


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
    
    def __len__(self):
        return len(self._data)
    
    def items(self):
        assert isinstance(self._data, dict)
        return self._data.items()
    
    @staticmethod
    def _wrap(exported_data):
        """
        Wrap the exported data from Exportable objects to proper types
        """
        if isinstance(exported_data, (str, bytes)):
            return biplist.Data(exported_data)
        return exported_data

    @classmethod
    def _update_exportable(cls, container):
        if not isinstance(container, (dict, list, cls)):
            return
        if isinstance(container, cls):
            cls._update_exportable(container._data)
        if isinstance(container, dict):
            for k, v in container.items():
                if isinstance(v, Exportable):
                    container[k] = cls._wrap(v.export())
                else:
                    cls._update_exportable(v)
        if isinstance(container, list):
            for i, v in enumerate(container):
                if isinstance(v, Exportable):
                    container[i] = cls._wrap(v.export())
                else:
                    cls._update_exportable(v)

    def export(self):
        """
        Export `self` as a binary bplist.
        将自己导出成二进制的bplist
        """
        result = deepcopy(self._data)
        self._update_exportable(result)
        if self.src_type == "bp":
            return biplist.writePlistToString(result)
        elif self.src_type == "xml":
            return biplist.writePlistToString(result, False)