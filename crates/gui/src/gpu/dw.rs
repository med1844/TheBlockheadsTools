use super::{super::image_type::ImageType, VoxelType, coord::Coord};
use eframe::wgpu::{self, util::DeviceExt};
use snafu::Snafu;
use std::collections::HashMap;
use the_blockheads_tools_lib::game::{
    chunk::Chunk,
    coord::{BlockCoord, ChunkBlockCoord, ChunkCoord, CoordError},
    dynamic_object::{DynamicObjectType, workbench::WorkbenchType},
    dynamic_world::ChunkDynamicObjects,
    item::ItemType,
};

const EPSILON: f32 = 1e-6;

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

#[derive(Clone, Copy)]
pub struct DwIcon {
    position: [f32; 2],
    item_type: ItemType,
}

impl DwIcon {
    pub fn new(position: [f32; 2], item_type: ItemType) -> Self {
        Self {
            position,
            item_type: item_type,
        }
    }

    pub fn instance(self, id: DwChunkObjId, coord: ChunkCoord) -> DwIconInstanceRaw {
        DwIconInstanceRaw {
            position: self.position,
            item_type: self.item_type as u32,
            raw_id: id.to_raw_id(),
            chunk_x: coord.x(),
            chunk_y: coord.y() as u32,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DwIconInstanceRaw {
    pub position: [f32; 2],
    pub item_type: u32,
    pub raw_id: u32,
    pub chunk_x: u32,
    pub chunk_y: u32,
}

impl DwIconInstanceRaw {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        1 => Float32x2,
        2 => Uint32,
        3 => Uint32,
        4 => Uint32,
        5 => Uint32,
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DwIconInstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
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
    uv_min: [f32; 2],
    uv_max: [f32; 2],
}

impl DwFace {
    pub fn from_tile_map(
        bottom_left_tile: ImageType,
        face_direction: FaceDirection,
        bottom_left: [f32; 3],
        size: [u8; 2],
    ) -> Self {
        let [_, h] = size;
        let (u_min_tile, v_max_tile) = bottom_left_tile.to_tile_xy();
        let v_min_tile = v_max_tile + 1 - h as u32;

        let u_min = (u_min_tile as f32) * ImageType::TILE_SIZE;
        let v_min = (v_min_tile as f32) * ImageType::TILE_SIZE;
        let [w, h] = size.map(|v| v as f32);
        let [u_max, v_max] = [
            u_min + w * ImageType::TILE_SIZE,
            v_min + h * ImageType::TILE_SIZE,
        ];

        Self {
            face_direction,
            bottom_left,
            size: [w, h],
            uv_min: [u_min, v_min].map(|v| v + EPSILON),
            uv_max: [u_max, v_max].map(|v| v - EPSILON),
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

    fn to_vertices(
        &self,
        id: &DwChunkObjId,
        chunk_coord: &ChunkCoord,
    ) -> ([DwVertex; 4], [u32; 6]) {
        let [x, y, z] = self.bottom_left;
        let [w, h] = self.size;

        let [bottom_left, bottom_right, top_right, top_left] = match self.face_direction {
            FaceDirection::Up | FaceDirection::Down => {
                [[x, y, z], [x + w, y, z], [x + w, y, z - h], [x, y, z - h]]
            }
            FaceDirection::Left | FaceDirection::Right => {
                [[x, y, z], [x, y, z + w], [x, y + h, z + w], [x, y + h, z]]
            }
            FaceDirection::Front => [[x, y, z], [x + w, y, z], [x + w, y + h, z], [x, y + h, z]],
        };

        let normal = match self.face_direction {
            FaceDirection::Up => [0, 1, 0],
            FaceDirection::Down => [0, -1, 0],
            FaceDirection::Left => [-1, 0, 0],
            FaceDirection::Right => [1, 0, 0],
            FaceDirection::Front => [0, 0, 1],
        }
        .map(|v| v as f32);
        let [u_min, v_min] = self.uv_min;
        let [u_max, v_max] = self.uv_max;

        let raw_id = id.to_raw_id();
        let chunk_x = chunk_coord.x();
        let chunk_y = chunk_coord.y() as u32;

        (
            [
                DwVertex {
                    raw_id,
                    chunk_x,
                    chunk_y,
                    position: bottom_left.map(|v| v as f32),
                    normal,
                    tex_coords: [u_min, v_max],
                },
                DwVertex {
                    raw_id,
                    chunk_x,
                    chunk_y,
                    position: bottom_right.map(|v| v as f32),
                    normal,
                    tex_coords: [u_max, v_max],
                },
                DwVertex {
                    raw_id,
                    chunk_x,
                    chunk_y,
                    position: top_right.map(|v| v as f32),
                    normal,
                    tex_coords: [u_max, v_min],
                },
                DwVertex {
                    raw_id,
                    chunk_x,
                    chunk_y,
                    position: top_left.map(|v| v as f32),
                    normal,
                    tex_coords: [u_min, v_min],
                },
            ],
            [0, 1, 2, 0, 2, 3],
        )
    }
}

pub struct DwBlock {
    coord: BlockCoord,
    voxel_type: VoxelType,
}

impl DwBlock {
    pub fn new(coord: BlockCoord, voxel_type: VoxelType) -> Self {
        Self { coord, voxel_type }
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
        let uv = block.voxel_type.uv();

        let dw_face = DwFace::from_tile_map(
            match face_direction {
                FaceDirection::Up { .. } => uv.up(),
                FaceDirection::Down { .. } => uv.down(),
                FaceDirection::Left { .. } | FaceDirection::Right { .. } | FaceDirection::Front => {
                    uv.side()
                }
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
        let (vertices, indices) = dw_face.to_vertices(id, chunk_coord);
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
                    unreachable!("add_faces must not be called with Face::Front")
                }
            };
            // SAFETY: non-zero u32 must have < 32 trailing zeros; thus lsb_index < 32.
            // Face ensures the attached values < 32 by loop in build_culled_mesh.
            let coord = ChunkBlockCoord::new(x, y).unwrap();

            // SAFETY: x, y is 1 in mask, meaning block_at result must have is_some() == true.
            let (block, id) = self.block_at(coord).unwrap();
            Self::add_face(face, &chunk_coord, block, id, builder);
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
    #[snafu(display("Item type {item_type:?} is not door"))]
    InvalidItemTypeForDoor { item_type: ItemType },
}

pub trait BuildDwMesh {
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
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DwChunkObjIdUniform([u32; 4]);

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

    pub fn to_raw_id(&self) -> u32 {
        (self.obj_type as u32) << Self::MAX_INDEX_BITS | self.index as u32
    }
}

impl Default for DwChunkObjIdUniform {
    fn default() -> Self {
        Self([0; 4])
    }
}

impl From<Option<(DwChunkObjId, ChunkCoord)>> for DwChunkObjIdUniform {
    fn from(value: Option<(DwChunkObjId, ChunkCoord)>) -> Self {
        let mut uniform = [0; 4];
        if let Some((id, chunk_coord)) = value {
            uniform[0] = 1;
            uniform[1] = id.to_raw_id();
            uniform[2] = chunk_coord.x_u32();
            uniform[3] = chunk_coord.y_u32();
        }
        Self(uniform)
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
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DwVertex {
    pub raw_id: u32,
    pub chunk_x: u32,
    pub chunk_y: u32,
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl DwVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![0 => Uint32, 1 => Uint32, 2 => Uint32, 3 => Float32x3, 4 => Float32x3, 5 => Float32x2];
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
    icon_instances: Vec<DwIconInstanceRaw>,
    faces_mesh_builder: MeshAggregator<[DwVertex; 4], [u32; 6]>,
    blocks: ChunkDwBlock,
}

impl DwChunkBufBuilder {
    pub fn set_id(&mut self, id: DwChunkObjId) {
        self.id = id;
    }

    pub fn add_icon(&mut self, icon: DwIcon) {
        self.icon_instances.push(icon.instance(self.id, self.coord));
    }

    pub fn add_block(&mut self, block: DwBlock) {
        self.blocks.add(block, self.id);
    }

    pub fn add_face(&mut self, face: DwFace) {
        let (vertices, indices) = face.to_vertices(&self.id, &self.coord);
        self.faces_mesh_builder.add(vertices, indices);
    }
}

#[derive(Clone)]
pub struct DwChunkBuf {
    // icon instances
    pub instance_buf: wgpu::Buffer,
    pub num_instances: u32,

    // faces (sprites, blocks, custom ones like kelp, doors)
    pub faces_vertex_buf: wgpu::Buffer,
    pub faces_index_buf: wgpu::Buffer,
    pub faces_num_indices: u32,
}

impl DwChunkBuf {
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

        let num_objs = chunk.num_objects();
        if num_objs == 0 {
            return None;
        }

        let mut builder = DwChunkBufBuilder {
            icon_instances: Vec::with_capacity(num_objs),
            faces_mesh_builder: MeshAggregator::with_capacity(num_objs),
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
        add(chunk.corn_plant.iter(), CornPlant, &mut builder);
        add(chunk.door.iter(), Door, &mut builder);
        add(chunk.carrot_plant.iter(), CarrotPlant, &mut builder);
        add(chunk.kelp_plant.iter(), KelpPlant, &mut builder);
        add(chunk.lime_tree.iter(), LimeTree, &mut builder);
        add(chunk.workbench.iter(), Workbench, &mut builder);
        add(chunk.chest.iter(), Chest, &mut builder);
        add(chunk.tomato_plant.iter(), TomatoPlant, &mut builder);

        let DwChunkBufBuilder {
            icon_instances,
            mut faces_mesh_builder,
            blocks,
            ..
        } = builder;

        let instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dynamic Object Icon Instance Buffer"),
            contents: bytemuck::cast_slice(&icon_instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        blocks.build_culled_mesh(&coord, &mut faces_mesh_builder);

        let (faces_vertex_buf, faces_index_buf, faces_num_indices) =
            faces_mesh_builder.build_const(device);

        let num_instances = icon_instances.len() as u32;
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
