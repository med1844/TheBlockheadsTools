# encoding: utf-8
import io
import plistlib
from typing import Literal
from exportable import Exportable
from copy import deepcopy


class BPList(Exportable):
    """
    This is the support class for bplist, making export easier.
    bplist的支持类。封装只是为了让导出变得更方便。
    """

    def __init__(self, data, src_type: Literal["bp", "xml"]):
        self._data = data
        self.src_type = src_type

    def __repr__(self):
        return repr(self._data)

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

    @classmethod
    def _update_exportable(cls, container):
        if not isinstance(container, (dict, list, cls)):
            return
        if isinstance(container, cls):
            cls._update_exportable(container._data)
        if isinstance(container, dict):
            for k, v in container.items():
                if isinstance(v, Exportable):
                    container[k] = v.export()
                else:
                    cls._update_exportable(v)
        if isinstance(container, list):
            for i, v in enumerate(container):
                if isinstance(v, Exportable):
                    container[i] = v.export()
                else:
                    cls._update_exportable(v)

    def export(self) -> bytes:
        result = deepcopy(self._data)
        self._update_exportable(result)
        match self.src_type:
            case "bp":
                return plistlib.dumps(result, fmt=plistlib.FMT_BINARY)
            case "xml":
                return plistlib.dumps(result, fmt=plistlib.FMT_XML)
        assert False
