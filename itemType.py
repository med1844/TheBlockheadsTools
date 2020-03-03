from enum import Enum


class ItemType(Enum):
    """
    The enumeration of item types.
    物品枚举类。
    """
    UNKNOWN = 0
    CLOTHING = 1
    DEPRECATED_DIRT_BLOCK = 2
    FLINT = 3
    STICK = 4
    DEPRECATED_WOOD_BLOCK = 5
    FLINT_AXE = 6
    FIINT_SPEAR = 7
    FLINT_PICKAXE = 8