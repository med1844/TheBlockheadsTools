# encoding: utf-8
from item import Item


class Inventory:

    def __init__(self, item_list):
        self._list = item_list
        for i, raw_item_list in enumerate(self._list):
            self._list[i] = Item(raw_item_list)
    
    def __repr__(self):
        result = ["["]
        for i, item in enumerate(self._list):
            result.append("   " * 4 + "%d: " % i + repr(item))
        result.append("]")
        return '\n'.join(result)
    
    def __getitem__(self, index):
        return self._list[index]
    
    def __setitem__(self, index, value):
        self._list[index] = value