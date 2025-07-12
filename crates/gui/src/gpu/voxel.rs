use std::collections::HashSet;

use egui_wgpu::wgpu::{self, util::DeviceExt};
use the_blockheads_tools_lib::{
    BhResult, Block, BlockContent, BlockType, BlockView, Chunk, ChunkBlockCoord, ChunkCoord,
};

type BlockIdType = u16;

// Basically the same as BlockType, but treats block with tile content as separate type
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, PartialEq, Eq)]
pub struct VoxelType(BlockIdType);

impl From<BlockIdType> for VoxelType {
    fn from(value: BlockIdType) -> Self {
        Self(value)
    }
}

impl VoxelType {
    pub const AIR: Self = Self(2);
    pub const UNKNOWN: Self = Self(0);

    // [PX, NX, PY, NY, PZ, NZ]
    pub(crate) const UV_AT_FACE: &[[u32; 6]] = &[
        [371; 6],                       // Unknown
        [32; 6],                        // Stone
        [0; 6],                         // Air
        [14; 6],                        // Water
        [110; 6],                       // Ice
        [129; 6],                       // Snow
        [64; 6],                        // Dirt
        [65; 6],                        // Sand
        [65; 6],                        // MinedSand
        [196; 6],                       // Wood
        [33; 6],                        // MinedStone
        [34; 6],                        // RedBrick
        [66; 6],                        // Limestone
        [35; 6],                        // MinedLimestone
        [67; 6],                        // Marble
        [76; 6],                        // MinedMarble
        [77; 6],                        // TimeCrystal
        [79; 6],                        // SandStone
        [80; 6],                        // MinedSandStone
        [81; 6],                        // RedMarble
        [82; 6],                        // MinedRedMarble
        [0; 6],                         // Missing id 21
        [0; 6],                         // Missing id 22
        [0; 6],                         // Missing id 23
        [108; 6],                       // Glass
        [132; 6],                       // SpawnPortalBase
        [85; 6],                        // GoldBlock
        [160, 160, 161, 64, 160, 160],  // GrassDirt
        [162, 162, 163, 64, 162, 162],  // SnowDirt
        [86; 6],                        // LapisLazuli
        [87; 6],                        // MinedLapisLazuli
        [46; 6],                        // Lava
        [164; 6],                       // ReinforcedPlatform
        [165; 6],                       // SpawnPortalBaseAmethyst
        [166; 6],                       // SpawnPortalBaseSapphire
        [167; 6],                       // SpawnPortalBaseEmerald
        [168; 6],                       // SpawnPortalBaseRuby
        [169; 6],                       // SpawnPortalBaseDiamond
        [552; 6],                       // NorthPole
        [553; 6],                       // SouthPole
        [554; 6],                       // WestPole
        [554; 6],                       // EastPole
        [170; 6],                       // PortalBase
        [171; 6],                       // PortalBaseAmethyst
        [172; 6],                       // PortalBaseSapphire
        [173; 6],                       // PortalBaseEmerald
        [174; 6],                       // PortalBaseRuby
        [175; 6],                       // PortalBaseDiamond
        [97; 6],                        // Compost
        [98, 98, 294, 97, 98, 98],      // GrassCompost
        [130, 130, 163, 97, 130, 130],  // SnowCompost
        [88; 6],                        // Basalt
        [88; 6],                        // MinedBasalt
        [112; 6],                       // CopperBlock
        [113; 6],                       // TinBlock
        [114; 6],                       // BronzeBlock
        [115; 6],                       // IronBlock
        [116; 6],                       // SteelBlock
        [89; 6],                        // BlackSand
        [117; 6],                       // BlackGlass
        [170; 6],                       // TradePortalBase
        [171; 6],                       // TradePortalBaseAmethyst
        [172; 6],                       // TradePortalBaseSapphire
        [173; 6],                       // TradePortalBaseEmerald
        [174; 6],                       // TradePortalBaseRuby
        [175; 6],                       // TradePortalBaseDiamond
        [0; 6],                         // Missing id 66
        [672; 6],                       // PlatinumBlock
        [190; 6],                       // TitaniumBlock
        [188; 6],                       // CarbonFiberBlock
        [478; 6],                       // Gravel
        [697; 6],                       // AmethystBlock
        [698; 6],                       // SapphireBlock
        [699; 6],                       // EmeraldBlock
        [700; 6],                       // RubyBlock
        [701; 6],                       // DiamondBlock
        [741; 6],                       // Plaster
        [746; 6],                       // LuminousPlaster
        [290; 6],                       // Dirt + Clay
        [288; 6],                       // Dirt + Flint
        [291, 291, 161, 64, 291, 291],  // GrassDirt + Clay
        [289, 289, 161, 64, 289, 289],  // GrassDirt + Flint
        [293, 293, 163, 64, 293, 293],  // SnowDirt + Clay
        [292, 292, 163, 64, 292, 292],  // SnowDirt + Flint
        [1; 6],                         // Stone + CopperOre
        [3; 6],                         // Stone + TinOre
        [2; 6],                         // Stone + IronOre
        [83; 6],                        // Stone + Coal
        [84; 6],                        // Stone + GoldNuggets
        [157; 6],                       // Stone + PlatinumOre
        [219; 6],                       // Stone + TitaniumOre
        [78; 6],                        // Limestone + Oil
        [256; 6],                       // AppleTreeLeafEarlySummer
        [264, 264, 193, 193, 264, 264], // AppleTreeTrunk
        [224; 6],                       // AppleTreeTrunkWithLeafEarlySummer
        [237; 6],                       // PineTreeLeaf
        [192, 192, 193, 193, 192, 192], // PineTreeTrunk,
        [236; 6],                       // PineTreeTrunkWithLeaf
        [512; 6],                       // MapleTreeLeafEarlySummer
        [488, 488, 193, 193, 488, 488], // MapleTreeTrunk
        [480; 6],                       // MapleTreeTrunkWithLeafEarlySummer
        [233; 6],                       // MangoTreeLeaf
        [192, 192, 193, 193, 192, 192], // MangoTreeTrunk
        [232; 6],                       // MangoTreeTrunkWithLeaf
        [489; 6],                       // CoconutTreeLeaf
        [490; 6],                       // CoconutTreeTrunk
        [493; 6],                       // OrangeTreeLeaf
        [495, 495, 193, 193, 495, 495], // OrangeTreeTrunk
        [494; 6],                       // OrangeTreeTrunkWithLeaf
        [265; 6],                       // CherryTreeLeafEarlySummer
        [273, 273, 193, 193, 273, 273], // CherryTreeTrunk
        [274; 6],                       // CherryTreeTrunkWithLeafEarlySummer
        [283; 6],                       // CoffeeTreeLeaf
        [282, 282, 193, 193, 282, 282], // CoffeeTreeTrunk
        [284; 6],                       // CoffeeTreeTrunkWithLeaf
        [234; 6],                       // Cactus
        [235; 6],                       // DeadCactus
        [520; 6],                       // LimeTreeLeaf
        [522, 522, 193, 193, 522, 522], // LimeTreeTrunk
        [521; 6],                       // LimeTreeTrunkWithLeaf
        [220, 220, 193, 193, 220, 220], // AmethystTreeTrunk
        [570; 6],                       // AmethystTreeLeaf
        [602; 6],                       // AmethystTreeTrunkWithLeaf
        [221, 221, 193, 193, 221, 221], // SapphireTreeTrunk
        [571; 6],                       // SapphireTreeLeaf
        [603; 6],                       // SapphireTreeTrunkWithLeaf
        [222, 222, 193, 193, 222, 222], // EmeraldTreeTrunk
        [572; 6],                       // EmeraldTreeLeaf
        [604; 6],                       // EmeraldTreeTrunkWithLeaf
        [253, 253, 193, 193, 253, 253], // RubyTreeTrunk
        [573; 6],                       // RubyTreeLeaf
        [605; 6],                       // RubyTreeTrunkWithLeaf
        [254, 254, 193, 193, 254, 254], // DiamondTreeTrunk
        [574; 6],                       // DiamondTreeLeaf
        [606; 6],                       // DiamondTreeTrunkWithLeaf
        [449; 6],                       // Any dead tree leaf
        [194, 194, 195, 195, 194, 194], // Any dead tree trunk
    ];
}

impl VoxelType {
    fn fg_from_block_inner<'b>(block: Block<'b>) -> BhResult<Self> {
        Ok(Self(match (block.fg()?, block.content()?) {
            (BlockType::Air, _) => 2,
            (BlockType::Snow, _) => 5,
            (block_type, BlockContent::None) => block_type as u16,
            (BlockType::Dirt, BlockContent::Clay) => 78,
            (BlockType::Dirt, BlockContent::Flint) => 79,
            (BlockType::GrassDirt, BlockContent::Clay) => 80,
            (BlockType::GrassDirt, BlockContent::Flint) => 81,
            (BlockType::SnowDirt, BlockContent::Clay) => 82,
            (BlockType::SnowDirt, BlockContent::Flint) => 83,
            (BlockType::Stone, BlockContent::CopperOre) => 84,
            (BlockType::Stone, BlockContent::TinOre) => 85,
            (BlockType::Stone, BlockContent::IronOre) => 86,
            (BlockType::Stone, BlockContent::Coal) => 87,
            (BlockType::Stone, BlockContent::GoldNuggets) => 88,
            (BlockType::Stone, BlockContent::PlatinumOre) => 89,
            (BlockType::Stone, BlockContent::TitaniumOre) => 90,
            (BlockType::Limestone, BlockContent::Oil) => 91,
            _ => 0,
        }))
    }

    pub fn fg_from_block<'b>(block: Block<'b>) -> Self {
        Self::fg_from_block_inner(block).unwrap_or(Self::UNKNOWN)
    }

    fn mg_from_block_inner<'b>(block: Block<'b>) -> BhResult<Self> {
        Ok(Self(match block.content()? {
            BlockContent::None => 2,
            BlockContent::AppleTreeLeaf => 92,
            BlockContent::AppleTreeTrunk => 93,
            BlockContent::AppleTreeTrunkWithLeaf => 94,
            BlockContent::PineTreeLeaf => 95,
            BlockContent::PineTreeTrunk => 96,
            BlockContent::PineTreeTrunkWithLeaf => 97,
            BlockContent::MapleTreeLeaf => 98,
            BlockContent::MapleTreeTrunk => 99,
            BlockContent::MapleTreeTrunkWithLeaf => 100,
            BlockContent::MangoTreeLeaf => 101,
            BlockContent::MangoTreeTrunk => 102,
            BlockContent::MangoTreeTrunkWithLeaf => 103,
            BlockContent::CoconutTreeLeaf => 104,
            BlockContent::CoconutTreeTrunk => 105,
            BlockContent::OrangeTreeLeaf => 106,
            BlockContent::OrangeTreeTrunk => 107,
            BlockContent::OrangeTreeTrunkWithLeaf => 108,
            BlockContent::CherryTreeLeaf => 109,
            BlockContent::CherryTreeTrunk => 110,
            BlockContent::CherryTreeTrunkWithLeaf => 111,
            BlockContent::CoffeeTreeLeaf => 112,
            BlockContent::CoffeeTreeTrunk => 113,
            BlockContent::CoffeeTreeTrunkWithLeaf => 114,
            BlockContent::Cactus => 115,
            BlockContent::DeadCactus => 116,
            BlockContent::LimeTreeLeaf => 117,
            BlockContent::LimeTreeTrunk => 118,
            BlockContent::LimeTreeTrunkWithLeaf => 119,
            BlockContent::AmethystTreeTrunk => 120,
            BlockContent::AmethystTreeLeaf => 121,
            BlockContent::AmethystTreeTrunkWithLeaf => 122,
            BlockContent::SapphireTreeTrunk => 123,
            BlockContent::SapphireTreeLeaf => 124,
            BlockContent::SapphireTreeTrunkWithLeaf => 125,
            BlockContent::EmeraldTreeTrunk => 126,
            BlockContent::EmeraldTreeLeaf => 127,
            BlockContent::EmeraldTreeTrunkWithLeaf => 128,
            BlockContent::RubyTreeTrunk => 129,
            BlockContent::RubyTreeLeaf => 130,
            BlockContent::RubyTreeTrunkWithLeaf => 131,
            BlockContent::DiamondTreeTrunk => 132,
            BlockContent::DiamondTreeLeaf => 133,
            BlockContent::DiamondTreeTrunkWithLeaf => 134,
            BlockContent::DeadPineTreeLeaf
            | BlockContent::DeadOrangeTreeLeaf
            | BlockContent::DeadCherryTreeLeaf
            | BlockContent::DeadLimeTreeLeaf => 135,
            BlockContent::DeadPineTreeTrunk
            | BlockContent::DeadOrangeTreeTrunk
            | BlockContent::DeadCherryTreeTrunk
            | BlockContent::DeadLimeTreeTrunk => 136,
            _ => 0,
        }))
    }

    pub fn mg_from_block<'b>(block: Block<'b>) -> Self {
        Self::mg_from_block_inner(block).unwrap_or(Self::UNKNOWN)
    }

    fn bg_from_block_inner<'b>(block: Block<'b>) -> BhResult<Self> {
        Ok(Self(block.bg()? as u16))
    }

    pub fn bg_from_block<'b>(block: Block<'b>) -> Self {
        Self::bg_from_block_inner(block).unwrap_or(Self::UNKNOWN)
    }
}

pub struct VoxelBuf {
    // Contains flattened block type, 512 * 32 * (32 * 32 * 3) blocks
    pub buf: wgpu::Buffer,
    chunk_keys: HashSet<ChunkCoord>,
    pub world_width_macro: usize,
}

impl VoxelBuf {
    const NUM_BLOCK_PER_CHUNK: usize = Chunk::NUM_BLOCK_PER_ROW * Chunk::NUM_BLOCK_PER_COL * 3;

    // Costly function - only call this once or VRAM nuked
    pub fn new(device: &wgpu::Device, world_width_macro: usize) -> Self {
        Self {
            buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Global Voxel Data Buffer"),
                contents: bytemuck::cast_slice(&vec![
                    VoxelType::AIR;
                    Self::NUM_BLOCK_PER_CHUNK
                        * 32
                        * world_width_macro
                ]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }),
            chunk_keys: HashSet::new(),
            world_width_macro,
        }
    }

    /// Clean up all voxels and registered chunks.
    pub fn clear(&mut self, queue: &wgpu::Queue) {
        self.chunk_keys.clear();
        queue.write_buffer(
            &self.buf,
            0,
            bytemuck::cast_slice(&vec![
                VoxelType::AIR;
                Self::NUM_BLOCK_PER_CHUNK * 32 * self.world_width_macro
            ]),
        );
    }

    pub fn has_chunk<I: Into<ChunkCoord>>(&self, key: I) -> bool {
        self.chunk_keys.contains(&key.into())
    }

    pub fn set_chunk<I: Into<ChunkCoord>>(
        &mut self,
        queue: &wgpu::Queue,
        coord: I,
        chunk: &Chunk,
    ) -> BhResult<()> {
        let mut blocks = [VoxelType(0); Self::NUM_BLOCK_PER_CHUNK];
        for y in 0..Chunk::NUM_BLOCK_PER_COL {
            for x in 0..Chunk::NUM_BLOCK_PER_ROW {
                let block = chunk.block_at(&ChunkBlockCoord::new(x as u8, y as u8)?);
                let fg_type = VoxelType::fg_from_block(block);
                let mg_type = if fg_type == VoxelType::AIR {
                    VoxelType::mg_from_block(block)
                } else {
                    fg_type
                };
                let bg_type = VoxelType::bg_from_block(block);
                let index = (y * Chunk::NUM_BLOCK_PER_ROW + x) * 3;
                blocks[index + 0] = bg_type;
                blocks[index + 1] = mg_type;
                blocks[index + 2] = fg_type;
            }
        }

        let chunk_coord: ChunkCoord = coord.into();
        let offset = (chunk_coord.x * 32 + chunk_coord.y as u64) * Self::NUM_BLOCK_PER_CHUNK as u64;

        queue.write_buffer(
            &self.buf,
            offset * size_of::<u16>() as u64,
            bytemuck::cast_slice(&blocks),
        );
        self.chunk_keys.insert(chunk_coord);
        Ok(())
    }
}
