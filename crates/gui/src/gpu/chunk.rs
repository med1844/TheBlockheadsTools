use super::{Vertex, VoxelBuf};
use egui_wgpu::wgpu::{self, util::DeviceExt};
use glam::Vec3;

pub struct ChunkMesh {
    vertices: [Vertex; 8],
}

impl ChunkMesh {
    const INDICES: &[u16] = &[
        0, 1, 2, 2, 3, 0, // front
        1, 5, 6, 6, 2, 1, // right
        5, 4, 7, 7, 6, 5, // back
        4, 0, 3, 3, 7, 4, // left
        3, 2, 6, 6, 7, 3, // top
        4, 5, 1, 1, 0, 4, // bottom
    ];

    pub fn new(offset: Vec3) -> Self {
        let [min_x, min_y, min_z] = offset.to_array();
        let max_x = min_x + 32.;
        let max_y = min_y + 32.;
        let max_z = min_z + 3.;
        Self {
            vertices: [
                // Front face
                Vertex {
                    pos: [min_x, min_y, max_z],
                },
                Vertex {
                    pos: [max_x, min_y, max_z],
                },
                Vertex {
                    pos: [max_x, max_y, max_z],
                },
                Vertex {
                    pos: [min_x, max_y, max_z],
                },
                // Back face
                Vertex {
                    pos: [min_x, min_y, min_z],
                },
                Vertex {
                    pos: [max_x, min_y, min_z],
                },
                Vertex {
                    pos: [max_x, max_y, min_z],
                },
                Vertex {
                    pos: [min_x, max_y, min_z],
                },
            ],
        }
    }

    pub fn to_vertex_index_buffer(
        &self,
        device: &wgpu::Device,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(Self::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buf, index_buf, Self::INDICES.len() as u32)
    }
}

pub struct ChunkBuffers {
    pub voxel_buf: VoxelBuf,

    // TODO make these 3 static after instancing
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub num_indices: u32,
}

impl ChunkBuffers {
    pub fn new(device: &wgpu::Device, voxel_buf: VoxelBuf) -> Self {
        let chunk = ChunkMesh::new(Vec3::ZERO);
        let (vertex_buf, index_buf, num_indices) = chunk.to_vertex_index_buffer(device);

        Self {
            voxel_buf,

            vertex_buf,
            index_buf,
            num_indices,
        }
    }
}
