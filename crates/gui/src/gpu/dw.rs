use super::ToSprite;
use egui_wgpu::wgpu::{self, util::DeviceExt};
use std::collections::HashMap;
use the_blockheads_tools_lib::game::{coord::ChunkCoord, dw::dynamic_world::ChunkDynamicObjects};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DwSpriteVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl DwSpriteVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DwSpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DwIconVertex {
    pub position: [f32; 2],
}

impl DwIconVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DwIconVertex>() as wgpu::BufferAddress,
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
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DwIconInstanceRaw {
    pub position: [f32; 2],
    pub item_type: u32,
}

impl DwIconInstanceRaw {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![1 => Float32x2, 2 => Uint32];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DwIconInstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub trait ToIconInstance {
    fn to_icon_instance(&self) -> DwIconInstanceRaw;
}

// Concats vertex buf and index buf
struct DwSpriteBufBuilder {
    vi_pairs: Vec<([DwSpriteVertex; 4], [u32; 6])>,
}

impl DwSpriteBufBuilder {
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

    fn add(&mut self, vertices: [DwSpriteVertex; 4], indices: [u32; 6]) {
        self.vi_pairs.push((vertices, indices));
    }

    fn build(self, device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let mut acc_vertices = 0;
        let mut vertices = Vec::with_capacity(self.vi_pairs.len() * 4);
        let mut indices = Vec::with_capacity(self.vi_pairs.len() * 6);
        for (v, i) in self.vi_pairs {
            vertices.extend_from_slice(&v);
            indices.extend(i.into_iter().map(|i| i + acc_vertices));
            acc_vertices += v.len() as u32;
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
}

pub struct DwChunkIconBuf {
    pub instance_buf: wgpu::Buffer,
    pub capacity: u32,
    pub num_instances: u32,
}

pub struct DwChunkSpriteBuf {
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub num_indices: u32,
}

pub struct DwChunkBuf {
    pub icon_buf: DwChunkIconBuf,
    pub sprite_buf: DwChunkSpriteBuf,
}

impl DwChunkBuf {
    pub fn from_chunk(chunk: &ChunkDynamicObjects, device: &wgpu::Device) -> Option<Self> {
        fn add<'a, T: ToSprite + ToIconInstance + 'a, I: Iterator<Item = &'a T>>(
            i: I,
            icon_instances: &mut Vec<DwIconInstanceRaw>,
            sprite_builder: &mut DwSpriteBufBuilder,
        ) {
            for obj in i {
                let (v, i) = obj.to_sprite().to_vertices();
                sprite_builder.add(v, i);
                icon_instances.push(obj.to_icon_instance());
            }
        }

        let num_objs = chunk.num_objects();
        if num_objs == 0 {
            return None;
        }

        let mut icons = Vec::with_capacity(num_objs);
        let mut builder = DwSpriteBufBuilder::with_capacity(num_objs);
        add(chunk.tomato_plants.iter(), &mut icons, &mut builder);

        let (vertex_buf, index_buf, num_indices) = builder.build(device);
        let icon_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dynamic Object Icon Instance Buffer"),
            contents: bytemuck::cast_slice(&icons),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let num_instances = icons.len() as u32;
        Some(Self {
            icon_buf: DwChunkIconBuf {
                instance_buf: icon_buf,
                capacity: num_instances,
                num_instances,
            },
            sprite_buf: DwChunkSpriteBuf {
                vertex_buf,
                index_buf,
                num_indices,
            },
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
        self.chunks.get(&coord.into()).map(|v| v.as_ref()).flatten()
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
        let buf = DwChunkBuf::from_chunk(chunk, device);
        self.chunks.insert(coord.into(), buf);
    }
}
