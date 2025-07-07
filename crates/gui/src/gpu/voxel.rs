use std::collections::HashSet;

use egui_wgpu::wgpu::{self, util::DeviceExt};
use the_blockheads_tools_lib::{
    BhResult, Block, BlockContent, BlockType, BlockView, Chunk, ChunkBlockCoord, ChunkCoord,
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
    pub(crate) const UV_AT_FACE: &[[u32; 6]] = &[
        [0; 6],                                // NONE,
        [32; 6],                               // STONE,
        [0; 6],                                // AIR
        [14; 6],                               // WATER
        [110; 6],                              // ICE
        [129; 6],                              // SNOW
        [64; 6],                               // DIRT
        [65; 6],                               // SAND
        [65; 6],                               // SAND_
        [196; 6],                              // WOOD
        [33; 6],                               // MINED_STONE
        [34; 6],                               // RED_BRICK
        [66; 6],                               // LIMESTONE
        [35; 6],                               // MINED_LIMESTONE
        [67; 6],                               // MARBLE
        [76; 6],                               // MINED_MARBLE
        [77; 6],                               // TIME_CRYSTAL
        [79; 6],                               // SAND_STONE
        [80; 6],                               // MINED_SAND_STONE
        [81; 6],                               // RED_MARBLE
        [82; 6],                               // MINED_RED_MARBLE
        [0; 6],                                // Missing ID 21
        [0; 6],                                // Missing ID 22
        [0; 6],                                // Missing ID 23
        [108; 6],                              // GLASS
        [132; 6],                              // SPAWN_PORTAL_BASE
        [288; 6],                              // GOLD_BLOCK
        [160, 160, 161, 64, 160, 160],         // GRASS_DIRT
        [162, 162, 163, 64, 162, 162],         // SNOW_DIRT
        [86; 6],                               // LAPIS_LAZULI
        [87; 6],                               // MINED_LAPIS_LAZULI
        [46; 6],                               // LAVA
        [164; 6],                              // REINFORCED_PLATFORM
        [165; 6],                              // SPAWN_PORTAL_BASE_AMETHYST
        [166; 6],                              // SPAWN_PORTAL_BASE_SAPPHIRE
        [167; 6],                              // SPAWN_PORTAL_BASE_EMERALD
        [168; 6],                              // SPAWN_PORTAL_BASE_RUBY
        [169; 6],                              // SPAWN_PORTAL_BASE_DIAMOND
        [89; 6],                               // NORTH_POLE
        [112; 6],                              // SOUTH_POLE
        [113; 6],                              // WEST_POLE
        [114; 6],                              // EAST_POLE
        [672; 6],                              // PORTAL_BASE
        [190; 6],                              // PORTAL_BASE_AMETHYST
        [188; 6],                              // PORTAL_BASE_SAPPHIRE
        [478; 6],                              // PORTAL_BASE_EMERALD
        [700; 6],                              // PORTAL_BASE_RUBY
        [699; 6],                              // PORTAL_BASE_DIAMOND
        [741; 6],                              // COMPOST
        [746; 6],                              // GRASS_COMPOST
        [0; 6],                                // SNOW_COMPOST
        [97; 6],                               // BASALT
        [294; 6],                              // MINED_BASALT
        [130; 6],                              // COPPER_BLOCK
        [0; 6],                                // TIN_BLOCK
        [0; 6],                                // BRONZE_BLOCK
        [0; 6],                                // IRON_BLOCK
        [0; 6],                                // STEEL_BLOCK
        [89; 6],                               // BLACK_SAND
        [0; 6],                                // BLACK_GLASS
        [170; 6],                              // TRADE_PORTAL_BASE
        [171; 6],                              // TRADE_PORTAL_BASE_AMETHYST
        [172; 6],                              // TRADE_PORTAL_BASE_SAPPHIRE
        [173; 6],                              // TRADE_PORTAL_BASE_EMERALD
        [174; 6],                              // TRADE_PORTAL_BASE_RUBY
        [175; 6],                              // TRADE_PORTAL_BASE_DIAMOND
        [0; 6],                                // Missing ID 66
        [117; 6],                              // PLATINUM_BLOCK
        [0; 6],                                // TITANIUM_BLOCK
        [0; 6],                                // CARBON_FIBER_BLOCK
        [170; 6],                              // GRAVEL
        [171; 6],                              // AMETHYST_BLOCK
        [172; 6],                              // SAPPHIRE_BLOCK
        [173; 6],                              // EMERALD_BLOCK
        [174; 6],                              // RUBY_BLOCK
        [175; 6],                              // DIAMOND_BLOCK
        [0; 6],                                // PLASTER
        [0; 6],                                // LUMINOUS_PLASTER
        [0x122; 6],                            // DIRT + CLAY
        [0x120; 6],                            // DIRT + FLINT
        [0x123, 0x123, 161, 64, 0x123, 0x123], // GRASS DIRT + CLAY
        [0x121, 0x121, 161, 64, 0x121, 0x121], // GRASS DIRT + FLINT
        [0x125, 0x125, 161, 64, 0x125, 0x125], // SNOW DIRT + CLAY
        [0x124, 0x124, 161, 64, 0x124, 0x124], // SNOW DIRT + FLINT
        [1; 6],                                // STONE + COPPER ORE
        [3; 6],                                // STONE + TIN ORE
        [2; 6],                                // STONE + IRON ORE
        [83; 6],                               // STONE + COAL
        [84; 6],                               // STONE + GOLD NUGGETS
        [157; 6],                              // STONE + PLATINUM ORE
        [219; 6],                              // STONE + TITANIUM ORE
        [78; 6],                               // LIMESTONE + OIL
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
                    VoxelType(0);
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
                VoxelType(0);
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
