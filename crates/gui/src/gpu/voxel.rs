use std::collections::HashSet;

use egui_wgpu::wgpu::{self, util::DeviceExt};
use the_blockheads_tools_lib::{
    BhResult, Block, BlockContent, BlockCoord, BlockType, BlockView, Chunk, ChunkBlockCoord,
    ChunkCoord, WorldDb,
};

type BlockIdType = u16;

// Basically the same as BlockType, but treats block with tile content as separate type
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VoxelType(BlockIdType);

impl From<BlockIdType> for VoxelType {
    fn from(value: BlockIdType) -> Self {
        Self(value)
    }
}

impl VoxelType {
    // [PX, NX, PY, NY, PZ, NZ]
    pub(crate) const UV_AT_FACE: [[u32; 6]; 78] = [
        [0; 6],                        // NONE,
        [32; 6],                       // STONE,
        [0; 6],                        // AIR
        [14; 6],                       // WATER
        [110; 6],                      // ICE
        [129; 6],                      // SNOW
        [64; 6],                       // DIRT
        [65; 6],                       // SAND
        [65; 6],                       // SAND_ (assuming same as SAND)
        [196; 6],                      // WOOD
        [33; 6],                       // MINED_STONE (from 0x21)
        [34; 6],                       // RED_BRICK (from 0x22)
        [66; 6],                       // LIMESTONE (from 0x42, default)
        [35; 6],                       // MINED_LIMESTONE (from 0x23)
        [67; 6],                       // MARBLE (from 0x43)
        [76; 6],                       // MINED_MARBLE (from 0x4c)
        [77; 6],                       // TIME_CRYSTAL (from 0x4d)
        [79; 6],                       // SAND_STONE (from 0x4f)
        [80; 6],                       // MINED_SAND_STONE (from 0x50)
        [81; 6],                       // RED_MARBLE (from 0x51)
        [82; 6],                       // MINED_RED_MARBLE (from 0x52)
        [0; 6],                        // Missing ID 21
        [0; 6],                        // Missing ID 22
        [0; 6],                        // Missing ID 23
        [108; 6],                      // GLASS (from 0x6c)
        [132; 6],                      // SPAWN_PORTAL_BASE (from 0x55)
        [288; 6], // GOLD_BLOCK (from 0x120, default. Note that this block has sub-types for gold and silver, but the prompt only asks for the base UV, which is the default in the code)
        [160, 160, 161, 64, 160, 160], // GRASS_DIRT (from 0xa0, default)
        [162, 162, 163, 64, 162, 162], // SNOW_DIRT (from 0xa2, default)
        [86; 6],  // LAPIS_LAZULI (from 0x56)
        [87; 6],  // MINED_LAPIS_LAZULI (from 0x57)
        [46; 6],  // LAVA (dynamic, using formula)
        [164; 6], // REINFORCED_PLATFORM (from 0xa4)
        [165; 6], // SPAWN_PORTAL_BASE_AMETHYST (from 0xa5)
        [166; 6], // SPAWN_PORTAL_BASE_SAPPHIRE (from 0xa6)
        [167; 6], // SPAWN_PORTAL_BASE_EMERALD (from 0xa7)
        [168; 6], // SPAWN_PORTAL_BASE_RUBY (from 0xa8)
        [169; 6], // SPAWN_PORTAL_BASE_DIAMOND (from 0xa9)
        [89; 6],  // NORTH_POLE (from 0x59)
        [112; 6], // SOUTH_POLE (from 0x70)
        [113; 6], // WEST_POLE (from 0x71)
        [114; 6], // EAST_POLE (from 0x72)
        [672; 6], // PORTAL_BASE (dynamic, using formula)
        [190; 6], // PORTAL_BASE_AMETHYST (from 0xbe)
        [188; 6], // PORTAL_BASE_SAPPHIRE (from 0xbc)
        [478; 6], // PORTAL_BASE_EMERALD (from 0x1de)
        [700; 6], // PORTAL_BASE_RUBY (from 0x2ba)
        [699; 6], // PORTAL_BASE_DIAMOND (from 0x2b9)
        [741; 6], // COMPOST (from 0x2e5)
        [746; 6], // GRASS_COMPOST (from 0x2ea)
        [0; 6],   // SNOW_COMPOST (Missing in code)
        [97; 6],  // BASALT (from 0x61)
        [294; 6], // MINED_BASALT (from 0x126 from 0x31)
        [130; 6], // COPPER_BLOCK (from 0x82)
        [0; 6],   // TIN_BLOCK (Missing in code)
        [0; 6],   // BRONZE_BLOCK (Missing in code)
        [0; 6],   // IRON_BLOCK (Missing in code)
        [0; 6],   // STEEL_BLOCK (Missing in code)
        [0; 6],   // BLACK_SAND (Missing in code)
        [0; 6],   // BLACK_GLASS (Missing in code)
        [170; 6], // TRADE_PORTAL_BASE (from 0xaa)
        [171; 6], // TRADE_PORTAL_BASE_AMETHYST (from 0xab)
        [172; 6], // TRADE_PORTAL_BASE_SAPPHIRE (from 0xac)
        [173; 6], // TRADE_PORTAL_BASE_EMERALD (from 0xad)
        [174; 6], // TRADE_PORTAL_BASE_RUBY (from 0xae)
        [175; 6], // TRADE_PORTAL_BASE_DIAMOND (from 0xaf)
        [0; 6],   // Missing ID 66
        [117; 6], // PLATINUM_BLOCK (from 0x75)
        [0; 6],   // TITANIUM_BLOCK (Missing in code)
        [0; 6],   // CARBON_FIBER_BLOCK (Missing in code)
        [170; 6], // GRAVEL (from 0xaa - duplicates TRADE_PORTAL_BASE, which is probably a copy-paste error in the original code, but following the provided logic)
        [171; 6], // AMETHYST_BLOCK (from 0xab)
        [172; 6], // SAPPHIRE_BLOCK (from 0xac)
        [173; 6], // EMERALD_BLOCK (from 0xad)
        [174; 6], // RUBY_BLOCK (from 0xae)
        [175; 6], // DIAMOND_BLOCK (from 0xaf)
        [0; 6],   // PLASTER (Missing in code)
        [0; 6],   // LUMINOUS_PLASTER (Missing in code)
    ];
}

impl From<(BlockType, BlockContent)> for VoxelType {
    fn from(value: (BlockType, BlockContent)) -> Self {
        Self(match value {
            (BlockType::Air, _) => 0,
            (block_type, BlockContent::None) => block_type as u16,
            _ => 0,
        })
    }
}

impl VoxelType {
    fn fg_from_block_inner<'b>(block: Block<'b>) -> BhResult<Self> {
        Ok((
            block.fg()?,
            block.fg_content().unwrap_or(BlockContent::None),
        )
            .into())
    }

    pub fn fg_from_block<'b>(block: Block<'b>) -> Self {
        Self::fg_from_block_inner(block).unwrap_or(Self(0))
    }

    fn bg_from_block_inner<'b>(block: Block<'b>) -> BhResult<Self> {
        Ok((block.bg()?, BlockContent::None).into())
    }

    pub fn bg_from_block<'b>(block: Block<'b>) -> Self {
        Self::bg_from_block_inner(block).unwrap_or(Self(0))
    }
}

pub struct VoxelBuf {
    // Contains flattened block type, 512 * 32 * (32 * 32 * 3) blocks
    pub buf: wgpu::Buffer,
    chunk_keys: HashSet<ChunkCoord>,
}

impl VoxelBuf {
    const NUM_BLOCK_PER_CHUNK: usize = Chunk::NUM_BLOCK_PER_ROW * Chunk::NUM_BLOCK_PER_COL * 3;

    // Costly function - only call this once or VRAM nuked
    pub fn new(device: &wgpu::Device, world_width_macro: usize) -> Self {
        Self {
            buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Global Voxel Data Buffer"),
                contents: bytemuck::cast_slice(&vec![
                    VoxelType(0);
                    Self::NUM_BLOCK_PER_CHUNK
                        * 32
                        * world_width_macro
                ]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }),
            chunk_keys: HashSet::new(),
        }
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
                let bg_type = VoxelType::bg_from_block(block);
                let index = (y * Chunk::NUM_BLOCK_PER_ROW + x) * 3;
                blocks[index + 0] = bg_type;
                blocks[index + 1] = fg_type;
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
