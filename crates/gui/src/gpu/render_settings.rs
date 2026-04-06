use eframe::wgpu::{self, util::DeviceExt};
use glam::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct RenderSettings {
    pub light_dir: Vec3,
    pub enable_reflect: bool,
    pub enable_destruct: bool,
    pub enable_ssao: bool,
    pub ambient_light: f32,
    pub shininess: f32,
    pub specular_intensity: f32,
    pub min_depth_factor: f32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            light_dir: Vec3::new(-2.5, 2.0, 3.0),
            enable_reflect: true,
            enable_destruct: true,
            enable_ssao: true,
            ambient_light: 0.1,
            shininess: 256.0,
            specular_intensity: 1.5,
            min_depth_factor: 0.6,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderSettingsUniform {
    pub light_dir: [f32; 3],     // 12 bytes
    pub enable_reflect: u32,     // 4 bytes
    pub enable_destruct: u32,    // 4 bytes
    pub enable_ssao: u32,        // 4 bytes (offset 20)
    pub ambient_light: f32,      // 4 bytes (offset 24)
    pub shininess: f32,          // 4 bytes (offset 28)
    pub specular_intensity: f32, // 4 bytes (offset 32)
    pub min_depth_factor: f32,   // 4 bytes (offset 36)
    pub _padding: [u32; 2],      // 8 bytes (pad to 48)
}

impl RenderSettings {
    pub fn uniform(&self) -> RenderSettingsUniform {
        RenderSettingsUniform {
            light_dir: [self.light_dir.x, self.light_dir.y, self.light_dir.z],
            enable_reflect: self.enable_reflect as u32,
            enable_destruct: self.enable_destruct as u32,
            enable_ssao: self.enable_ssao as u32,
            ambient_light: self.ambient_light,
            shininess: self.shininess,
            specular_intensity: self.specular_intensity,
            min_depth_factor: self.min_depth_factor,
            _padding: [0; 2],
        }
    }

    pub fn buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let uniform = self.uniform();
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Render Settings Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
}
