use egui_wgpu::wgpu::{self, util::DeviceExt};
use the_blockheads_tools_lib::BlockCoord;

pub struct SelectedBlock {
    coord: Option<BlockCoord>,
    uniform: [u32; 4],
    pub buf: wgpu::Buffer,
}

impl SelectedBlock {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform = [0; 4];
        Self {
            coord: None,
            uniform,
            buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }

    pub fn coord(&self) -> &Option<BlockCoord> {
        &self.coord
    }

    pub fn update(&mut self, queue: &wgpu::Queue, new_coord: Option<BlockCoord>) {
        if new_coord == self.coord {
            self.coord = None;
            self.uniform = [0; 4];
        } else {
            self.coord = new_coord;
            match &self.coord {
                Some(coord) => {
                    self.uniform[0] = 1;
                    self.uniform[1] = coord.x() as u32;
                    self.uniform[2] = coord.y() as u32;
                }
                None => {
                    self.uniform = [0; 4];
                }
            }
        }
        queue.write_buffer(&self.buf, 0, bytemuck::cast_slice(self.uniform.as_slice()));
    }
}
