use crate::image_type::ImageType;
use num_enum::TryFromPrimitive;
use the_blockheads_tools_lib::game::block::{
    Block, BlockContentType, BlockError, BlockType, BlockView,
};

// Basically the same as BlockType, but treats block with tile content as separate type
#[repr(u16)]
#[derive(
    Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::NoUninit, PartialEq, Eq, TryFromPrimitive,
)]
pub enum VoxelType {
    Unknown = 0,
    Stone = 1,
    Air = 2,
    Water = 3,
    Ice = 4,
    Snow = 5,
    Dirt = 6,
    Sand = 7,
    MinedSand = 8,
    Wood = 9,
    MinedStone = 10,
    RedBrick = 11,
    Limestone = 12,
    MinedLimestone = 13,
    Marble = 14,
    MinedMarble = 15,
    TimeCrystal = 16,
    SandStone = 17,
    MinedSandStone = 18,
    RedMarble = 19,
    MinedRedMarble = 20,
    Glass = 24,
    SpawnPortalBase = 25,
    GoldBlock = 26,
    GrassDirt = 27,
    SnowDirt = 28,
    LapisLazuli = 29,
    MinedLapisLazuli = 30,
    Lava = 31,
    ReinforcedPlatform = 32,
    SpawnPortalBaseAmethyst = 33,
    SpawnPortalBaseSapphire = 34,
    SpawnPortalBaseEmerald = 35,
    SpawnPortalBaseRuby = 36,
    SpawnPortalBaseDiamond = 37,
    NorthPole = 38,
    SouthPole = 39,
    WestPole = 40,
    EastPole = 41,
    PortalBase = 42,
    PortalBaseAmethyst = 43,
    PortalBaseSapphire = 44,
    PortalBaseEmerald = 45,
    PortalBaseRuby = 46,
    PortalBaseDiamond = 47,
    Compost = 48,
    GrassCompost = 49,
    SnowCompost = 50,
    Basalt = 51,
    MinedBasalt = 52,
    CopperBlock = 53,
    TinBlock = 54,
    BronzeBlock = 55,
    IronBlock = 56,
    SteelBlock = 57,
    BlackSand = 58,
    BlackGlass = 59,
    TradePortalBase = 60,
    TradePortalBaseAmethyst = 61,
    TradePortalBaseSapphire = 62,
    TradePortalBaseEmerald = 63,
    TradePortalBaseRuby = 64,
    TradePortalBaseDiamond = 65,
    PlatinumBlock = 67,
    TitaniumBlock = 68,
    CarbonFiberBlock = 69,
    Gravel = 70,
    AmethystBlock = 71,
    SapphireBlock = 72,
    EmeraldBlock = 73,
    RubyBlock = 74,
    DiamondBlock = 75,
    Plaster = 76,
    LuminousPlaster = 77,
    DirtClay = 78,
    DirtFlint = 79,
    GrassDirtClay = 80,
    GrassDirtFlint = 81,
    SnowDirtClay = 82,
    SnowDirtFlint = 83,
    StoneCopperOre = 84,
    StoneTinOre = 85,
    StoneIronOre = 86,
    StoneCoal = 87,
    StoneGoldNuggets = 88,
    StonePlatinumOre = 89,
    StoneTitaniumOre = 90,
    LimestoneOil = 91,
    AppleTreeLeaf = 92,
    AppleTreeTrunk = 93,
    AppleTreeTrunkLeaf = 94,
    PineTreeLeaf = 95,
    PineTreeTrunk = 96,
    PineTreeTrunkLeaf = 97,
    MapleTreeLeaf = 98,
    MapleTreeTrunk = 99,
    MapleTreeTrunkLeaf = 100,
    MangoTreeLeaf = 101,
    MangoTreeTrunk = 102,
    MangoTreeTrunkLeaf = 103,
    CoconutTreeLeaf = 104,
    CoconutTreeTrunk = 105,
    OrangeTreeLeaf = 106,
    OrangeTreeTrunk = 107,
    OrangeTreeTrunkLeaf = 108,
    CherryTreeLeaf = 109,
    CherryTreeTrunk = 110,
    CherryTreeTrunkLeaf = 111,
    CoffeeTreeLeaf = 112,
    CoffeeTreeTrunk = 113,
    CoffeeTreeTrunkLeaf = 114,
    Cactus = 115,
    DeadCactus = 116,
    LimeTreeLeaf = 117,
    LimeTreeTrunk = 118,
    LimeTreeTrunkLeaf = 119,
    AmethystTreeTrunk = 120,
    AmethystTreeLeaf = 121,
    AmethystTreeTrunkLeaf = 122,
    SapphireTreeTrunk = 123,
    SapphireTreeLeaf = 124,
    SapphireTreeTrunkLeaf = 125,
    EmeraldTreeTrunk = 126,
    EmeraldTreeLeaf = 127,
    EmeraldTreeTrunkLeaf = 128,
    RubyTreeTrunk = 129,
    RubyTreeLeaf = 130,
    RubyTreeTrunkLeaf = 131,
    DiamondTreeTrunk = 132,
    DiamondTreeLeaf = 133,
    DiamondTreeTrunkLeaf = 134,
    AnyDeadTreeLeaf = 135,
    AnyDeadTreeTrunk = 136,
    GoldChest = 137,
}

impl VoxelType {
    pub const MAX_VALUE: u16 = 138;
}

#[derive(Debug, Clone, Copy)]
pub enum BlockUv {
    All(ImageType),
    TopSide {
        top: ImageType,
        side: ImageType,
    },
    TopBottomSide {
        top: ImageType,
        bottom: ImageType,
        side: ImageType,
    },
}

impl BlockUv {
    pub fn face_images(&self) -> [ImageType; 6] {
        // [PX, NX, PY, NY, PZ, NZ]
        match self {
            BlockUv::All(image_type) => [*image_type; 6],
            BlockUv::TopSide { top, side } => [*side, *side, *top, *top, *side, *side],
            BlockUv::TopBottomSide { top, bottom, side } => {
                [*side, *side, *top, *bottom, *side, *side]
            }
        }
    }

    pub fn up(&self) -> ImageType {
        match self {
            BlockUv::All(image_type) => *image_type,
            BlockUv::TopSide { top, .. } | BlockUv::TopBottomSide { top, .. } => *top,
        }
    }

    pub fn bottom(&self) -> ImageType {
        match self {
            BlockUv::All(image_type) => *image_type,
            BlockUv::TopSide { top, .. } => *top,
            BlockUv::TopBottomSide { bottom, .. } => *bottom,
        }
    }

    pub fn side(&self) -> ImageType {
        match self {
            BlockUv::All(image_type) => *image_type,
            BlockUv::TopSide { side, .. } | BlockUv::TopBottomSide { side, .. } => *side,
        }
    }
}

impl From<BlockType> for VoxelType {
    fn from(value: BlockType) -> Self {
        Self::try_from(value as u16).expect("all block types should be able to map to voxel type")
    }
}

impl VoxelType {
    pub(crate) fn uv(&self) -> BlockUv {
        use BlockUv::*;
        use ImageType::*;
        match self {
            Self::Unknown => All(WorkbenchTool5Top),
            Self::Stone => All(Stone),
            Self::Air => All(Air),
            Self::Water => All(Water0),
            Self::Ice => All(Ice),
            Self::Snow => All(Snow),
            Self::Dirt => All(Dirt),
            Self::Sand => All(Sand),
            Self::MinedSand => All(Sand),
            Self::Wood => All(Wood),
            Self::MinedStone => All(MinedStone),
            Self::RedBrick => All(RedBrick),
            Self::Limestone => All(Limestone),
            Self::MinedLimestone => All(MinedLimestone),
            Self::Marble => All(Marble),
            Self::MinedMarble => All(MinedMarble),
            Self::TimeCrystal => All(TimeCrystal),
            Self::SandStone => All(SandStone),
            Self::MinedSandStone => All(MinedSandStone),
            Self::RedMarble => All(RedMarble),
            Self::MinedRedMarble => All(MinedRedMarble),
            Self::Glass => All(Glass),
            Self::SpawnPortalBase => All(SpawnPortalBase),
            Self::GoldBlock => All(GoldBlock),
            Self::GrassDirt => TopBottomSide {
                top: Grass,
                bottom: Dirt,
                side: GrassDirt,
            },
            Self::SnowDirt => TopBottomSide {
                top: SnowGrass,
                bottom: Dirt,
                side: SnowGrassDirt,
            },
            Self::LapisLazuli => All(LapisLazuli),
            Self::MinedLapisLazuli => All(MinedLapisLazuli),
            Self::Lava => All(Lava0),
            Self::ReinforcedPlatform => All(ReinforcedPlatform),
            Self::SpawnPortalBaseAmethyst => All(SpawnPortalBaseAmethyst),
            Self::SpawnPortalBaseSapphire => All(SpawnPortalBaseSapphire),
            Self::SpawnPortalBaseEmerald => All(SpawnPortalBaseEmerald),
            Self::SpawnPortalBaseRuby => All(SpawnPortalBaseRuby),
            Self::SpawnPortalBaseDiamond => All(SpawnPortalBaseDiamond),
            Self::NorthPole => All(NorthPole),
            Self::SouthPole => All(SouthPole),
            Self::WestPole => All(EquatorPole),
            Self::EastPole => All(EquatorPole),
            Self::PortalBase => All(PortalBase),
            Self::PortalBaseAmethyst => All(PortalBaseAmethyst),
            Self::PortalBaseSapphire => All(PortalBaseSapphire),
            Self::PortalBaseEmerald => All(PortalBaseEmerald),
            Self::PortalBaseRuby => All(PortalBaseRuby),
            Self::PortalBaseDiamond => All(PortalBaseDiamond),
            Self::Compost => All(Compost),
            Self::GrassCompost => TopBottomSide {
                top: CompostGrass,
                bottom: Compost,
                side: CompostSide,
            },
            Self::SnowCompost => TopBottomSide {
                top: SnowGrass,
                bottom: Compost,
                side: SnowCompostSide,
            },
            Self::Basalt => All(Basalt),
            Self::MinedBasalt => All(Basalt),
            Self::CopperBlock => All(CopperBlock),
            Self::TinBlock => All(TinBlock),
            Self::BronzeBlock => All(BronzeBlock),
            Self::IronBlock => All(IronBlock),
            Self::SteelBlock => All(SteelBlock),
            Self::BlackSand => All(BlackSand),
            Self::BlackGlass => All(BlackGlass),
            Self::TradePortalBase => All(PortalBase),
            Self::TradePortalBaseAmethyst => All(PortalBaseAmethyst),
            Self::TradePortalBaseSapphire => All(PortalBaseSapphire),
            Self::TradePortalBaseEmerald => All(PortalBaseEmerald),
            Self::TradePortalBaseRuby => All(PortalBaseRuby),
            Self::TradePortalBaseDiamond => All(PortalBaseDiamond),
            Self::PlatinumBlock => All(PlatinumBlock0),
            Self::TitaniumBlock => All(TitaniumBlock),
            Self::CarbonFiberBlock => All(CarbonFiberBlock),
            Self::Gravel => All(Gravel),
            Self::AmethystBlock => All(AmethystBlock),
            Self::SapphireBlock => All(SapphireBlock),
            Self::EmeraldBlock => All(EmeraldBlock),
            Self::RubyBlock => All(RubyBlock),
            Self::DiamondBlock => All(DiamondBlock),
            Self::Plaster => All(Plaster),
            Self::LuminousPlaster => All(LuminousPlaster),
            Self::DirtClay => All(DirtClay),
            Self::DirtFlint => All(DirtFlint),
            Self::GrassDirtClay => TopBottomSide {
                top: Grass,
                bottom: Dirt,
                side: GrassDirtClay,
            },
            Self::GrassDirtFlint => TopBottomSide {
                top: Grass,
                bottom: Dirt,
                side: GrassDirtFlint,
            },
            Self::SnowDirtClay => TopBottomSide {
                top: SnowGrass,
                bottom: Dirt,
                side: SnowGrassDirtClay,
            },
            Self::SnowDirtFlint => TopBottomSide {
                top: SnowGrass,
                bottom: Dirt,
                side: SnowGrassDirtFlint,
            },
            Self::StoneCopperOre => All(StoneCopper),
            Self::StoneTinOre => All(StoneTin),
            Self::StoneIronOre => All(StoneIron),
            Self::StoneCoal => All(StoneCoal),
            Self::StoneGoldNuggets => All(StoneGold),
            Self::StonePlatinumOre => All(StonePlatinum),
            Self::StoneTitaniumOre => All(StoneTitanium),
            Self::LimestoneOil => All(LimestoneOil),
            Self::AppleTreeLeaf => All(AppleTreeLeaf),
            Self::AppleTreeTrunk => TopSide {
                top: TrunkTop,
                side: AppleTreeTrunk,
            },
            Self::AppleTreeTrunkLeaf => All(AppleTreeTrunkLeaf),
            Self::PineTreeLeaf => All(PineTreeLeaf),
            Self::PineTreeTrunk => TopSide {
                top: TrunkTop,
                side: Trunk,
            },
            Self::PineTreeTrunkLeaf => All(PineTreeTrunkLeaf),
            Self::MapleTreeLeaf => All(MapleTreeLeaf),
            Self::MapleTreeTrunk => TopSide {
                top: TrunkTop,
                side: MapleTreeTrunk,
            },
            Self::MapleTreeTrunkLeaf => All(MapleTreeTrunkLeaf),
            Self::MangoTreeLeaf => All(MangoTreeLeaf),
            Self::MangoTreeTrunk => TopSide {
                top: TrunkTop,
                side: Trunk,
            },
            Self::MangoTreeTrunkLeaf => All(MangoTreeTrunkLeaf),
            Self::CoconutTreeLeaf => All(CoconutTreeLeaf),
            Self::CoconutTreeTrunk => All(CoconutTreeTrunk),
            Self::OrangeTreeLeaf => All(OrangeTreeLeaf),
            Self::OrangeTreeTrunk => TopSide {
                top: TrunkTop,
                side: OrangeTreeTrunk,
            },
            Self::OrangeTreeTrunkLeaf => All(OrangeTreeTrunkLeaf),
            Self::CherryTreeLeaf => All(CherryTreeLeaf),
            Self::CherryTreeTrunk => TopSide {
                top: TrunkTop,
                side: CherryTreeTrunk,
            },
            Self::CherryTreeTrunkLeaf => All(CherryTreeTrunkLeaf),
            Self::CoffeeTreeLeaf => All(CoffeeTreeLeaf),
            Self::CoffeeTreeTrunk => TopSide {
                top: TrunkTop,
                side: CoffeeTreeTrunk,
            },
            Self::CoffeeTreeTrunkLeaf => All(CoffeeTreeTrunkLeaf),
            Self::Cactus => All(Cactus),
            Self::DeadCactus => All(DeadCactus),
            Self::LimeTreeLeaf => All(LimeTreeLeaf),
            Self::LimeTreeTrunk => TopSide {
                top: TrunkTop,
                side: LimeTreeTrunk,
            },
            Self::LimeTreeTrunkLeaf => All(LimeTreeTrunkLeaf),
            Self::AmethystTreeTrunk => TopSide {
                top: TrunkTop,
                side: AmethystTreeTrunk,
            },
            Self::AmethystTreeLeaf => All(AmethystTreeLeaf),
            Self::AmethystTreeTrunkLeaf => All(AmethystTreeTrunkLeaf),
            Self::SapphireTreeTrunk => TopSide {
                top: TrunkTop,
                side: SapphireTreeTrunk,
            },
            Self::SapphireTreeLeaf => All(SapphireTreeLeaf),
            Self::SapphireTreeTrunkLeaf => All(SapphireTreeTrunkLeaf),
            Self::EmeraldTreeTrunk => TopSide {
                top: TrunkTop,
                side: EmeraldTreeTrunk,
            },
            Self::EmeraldTreeLeaf => All(EmeraldTreeLeaf),
            Self::EmeraldTreeTrunkLeaf => All(EmeraldTreeTrunkLeaf),
            Self::RubyTreeTrunk => TopSide {
                top: TrunkTop,
                side: RubyTreeTrunk,
            },
            Self::RubyTreeLeaf => All(RubyTreeLeaf),
            Self::RubyTreeTrunkLeaf => All(RubyTreeTrunkLeaf),
            Self::DiamondTreeTrunk => TopSide {
                top: TrunkTop,
                side: DiamondTreeTrunk,
            },
            Self::DiamondTreeLeaf => All(DiamondTreeLeaf),
            Self::DiamondTreeTrunkLeaf => All(DiamondTreeTrunkLeaf),
            Self::AnyDeadTreeLeaf => All(DeadTreeLeaf),
            Self::AnyDeadTreeTrunk => TopSide {
                top: DeadTrunkTop,
                side: DeadTrunk,
            },
            Self::GoldChest => TopSide {
                top: ChestGoldTop,
                side: ChestGold,
            },
        }
    }
}

impl VoxelType {
    fn fg_from_block_inner<'b>(block: BlockView<'b>) -> Result<Self, BlockError> {
        Ok(match (block.fg()?, block.content()?) {
            (BlockType::Air, _) => Self::Air,
            (BlockType::Snow, _) => Self::Snow,
            (BlockType::Water, _) => Self::Water,
            (block_type, BlockContentType::Nothing)
            | (block_type, BlockContentType::Workbench)
            | (block_type, BlockContentType::WorkbenchSprite)
            | (block_type, BlockContentType::Sprite)
            | (block_type, BlockContentType::Wire) => block_type.into(),
            (BlockType::Dirt, BlockContentType::Clay) => Self::DirtClay,
            (BlockType::Dirt, BlockContentType::Flint) => Self::DirtFlint,
            (BlockType::GrassDirt, BlockContentType::Clay) => Self::GrassDirtClay,
            (BlockType::GrassDirt, BlockContentType::Flint) => Self::GrassDirtFlint,
            (BlockType::SnowDirt, BlockContentType::Clay) => Self::SnowDirtClay,
            (BlockType::SnowDirt, BlockContentType::Flint) => Self::SnowDirtFlint,
            (BlockType::Stone, BlockContentType::CopperOre) => Self::StoneCopperOre,
            (BlockType::Stone, BlockContentType::TinOre) => Self::StoneTinOre,
            (BlockType::Stone, BlockContentType::IronOre) => Self::StoneIronOre,
            (BlockType::Stone, BlockContentType::Coal) => Self::StoneCoal,
            (BlockType::Stone, BlockContentType::GoldNuggets) => Self::StoneGoldNuggets,
            (BlockType::Stone, BlockContentType::PlatinumOre) => Self::StonePlatinumOre,
            (BlockType::Stone, BlockContentType::TitaniumOre) => Self::StoneTitaniumOre,
            (BlockType::Limestone, BlockContentType::Oil) => Self::LimestoneOil,
            _ => Self::Unknown,
        })
    }

    pub fn fg_from_block<'b>(block: BlockView<'b>) -> Self {
        Self::fg_from_block_inner(block).unwrap_or(Self::Unknown)
    }

    fn mg_from_block_inner<'b>(block: BlockView<'b>) -> Result<Self, BlockError> {
        Ok(match block.content()? {
            BlockContentType::Nothing
            | BlockContentType::Clay
            | BlockContentType::Flint
            | BlockContentType::CopperOre
            | BlockContentType::TinOre
            | BlockContentType::IronOre
            | BlockContentType::Coal
            | BlockContentType::GoldNuggets
            | BlockContentType::PlatinumOre
            | BlockContentType::TitaniumOre
            | BlockContentType::Oil => Self::fg_from_block_inner(block)?,
            BlockContentType::Workbench
            | BlockContentType::WorkbenchSprite
            | BlockContentType::Sprite
            | BlockContentType::Wire => Self::Air,
            BlockContentType::AppleTreeLeaf => Self::AppleTreeLeaf,
            BlockContentType::AppleTreeTrunk => Self::AppleTreeTrunk,
            BlockContentType::AppleTreeTrunkLeaf => Self::AppleTreeTrunkLeaf,
            BlockContentType::PineTreeLeaf => Self::PineTreeLeaf,
            BlockContentType::PineTreeTrunk => Self::PineTreeTrunk,
            BlockContentType::PineTreeTrunkLeaf => Self::PineTreeTrunkLeaf,
            BlockContentType::MapleTreeLeaf => Self::MapleTreeLeaf,
            BlockContentType::MapleTreeTrunk => Self::MapleTreeTrunk,
            BlockContentType::MapleTreeTrunkLeaf => Self::MapleTreeTrunkLeaf,
            BlockContentType::MangoTreeLeaf => Self::MangoTreeLeaf,
            BlockContentType::MangoTreeTrunk => Self::MangoTreeTrunk,
            BlockContentType::MangoTreeTrunkLeaf => Self::MangoTreeTrunkLeaf,
            BlockContentType::CoconutTreeLeaf => Self::CoconutTreeLeaf,
            BlockContentType::CoconutTreeTrunk => Self::CoconutTreeTrunk,
            BlockContentType::OrangeTreeLeaf => Self::OrangeTreeLeaf,
            BlockContentType::OrangeTreeTrunk => Self::OrangeTreeTrunk,
            BlockContentType::OrangeTreeTrunkLeaf => Self::OrangeTreeTrunkLeaf,
            BlockContentType::CherryTreeLeaf => Self::CherryTreeLeaf,
            BlockContentType::CherryTreeTrunk => Self::CherryTreeTrunk,
            BlockContentType::CherryTreeTrunkLeaf => Self::CherryTreeTrunkLeaf,
            BlockContentType::CoffeeTreeLeaf => Self::CoffeeTreeLeaf,
            BlockContentType::CoffeeTreeTrunk => Self::CoffeeTreeTrunk,
            BlockContentType::CoffeeTreeTrunkLeaf => Self::CoffeeTreeTrunkLeaf,
            BlockContentType::Cactus => Self::Cactus,
            BlockContentType::DeadCactus => Self::DeadCactus,
            BlockContentType::LimeTreeLeaf => Self::LimeTreeLeaf,
            BlockContentType::LimeTreeTrunk => Self::LimeTreeTrunk,
            BlockContentType::LimeTreeTrunkLeaf => Self::LimeTreeTrunkLeaf,
            BlockContentType::AmethystTreeTrunk => Self::AmethystTreeTrunk,
            BlockContentType::AmethystTreeLeaf => Self::AmethystTreeLeaf,
            BlockContentType::AmethystTreeTrunkLeaf => Self::AmethystTreeTrunkLeaf,
            BlockContentType::SapphireTreeTrunk => Self::SapphireTreeTrunk,
            BlockContentType::SapphireTreeLeaf => Self::SapphireTreeLeaf,
            BlockContentType::SapphireTreeTrunkLeaf => Self::SapphireTreeTrunkLeaf,
            BlockContentType::EmeraldTreeTrunk => Self::EmeraldTreeTrunk,
            BlockContentType::EmeraldTreeLeaf => Self::EmeraldTreeLeaf,
            BlockContentType::EmeraldTreeTrunkLeaf => Self::EmeraldTreeTrunkLeaf,
            BlockContentType::RubyTreeTrunk => Self::RubyTreeTrunk,
            BlockContentType::RubyTreeLeaf => Self::RubyTreeLeaf,
            BlockContentType::RubyTreeTrunkLeaf => Self::RubyTreeTrunkLeaf,
            BlockContentType::DiamondTreeTrunk => Self::DiamondTreeTrunk,
            BlockContentType::DiamondTreeLeaf => Self::DiamondTreeLeaf,
            BlockContentType::DiamondTreeTrunkLeaf => Self::DiamondTreeTrunkLeaf,
            BlockContentType::DeadPineTreeLeaf
            | BlockContentType::DeadOrangeTreeLeaf
            | BlockContentType::DeadCherryTreeLeaf
            | BlockContentType::DeadLimeTreeLeaf => Self::AnyDeadTreeLeaf,
            BlockContentType::DeadPineTreeTrunk
            | BlockContentType::DeadOrangeTreeTrunk
            | BlockContentType::DeadCherryTreeTrunk
            | BlockContentType::DeadLimeTreeTrunk => Self::AnyDeadTreeTrunk,
            BlockContentType::GoldChest => Self::GoldChest,
        })
    }

    pub fn mg_from_block<'b>(block: BlockView<'b>) -> Self {
        Self::mg_from_block_inner(block).unwrap_or(Self::Unknown)
    }

    fn bg_from_block_inner<'b>(block: BlockView<'b>) -> Result<Self, BlockError> {
        Ok(block.bg()?.into())
    }

    pub fn bg_from_block<'b>(block: BlockView<'b>) -> Self {
        Self::bg_from_block_inner(block).unwrap_or(Self::Unknown)
    }

    pub(crate) fn uv_at_face() -> Vec<[ImageType; 6]> {
        (0..VoxelType::MAX_VALUE)
            .map(|voxel_type_id| {
                VoxelType::try_from(voxel_type_id)
                    .unwrap_or(VoxelType::Unknown)
                    .uv()
                    .face_images()
            })
            .collect()
    }
}

pub mod voxel_util {
    use super::VoxelType;
    use eframe::wgpu::{self, util::DeviceExt};
    use the_blockheads_tools_lib::game::{
        chunk::{Chunk, ChunkView, Chunks},
        coord::{ChunkBlockCoord, ChunkCoord},
    };

    const NUM_VOXEL_DEPTH: usize = 3;
    const NUM_BLOCK_PER_CHUNK: usize =
        Chunk::NUM_BLOCK_PER_ROW * Chunk::NUM_BLOCK_PER_COL * NUM_VOXEL_DEPTH; // 3 layers

    // Costly function - only call this once or VRAM nuked
    // Contains flattened block type, 512 * 32 * (32 * 32 * 3) blocks
    pub fn create_buffer(device: &wgpu::Device, world_width_macro: usize) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Voxel Data Buffer"),
            contents: bytemuck::cast_slice(&new_world_voxel(world_width_macro)),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn new_world_voxel(world_width_macro: usize) -> Vec<VoxelType> {
        vec![VoxelType::Air; NUM_BLOCK_PER_CHUNK * Chunks::NUM_CHUNK_PER_COL * world_width_macro]
    }

    fn fill_chunk_voxel(chunk: ChunkView<'_>, chunk_voxel: &mut [VoxelType]) {
        for y in 0..Chunk::NUM_BLOCK_PER_COL {
            for x in 0..Chunk::NUM_BLOCK_PER_ROW {
                let block = chunk.block_at(
                    ChunkBlockCoord::new(x as u8, y as u8).expect("x and y must be within limit"),
                );
                let fg_type = VoxelType::fg_from_block(block);
                let mg_type = VoxelType::mg_from_block(block);
                let mut bg_type = VoxelType::bg_from_block(block);
                if bg_type == VoxelType::Air && fg_type != VoxelType::Air {
                    bg_type = fg_type;
                }
                let index = (y * Chunk::NUM_BLOCK_PER_ROW + x) * NUM_VOXEL_DEPTH;
                chunk_voxel[index] = bg_type;
                chunk_voxel[index + 1] = mg_type;
                chunk_voxel[index + 2] = fg_type;
            }
        }
    }

    // On wasm32, malloc is extremely slow (20-30ms PER call) if we do too much.
    // This method builds world_voxel with only 2 malloc!
    // This also turns out to help with I/O speed, based on observation that the editor
    // will likely never edit >95% of the chunks decompressed.
    fn build_world_voxel(chunks: &Chunks, world_width_macro: usize) -> Vec<VoxelType> {
        let mut world_voxel = new_world_voxel(world_width_macro);
        let mut decompress_output = Vec::with_capacity(Chunk::NUM_BYTES);
        chunks
            .inner()
            .iter()
            .zip(world_voxel.chunks_exact_mut(NUM_BLOCK_PER_CHUNK))
            .for_each(|(chunk, chunk_voxel)| {
                if let Some(chunk) = chunk
                    && let Ok(chunk_slice) = chunk.decompress_view(&mut decompress_output)
                {
                    fill_chunk_voxel(chunk_slice, chunk_voxel);
                }
            });
        world_voxel
    }

    pub fn set_chunks(
        queue: &wgpu::Queue,
        voxel_buffer: &wgpu::Buffer,
        chunks: &mut Chunks,
        world_width_macro: usize,
    ) {
        let world_voxel = build_world_voxel(chunks, world_width_macro);
        queue.write_buffer(voxel_buffer, 0, bytemuck::cast_slice(&world_voxel));
    }

    pub fn set_chunk<I: Into<ChunkCoord>>(
        queue: &wgpu::Queue,
        voxel_buffer: &wgpu::Buffer,
        coord: I,
        chunk: &Chunk,
    ) {
        let mut blocks = [VoxelType::Unknown; NUM_BLOCK_PER_CHUNK];
        fill_chunk_voxel(chunk.view(), &mut blocks);

        let chunk_coord: ChunkCoord = coord.into();
        let offset = (chunk_coord.x() * Chunk::NUM_BLOCK_PER_COL as u32 + chunk_coord.y() as u32)
            * NUM_BLOCK_PER_CHUNK as u32;

        queue.write_buffer(
            voxel_buffer,
            offset as u64 * size_of::<u16>() as u64,
            bytemuck::cast_slice(&blocks),
        );
    }
}
