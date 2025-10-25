use egui_wgpu::wgpu::{self, util::DeviceExt};
use std::collections::HashMap;
use the_blockheads_tools_lib::game::{coord::ChunkCoord, dw::dynamic_world::ChunkDynamicObjects};

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

pub struct DwChunkIconBuf {
    pub instance_buf: wgpu::Buffer,
    pub num_instances: u32,
}

pub struct DwChunkBuf {
    pub icon_buf: DwChunkIconBuf,
}

impl DwChunkBuf {
    pub fn from_chunk(chunk: &ChunkDynamicObjects, device: &wgpu::Device) -> Option<Self> {
        fn add<'a, T: ToIconInstance + 'a, I: Iterator<Item = &'a T>>(
            i: I,
            icon_instances: &mut Vec<DwIconInstanceRaw>,
        ) {
            for obj in i {
                icon_instances.push(obj.to_icon_instance());
            }
        }

        let num_objs = chunk.num_objects();
        if num_objs == 0 {
            return None;
        }

        let mut icons = Vec::with_capacity(num_objs);

        add(chunk.corn_plant.iter(), &mut icons);
        add(chunk.carrot_plant.iter(), &mut icons);
        add(chunk.tomato_plants.iter(), &mut icons);

        let icon_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dynamic Object Icon Instance Buffer"),
            contents: bytemuck::cast_slice(&icons),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let num_instances = icons.len() as u32;
        Some(Self {
            icon_buf: DwChunkIconBuf {
                instance_buf: icon_buf,
                num_instances,
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
        let buf = DwChunkBuf::from_chunk(chunk, device);
        self.chunks.insert(coord.into(), buf);
    }
}
