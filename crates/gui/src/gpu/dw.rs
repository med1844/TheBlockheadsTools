use super::sprite::ToSprite;
use eframe::wgpu::{self, util::DeviceExt};
use std::collections::HashMap;
use the_blockheads_tools_lib::game::{
    coord::ChunkCoord, dynamic_object::DynamicObjectType, dynamic_world::ChunkDynamicObjects,
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

struct MeshAggregator<V, I> {
    vi_pairs: Vec<(V, I)>,
}

impl<V, I> MeshAggregator<V, I> {
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DwChunkObjId(u32);

impl DwChunkObjId {
    pub fn from_obj_type_and_index(obj_type: DynamicObjectType, index: usize) -> Self {
        Self(((obj_type as u32) << 24) | (index as u32))
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DwVertex {
    pub id: DwChunkObjId,
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl DwVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Uint32, 1 => Float32x3, 2 => Float32x2];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DwVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[derive(Clone)]
pub struct DwChunkBuf {
    // icon instances
    pub instance_buf: wgpu::Buffer,
    pub num_instances: u32,

    // sprites
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub num_indices: u32,
}

impl DwChunkBuf {
    pub fn from_chunk(chunk: &ChunkDynamicObjects, device: &wgpu::Device) -> Option<Self> {
        fn add<'a, T: ToSprite + ToIconInstance + 'a, I: Iterator<Item = &'a T>>(
            i: I,
            obj_type: DynamicObjectType,
            icon_instances: &mut Vec<DwIconInstanceRaw>,
            sprite_builder: &mut MeshAggregator<[DwVertex; 4], [u32; 6]>,
        ) {
            for (index, obj) in i.enumerate() {
                if let Some(sprite) = obj.to_sprite() {
                    let (vertices, indices) =
                        sprite.to_vertices(DwChunkObjId::from_obj_type_and_index(obj_type, index));
                    sprite_builder.add(vertices, indices);
                }
                icon_instances.push(obj.to_icon_instance());
            }
        }

        let num_objs = chunk.num_objects();
        if num_objs == 0 {
            return None;
        }

        let mut icons = Vec::with_capacity(num_objs);
        let mut builder = MeshAggregator::with_capacity(num_objs);

        use DynamicObjectType::*;
        add(chunk.corn_plant.iter(), CornPlant, &mut icons, &mut builder);
        add(
            chunk.carrot_plant.iter(),
            CarrotPlant,
            &mut icons,
            &mut builder,
        );
        add(
            chunk.tomato_plant.iter(),
            TomatoPlant,
            &mut icons,
            &mut builder,
        );

        let instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dynamic Object Icon Instance Buffer"),
            contents: bytemuck::cast_slice(&icons),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let (vertex_buf, index_buf, num_indices) = builder.build_const(device);

        let num_instances = icons.len() as u32;
        if num_instances + num_indices == 0 {
            return None;
        }

        Some(Self {
            instance_buf,
            num_instances,

            vertex_buf,
            index_buf,
            num_indices,
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
