use eframe::wgpu::{self, util::DeviceExt};
use the_blockheads_tools_lib::game::coord::{BlockCoord, ChunkCoord};

pub(crate) trait Coord {
    fn x_u32(&self) -> u32;
    fn y_u32(&self) -> u32;
}

impl Coord for BlockCoord {
    fn x_u32(&self) -> u32 {
        self.x()
    }

    fn y_u32(&self) -> u32 {
        self.y() as u32
    }
}

impl Coord for ChunkCoord {
    fn x_u32(&self) -> u32 {
        self.x()
    }

    fn y_u32(&self) -> u32 {
        self.y() as u32
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GpuCoord<T> {
    coord: Option<T>,
}

impl<T> Default for GpuCoord<T> {
    fn default() -> Self {
        Self { coord: None }
    }
}

impl<T> From<GpuCoord<T>> for Option<T> {
    fn from(value: GpuCoord<T>) -> Self {
        value.coord
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuCoordUniform([u32; 4]);

impl GpuCoordUniform {
    pub fn create_buffer(self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block Coord Buffer"),
            contents: bytemuck::cast_slice(&[self]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
}

impl<T: Coord> GpuCoord<T> {
    pub fn uniform(&self) -> GpuCoordUniform {
        let mut uniform = [0; 4];
        if let Some(coord) = &self.coord {
            uniform[0] = 1;
            uniform[1] = coord.x_u32();
            uniform[2] = coord.y_u32();
        }
        GpuCoordUniform(uniform)
    }
}

impl<T: Copy> GpuCoord<T> {
    pub fn coord(&self) -> Option<T> {
        self.coord
    }
}

impl<T: Eq> GpuCoord<T> {
    pub fn toggle(&mut self, new_coord: Option<T>) {
        if new_coord == self.coord {
            self.coord = None;
        } else {
            self.coord = new_coord;
        }
    }
}

impl<T> GpuCoord<T> {
    pub fn update(&mut self, new_coord: Option<T>) {
        self.coord = new_coord;
    }
}
