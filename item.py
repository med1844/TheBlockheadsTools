# encoding: utf-8
from exportable import Exportable
from gzipWrapper import GzipWrapper
from bplist import BPList
import biplist
import struct
from itemType import ItemType


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
            elif not src:
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
    
    def get_damage(self):
        return struct.unpack("<H", self._data[2:4])[0]

    def set_damage(self, new_damage):
        new_damage = struct.pack("<H", new_damage)
        self._data = self._data[:2] + new_damage + self._data[4:]
    
    def __getitem__(self, key):
        return self._zip._data[0][key]
    
    def __setitem__(self, key, value):
        self._zip._data[0][key] = value

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
        else:
            # empty slot
            self.stack = False
    
    def __repr__(self):
        if self.count and self.stack:
            return "%r * %d" % (self.items[0], self.count)
        else:
            if self.count:
                return repr(self.items)
            return "empty"
        
    def __getitem__(self, index):
        return self.items[index]
    
    def __setitem__(self, index, value):
        self.items[index] = value
    
    def get_id(self):
        return self.items[0].get_id()
    
    def set_id(self, new_id):
        if isinstance(new_id, ItemType):
            new_id = new_id.value
        if self.items:
            for item in self.items:
                item.set_id(new_id)
        else:
            self.items = \
                [SingleItem(struct.pack("<H", new_id) + "\x00" * 5 + '\x0c')]
            self.count = 1
            self.stack = True
            # TODO add container support or add container check
    
    def get_count(self):
        return self.count

    def set_count(self, new_count):
        self.count = new_count
    
    def export(self):
        if not self.count:
            return []
        if self.stack:
            return [self.items[0].export()] * self.count
        else:
            return [v.export() for v in self.items]