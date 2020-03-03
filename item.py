# encoding: utf-8
from exportable import Exportable
from gzipWrapper import GzipWrapper
from bplist import BPList
import biplist
import struct


class SingleItem(Exportable):
    """
    The support class for single item. Stacked items would be optimized by the 
    `Item` class.
    对单个物品的支持类。针对堆叠物品的优化将由Item类来处理。
    """

    def __init__(self, src_bytes):
        self._data = src_bytes if len(src_bytes) == 8 else src_bytes[:8]
        self._zip = None
        if len(src_bytes) > 8:
            self._zip = GzipWrapper(src_bytes[8:])
            if self._zip._data[0].startswith(b"<?xml"):
                self._zip._data[0] = BPList(
                    biplist.readPlistFromString(self._zip._data[0]), "xml"
                )
            elif self._zip._data[0].startswith(b"bplist00"):
                self._zip._data[0] = BPList(
                    biplist.readPlistFromString(self._zip._data[0]), "bp"
                )
            self._zip._data[0] = self._parse(self._zip._data[0])
        self.is_container = self._zip is not None
    
    def __repr__(self):
        if self._zip is None:
            return "'item %d'" % self.get_id()
        return "'item %d': %r" % (self.get_id(), self._zip._data[0])

    def _parse(self, src):
        if isinstance(src, BPList):
            src._data = self._parse(src._data)
            return src
        if isinstance(src, dict):
            for k, v in src.items():
                src[k] = self._parse(v)
            return src
        if isinstance(src, list):
            if src and isinstance(src[0], (str, bytes, biplist.Data)):
                src = Item(src)
            else:
                for i, v in enumerate(src):
                    src[i] = self._parse(v)
            return src
        return src

    def get_id(self):
        return struct.unpack("<H", self._data[:2])[0]
    
    def set_id(self, new_id):
        new_id = struct.pack("<H", new_id)
        self._data = new_id + self._data[2:]

    def export(self):
        """
        Export the item object to binary string.
        将item对象导出为二进制数据流。
        """
        if self._zip is None:
            return biplist.Data(self._data)
        return biplist.Data(self._data + self._zip.export())


class Item(Exportable):

    def __init__(self, src_list):
        self.items = None
        self.count = len(src_list)
        self.stack = True  # stacked single items
        if self.count:
            first_item = SingleItem(src_list[0])
            if first_item.is_container:
                # if the first item is container, then we shall not stack
                self.stack = False
                self.items = [SingleItem(v) for v in src_list]
            else:
                self.items = [first_item]
    
    def __repr__(self):
        if self.count and self.stack:
            return "%r * %d" % (self.items[0], self.count)
        else:
            if self.count:
                return repr(self.items)
            return "empty"
    
    def export(self):
        if not self.count:
            return []
        if self.stack:
            return [self.items[0].export()] * self.count
        else:
            return [v.export() for v in self.items]