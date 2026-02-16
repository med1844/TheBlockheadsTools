from enum import Enum
from typing import Optional, Self, Iterator
from pathlib import Path


class PigmentColor(Enum):
    Transparent = 0
    MarbleWhite = 1
    CarbonBlack = 2
    RedOchre = 3
    IndianYellow = 4
    UltraMarineBlue = 5
    EmeraldGreen = 6
    TyrianPurple = 7
    CopperBlue = 8

    def __int__(self) -> int: ...


class ItemType(Enum):
    Unknown = 0
    Clothing = 1
    DeprecatedDirtBlock = 2
    Flint = 3
    Stick = 4
    DeprecatedWoodBlock = 5
    FlintAxe = 6
    FlintSpear = 7
    FlintPickaxe = 8
    DoubleTime = 9
    DeprecatedWorkbench = 10
    TimeCrystal = 11
    Basket = 12
    Ember = 13
    Charcoal = 14
    Campfire = 15
    FlintSpade = 16
    Torch = 17
    DeprecatedSand = 18
    Blockhead = 19
    Food = 20
    Apple = 21
    Mango = 22
    MapleSeed = 23
    PricklyPear = 24
    FlintMachete = 25
    DeprecatedStoneWorkbench = 26
    Pinecone = 27
    Clay = 28
    DodoMeat = 29
    DodoFeather = 30
    CopperOre = 31
    IronOre = 32
    StoneAxe = 33
    StonePickaxe = 34
    CopperIngot = 35
    TinOre = 36
    TinIngot = 37
    BronzeIngot = 38
    CopperSpear = 39
    TinSpade = 40
    CopperArrow = 41
    CopperBowAndArrows = 42
    BronzePickaxe = 43
    String = 44
    ClayJug = 45
    Coconut = 46
    OilLantern = 47
    Oil = 48
    BronzeMachete = 49
    BronzeSword = 50
    Coal = 51
    Door = 52
    Ladder = 53
    FlaxSeed = 54
    Flax = 55
    IndianYellow = 56
    RedOchre = 57
    Window = 58
    CookedDodoMeat = 59
    Orange = 60
    SunflowerSeed = 61
    Corn = 62
    Bed = 63
    StoneSpade = 64
    IronIngot = 65
    IronPickaxe = 66
    IronMachete = 67
    IronSword = 68
    Trapdoor = 69
    IronAxe = 70
    Carrot = 71
    GoldIngot = 72
    GoldNugget = 73
    CarrotOnAStick = 74
    Ruby = 75
    Emerald = 76
    Cherry = 77
    CoffeeCherry = 78
    GreenCoffeeBean = 79
    Cup = 80
    Coffee = 81
    RoastedCoffeeBean = 82
    Linen = 83
    LinenPants = 84
    LinenShirt = 85
    Sapphire = 86
    Amethyst = 87
    Diamond = 88
    GoldSpade = 89
    GoldPickaxe = 90
    DodoEgg = 91
    SteelIngot = 92
    SteelPickaxe = 93
    AmethystPickaxe = 94
    SapphirePickaxe = 95
    EmeraldPickaxe = 96
    RubyPickaxe = 97
    DiamondPickaxe = 98
    UltramarineBlue = 99
    CarbonBlack = 100
    MarbleWhite = 101
    TinBucket = 102
    Paint = 103
    PaintStripper = 104
    BucketOfWater = 105
    Pigment = 106
    RainbowPaintCap = 107
    InvalidPigment = 108
    EmeraldGreen = 109
    TyrianPurple = 110
    Boat = 111
    Chilli = 112
    RainbowLinenPants = 113
    RainbowShirt = 114
    LinenCap = 115
    RainbowCap = 116
    LinenBrimmedHat = 117
    RainbowBrimmedHat = 118
    CopperBlue = 119
    Leather = 120
    Fur = 121
    LeatherJacket = 122
    RainbowJacket = 123
    LeatherBoots = 124
    RainbowLeatherBoots = 125
    FurCoat = 126
    FurBoots = 127
    RainbowCoat = 128
    RainbowFurBoots = 129
    LeatherPants = 130
    RainbowLeatherPants = 131
    Upgrade = 132
    Camera = 133
    Portal = 134
    AmethystPortal = 135
    SapphirePortal = 136
    EmeraldPortal = 137
    RubyPortal = 138
    DiamondPortal = 139
    SunriseHatOfFullness = 140
    SunsetSkirtOfHappiness = 141
    NorthPoleHatOfWarmth = 142
    SouthPoleBootsOfSpeed = 143
    Kelp = 144
    AmethystChandelier = 145
    SapphireChandelier = 146
    EmeraldChandelier = 147
    RubyChandelier = 148
    DiamondChandelier = 149
    SteelLantern = 150
    RawFish = 151
    CookedFish = 152
    TinFoil = 153
    TinFoilHat = 154
    Worm = 155
    FishingRod = 156
    SharkJaw = 157
    FishBucket = 158
    SharkBucket = 159
    Lime = 160
    Shelf = 161
    TeleportHere = 162
    Sign = 163
    IronDoor = 164
    IronTrapdoor = 165
    CopperCoin = 166
    GoldCoin = 167
    Shop = 168
    SoftBed = 169
    GoldenBed = 170
    BedBlanket = 171
    RainbowSoftBed = 172
    RainbowGoldenBed = 173
    BlackWindow = 174
    Magnet = 175
    CopperBoiler = 176
    ElectronicMotor = 177
    CopperWire = 178
    SteamEngine = 179
    IronPot = 180
    FishCurry = 181
    DodoStew = 182
    IceTorch = 183
    SiliconIngot = 184
    SiliconCrystal = 185
    SiliconWafer = 186
    TinArmorLeggings = 187
    TinChestPlate = 188
    TinHelmet = 189
    TinBoots = 190
    IronArmorLeggings = 191
    IronChestPlate = 192
    IronHelmet = 193
    IronBoots = 194
    IceArmorLeggings = 195
    IceChestPlate = 196
    IceHelmet = 197
    IceBoots = 198
    Rail = 199
    TrainStation = 200
    PigIron = 201
    CrushedLimestone = 202
    TrainWheel = 203
    RailHandcar = 204
    SteamLocomotive = 205
    FreightCar = 206
    DisplayCabinet = 207
    PassengerCar = 208
    Crowbar = 209
    TradePortal = 210
    DeprecatedGoldChest = 211
    LargeSquarePainting = 212
    LargeLandscapePainting = 213
    LargePortraitPainting = 214
    MedSquarePainting = 215
    MedLandscapePainting = 216
    MedPortraitPainting = 217
    SmallSquarePainting = 218
    SmallLandscapePainting = 219
    SmallPortraitPainting = 220
    Easel = 221
    StoneColumn = 222
    LimestoneColumn = 223
    MarbleColumn = 224
    SandstoneColumn = 225
    RedMarbleColumn = 226
    LapisLazuliColumn = 227
    BasaltColumn = 228
    StoneStairs = 229
    LimestoneStairs = 230
    MarbleStairs = 231
    SandstoneStairs = 232
    RedMarbleStairs = 233
    LapisLazuliStairs = 234
    BasaltStairs = 235
    CopperColumn = 236
    TinColumn = 237
    BronzeColumn = 238
    IronColumn = 239
    SteelColumn = 240
    GoldColumn = 241
    WoodColumn = 242
    BrickColumn = 243
    IceColumn = 244
    CopperStairs = 245
    TinStairs = 246
    BronzeStairs = 247
    IronStairs = 248
    SteelStairs = 249
    GoldStairs = 250
    WoodStairs = 251
    BrickStairs = 252
    IceStairs = 253
    SteelDownlight = 254
    Poison = 255
    PoisonArrow = 256
    GoldBowAndPoisonArrows = 257
    SteelUplight = 258
    WorldCredit = 259
    PlatiumCoin = 260
    PlatiumNugget = 261
    PlatiumIngot = 262
    PlatiumStairs = 263
    PlatiumColumn = 264
    GlassStairs = 265
    GlassColumn = 266
    BlackGlassStairs = 267
    BlackGlassColumn = 268
    Fuel = 269
    Refinery = 270
    Epoxy = 271
    RawResin = 272
    CarbonFibers = 273
    CarbonFiberSheet = 274
    CarbonFiberWing = 275
    JetpackChassis = 276
    JetEngine = 277
    Jetpack = 278
    TitaniumOre = 279
    TitaniumIngot = 280
    TitaniumStairs = 281
    TitaniumColumn = 282
    CarbonFiberStairs = 283
    CarbonFiberColumn = 284
    TitaniumPickaxe = 285
    TitaniumSword = 286
    TitaniumLeggings = 287
    TitaniumChestPlate = 288
    TitaniumHelmet = 289
    TitaniumBoots = 290
    CarbonFiberLeggings = 291
    CarbonFiberChestPlate = 292
    CarbonFiberHelmet = 293
    CarbonFiberBoots = 294
    Vine = 295
    TulipBulb = 296
    TulipSeed = 297
    Coins = 298
    RandomOre = 299
    ElectricSluice = 300
    OwnershipSign = 301
    Cage = 302
    CagedDodo = 303
    WoodenGate = 304
    AmethystShard = 305
    SapphireShard = 306
    EmeraldShard = 307
    RubyShard = 308
    DiamondShard = 309
    Wheat = 310
    Flour = 311
    Yeast = 312
    Salt = 313
    Dough = 314
    Bread = 315
    Tomato = 316
    Pizza = 317
    Flatbread = 318
    Milk = 319
    Mozzarella = 320
    YakHorn = 321
    Razor = 322
    YakShavings = 323
    CagedDonkey = 324
    CagedYak = 325
    CagedDropbear = 326
    CagedScorpion = 327
    RainbowCake = 328
    RainbowEssence = 329
    CagedUnicorn = 330
    Mirror = 331
    PlasterColumn = 332
    PlasterStairs = 333
    AmethystColumn = 334
    SapphireColumn = 335
    EmeraldColumn = 336
    RubyColumn = 337
    DiamondColumn = 338
    AmethystStairs = 339
    SapphireStairs = 340
    EmeraldStairs = 341
    RubyStairs = 342
    DiamondStairs = 343

    Stone = 1024
    Kiln = 1025
    Brick = 1026
    Limestone = 1027
    MinedLimestone = 1028
    Marble = 1029
    MinedMarble = 1030
    Furnace = 1031
    WoodworkBench = 1032
    TaylorsBench = 1033
    Press = 1034
    Sandstone = 1035
    MinedSandstone = 1036
    RedMarble = 1037
    MinedRedMarble = 1038
    WovenFlaxMat = 1039
    YellowFlaxMat = 1040
    RedFlaxMat = 1041
    Glass = 1042
    Chest = 1043
    DeprecatedFood = 1044
    GoldBlock = 1045
    DeprecatedMango = 1046
    Rock = 1047
    Dirt = 1048
    Wood = 1049
    WorkBench = 1050
    Sand = 1051
    ToolBench = 1052
    LapisLazuli = 1053
    MinedLapisLazuli = 1054
    CraftBench = 1055
    MixingBench = 1056
    ReinforcedPlatform = 1057
    DeprecatedStonePickaxe = 1058
    DeprecatedCopperIngot = 1059
    Ice = 1060
    DyeBench = 1061
    Compost = 1062
    Basalt = 1063
    MinedBasalt = 1064
    Safe = 1065
    CopperBlock = 1066
    TinBlock = 1067
    BronzeBlock = 1068
    IronBlock = 1069
    SteelBlock = 1070
    MetalworkBench = 1071
    GoldenChest = 1072
    DeprecatedBronzeMachete = 1073
    PortalChest = 1074
    BlackSand = 1075
    BlackGlass = 1076
    SteamGenerator = 1077
    ElectricKiln = 1078
    ElectricFurnace = 1079
    ElectricMetalworkBench = 1080
    ElectricStove = 1081
    SolarPanel = 1082
    Flywheel = 1083
    ArmorBench = 1084
    TrainYard = 1085
    BuildersBench = 1086
    ElevatorShaft = 1087
    ElectricElevatorMotor = 1088
    PlatiumBlock = 1089
    CarbonFiberBlock = 1090
    TitaniumBlock = 1091
    DeprecatedIronSword = 1092
    ElectricPress = 1093
    Gravel = 1094
    CompostBin = 1095
    EggExtractor = 1096
    PizzaOven = 1097
    AmethystBlock = 1098
    SapphireBlock = 1099
    EmeraldBlock = 1100
    RubyBlock = 1101
    DiamondBlock = 1102
    Plaster = 1103
    FeederChest = 1104
    LuminousPlaster = 1105

    def __int__(self) -> int: ...


class ChestType(Enum):
    Standard = 0
    Safe = 1
    Shelf = 2
    Gold = 3
    Portal = 4
    DisplayCabinet = 5
    Feeder = 6

    def __int__(self) -> int: ...


class WorkbenchType(Enum):
    Undefined = 0
    BasicPortal = 1
    Workbench = 2
    Campfire = 3
    Weave = 4
    Wood = 5
    Tool = 6
    Press = 7
    Kiln = 8
    Furnace = 9
    Craft = 10
    Mix = 11
    Dye = 12
    PlacedPortal = 13
    Metalwork = 14
    SteamGenerator = 15
    ElectricKiln = 16
    ElectricFurnace = 17
    ElectricMetalworkBench = 18
    ElectricStove = 19
    SolarPanel = 20
    Flywheel = 21
    ArmorBench = 22
    TrainYard = 23
    Easel = 24
    Build = 25
    Refinery = 26
    ElectricPress = 27
    CompostBin = 28
    Sluice = 29
    EggExtractor = 30
    PizzaOven = 31

    def __int__(self) -> int: ...


class BasketExtra:
    def __init__(self, items: Optional[list[Slot]] = None) -> None: ...
    def __len__(self) -> int: ...
    def __getitem__(self, index: int) -> Slot: ...
    def __setitem__(self, index: int, value: Slot) -> None: ...
    def __repr__(self) -> str: ...


class ChestExtra:
    chest_type: ChestType
    owner_id: str
    is_in_use: bool
    flipped: bool
    paint_color: int
    pos_x: int
    pos_y: int
    float_pos: list[float]
    unique_id: int

    def __init__(
        self,
        items: Optional[list[Slot]] = None,
        chest_type: ChestType = ChestType.Standard,
        owner_id: str = "server",
    ) -> None: ...
    def __len__(self) -> int: ...
    def __getitem__(self, index: int) -> Slot: ...
    def __setitem__(self, index: int, value: Slot) -> None: ...
    def __repr__(self) -> str: ...


class WorkbenchExtra:
    workbench_type: WorkbenchType
    level: int
    owner_id: str
    is_in_use: bool
    flipped: bool
    paint_color: int
    pos_x: int
    pos_y: int
    float_pos: list[float]
    unique_id: int
    available_electricity: int
    craft_progress_count: float
    fire_spread_timer: float
    fuel_fraction: float
    has_fuel: bool
    hurry_cost: int
    hurry_seconds: float
    hurry_timer: float
    hurrying: bool
    last_world_time: float
    save_time: float
    selected_index: int
    x_scroll: float

    def __init__(
        self,
        workbench_type: WorkbenchType = WorkbenchType.Workbench,
        level: int = 1,
        owner_id: str = "server",
    ) -> None: ...
    def __repr__(self) -> str: ...


class Item:
    type_id: int
    data_a: int
    data_b: int
    selected_sub_item_index: int
    padding: int

    def __init__(
        self,
        item_type: ItemType,
        extra: Optional[BasketExtra | ChestExtra | WorkbenchExtra] = None,
    ) -> None: ...
    @property
    def item_type(self) -> ItemType: ...
    @item_type.setter
    def item_type(self, value: ItemType) -> None: ...
    @property
    def damage(self) -> int: ...
    @damage.setter
    def damage(self, value: int) -> None: ...
    @property
    def colors(self) -> list[PigmentColor]: ...
    @colors.setter
    def colors(self, value: list[PigmentColor]) -> None: ...
    @property
    def extra(self) -> Optional[BasketExtra | ChestExtra | WorkbenchExtra]: ...
    @extra.setter
    def extra(
        self, value: Optional[BasketExtra | ChestExtra | WorkbenchExtra]
    ) -> None: ...
    def __repr__(self) -> str: ...


class Slot:
    items: list[Item]

    def __init__(self, items: Optional[list[Item]] = None) -> None: ...
    def __len__(self) -> int: ...
    def __getitem__(self, index: int) -> Item: ...
    def __setitem__(self, index: int, value: Item) -> None: ...
    def __delitem__(self, index: int) -> None: ...
    def append(self, item: Item) -> None: ...
    def extend(self, items: list[Item]) -> None: ...
    def insert(self, index: int, item: Item) -> None: ...
    def pop(self, index: Optional[int] = -1) -> Item: ...
    def clear(self) -> None: ...
    def __iter__(self) -> Iterator[Item]: ...
    def __repr__(self) -> str: ...


class Inventory:
    def __init__(self, slots: Optional[list[Slot]] = None) -> None: ...
    def __len__(self) -> int: ...
    def __getitem__(self, index: int) -> Slot: ...
    def __setitem__(self, index: int, value: Slot) -> None: ...
    def __repr__(self) -> str: ...


class ChunkCoord:
    """0 <= x < 2^32 - 1, 0 <= y < 32"""

    def __new__(cls, x: int, y: int) -> Self: ...
    def __init__(self, x: int, y: int) -> None: ...
    @property
    def x(self) -> int: ...
    @property
    def y(self) -> int: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __hash__(self) -> int: ...


class ChunkBlockCoord:
    """0 <= x < 32, 0 <= y < 32"""

    def __new__(cls, x: int, y: int) -> Self: ...
    def __init__(self, x: int, y: int) -> None: ...
    @property
    def x(self) -> int: ...
    @property
    def y(self) -> int: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __hash__(self) -> int: ...


class BlockCoord:
    """0 <= x < 2^32 - 1, 0 <= y < 1024"""

    def __new__(cls, x: int, y: int) -> Self: ...
    def __init__(self, x: int, y: int) -> None: ...
    @property
    def x(self) -> int: ...
    @property
    def y(self) -> int: ...
    def decompose(self) -> tuple[ChunkCoord, ChunkBlockCoord]: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __hash__(self) -> int: ...


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

    def __int__(self) -> int: ...


class BlockContentType(Enum):
    Nothing = 0
    Flint = 1
    Clay = 2
    AppleTreeLeaf = 3
    AppleTreeTrunk = 4
    AppleTreeTrunkLeaf = 5
    PineTreeLeaf = 6
    PineTreeTrunk = 7
    PineTreeTrunkLeaf = 8
    MapleTreeLeaf = 9
    MapleTreeTrunk = 10
    MapleTreeTrunkLeaf = 11
    MangoTreeLeaf = 12
    MangoTreeTrunk = 13
    MangoTreeTrunkLeaf = 14
    CoconutTreeLeaf = 15
    CoconutTreeTrunk = 16
    OrangeTreeLeaf = 18
    OrangeTreeTrunk = 19
    OrangeTreeTrunkLeaf = 20
    CherryTreeLeaf = 21
    CherryTreeTrunk = 22
    CherryTreeTrunkLeaf = 23
    CoffeeTreeLeaf = 24
    CoffeeTreeTrunk = 25
    CoffeeTreeTrunkLeaf = 26
    DeadPineTreeTrunk = 29
    DeadPineTreeLeaf = 34
    DeadOrangeTreeLeaf = 37
    DeadOrangeTreeTrunk = 38
    DeadCherryTreeLeaf = 39
    DeadCherryTreeTrunk = 40
    Cactus = 43
    DeadCactus = 44
    Workbench = 46
    WorkbenchSprite = 47
    CopperOre = 61
    TinOre = 62
    IronOre = 63
    Oil = 64
    Coal = 65
    GoldNuggets = 77
    LimeTreeLeaf = 89
    LimeTreeTrunk = 90
    LimeTreeTrunkLeaf = 91
    DeadLimeTreeLeaf = 92
    DeadLimeTreeTrunk = 93
    GoldChest = 94
    PlatinumOre = 106
    TitaniumOre = 107
    AmethystTreeTrunk = 109
    AmethystTreeLeaf = 110
    AmethystTreeTrunkLeaf = 111
    SapphireTreeTrunk = 112
    SapphireTreeLeaf = 113
    SapphireTreeTrunkLeaf = 114
    EmeraldTreeTrunk = 115
    EmeraldTreeLeaf = 116
    EmeraldTreeTrunkLeaf = 117
    RubyTreeTrunk = 118
    RubyTreeLeaf = 119
    RubyTreeTrunkLeaf = 120
    DiamondTreeTrunk = 121
    DiamondTreeLeaf = 122
    DiamondTreeTrunkLeaf = 123

    def __int__(self) -> int: ...


class Block:
    def fg(self) -> BlockType: ...
    def set_fg(self, block_type: BlockType): ...
    def bg(self) -> BlockType: ...
    def set_bg(self, block_type: BlockType): ...
    def content(self) -> BlockContentType: ...
    def set_content(self, block_content_type: BlockContentType): ...
    def height(self) -> int: ...
    def set_height(self, height: int): ...
    def damage(self) -> int: ...
    def set_damage(self, damage: int): ...
    def visibility(self) -> int: ...
    def set_visibility(self, visibility: int): ...
    def brightness(self) -> int: ...
    def set_brightness(self, brightness: int): ...


class Chunk:
    WIDTH: int = 32
    HEIGHT: int = 32

    def as_bytes(self) -> bytes: ...
    def block_at(self, coord: ChunkBlockCoord) -> Block: ...


class Chunks:
    def __contains__(self, coord: ChunkCoord) -> bool: ...
    def keys(self) -> set[ChunkCoord]: ...
    def chunk_at(self, coord: BlockCoord | ChunkCoord) -> Optional[Chunk]: ...
    def set_chunk_at(self, coord: BlockCoord | ChunkCoord, chunk: Chunk): ...


class WorldV2:
    @property
    def blockhead_datas_v2(self) -> str: ...

    @property
    def circum_navigate_booleans_data(self) -> bytes: ...
    @circum_navigate_booleans_data.setter
    def circum_navigate_booleans_data(self, value: bytes): ...

    @property
    def creation_date(self) -> str: ...
    @property
    def distance_ordered_food_types(self) -> bytes: ...
    @distance_ordered_food_types.setter
    def distance_ordered_food_types(self, value: bytes): ...

    @property
    def expert_mode(self) -> bool: ...
    @expert_mode.setter
    def expert_mode(self, value: bool): ...

    @property
    def found_items(self) -> bytes: ...
    @found_items.setter
    def found_items(self, value: bytes): ...

    @property
    def host_port(self) -> str: ...
    @host_port.setter
    def host_port(self, value: str): ...

    @property
    def max_players(self) -> str: ...
    @max_players.setter
    def max_players(self, value: str): ...

    @property
    def migration_complete_v1_7(self) -> bool: ...
    @migration_complete_v1_7.setter
    def migration_complete_v1_7(self, value: bool): ...

    @property
    def no_rain_timer(self) -> float: ...
    @no_rain_timer.setter
    def no_rain_timer(self, value: float): ...

    @property
    def portal_level(self) -> int: ...
    @portal_level.setter
    def portal_level(self, value: int): ...

    @property
    def random_seed(self) -> int: ...
    @random_seed.setter
    def random_seed(self, value: int): ...

    @property
    def remote_game(self) -> bool: ...
    @remote_game.setter
    def remote_game(self, value: bool): ...

    @property
    def run_at_launch(self) -> bool: ...
    @run_at_launch.setter
    def run_at_launch(self, value: bool): ...

    @property
    def save_date(self) -> str: ...

    @property
    def save_id(self) -> str: ...
    @save_id.setter
    def save_id(self, value: str): ...

    @property
    def save_version(self) -> int: ...
    @save_version.setter
    def save_version(self, value: int): ...

    @property
    def start_portal_pos_x(self) -> int: ...
    @start_portal_pos_x.setter
    def start_portal_pos_x(self, value: int): ...

    @property
    def start_portal_pos_y(self) -> int: ...
    @start_portal_pos_y.setter
    def start_portal_pos_y(self, value: int): ...

    @property
    def translation(self) -> tuple[float, float]: ...
    @translation.setter
    def translation(self, value: tuple[float, float]): ...

    @property
    def world_name(self) -> str: ...
    @world_name.setter
    def world_name(self, value: str): ...

    @property
    def world_time(self) -> float: ...
    @world_time.setter
    def world_time(self, value: float): ...

    @property
    def world_width_macro(self) -> int: ...
    @world_width_macro.setter
    def world_width_macro(self, value: int): ...


class DynamicWorldV2:
    @property
    def active_blockhead_index(self) -> int: ...
    @active_blockhead_index.setter
    def active_blockhead_index(self, value: int): ...

    @property
    def dynamic_object_id_count(self) -> int: ...
    @dynamic_object_id_count.setter
    def dynamic_object_id_count(self, value: int): ...

    @property
    def save_version(self) -> int: ...
    @save_version.setter
    def save_version(self, value: int): ...

    @property
    def saved_glow_indices(self) -> bytes: ...
    @saved_glow_indices.setter
    def saved_glow_indices(self, value: bytes): ...

    @property
    def workbench_has_been_crafted(self) -> bool: ...
    @workbench_has_been_crafted.setter
    def workbench_has_been_crafted(self, value: bool): ...

    def __repr__(self) -> str: ...


class Blockhead:
    @property
    def name(self) -> str: ...
    @name.setter
    def name(self, value: str): ...

    @property
    def clothing_increment_timer(self) -> int: ...
    @clothing_increment_timer.setter
    def clothing_increment_timer(self, value: int): ...

    @property
    def double_time_unlocked(self) -> bool: ...
    @double_time_unlocked.setter
    def double_time_unlocked(self, value: bool): ...

    @property
    def skin_options(self) -> bytes: ...
    @skin_options.setter
    def skin_options(self, value: bytes): ...

    @property
    def state(self) -> bytes: ...
    @state.setter
    def state(self, value: bytes): ...

    @property
    def inventory(self) -> Optional[Inventory]: ...
    @inventory.setter
    def inventory(self, value: Optional[Inventory]) -> None: ...

    def __repr__(self) -> str: ...


class WorldDbMain:
    @property
    def blockheads(self) -> list[Blockhead]: ...
    @property
    def blockhead_inventory_keys(self) -> set[int]: ...
    @property
    def dynamic_world_v2(self) -> DynamicWorldV2: ...
    @property
    def world_v2(self) -> WorldV2: ...
    def get_blockhead_inventory(self, id: int) -> Optional[Inventory]: ...
    def set_blockhead_inventory(
        self, id: int, inventory: Optional[Inventory]
    ) -> None: ...


class Arch(Enum):
    Arch32 = 0
    Arch64 = 1

    def __int__(self) -> int: ...


class WorldDb:
    @classmethod
    def open_path(cls, path: str | Path) -> Self: ...
    def save_path(self, path: str | Path, arch: Arch): ...
    @classmethod
    def open_bytes(cls, data: bytes | bytearray) -> Self: ...
    def save_bytes(self, arch: Arch) -> bytes: ...

    @property
    def chunks(self) -> Chunks: ...

    @property
    def main(self) -> WorldDbMain: ...
