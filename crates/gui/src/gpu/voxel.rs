use crate::image_type::ImageType;
use the_blockheads_tools_lib::game::block::{
    Block, BlockContentType, BlockError, BlockType, BlockView,
};
// use snafu::prelude::*;

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
    pub(crate) const UV_AT_FACE: &[[ImageType; 6]] = {
        use ImageType::*;
        &[
            [WorkbenchTool5Top; 6],                                    // Unknown
            [Stone; 6],                                                // Stone
            [Air; 6],                                                  // Air
            [Water0; 6],                                               // Water
            [Ice; 6],                                                  // Ice
            [Snow; 6],                                                 // Snow
            [Dirt; 6],                                                 // Dirt
            [Sand; 6],                                                 // Sand
            [Sand; 6],                                                 // MinedSand
            [Wood; 6],                                                 // Wood
            [MinedStone; 6],                                           // MinedStone
            [RedBrick; 6],                                             // RedBrick
            [Limestone; 6],                                            // Limestone
            [MinedLimestone; 6],                                       // MinedLimestone
            [Marble; 6],                                               // Marble
            [MinedMarble; 6],                                          // MinedMarble
            [TimeCrystal; 6],                                          // TimeCrystal
            [SandStone; 6],                                            // SandStone
            [MinedSandStone; 6],                                       // MinedSandStone
            [RedMarble; 6],                                            // RedMarble
            [MinedRedMarble; 6],                                       // MinedRedMarble
            [Air; 6],                                                  // Missing id 21
            [Air; 6],                                                  // Missing id 22
            [Air; 6],                                                  // Missing id 23
            [Glass; 6],                                                // Glass
            [SpawnPortalBase; 6],                                      // SpawnPortalBase
            [GoldBlock; 6],                                            // GoldBlock
            [GrassDirt, GrassDirt, Grass, Dirt, GrassDirt, GrassDirt], // GrassDirt
            [
                SnowGrassDirt,
                SnowGrassDirt,
                SnowGrass,
                Dirt,
                SnowGrassDirt,
                SnowGrassDirt,
            ], // SnowDirt
            [LapisLazuli; 6],                                          // LapisLazuli
            [MinedLapisLazuli; 6],                                     // MinedLapisLazuli
            [Lava0; 6],                                                // Lava
            [ReinforcedPlatform; 6],                                   // ReinforcedPlatform
            [SpawnPortalBaseAmethyst; 6],                              // SpawnPortalBaseAmethyst
            [SpawnPortalBaseSapphire; 6],                              // SpawnPortalBaseSapphire
            [SpawnPortalBaseEmerald; 6],                               // SpawnPortalBaseEmerald
            [SpawnPortalBaseRuby; 6],                                  // SpawnPortalBaseRuby
            [SpawnPortalBaseDiamond; 6],                               // SpawnPortalBaseDiamond
            [NorthPole; 6],                                            // NorthPole
            [SouthPole; 6],                                            // SouthPole
            [EquatorPole; 6],                                          // WestPole
            [EquatorPole; 6],                                          // EastPole
            [PortalBase; 6],                                           // PortalBase
            [PortalBaseAmethyst; 6],                                   // PortalBaseAmethyst
            [PortalBaseSapphire; 6],                                   // PortalBaseSapphire
            [PortalBaseEmerald; 6],                                    // PortalBaseEmerald
            [PortalBaseRuby; 6],                                       // PortalBaseRuby
            [PortalBaseDiamond; 6],                                    // PortalBaseDiamond
            [Compost; 6],                                              // Compost
            [
                CompostSide,
                CompostSide,
                CompostGrass,
                Compost,
                CompostSide,
                CompostSide,
            ], // GrassCompost
            [
                SnowCompostSide,
                SnowCompostSide,
                SnowGrass,
                Compost,
                SnowCompostSide,
                SnowCompostSide,
            ], // SnowCompost
            [Basalt; 6],                                               // Basalt
            [Basalt; 6],                                               // MinedBasalt
            [CopperBlock; 6],                                          // CopperBlock
            [TinBlock; 6],                                             // TinBlock
            [BronzeBlock; 6],                                          // BronzeBlock
            [IronBlock; 6],                                            // IronBlock
            [SteelBlock; 6],                                           // SteelBlock
            [BlackSand; 6],                                            // BlackSand
            [BlackGlass; 6],                                           // BlackGlass
            [PortalBase; 6],                                           // TradePortalBase
            [PortalBaseAmethyst; 6],                                   // TradePortalBaseAmethyst
            [PortalBaseSapphire; 6],                                   // TradePortalBaseSapphire
            [PortalBaseEmerald; 6],                                    // TradePortalBaseEmerald
            [PortalBaseRuby; 6],                                       // TradePortalBaseRuby
            [PortalBaseDiamond; 6],                                    // TradePortalBaseDiamond
            [Air; 6],                                                  // Missing id 66
            [PlatinumBlock0; 6],                                       // PlatinumBlock
            [TitaniumBlock; 6],                                        // TitaniumBlock
            [CarbonFiberBlock; 6],                                     // CarbonFiberBlock
            [Gravel; 6],                                               // Gravel
            [AmethystBlock; 6],                                        // AmethystBlock
            [SapphireBlock; 6],                                        // SapphireBlock
            [EmeraldBlock; 6],                                         // EmeraldBlock
            [RubyBlock; 6],                                            // RubyBlock
            [DiamondBlock; 6],                                         // DiamondBlock
            [Plaster; 6],                                              // Plaster
            [LuminousPlaster; 6],                                      // LuminousPlaster
            [DirtClay; 6],                                             // Dirt + Clay
            [DirtFlint; 6],                                            // Dirt + Flint
            [
                GrassDirtClay,
                GrassDirtClay,
                Grass,
                Dirt,
                GrassDirtClay,
                GrassDirtClay,
            ], // GrassDirt + Clay
            [
                GrassDirtFlint,
                GrassDirtFlint,
                Grass,
                Dirt,
                GrassDirtFlint,
                GrassDirtFlint,
            ], // GrassDirt + Flint
            [
                SnowGrassDirtClay,
                SnowGrassDirtClay,
                SnowGrass,
                Dirt,
                SnowGrassDirtClay,
                SnowGrassDirtClay,
            ], // SnowDirt + Clay
            [
                SnowGrassDirtFlint,
                SnowGrassDirtFlint,
                SnowGrass,
                Dirt,
                SnowGrassDirtFlint,
                SnowGrassDirtFlint,
            ], // SnowDirt + Flint
            [StoneCopper; 6],                                          // Stone + CopperOre
            [StoneTin; 6],                                             // Stone + TinOre
            [StoneIron; 6],                                            // Stone + IronOre
            [StoneCoal; 6],                                            // Stone + Coal
            [StoneGold; 6],                                            // Stone + GoldNuggets
            [StonePlatinum; 6],                                        // Stone + PlatinumOre
            [StoneTitanium; 6],                                        // Stone + TitaniumOre
            [LimestoneOil; 6],                                         // Limestone + Oil
            [AppleTreeLeaf; 6],                                        // AppleTreeLeaf
            [
                AppleTreeTrunk,
                AppleTreeTrunk,
                TrunkTop,
                TrunkTop,
                AppleTreeTrunk,
                AppleTreeTrunk,
            ], // AppleTreeTrunk
            [AppleTreeTrunkLeaf; 6],                                   // AppleTreeTrunkLeaf
            [PineTreeLeaf; 6],                                         // PineTreeLeaf
            [Trunk, Trunk, TrunkTop, TrunkTop, Trunk, Trunk],          // PineTreeTrunk,
            [PineTreeTrunkLeaf; 6],                                    // PineTreeTrunkLeaf
            [MapleTreeLeaf; 6],                                        // MapleTreeLeaf
            [
                MapleTreeTrunk,
                MapleTreeTrunk,
                TrunkTop,
                TrunkTop,
                MapleTreeTrunk,
                MapleTreeTrunk,
            ], // MapleTreeTrunk
            [MapleTreeTrunkLeaf; 6],                                   // MapleTreeTrunkLeaf
            [MangoTreeLeaf; 6],                                        // MangoTreeLeaf
            [Trunk, Trunk, TrunkTop, TrunkTop, Trunk, Trunk],          // MangoTreeTrunk
            [MangoTreeTrunkLeaf; 6],                                   // MangoTreeTrunkLeaf
            [CoconutTreeLeaf; 6],                                      // CoconutTreeLeaf
            [CoconutTreeTrunk; 6],                                     // CoconutTreeTrunk
            [OrangeTreeLeaf; 6],                                       // OrangeTreeLeaf
            [
                OrangeTreeTrunk,
                OrangeTreeTrunk,
                TrunkTop,
                TrunkTop,
                OrangeTreeTrunk,
                OrangeTreeTrunk,
            ], // OrangeTreeTrunk
            [OrangeTreeTrunkLeaf; 6],                                  // OrangeTreeTrunkLeaf
            [CherryTreeLeaf; 6],                                       // CherryTreeLeaf
            [
                CherryTreeTrunk,
                CherryTreeTrunk,
                TrunkTop,
                TrunkTop,
                CherryTreeTrunk,
                CherryTreeTrunk,
            ], // CherryTreeTrunk
            [CherryTreeTrunkLeaf; 6],                                  // CherryTreeTrunkLeaf
            [CoffeeTreeLeaf; 6],                                       // CoffeeTreeLeaf
            [
                CoffeeTreeTrunk,
                CoffeeTreeTrunk,
                TrunkTop,
                TrunkTop,
                CoffeeTreeTrunk,
                CoffeeTreeTrunk,
            ], // CoffeeTreeTrunk
            [CoffeeTreeTrunkLeaf; 6],                                  // CoffeeTreeTrunkLeaf
            [Cactus; 6],                                               // Cactus
            [DeadCactus; 6],                                           // DeadCactus
            [LimeTreeLeaf; 6],                                         // LimeTreeLeaf
            [
                LimeTreeTrunk,
                LimeTreeTrunk,
                TrunkTop,
                TrunkTop,
                LimeTreeTrunk,
                LimeTreeTrunk,
            ], // LimeTreeTrunk
            [LimeTreeTrunkLeaf; 6],                                    // LimeTreeTrunkLeaf
            [
                AmethystTreeTrunk,
                AmethystTreeTrunk,
                TrunkTop,
                TrunkTop,
                AmethystTreeTrunk,
                AmethystTreeTrunk,
            ], // AmethystTreeTrunk
            [AmethystTreeLeaf; 6],                                     // AmethystTreeLeaf
            [AmethystTreeTrunkLeaf; 6],                                // AmethystTreeTrunkLeaf
            [
                SapphireTreeTrunk,
                SapphireTreeTrunk,
                TrunkTop,
                TrunkTop,
                SapphireTreeTrunk,
                SapphireTreeTrunk,
            ], // SapphireTreeTrunk
            [SapphireTreeLeaf; 6],                                     // SapphireTreeLeaf
            [SapphireTreeTrunkLeaf; 6],                                // SapphireTreeTrunkLeaf
            [
                EmeraldTreeTrunk,
                EmeraldTreeTrunk,
                TrunkTop,
                TrunkTop,
                EmeraldTreeTrunk,
                EmeraldTreeTrunk,
            ], // EmeraldTreeTrunk
            [EmeraldTreeLeaf; 6],                                      // EmeraldTreeLeaf
            [EmeraldTreeTrunkLeaf; 6],                                 // EmeraldTreeTrunkLeaf
            [
                RubyTreeTrunk,
                RubyTreeTrunk,
                TrunkTop,
                TrunkTop,
                RubyTreeTrunk,
                RubyTreeTrunk,
            ], // RubyTreeTrunk
            [RubyTreeLeaf; 6],                                         // RubyTreeLeaf
            [RubyTreeTrunkLeaf; 6],                                    // RubyTreeTrunkLeaf
            [
                DiamondTreeTrunk,
                DiamondTreeTrunk,
                TrunkTop,
                TrunkTop,
                DiamondTreeTrunk,
                DiamondTreeTrunk,
            ], // DiamondTreeTrunk
            [DiamondTreeLeaf; 6],                                      // DiamondTreeLeaf
            [DiamondTreeTrunkLeaf; 6],                                 // DiamondTreeTrunkLeaf
            [DeadTreeLeaf; 6],                                         // Any dead tree leaf
            [
                DeadTrunk,
                DeadTrunk,
                DeadTrunkTop,
                DeadTrunkTop,
                DeadTrunk,
                DeadTrunk,
            ], // Any dead tree trunk
            [
                ChestGold,
                ChestGold,
                ChestGoldTop,
                ChestGoldTop,
                ChestGold,
                ChestGold,
            ], // GoldChest
        ]
    };
}

impl VoxelType {
    fn fg_from_block_inner<'b>(block: BlockView<'b>) -> Result<Self, BlockError> {
        Ok(Self(match (block.fg()?, block.content()?) {
            (BlockType::Air, _) => 2,
            (BlockType::Snow, _) => 5,
            (block_type, BlockContentType::Nothing) => block_type as u16,
            (BlockType::Dirt, BlockContentType::Clay) => 78,
            (BlockType::Dirt, BlockContentType::Flint) => 79,
            (BlockType::GrassDirt, BlockContentType::Clay) => 80,
            (BlockType::GrassDirt, BlockContentType::Flint) => 81,
            (BlockType::SnowDirt, BlockContentType::Clay) => 82,
            (BlockType::SnowDirt, BlockContentType::Flint) => 83,
            (BlockType::Stone, BlockContentType::CopperOre) => 84,
            (BlockType::Stone, BlockContentType::TinOre) => 85,
            (BlockType::Stone, BlockContentType::IronOre) => 86,
            (BlockType::Stone, BlockContentType::Coal) => 87,
            (BlockType::Stone, BlockContentType::GoldNuggets) => 88,
            (BlockType::Stone, BlockContentType::PlatinumOre) => 89,
            (BlockType::Stone, BlockContentType::TitaniumOre) => 90,
            (BlockType::Limestone, BlockContentType::Oil) => 91,
            _ => 0,
        }))
    }

    pub fn fg_from_block<'b>(block: BlockView<'b>) -> Self {
        Self::fg_from_block_inner(block).unwrap_or(Self::UNKNOWN)
    }

    fn mg_from_block_inner<'b>(block: BlockView<'b>) -> Result<Self, BlockError> {
        Ok(Self(match block.content()? {
            BlockContentType::Nothing => 2,
            BlockContentType::AppleTreeLeaf => 92,
            BlockContentType::AppleTreeTrunk => 93,
            BlockContentType::AppleTreeTrunkLeaf => 94,
            BlockContentType::PineTreeLeaf => 95,
            BlockContentType::PineTreeTrunk => 96,
            BlockContentType::PineTreeTrunkLeaf => 97,
            BlockContentType::MapleTreeLeaf => 98,
            BlockContentType::MapleTreeTrunk => 99,
            BlockContentType::MapleTreeTrunkLeaf => 100,
            BlockContentType::MangoTreeLeaf => 101,
            BlockContentType::MangoTreeTrunk => 102,
            BlockContentType::MangoTreeTrunkLeaf => 103,
            BlockContentType::CoconutTreeLeaf => 104,
            BlockContentType::CoconutTreeTrunk => 105,
            BlockContentType::OrangeTreeLeaf => 106,
            BlockContentType::OrangeTreeTrunk => 107,
            BlockContentType::OrangeTreeTrunkLeaf => 108,
            BlockContentType::CherryTreeLeaf => 109,
            BlockContentType::CherryTreeTrunk => 110,
            BlockContentType::CherryTreeTrunkLeaf => 111,
            BlockContentType::CoffeeTreeLeaf => 112,
            BlockContentType::CoffeeTreeTrunk => 113,
            BlockContentType::CoffeeTreeTrunkLeaf => 114,
            BlockContentType::Cactus => 115,
            BlockContentType::DeadCactus => 116,
            BlockContentType::LimeTreeLeaf => 117,
            BlockContentType::LimeTreeTrunk => 118,
            BlockContentType::LimeTreeTrunkLeaf => 119,
            BlockContentType::AmethystTreeTrunk => 120,
            BlockContentType::AmethystTreeLeaf => 121,
            BlockContentType::AmethystTreeTrunkLeaf => 122,
            BlockContentType::SapphireTreeTrunk => 123,
            BlockContentType::SapphireTreeLeaf => 124,
            BlockContentType::SapphireTreeTrunkLeaf => 125,
            BlockContentType::EmeraldTreeTrunk => 126,
            BlockContentType::EmeraldTreeLeaf => 127,
            BlockContentType::EmeraldTreeTrunkLeaf => 128,
            BlockContentType::RubyTreeTrunk => 129,
            BlockContentType::RubyTreeLeaf => 130,
            BlockContentType::RubyTreeTrunkLeaf => 131,
            BlockContentType::DiamondTreeTrunk => 132,
            BlockContentType::DiamondTreeLeaf => 133,
            BlockContentType::DiamondTreeTrunkLeaf => 134,
            BlockContentType::DeadPineTreeLeaf
            | BlockContentType::DeadOrangeTreeLeaf
            | BlockContentType::DeadCherryTreeLeaf
            | BlockContentType::DeadLimeTreeLeaf => 135,
            BlockContentType::DeadPineTreeTrunk
            | BlockContentType::DeadOrangeTreeTrunk
            | BlockContentType::DeadCherryTreeTrunk
            | BlockContentType::DeadLimeTreeTrunk => 136,
            BlockContentType::GoldChest => 137,
            _ => 0,
        }))
    }

    pub fn mg_from_block<'b>(block: BlockView<'b>) -> Self {
        Self::mg_from_block_inner(block).unwrap_or(Self::UNKNOWN)
    }

    fn bg_from_block_inner<'b>(block: BlockView<'b>) -> Result<Self, BlockError> {
        Ok(Self(block.bg()? as u16))
    }

    pub fn bg_from_block<'b>(block: BlockView<'b>) -> Self {
        Self::bg_from_block_inner(block).unwrap_or(Self::UNKNOWN)
    }
}

pub mod voxel_util {
    use super::VoxelType;
    use eframe::wgpu::{self, util::DeviceExt};
    use the_blockheads_tools_lib::game::{
        chunk::{Chunk, ChunkView, Chunks},
        coord::{ChunkBlockCoord, ChunkCoord},
    };

    const NUM_BLOCK_PER_CHUNK: usize = Chunk::NUM_BLOCK_PER_ROW * Chunk::NUM_BLOCK_PER_COL * 3; // 3 layers

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
        vec![VoxelType::AIR; NUM_BLOCK_PER_CHUNK * Chunks::NUM_CHUNK_PER_COL * world_width_macro]
    }

    fn fill_chunk_voxel(chunk: ChunkView<'_>, chunk_voxel: &mut [VoxelType]) {
        for y in 0..Chunk::NUM_BLOCK_PER_COL {
            for x in 0..Chunk::NUM_BLOCK_PER_ROW {
                let block = chunk.block_at(
                    ChunkBlockCoord::new(x as u8, y as u8).expect("x and y must be within limit"),
                );
                let fg_type = VoxelType::fg_from_block(block);
                let mg_type = if fg_type == VoxelType::AIR {
                    VoxelType::mg_from_block(block)
                } else {
                    fg_type
                };
                let mut bg_type = VoxelType::bg_from_block(block);
                if bg_type == VoxelType::AIR && fg_type != VoxelType::AIR {
                    bg_type = fg_type;
                }
                let index = (y * Chunk::NUM_BLOCK_PER_ROW + x) * 3;
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

    #[allow(dead_code)] // will be used once we support edit modes
    pub fn set_chunk<I: Into<ChunkCoord>>(
        queue: &wgpu::Queue,
        voxel_buffer: &wgpu::Buffer,
        coord: I,
        chunk: &Chunk,
    ) {
        let mut blocks = [VoxelType(0); NUM_BLOCK_PER_CHUNK];
        fill_chunk_voxel(chunk.view(), &mut blocks);

        let chunk_coord: ChunkCoord = coord.into();
        let offset = (chunk_coord.x() * 32 + chunk_coord.y() as u32) * NUM_BLOCK_PER_CHUNK as u32;

        queue.write_buffer(
            voxel_buffer,
            offset as u64 * size_of::<u16>() as u64,
            bytemuck::cast_slice(&blocks),
        );
    }
}
