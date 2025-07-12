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

    // [PX, NX, PY, NY, PZ, NZ]
    pub(crate) const UV_AT_FACE: &[[u32; 6]] = &[
        [0; 6],                         // None
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
        [293, 293, 161, 64, 293, 293],  // SnowDirt + Clay
        [292, 292, 161, 64, 292, 292],  // SnowDirt + Flint
        [1; 6],                         // Stone + CopperOre
        [3; 6],                         // Stone + TinOre
        [2; 6],                         // Stone + IronOre
        [83; 6],                        // Stone + Coal
        [84; 6],                        // Stone + GoldNuggets
        [157; 6],                       // Stone + PlatinumOre
        [219; 6],                       // Stone + TitaniumOre
        [78; 6],                        // Limestone + Oil
        [237; 6],                       // PineTreeLeaf
        [192, 192, 193, 193, 192, 192], // PineTreeTrunk,
        [236; 6],                       // PineTreeTrunkWithLeaf
        [493; 6],                       // OrangeTreeLeaf
        [495, 495, 193, 193, 495, 495], // OrangeTreeTrunk
        [494; 6],                       // OrangeTreeTrunkWithLeaf
    ];
}

impl From<(BlockType, BlockContent)> for VoxelType {
    fn from(value: (BlockType, BlockContent)) -> Self {
        Self(match value {
            (BlockType::Air, _) => 0,
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
        })
    }
}

impl VoxelType {
    fn fg_from_block_inner<'b>(block: Block<'b>) -> BhResult<Self> {
        Ok(Self(match (block.fg()?, block.content()?) {
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
        Self::fg_from_block_inner(block).unwrap_or(Self(0))
    }

    fn mg_from_block_inner<'b>(block: Block<'b>) -> BhResult<Self> {
        Ok(Self(match block.content()? {
            BlockContent::PineTreeLeaf => 92,
            BlockContent::PineTreeTrunk => 93,
            BlockContent::PineTreeTrunkWithLeaf => 94,
            BlockContent::OrangeTreeLeaf => 95,
            BlockContent::OrangeTreeTrunk => 96,
            BlockContent::OrangeTreeTrunkWithLeaf => 97,
            _ => 0,
        }))
    }

    pub fn mg_from_block<'b>(block: Block<'b>) -> Self {
        Self::mg_from_block_inner(block).unwrap_or(Self(0))
    }

    fn bg_from_block_inner<'b>(block: Block<'b>) -> BhResult<Self> {
        Ok(Self(block.bg()? as u16))
    }

    pub fn bg_from_block<'b>(block: Block<'b>) -> Self {
        Self::bg_from_block_inner(block).unwrap_or(Self(0))
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
                let mg_type = if fg_type == VoxelType(0) {
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
