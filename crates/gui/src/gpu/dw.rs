use super::{super::image_type::ImageType, voxel::BlockUv};
use eframe::wgpu::{self, util::DeviceExt};
use snafu::Snafu;
use std::{
    collections::HashMap,
    ops::{AddAssign, Mul},
};
use the_blockheads_tools_lib::game::{
    chunk::Chunk,
    coord::{BlockCoord, ChunkBlockCoord, ChunkCoord, CoordError},
    dynamic_object::{
        DynamicObjectType,
        craft::{Door, Torch},
        workbench::WorkbenchType,
    },
    dynamic_world::ChunkDynamicObjects,
    item::ItemType,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DwIconVertex {
    pub position: [f32; 2],
}

impl DwIconVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }

    pub const VERTICES: &[Self] = &[
        Self {
            position: [-0.5, -0.5],
        }, // bottom-left
        Self {
            position: [0.5, -0.5],
        }, // bottom-right
        Self {
            position: [0.5, 0.5],
        }, // top-right
        Self {
            position: [-0.5, 0.5],
        }, // top-left
    ];

    pub const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct PackedChunkCoord {
    coord: u32,
}

impl PackedChunkCoord {
    const Y_NUM_BITS: usize = 5;
    const Y_MASK: u32 = (1 << Self::Y_NUM_BITS) - 1;
}

impl From<ChunkCoord> for PackedChunkCoord {
    fn from(value: ChunkCoord) -> Self {
        Self {
            coord: (value.x() << Self::Y_NUM_BITS) | value.y() as u32,
        }
    }
}

impl From<PackedChunkCoord> for ChunkCoord {
    fn from(value: PackedChunkCoord) -> Self {
        Self::new(
            value.coord >> PackedChunkCoord::Y_NUM_BITS,
            (value.coord & PackedChunkCoord::Y_MASK) as u8, // Y_MASK is guaranteed to be smaller than u8
        ).expect("type safety violation: PackedChunkCoord must come from ChunkCoord instance, reconstructing should not violate invariance")
    }
}

#[derive(Clone, Copy)]
pub struct DwItem {
    position: [f32; 2],
    item_type: ItemType,
    block_uv: Option<BlockUv>,
}

impl DwItem {
    fn map_deprecated(item_type: ItemType) -> ItemType {
        match item_type {
            ItemType::WovenFlaxMat
            | ItemType::YellowFlaxMat
            | ItemType::RedFlaxMat
            | ItemType::DeprecatedFood
            | ItemType::DeprecatedMango
            | ItemType::DeprecatedStonePickaxe
            | ItemType::DeprecatedCopperIngot
            | ItemType::DeprecatedBronzeMachete
            | ItemType::DeprecatedIronSword => ItemType::Unknown,
            _ => item_type,
        }
    }

    fn default_block_uv(item_type: ItemType) -> Option<BlockUv> {
        use BlockUv::*;
        use ImageType::*;
        Some(match item_type {
            ItemType::Stone => All(Stone),
            ItemType::Kiln => TopSide {
                top: KilnTop,
                side: Kiln,
            },
            ItemType::Brick => All(RedBrick),
            ItemType::Limestone => All(Limestone),
            ItemType::MinedLimestone => All(MinedLimestone),
            ItemType::Marble => All(Marble),
            ItemType::MinedMarble => All(MinedMarble),
            ItemType::Furnace => TopSide {
                top: Furnace1Top,
                side: Furnace1,
            },
            ItemType::WoodworkBench => TopSide {
                top: WorkbenchWood1Top,
                side: WorkbenchWood1,
            },
            ItemType::TaylorsBench => TopSide {
                top: WorkbenchWeave1Top,
                side: WorkbenchWeave1,
            },
            ItemType::Press => TopSide {
                top: WorkbenchPress1Top,
                side: WorkbenchPress1,
            },
            ItemType::Sandstone => All(SandStone),
            ItemType::MinedSandstone => All(MinedSandStone),
            ItemType::RedMarble => All(RedMarble),
            ItemType::MinedRedMarble => All(MinedRedMarble),
            // These blocks are rendered as stone in game
            ItemType::Glass => All(Glass),
            ItemType::Chest => TopSide {
                top: ChestTop,
                side: Chest,
            },
            ItemType::GoldBlock => All(GoldBlock),
            ItemType::Rock => All(Stone),
            ItemType::Dirt => All(Dirt),
            ItemType::Wood => All(Wood),
            ItemType::WorkBench => TopSide {
                top: WorkbenchLevel1Top,
                side: WorkbenchLevel1,
            },
            ItemType::Sand => All(Sand),
            ItemType::ToolBench => TopSide {
                top: WorkbenchTool1Top,
                side: WorkbenchTool1,
            },
            ItemType::LapisLazuli => All(LapisLazuli),
            ItemType::MinedLapisLazuli => All(MinedLapisLazuli),
            ItemType::CraftBench => TopSide {
                top: CraftBenchLevel1Top,
                side: CraftBenchLevel1,
            },
            ItemType::MixingBench => TopSide {
                top: MixBenchLevel1Top,
                side: MixBenchLevel1,
            },
            ItemType::ReinforcedPlatform => All(ReinforcedPlatform),
            ItemType::Ice => All(Ice),
            ItemType::DyeBench => TopSide {
                top: DyeBenchLevel1Top,
                side: DyeBenchLevel1,
            },
            ItemType::Compost => All(Compost),
            ItemType::Basalt => All(Basalt),
            ItemType::MinedBasalt => All(Basalt),
            ItemType::Safe => All(Safe),
            ItemType::CopperBlock => All(CopperBlock),
            ItemType::TinBlock => All(TinBlock),
            ItemType::BronzeBlock => All(BronzeBlock),
            ItemType::IronBlock => All(IronBlock),
            ItemType::SteelBlock => All(SteelBlock),
            ItemType::MetalworkBench => TopSide {
                top: MetalworkBenchLevel1Top,
                side: MetalworkBenchLevel1,
            },
            ItemType::GoldenChest => TopSide {
                top: ChestGoldTop,
                side: ChestGold,
            },
            ItemType::PortalChest => TopSide {
                top: ChestPortalTop,
                side: ChestPortal,
            },
            ItemType::BlackSand => All(BlackSand),
            ItemType::BlackGlass => All(BlackGlass),
            ItemType::SteamGenerator => TopSide {
                top: SteamGeneratorTop,
                side: SteamGenerator,
            },
            ItemType::ElectricKiln => TopSide {
                top: ElectricKilnTop,
                side: ElectricKiln,
            },
            ItemType::ElectricFurnace => TopSide {
                top: ElectricFurnaceTop,
                side: ElectricFurnace,
            },
            ItemType::ElectricMetalworkBench => TopSide {
                top: ElectricMetalworkBenchTop,
                side: ElectricMetalworkBench,
            },
            ItemType::ElectricStove => TopSide {
                top: ElectricStoveTop,
                side: ElectricStove,
            },
            ItemType::SolarPanel => TopSide {
                top: SolarPanelTop,
                side: SolarPanel,
            },
            ItemType::Flywheel => TopSide {
                top: FlywheelTop,
                side: Flywheel,
            },
            ItemType::ArmorBench => TopSide {
                top: ArmorBenchLevel1Top,
                side: ArmorBenchLevel1,
            },
            ItemType::TrainYard => TopSide {
                top: TrainYardTop,
                side: TrainYard,
            },
            ItemType::BuildersBench => TopSide {
                top: BuildersBenchLevel1Top,
                side: BuildersBenchLevel1,
            },
            ItemType::ElevatorShaft => All(ElevatorShaft),
            ItemType::ElectricElevatorMotor => All(ElevatorMotor),
            ItemType::PlatiumBlock => All(PlatinumBlock0),
            ItemType::CarbonFiberBlock => All(CarbonFiberBlock),
            ItemType::TitaniumBlock => All(TitaniumBlock),
            ItemType::DeprecatedIronSword => None?,
            ItemType::ElectricPress => TopSide {
                top: ElectricPressTop,
                side: ElectricPress,
            },
            ItemType::Gravel => All(Gravel),
            ItemType::CompostBin => TopSide {
                top: CompostBinTop,
                side: CompostBin,
            },
            ItemType::EggExtractor => TopSide {
                top: EggExtractorTop,
                side: EggExtractor,
            },
            ItemType::PizzaOven => TopSide {
                top: PizzaOvenTop,
                side: PizzaOven,
            },
            ItemType::AmethystBlock => All(AmethystBlock),
            ItemType::SapphireBlock => All(SapphireBlock),
            ItemType::EmeraldBlock => All(EmeraldBlock),
            ItemType::RubyBlock => All(RubyBlock),
            ItemType::DiamondBlock => All(DiamondBlock),
            ItemType::Plaster => All(Plaster),
            ItemType::FeederChest => TopSide {
                top: ChestFeederTop,
                side: ChestFeeder,
            },
            ItemType::LuminousPlaster => All(LuminousPlaster),
            _ => None?,
        })
    }

    pub fn new(position: [f32; 2], item_type: ItemType) -> Self {
        let [x, y] = position;
        let item_type = Self::map_deprecated(item_type);
        let block_uv = Self::default_block_uv(item_type);
        Self {
            position: [x, y + 0.5],
            item_type,
            block_uv,
        }
    }

    pub fn instance(self, id: DwChunkObjId, coord: ChunkCoord) -> DwItemInstanceRaw {
        let is_block = self.block_uv.is_some() as u16;
        let [top, side] = self
            .block_uv
            .map(|block_uv| [block_uv.up(), block_uv.side()].map(|v| v as u16))
            .unwrap_or([0; 2]);
        DwItemInstanceRaw {
            position: self.position,
            item_type: self.item_type as u16,
            is_block,
            top,
            side,
            raw_id: id.raw_id(),
            chunk: coord.into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DwItemInstanceRaw {
    pub position: [f32; 2],
    pub item_type: u16,

    // If item_type highest bit is true, use these instead
    pub is_block: u16,
    pub top: u16,
    pub side: u16,

    pub raw_id: u32,
    pub chunk: PackedChunkCoord,
}

impl DwItemInstanceRaw {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        1 => Float32x2,
        2 => Uint32,
        3 => Uint32,
        4 => Uint32,
        5 => Uint32,
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DwItemInstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub trait DwQuad {
    fn quad(&self) -> [[f32; 3]; 4];
    fn normal(&self) -> [f32; 3];
    fn uv_min_max(&self) -> [[f32; 2]; 2];

    fn vertices(&self, id: &DwChunkObjId, chunk_coord: &ChunkCoord) -> ([DwVertex; 4], [u32; 6]) {
        let [bottom_left, bottom_right, top_right, top_left] = self.quad();

        let normal = self.normal();
        let [[u_min, v_min], [u_max, v_max]] = self.uv_min_max();

        let raw_id = id.raw_id();
        let chunk = (*chunk_coord).into();

        (
            [
                DwVertex {
                    raw_id,
                    chunk,
                    position: bottom_left.map(|v| v),
                    normal,
                    tex_coords: [u_min, v_max],
                },
                DwVertex {
                    raw_id,
                    chunk,
                    position: bottom_right.map(|v| v),
                    normal,
                    tex_coords: [u_max, v_max],
                },
                DwVertex {
                    raw_id,
                    chunk,
                    position: top_right.map(|v| v),
                    normal,
                    tex_coords: [u_max, v_min],
                },
                DwVertex {
                    raw_id,
                    chunk,
                    position: top_left.map(|v| v),
                    normal,
                    tex_coords: [u_min, v_min],
                },
            ],
            [0, 1, 2, 0, 2, 3],
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FaceDirection {
    Up,
    Down,
    Left,
    Right,
    Front,
}

pub struct DwFace {
    face_direction: FaceDirection,
    bottom_left: [f32; 3],
    size: [f32; 2],
    uv_min_max: [[f32; 2]; 2],
}

impl DwQuad for DwFace {
    fn quad(&self) -> [[f32; 3]; 4] {
        let [x, y, z] = self.bottom_left;
        let [w, h] = self.size;
        match self.face_direction {
            FaceDirection::Up | FaceDirection::Down => {
                [[x, y, z], [x + w, y, z], [x + w, y, z - h], [x, y, z - h]]
            }
            FaceDirection::Left | FaceDirection::Right => {
                [[x, y, z], [x, y, z + w], [x, y + h, z + w], [x, y + h, z]]
            }
            FaceDirection::Front => [[x, y, z], [x + w, y, z], [x + w, y + h, z], [x, y + h, z]],
        }
    }

    fn normal(&self) -> [f32; 3] {
        match self.face_direction {
            FaceDirection::Up => [0, 1, 0],
            FaceDirection::Down => [0, -1, 0],
            FaceDirection::Left => [-1, 0, 0],
            FaceDirection::Right => [1, 0, 0],
            FaceDirection::Front => [0, 0, 1],
        }
        .map(|v| v as f32)
    }

    fn uv_min_max(&self) -> [[f32; 2]; 2] {
        self.uv_min_max
    }
}

impl DwFace {
    pub fn from_tile_map(
        bottom_left_tile: ImageType,
        face_direction: FaceDirection,
        bottom_left: [f32; 3],
        size: [u8; 2],
    ) -> Self {
        let [w, h] = size;
        Self {
            face_direction,
            bottom_left,
            size: size.map(|v| v as f32),
            uv_min_max: bottom_left_tile.uv_min_max(w, h),
        }
    }

    pub fn new_sprite(
        bottom_left_tile: ImageType,
        local_center_pos: [f32; 2],
        global_center_pos: [f32; 2],
        size: [u8; 2],
        z: f32,
    ) -> Self {
        let [local_center_x_offset, local_center_y_offset] = local_center_pos;
        let [global_center_x, global_center_y] = global_center_pos;
        let min_x = global_center_x - local_center_x_offset;
        let min_y = global_center_y - local_center_y_offset;

        Self::from_tile_map(
            bottom_left_tile,
            FaceDirection::Front,
            [min_x, min_y, z],
            size,
        )
    }

    pub fn mirror_uv_h(&mut self) {
        let [[u_min, v_min], [u_max, v_max]] = self.uv_min_max;
        self.uv_min_max = [[u_max, v_min], [u_min, v_max]];
    }
}

pub struct DwBlock {
    coord: BlockCoord,
    block_uv: BlockUv,
}

impl DwBlock {
    pub fn new(coord: BlockCoord, block_uv: BlockUv) -> Self {
        Self { coord, block_uv }
    }
}

pub struct ChunkDwBlock {
    blocks: [Option<(DwBlock, DwChunkObjId)>; Chunk::NUM_BLOCK_PER_ROW * Chunk::NUM_BLOCK_PER_COL],
    num_blocks: usize,
}

impl Default for ChunkDwBlock {
    fn default() -> Self {
        Self {
            blocks: [const { None }; _],
            num_blocks: 0,
        }
    }
}

impl ChunkDwBlock {
    fn to_index(chunk_block_coord: ChunkBlockCoord) -> usize {
        chunk_block_coord.x() as usize * Chunk::NUM_BLOCK_PER_COL + chunk_block_coord.y() as usize
    }

    pub fn add(&mut self, dw_block: DwBlock, id: DwChunkObjId) {
        let index = Self::to_index(dw_block.coord.into());
        self.blocks[index] = Some((dw_block, id));
        self.num_blocks += 1;
    }

    fn block_at(&self, coord: ChunkBlockCoord) -> Option<&(DwBlock, DwChunkObjId)> {
        self.blocks[Self::to_index(coord)].as_ref()
    }

    fn add_face(
        face_direction: FaceDirection,
        chunk_coord: &ChunkCoord,
        block: &DwBlock,
        id: &DwChunkObjId,
        builder: &mut MeshAggregator<[DwVertex; 4], [u32; 6]>,
    ) {
        let z = 1;
        let x = block.coord.x();
        let y = block.coord.y() as u32;
        let uv = block.block_uv;

        let dw_face = DwFace::from_tile_map(
            match face_direction {
                FaceDirection::Up => uv.up(),
                FaceDirection::Down => uv.bottom(),
                FaceDirection::Left | FaceDirection::Right | FaceDirection::Front => uv.side(),
            },
            face_direction,
            match face_direction {
                FaceDirection::Up => [x, y + 1, z + 1],
                FaceDirection::Down => [x, y, z + 1],
                FaceDirection::Left => [x, y, z],
                FaceDirection::Right => [x + 1, y, z],
                FaceDirection::Front => [x, y, z + 1],
            }
            .map(|v| v as f32),
            [1, 1],
        );
        let (vertices, indices) = dw_face.vertices(id, chunk_coord);
        builder.add(vertices, indices);
    }

    fn add_faces(
        &self,
        mut mask: u32,
        face: FaceDirection,
        row_or_col_index: u8,
        chunk_coord: &ChunkCoord,
        builder: &mut MeshAggregator<[DwVertex; 4], [u32; 6]>,
    ) {
        while mask != 0 {
            let lsb = mask & mask.wrapping_neg();
            let lsb_index = lsb.trailing_zeros() as u8;
            mask ^= lsb;

            let (x, y) = match face {
                FaceDirection::Up | FaceDirection::Down => (row_or_col_index, lsb_index),
                FaceDirection::Left | FaceDirection::Right => (lsb_index, row_or_col_index),
                FaceDirection::Front => {
                    unreachable!(
                        "ChunkDwBlock::add_faces must not be called with FaceDirection::Front"
                    )
                }
            };
            // SAFETY: non-zero u32 must have < 32 trailing zeros; thus lsb_index < 32.
            // Face ensures the attached values < 32 by loop in build_culled_mesh.
            let coord = ChunkBlockCoord::new(x, y).unwrap();

            // SAFETY: x, y is 1 in mask, meaning block_at result must have is_some() == true.
            let (block, id) = self.block_at(coord).unwrap();
            Self::add_face(face, chunk_coord, block, id, builder);
        }
    }

    fn build_culled_mesh(
        self,
        chunk_coord: &ChunkCoord,
        builder: &mut MeshAggregator<[DwVertex; 4], [u32; 6]>,
    ) {
        if self.num_blocks == 0 {
            return;
        }

        builder.reserve(self.num_blocks * 6); // at most 6 faces for each block

        // 0th bit is y=0, 31th bit is y=31; [0] is first column (leftmost)
        // bit = 1 means there's a block
        let mut v_blocks = [0u32; Chunk::NUM_BLOCK_PER_ROW];

        // 0th bit is x=0; [0] is bottom row
        let mut h_blocks = [0u32; Chunk::NUM_BLOCK_PER_COL];

        for y in (0..Chunk::NUM_BLOCK_PER_COL).rev() {
            for x in (0..Chunk::NUM_BLOCK_PER_ROW).rev() {
                // SAFETY: guaranteed to be success with Chunk constants
                let coord = ChunkBlockCoord::new(x as u8, y as u8).unwrap();
                let block = self.block_at(coord);
                let is_block = block.is_some() as u32;
                h_blocks[y] <<= 1;
                h_blocks[y] |= is_block;
                v_blocks[x] <<= 1;
                v_blocks[x] |= is_block;
                if let Some((block, id)) = block {
                    Self::add_face(FaceDirection::Front, chunk_coord, block, id, builder);
                }
            }
        }

        for (x, column) in v_blocks.iter().enumerate() {
            self.add_faces(
                column & !(column >> 1),
                FaceDirection::Up,
                x as u8,
                chunk_coord,
                builder,
            );
            self.add_faces(
                column & !(column << 1),
                FaceDirection::Down,
                x as u8,
                chunk_coord,
                builder,
            );
        }

        for (y, row) in h_blocks.iter().enumerate() {
            self.add_faces(
                row & !(row << 1),
                FaceDirection::Left,
                y as u8,
                chunk_coord,
                builder,
            );
            self.add_faces(
                row & !(row >> 1),
                FaceDirection::Right,
                y as u8,
                chunk_coord,
                builder,
            );
        }
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum BuildDwMeshError {
    #[snafu(display("Object coordinate out of world bound: {source}"))]
    CoordOutOfBound { source: CoordError },
    #[snafu(display(
        "Workbench with type {workbench_type:?} has level {level} that exceeds maximum {maximum}"
    ))]
    InvalidWorkbenchLevel {
        workbench_type: WorkbenchType,
        level: u8,
        maximum: u8,
    },
    // TODO these should be checked when loading the dw xmls
    #[snafu(display("Item type {item_type:?} is not door: {door:?}"))]
    InvalidItemTypeForDoor {
        item_type: ItemType,
        door: Box<Door>,
    },
    #[snafu(display("Item type {item_type:?} is not torch: {torch:?}"))]
    InvalidItemTypeForTorch {
        item_type: ItemType,
        torch: Box<Torch>,
    },
}

#[derive(Default, Clone, Copy)]
pub struct DwCapacity {
    pub items: usize,
    pub quads: usize,
}

impl DwCapacity {
    fn is_empty(&self) -> bool {
        self.items == 0 && self.quads == 0
    }
}

impl AddAssign for DwCapacity {
    fn add_assign(&mut self, rhs: Self) {
        self.items += rhs.items;
        self.quads += rhs.quads;
    }
}

impl Mul<usize> for DwCapacity {
    type Output = DwCapacity;

    fn mul(self, rhs: usize) -> Self::Output {
        Self::Output {
            items: self.items * rhs,
            quads: self.quads * rhs,
        }
    }
}

pub trait BuildDwMesh {
    const STATIC_CAPACITY: Option<DwCapacity> = None;

    fn capacity(&self) -> DwCapacity {
        DwCapacity::default()
    }

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError>;
}

struct MeshAggregator<V, I> {
    vi_pairs: Vec<(V, I)>,
}

impl<V, I> MeshAggregator<V, I> {
    #[allow(unused)]
    fn new() -> Self {
        Self {
            vi_pairs: Vec::new(),
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            vi_pairs: Vec::with_capacity(capacity),
        }
    }

    fn reserve(&mut self, additional: usize) {
        self.vi_pairs.reserve(additional);
    }

    fn add(&mut self, vertices: V, indices: I) {
        self.vi_pairs.push((vertices, indices));
    }
}

impl<V, I> MeshAggregator<V, I> {
    fn build<T, U>(
        self,
        mut vertices: Vec<T>,
        mut indices: Vec<u32>,
        device: &wgpu::Device,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32)
    where
        V: AsRef<[T]>,
        I: AsRef<[U]> + IntoIterator<Item = U>,
        T: bytemuck::Pod + bytemuck::Zeroable,
        U: Into<u32> + std::ops::Add<u32, Output = u32>,
    {
        let mut acc_vertices = 0;
        for (v, i) in self.vi_pairs {
            vertices.extend_from_slice(v.as_ref());
            indices.extend(i.into_iter().map(|i| i + acc_vertices));
            acc_vertices += v.as_ref().len() as u32;
        }

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("DW Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("DW Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buf, index_buf, indices.len() as u32)
    }

    #[allow(unused)]
    pub fn build_var<T, U>(self, device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, u32)
    where
        V: AsRef<[T]>,
        I: AsRef<[U]> + IntoIterator<Item = U>,
        T: bytemuck::Pod + bytemuck::Zeroable,
        U: Into<u32> + std::ops::Add<u32, Output = u32>,
    {
        let vertices =
            Vec::with_capacity(self.vi_pairs.iter().map(|(v, _)| v.as_ref().len()).sum());
        let indices = Vec::with_capacity(self.vi_pairs.iter().map(|(_, i)| i.as_ref().len()).sum());
        self.build(vertices, indices, device)
    }

    pub fn build_const<T, U, const N: usize, const M: usize>(
        self,
        device: &wgpu::Device,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32)
    where
        V: AsRef<[T]> + Into<[T; N]>,
        I: AsRef<[U]> + Into<[U; M]> + IntoIterator<Item = U>,
        T: bytemuck::Pod + bytemuck::Zeroable,
        U: Into<u32> + std::ops::Add<u32, Output = u32>,
    {
        let vertices = Vec::with_capacity(self.vi_pairs.len() * N);
        let indices = Vec::with_capacity(self.vi_pairs.len() * M);
        self.build(vertices, indices, device)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DwChunkObjId {
    pub obj_type: DynamicObjectType,
    pub index: usize,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct DwChunkObjIdUniform {
    is_some: u32,
    raw_id: u32,
    chunk: PackedChunkCoord,
    _padding: u32,
}

impl DwChunkObjId {
    const MAX_INDEX_BITS: usize = 24;
    const MAX_INDEX: usize = (1 << Self::MAX_INDEX_BITS); // valid range: 0..MAX_INDEX
    const INDEX_MASK: usize = Self::MAX_INDEX - 1;

    fn obj_type_from_raw(raw_id: u32) -> Option<DynamicObjectType> {
        DynamicObjectType::try_from((raw_id >> Self::MAX_INDEX_BITS) as u16).ok()
    }

    fn index_from_raw(raw_id: u32) -> usize {
        (raw_id & ((1 << Self::MAX_INDEX_BITS) - 1)) as usize
    }

    pub fn try_from_u32(raw_id: u32) -> Option<Self> {
        Self::obj_type_from_raw(raw_id)
            .map(|obj_type| Self::new(obj_type, Self::index_from_raw(raw_id)))
    }

    pub fn new(obj_type: DynamicObjectType, index: usize) -> Self {
        if index > Self::INDEX_MASK {
            // should be very rare or nearly impossible to happen; should be warning
            println!("index {} interferes with obj_type {:?}", index, obj_type);
        }
        Self { obj_type, index }
    }

    pub fn raw_id(&self) -> u32 {
        (self.obj_type as u32) << Self::MAX_INDEX_BITS | self.index as u32
    }
}

impl From<Option<(DwChunkObjId, ChunkCoord)>> for DwChunkObjIdUniform {
    fn from(value: Option<(DwChunkObjId, ChunkCoord)>) -> Self {
        value
            .map(|(id, chunk_coord)| Self {
                is_some: 1,
                raw_id: id.raw_id(),
                chunk: chunk_coord.into(),
                _padding: 0,
            })
            .unwrap_or_default()
    }
}

impl DwChunkObjIdUniform {
    pub fn create_buffer(self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Id of Object in Chunk Buffer"),
            contents: bytemuck::cast_slice(&[self]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DwVertex {
    pub raw_id: u32,
    pub chunk: PackedChunkCoord,
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl DwVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![0 => Uint32, 1 => Uint32, 2 => Float32x3, 3 => Float32x3, 4 => Float32x2];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DwVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct DwChunkBufBuilder {
    id: DwChunkObjId,
    coord: ChunkCoord,
    item_instances: Vec<DwItemInstanceRaw>,
    faces_mesh_builder: MeshAggregator<[DwVertex; 4], [u32; 6]>,
    blocks: ChunkDwBlock,
}

impl DwChunkBufBuilder {
    pub fn set_id(&mut self, id: DwChunkObjId) {
        self.id = id;
    }

    pub fn add_vertices_and_indices(&mut self, vertices: [DwVertex; 4], indices: [u32; 6]) {
        self.faces_mesh_builder.add(vertices, indices);
    }

    pub fn add_item(&mut self, item: DwItem) {
        self.item_instances.push(item.instance(self.id, self.coord));
    }

    pub fn add_block(&mut self, block: DwBlock) {
        self.blocks.add(block, self.id);
    }

    pub fn add_quad<T: DwQuad>(&mut self, quad: T) {
        let (vertices, indices) = quad.vertices(&self.id, &self.coord);
        self.add_vertices_and_indices(vertices, indices);
    }

    pub fn add_face(&mut self, face: DwFace) {
        self.add_quad(face);
    }
}

#[derive(Clone)]
pub struct DwChunkBuf {
    // item instances
    pub instance_buf: wgpu::Buffer,
    pub num_instances: u32,

    // faces (sprites, blocks, custom ones like kelp, doors)
    pub faces_vertex_buf: wgpu::Buffer,
    pub faces_index_buf: wgpu::Buffer,
    pub faces_num_indices: u32,
}

impl DwChunkBuf {
    fn get_chunk_capacity(chunk: &ChunkDynamicObjects) -> DwCapacity {
        let mut sum = DwCapacity::default();
        fn accu<'a, T: BuildDwMesh + 'a, I: ExactSizeIterator<Item = &'a T>>(
            i: I,
            sum: &mut DwCapacity,
        ) {
            if let Some(cap) = T::STATIC_CAPACITY {
                *sum += cap * i.len();
            } else {
                for obj in i {
                    let cap = obj.capacity();
                    *sum += cap;
                }
            }
        }
        accu(chunk.apple_tree.iter(), &mut sum);
        accu(chunk.maple_tree.iter(), &mut sum);
        accu(chunk.mango_tree.iter(), &mut sum);
        accu(chunk.pine_tree.iter(), &mut sum);
        accu(chunk.cactus_tree.iter(), &mut sum);
        accu(chunk.coconut_tree.iter(), &mut sum);
        accu(chunk.orange_tree.iter(), &mut sum);
        accu(chunk.cherry_tree.iter(), &mut sum);
        accu(chunk.coffee_tree.iter(), &mut sum);
        // accu(chunk.flax_plant.iter(), &mut sum);
        // accu(chunk.sunflower_plant.iter(), &mut sum);
        accu(chunk.corn_plant.iter(), &mut sum);
        // accu(chunk.dodo.iter(), &mut sum);
        // accu(chunk.dropped_item.iter(), &mut sum);
        // accu(chunk.fire.iter(), &mut sum);
        accu(chunk.torch.iter(), &mut sum);
        // accu(chunk.glow_block.iter(), &mut sum);
        accu(chunk.ladder.iter(), &mut sum);
        accu(chunk.door.iter(), &mut sum);
        // accu(chunk.artificial_light.iter(), &mut sum);
        // accu(chunk.bed.iter(), &mut sum);
        // accu(chunk.dropbear.iter(), &mut sum);
        // accu(chunk.gather_block.iter(), &mut sum);
        accu(chunk.carrot_plant.iter(), &mut sum);
        // accu(chunk.donkey.iter(), &mut sum);
        accu(chunk.egg.iter(), &mut sum);
        // accu(chunk.window.iter(), &mut sum);
        // accu(chunk.boat.iter(), &mut sum);
        // accu(chunk.chilli_plant.iter(), &mut sum);
        accu(chunk.kelp_plant.iter(), &mut sum);
        // accu(chunk.clown_fish.iter(), &mut sum);
        // accu(chunk.shark.iter(), &mut sum);
        accu(chunk.lime_tree.iter(), &mut sum);
        // accu(chunk.wire.iter(), &mut sum);
        // accu(chunk.cave_troll.iter(), &mut sum);
        // accu(chunk.rail.iter(), &mut sum);
        // accu(chunk.hand_car.iter(), &mut sum);
        // accu(chunk.steam_locomotive.iter(), &mut sum);
        // accu(chunk.freight_car.iter(), &mut sum);
        // accu(chunk.passenger_car.iter(), &mut sum);
        accu(chunk.workbench.iter(), &mut sum);
        accu(chunk.chest.iter(), &mut sum);
        accu(chunk.sign.iter(), &mut sum);
        // accu(chunk.trading_post.iter(), &mut sum);
        // accu(chunk.train_station.iter(), &mut sum);
        // accu(chunk.trade_portal.iter(), &mut sum);
        // accu(chunk.scorpion.iter(), &mut sum);
        // accu(chunk.painting.iter(), &mut sum);
        // accu(chunk.column.iter(), &mut sum);
        // accu(chunk.stairs.iter(), &mut sum);
        // accu(chunk.elevator_motor.iter(), &mut sum);
        // accu(chunk.elevator_shaft.iter(), &mut sum);
        accu(chunk.gem_tree.iter(), &mut sum);
        // accu(chunk.vine_plant.iter(), &mut sum);
        // accu(chunk.tulip_plant.iter(), &mut sum);
        // accu(chunk.ownership_sign.iter(), &mut sum);
        // accu(chunk.wheat_plant.iter(), &mut sum);
        accu(chunk.tomato_plant.iter(), &mut sum);
        // accu(chunk.yak.iter(), &mut sum);
        // accu(chunk.mirror.iter(), &mut sum);

        sum
    }

    pub fn from_chunk(
        chunk: &ChunkDynamicObjects,
        coord: ChunkCoord,
        device: &wgpu::Device,
    ) -> Option<Self> {
        fn add<'a, T: BuildDwMesh + 'a, I: Iterator<Item = &'a T>>(
            i: I,
            obj_type: DynamicObjectType,
            builder: &mut DwChunkBufBuilder,
        ) {
            for (index, obj) in i.enumerate() {
                let id = DwChunkObjId::new(obj_type, index);
                builder.set_id(id);

                // should report error, for now we just skip
                let _ = obj.build_dw_mesh(builder);
            }
        }
        let need_capacity = Self::get_chunk_capacity(chunk);
        if need_capacity.is_empty() {
            return None;
        }

        let mut builder = DwChunkBufBuilder {
            item_instances: Vec::with_capacity(need_capacity.items),
            faces_mesh_builder: MeshAggregator::with_capacity(need_capacity.quads),
            blocks: ChunkDwBlock::default(),
            id: DwChunkObjId::new(DynamicObjectType::AppleTree, 0), // random id - won't be read.
            coord,
        };

        use DynamicObjectType::*;
        add(chunk.apple_tree.iter(), AppleTree, &mut builder);
        add(chunk.maple_tree.iter(), MapleTree, &mut builder);
        add(chunk.mango_tree.iter(), MangoTree, &mut builder);
        add(chunk.pine_tree.iter(), PineTree, &mut builder);
        add(chunk.cactus_tree.iter(), CactusTree, &mut builder);
        add(chunk.coconut_tree.iter(), CoconutTree, &mut builder);
        add(chunk.orange_tree.iter(), OrangeTree, &mut builder);
        add(chunk.cherry_tree.iter(), CherryTree, &mut builder);
        add(chunk.coffee_tree.iter(), CoffeeTree, &mut builder);
        // add(chunk.flax_plant.iter(), FlaxPlant, &mut builder);
        // add(chunk.sunflower_plant.iter(), SunflowerPlant, &mut builder);
        add(chunk.corn_plant.iter(), CornPlant, &mut builder);
        // add(chunk.dodo.iter(), Dodo, &mut builder);
        // add(chunk.dropped_item.iter(), DroppedItem, &mut builder);
        // add(chunk.fire.iter(), u8, &mut builder);
        add(chunk.torch.iter(), Torch, &mut builder);
        // add(chunk.glow_block.iter(), u8, &mut builder);
        add(chunk.ladder.iter(), Ladder, &mut builder);
        add(chunk.door.iter(), Door, &mut builder);
        // add(chunk.artificial_light.iter(), u8, &mut builder);
        // add(chunk.bed.iter(), Bed, &mut builder);
        // add(chunk.dropbear.iter(), DropBear, &mut builder);
        // add(chunk.gather_block.iter(), u8, &mut builder);
        add(chunk.carrot_plant.iter(), CarrotPlant, &mut builder);
        // add(chunk.donkey.iter(), Donkey, &mut builder);
        add(chunk.egg.iter(), Egg, &mut builder);
        // add(chunk.window.iter(), Window, &mut builder);
        // add(chunk.boat.iter(), Boat, &mut builder);
        // add(chunk.chilli_plant.iter(), ChilliPlant, &mut builder);
        add(chunk.kelp_plant.iter(), KelpPlant, &mut builder);
        // add(chunk.clown_fish.iter(), ClownFish, &mut builder);
        // add(chunk.shark.iter(), Shark, &mut builder);
        add(chunk.lime_tree.iter(), LimeTree, &mut builder);
        // add(chunk.wire.iter(), Wire, &mut builder);
        // add(chunk.cave_troll.iter(), CaveTroll, &mut builder);
        // add(chunk.rail.iter(), Rail, &mut builder);
        // add(chunk.hand_car.iter(), HandCar, &mut builder);
        // add(chunk.steam_locomotive.iter(), SteamLocomotive, &mut builder);
        // add(chunk.freight_car.iter(), FreightCar, &mut builder);
        // add(chunk.passenger_car.iter(), PassengerCar, &mut builder);
        add(chunk.workbench.iter(), Workbench, &mut builder);
        add(chunk.chest.iter(), Chest, &mut builder);
        add(chunk.sign.iter(), Sign, &mut builder);
        // add(chunk.trading_post.iter(), TradingPost, &mut builder);
        add(chunk.train_station.iter(), TrainStation, &mut builder);
        // add(chunk.trade_portal.iter(), TradePortal, &mut builder);
        // add(chunk.scorpion.iter(), Scorpion, &mut builder);
        // add(chunk.painting.iter(), u8, &mut builder);
        // add(chunk.column.iter(), Column, &mut builder);
        // add(chunk.stairs.iter(), Stairs, &mut builder);
        // add(chunk.elevator_motor.iter(), ElevatorMotor, &mut builder);
        // add(chunk.elevator_shaft.iter(), ElevatorShaft, &mut builder);
        add(chunk.gem_tree.iter(), GemTree, &mut builder);
        // add(chunk.vine_plant.iter(), VinePlant, &mut builder);
        // add(chunk.tulip_plant.iter(), TulipPlant, &mut builder);
        // add(chunk.ownership_sign.iter(), u8, &mut builder);
        // add(chunk.wheat_plant.iter(), WheatPlant, &mut builder);
        add(chunk.tomato_plant.iter(), TomatoPlant, &mut builder);
        // add(chunk.yak.iter(), Yak, &mut builder);
        // add(chunk.mirror.iter(), u8, &mut builder);

        let DwChunkBufBuilder {
            item_instances,
            mut faces_mesh_builder,
            blocks,
            ..
        } = builder;

        let instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dynamic Object Icon Instance Buffer"),
            contents: bytemuck::cast_slice(&item_instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        blocks.build_culled_mesh(&coord, &mut faces_mesh_builder);

        let (faces_vertex_buf, faces_index_buf, faces_num_indices) =
            faces_mesh_builder.build_const(device);

        let num_instances = item_instances.len() as u32;
        if num_instances + faces_num_indices == 0 {
            return None;
        }

        Some(Self {
            instance_buf,
            num_instances,

            faces_vertex_buf,
            faces_index_buf,
            faces_num_indices,
        })
    }
}

// Buffers storing all dynamic world objects
pub struct DwBuf {
    chunks: HashMap<ChunkCoord, Option<DwChunkBuf>>,
}

impl DwBuf {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    pub fn get_chunk<I: Into<ChunkCoord>>(&self, coord: I) -> Option<&DwChunkBuf> {
        self.chunks.get(&coord.into()).and_then(|v| v.as_ref())
    }

    pub fn has_chunk<I: Into<ChunkCoord>>(&self, coord: I) -> bool {
        self.get_chunk(coord).is_some()
    }

    pub fn set_chunk<I: Into<ChunkCoord>>(
        &mut self,
        device: &wgpu::Device,
        coord: I,
        chunk: &ChunkDynamicObjects,
    ) {
        let coord = coord.into();
        let buf = DwChunkBuf::from_chunk(chunk, coord, device);
        self.chunks.insert(coord, buf);
    }
}
