from enum import Enum
from typing import Self, Optional


class ChunkCoord:
    """0 <= x < 2^32 - 1, 0 <= y < 32"""

    def __new__(cls, x: int, y: int) -> Self: ...
    def __init__(self, x: int, y: int) -> None: ...
    @property
    def x(self) -> int: ...
    @property
    def y(self) -> int: ...
    def __str__(self) -> str: ...


class ChunkBlockCoord:
    """0 <= x < 32, 0 <= y < 32"""

    def __new__(cls, x: int, y: int) -> Self: ...
    def __init__(self, x: int, y: int) -> None: ...
    @property
    def x(self) -> int: ...
    @property
    def y(self) -> int: ...
    def __str__(self) -> str: ...


class BlockCoord:
    """0 <= x < 2^32 - 1, 0 <= y < 1024"""

    def __new__(cls, x: int, y: int) -> Self: ...
    def __init__(self, x: int, y: int) -> None: ...
    @property
    def x(self) -> int: ...
    @property
    def y(self) -> int: ...
    def __str__(self) -> str: ...


class BlockType(Enum):
    Stone = 1
    Air = 2
    Water = 3
    Ice = 4
    Snow = 5
    Dirt = 6
    DesertSand = 7
    BeachSand = 8
    Wood = 9
    MinedStone = 10
    RedBrick = 11
    Limestone = 12
    MinedLimestone = 13
    Marble = 14
    MinedMarble = 15
    TimeCrystal = 16
    SandStone = 17
    MinedSandStone = 18
    RedMarble = 19
    MinedRedMarble = 20
    Glass = 24
    SpawnPortalBase = 25
    GoldBlock = 26
    GrassDirt = 27
    SnowDirt = 28
    LapisLazuli = 29
    MinedLapisLazuli = 30
    Lava = 31
    ReinforcedPlatform = 32
    SpawnPortalBaseAmethyst = 33
    SpawnPortalBaseSapphire = 34
    SpawnPortalBaseEmerald = 35
    SpawnPortalBaseRuby = 36
    SpawnPortalBaseDiamond = 37
    NorthPole = 38
    SouthPole = 39
    WestPole = 40
    EastPole = 41
    PortalBase = 42
    PortalBaseAmethyst = 43
    PortalBaseSapphire = 44
    PortalBaseEmerald = 45
    PortalBaseRuby = 46
    PortalBaseDiamond = 47
    Compost = 48
    GrassCompost = 49
    SnowCompost = 50
    Basalt = 51
    MinedBasalt = 52
    CopperBlock = 53
    TinBlock = 54
    BronzeBlock = 55
    IronBlock = 56
    SteelBlock = 57
    BlackSand = 58
    BlackGlass = 59
    TradePortalBase = 60
    TradePortalBaseAmethyst = 61
    TradePortalBaseSapphire = 62
    TradePortalBaseEmerald = 63
    TradePortalBaseRuby = 64
    TradePortalBaseDiamond = 65
    PlatinumBlock = 67
    TitaniumBlock = 68
    CarbonFiberBlock = 69
    Gravel = 70
    AmethystBlock = 71
    SapphireBlock = 72
    EmeraldBlock = 73
    RubyBlock = 74
    DiamondBlock = 75
    Plaster = 76
    LuminousPlaster = 77


class Block:
    def fg(self) -> BlockTypePy: ...


class Chunk:
    def as_bytes(self) -> bytes: ...
    def block_at(self, coord: BlockCoord | ChunkBlockCoord) -> Block: ...


class Chunks:
    def __contains__(self, coord: ChunkCoord) -> bool: ...
    def keys(self) -> set[ChunkCoord]: ...
    def chunk_at(self, coord: BlockCoord | ChunkCoord) -> Optional[Chunk]: ...


class WorldDb:
    @staticmethod
    def open(path: str) -> Self: ...
    def save(path: str): ...

    @property
    def chunks(self) -> Chunks: ...
