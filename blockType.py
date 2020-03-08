# encoding: utf-8
from enum import Enum


class BlockType(Enum):
    """
    The enumeration of block types. Records the ID of each type of block.

    方块类型枚举类。保存了每种方块的ID。
    """
    STONE = 1
    AIR = 2
    UNKNOWN = 3
    ICE = 4
    SNOW = 5
    DIRT = 6
    SAND = 7
    SAND_ = 8  # guess this is mined sand?
    WOOD = 9
    MINED_STONE = 10
    RED_BRICK = 11
    LIMESTONE = 12
    MINED_LIMESTONE = 13
    MARBLE = 14
    MINED_MARBLE = 15
    TIME_CRYSTAL = 16
    SAND_STONE = 17
    MINED_SAND_STONE = 18
    RED_MARBLE = 19
    MINED_RED_MARBLE = 20
    GLASS = 24
    SPAWN_PORTAL_BASE = 25
    GOLD_BLOCK = 26
    GRASS_DIRT = 27
    SNOW_DIRT = 28
    LAPIS_LAZULI = 29
    MINED_LAPIS_LAZULI = 30
    LAVA = 31
    REINFORCED_PLATFORM = 32
    SPAWN_PORTAL_BASE_AMETHYST = 33
    SPAWN_PORTAL_BASE_SAPPHIRE = 34
    SPAWN_PORTAL_BASE_EMERALD = 35
    SPAWN_PORTAL_BASE_RUBY = 36
    SPAWN_PORTAL_BASE_DIAMOND = 37
    NORTH_POLE = 38
    SOUTH_POLE = 39
    WEST_POLE = 40
    EAST_POLE = 41
    PORTAL_BASE = 42
    PORTAL_BASE_AMETHYST = 43
    PORTAL_BASE_SAPPHIRE = 44
    PORTAL_BASE_EMERALD = 45
    PORTAL_BASE_RUBY = 46
    PORTAL_BASE_DIAMOND = 47
    COMPOST = 48
    GRASS_COMPOST = 49
    SNOW_COMPOST = 50
    BASALT = 51
    MINED_BASALT = 52
    COPPER_BLOCK = 53
    TIN_BLOCK = 54
    BRONZE_BLOCK = 55
    IRON_BLOCK = 56
    STEEL_BLOCK = 57
    BLACK_SAND = 58
    BLACK_GLASS = 59
    TRADE_PORTAL_BASE = 60
    TRADE_PORTAL_BASE_AMETHYST = 61
    TRADE_PORTAL_BASE_SAPPHIRE = 62
    TRADE_PORTAL_BASE_EMERALD = 63
    TRADE_PORTAL_BASE_RUBY = 64
    TRADE_PORTAL_BASE_DIAMOND = 65
    PLATINUM_BLOCK = 67
    TITANIUM_BLOCK = 68
    CARBON_FIBER_BLOCK = 69
    GRAVEL = 70
    AMETHYST_BLOCK = 71
    SAPPHIRE_BLOCK = 72
    EMERALD_BLOCK = 73
    RUBY_BLOCK = 74
    DIAMOND_BLOCK = 75
    PLASTER = 76
    LUMINOUS_PLASTER = 77


class SubType(Enum):
    COPPER_ORE = 61
    TIN_ORE = 62
    IRON_ORE = 63
    COAL = 65
    GOLD_NUGGETS = 77
    PLATIUM_ORE = 106
    TITANIUM_ORE = 107


def id_to_block_type(block_id):
    if not isinstance(block_id, int):
        block_id = ord(block_id)
    return BlockType._value2member_map_[block_id]


def id_to_block_name(block_id):
    if not isinstance(block_id, int):
        block_id = ord(block_id)
    return BlockType._value2member_map_[block_id].name


def block_name_to_id(block_name):
    assert isinstance(block_name, str)
    return BlockType._member_map_[block_name].value


if __name__ == "__main__":
    print(id_to_block_name(57))