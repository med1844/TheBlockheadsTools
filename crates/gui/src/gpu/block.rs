use eframe::wgpu::{self, util::DeviceExt};
use the_blockheads_tools_lib::game::coord::BlockCoord;

#[derive(Default)]
pub struct GpuBlockCoord {
    coord: Option<BlockCoord>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuBlockCoordUniform([u32; 4]);

impl GpuBlockCoord {
    pub fn to_uniform(&self) -> GpuBlockCoordUniform {
        let mut uniform = [0; 4];
        if let Some(coord) = &self.coord {
            uniform[0] = 1;
            uniform[1] = coord.x();
            uniform[2] = coord.y() as u32;
        }
        GpuBlockCoordUniform(uniform)
    }

    pub fn to_buf(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let uniform = self.to_uniform();
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Selected Block Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn toggle(&mut self, new_coord: Option<BlockCoord>) {
        if new_coord == self.coord {
            self.coord = None;
        } else {
            self.coord = new_coord;
        }
    }

    pub fn update(&mut self, new_coord: Option<BlockCoord>) {
        self.coord = new_coord;
    }
}
