use std::collections::HashSet;

use super::super::image_type::ImageType;
use egui_wgpu::wgpu::{self, util::DeviceExt};
use the_blockheads_tools_lib::{
    BhResult,
    game::{
        block::{Block, BlockContent, BlockType, BlockView},
        chunk::Chunk,
        coord::{ChunkBlockCoord, ChunkCoord},
    },
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
    pub(crate) const UV_AT_FACE: &[[ImageType; 6]] = {
        use ImageType::*;
        &[
            [Lava0; 6],                                                // Unknown
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
        ]
    };
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
            BlockContent::AppleTreeTrunkLeaf => 94,
            BlockContent::PineTreeLeaf => 95,
            BlockContent::PineTreeTrunk => 96,
            BlockContent::PineTreeTrunkLeaf => 97,
            BlockContent::MapleTreeLeaf => 98,
            BlockContent::MapleTreeTrunk => 99,
            BlockContent::MapleTreeTrunkLeaf => 100,
            BlockContent::MangoTreeLeaf => 101,
            BlockContent::MangoTreeTrunk => 102,
            BlockContent::MangoTreeTrunkLeaf => 103,
            BlockContent::CoconutTreeLeaf => 104,
            BlockContent::CoconutTreeTrunk => 105,
            BlockContent::OrangeTreeLeaf => 106,
            BlockContent::OrangeTreeTrunk => 107,
            BlockContent::OrangeTreeTrunkLeaf => 108,
            BlockContent::CherryTreeLeaf => 109,
            BlockContent::CherryTreeTrunk => 110,
            BlockContent::CherryTreeTrunkLeaf => 111,
            BlockContent::CoffeeTreeLeaf => 112,
            BlockContent::CoffeeTreeTrunk => 113,
            BlockContent::CoffeeTreeTrunkLeaf => 114,
            BlockContent::Cactus => 115,
            BlockContent::DeadCactus => 116,
            BlockContent::LimeTreeLeaf => 117,
            BlockContent::LimeTreeTrunk => 118,
            BlockContent::LimeTreeTrunkLeaf => 119,
            BlockContent::AmethystTreeTrunk => 120,
            BlockContent::AmethystTreeLeaf => 121,
            BlockContent::AmethystTreeTrunkLeaf => 122,
            BlockContent::SapphireTreeTrunk => 123,
            BlockContent::SapphireTreeLeaf => 124,
            BlockContent::SapphireTreeTrunkLeaf => 125,
            BlockContent::EmeraldTreeTrunk => 126,
            BlockContent::EmeraldTreeLeaf => 127,
            BlockContent::EmeraldTreeTrunkLeaf => 128,
            BlockContent::RubyTreeTrunk => 129,
            BlockContent::RubyTreeLeaf => 130,
            BlockContent::RubyTreeTrunkLeaf => 131,
            BlockContent::DiamondTreeTrunk => 132,
            BlockContent::DiamondTreeLeaf => 133,
            BlockContent::DiamondTreeTrunkLeaf => 134,
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

    pub fn set_chunk<I: Into<ChunkCoord>>(
        &mut self,
        queue: &wgpu::Queue,
        coord: I,
        chunk: &Chunk,
    ) -> BhResult<()> {
        let mut blocks = [VoxelType(0); Self::NUM_BLOCK_PER_CHUNK];
        for y in 0..Chunk::NUM_BLOCK_PER_COL {
            for x in 0..Chunk::NUM_BLOCK_PER_ROW {
                let block = chunk.block_at(ChunkBlockCoord::new(x as u8, y as u8)?);
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
                blocks[index] = bg_type;
                blocks[index + 1] = mg_type;
                blocks[index + 2] = fg_type;
            }
        }

        let chunk_coord: ChunkCoord = coord.into();
        let offset =
            (chunk_coord.x() * 32 + chunk_coord.y() as u32) * Self::NUM_BLOCK_PER_CHUNK as u32;

        queue.write_buffer(
            &self.buf,
            offset as u64 * size_of::<u16>() as u64,
            bytemuck::cast_slice(&blocks),
        );
        self.chunk_keys.insert(chunk_coord);
        Ok(())
    }
}
