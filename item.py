# encoding: utf-8
from exportable import Exportable
from gzipWrapper import GzipWrapper
from bplist import BPList
import biplist
import struct
from itemType import ItemType, ItemExtra


class NotWorkbenchError(Exception):
    """Raised when trying to set the level of a item that is not a workbench"""
    pass


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
        self.has_extra = self._zip is not None
    
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
    
    def set_color(self, *colors):
        assert 1 <= len(colors) <= 4
        val = 0
        for i in range(4):
            val <<= 4
            if i < len(colors):
                val |= colors[i]
        self._data = self._data[:4] + struct.pack("<H", val) + self._data[6:]
    
    def __getitem__(self, key):
        return self._zip._data[0][key]
    
    def __setitem__(self, key, value):
        self._zip._data[0][key] = value
    
    def init_extra(self, dict_):
        self._zip = GzipWrapper("")
        self._zip._data[0] = BPList(
            biplist.readPlistFromString(
                biplist.writePlistToString(dict_)
            ), "xml"
        )
        self._zip._data[0] = self._parse(self._zip._data[0])
        self.has_extra = True
    
    def remove_extra(self):
        self._zip = None
        self.has_extra = False

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
            if first_item.has_extra:
                # if the first item is extra data, then we shall not stack
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
    
    def get(self, *indexes):
        """
        A shortcut to get item at given position. Note that indexes should be
        0-indexed.
        eg:
        To get the first item in the second row in a chest:
        >>> item.get(1, 0) 
        """
        if len(indexes) == 1:
            # basket
            return self[0]['s'][3 - indexes[0]]
        elif len(indexes) == 2:
            return self[0]['saveItemSlots'][indexes[0]][3 - indexes[1]]
        raise TypeError("The count of parameter should not exceed 2.")
    
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
    
    def set_damage(self, damage):
        for item in self.items:
            item.set_damage(damage)

    def get_damage(self):
        return self.items[0].get_damage()
    
    def get_count(self):
        return self.count

    def set_count(self, new_count):
        self.count = new_count
    
    def get_level(self, index=0):
        """
        Get the workbench level of item at `index`
        """
        try:
            return self.items[index]['d']['level']
        except KeyError:
            raise NotWorkbenchError("Current item is not workbench")
    
    def set_level(self, value, index=0):
        assert isinstance(value, int)
        try:
            self.items[index]['d']['level'] = value
        except KeyError:
            raise NotWorkbenchError("Current item is not workbench")
    
    def set_color(self, *colors):
        self.items[0].set_color(*colors)
    
    def init_extra(self, dict_, index=0):
        if isinstance(dict_, ItemExtra):
            dict_ = dict_.value
        if self.stack:
            if self.items[0].has_extra:
                return
            self.stack = False
            first_data = self.items[0].export()
            for _ in range(1, self.count):
                self.items.append(SingleItem(first_data))
        else:
            if self.items[index].has_extra:
                return
        self.items[index].init_extra(dict_)
    
    def remove_extra(self, index=0):
        self.items[index].remove_extra()
    
    def clear(self):
        self.items = None
        self.count = 0
        self.stack = False

    def export(self):
        if not self.count:
            return []
        if self.stack:
            return [self.items[0].export()] * self.count
        else:
            return [v.export() for v in self.items]