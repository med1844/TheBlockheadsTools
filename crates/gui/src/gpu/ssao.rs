use super::texture::Texture;
use bytemuck::{Pod, Zeroable};
use eframe::wgpu;
use glam::Vec3;

pub const SSAO_KERNEL_SIZE: u32 = 64;
pub const SSAO_MAX_KERNEL_SIZE: usize = 256;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SsaoUniform {
    pub kernel_size: u32,
    pub _padding: [u32; 3],
    pub kernel: [[f32; 4]; SSAO_MAX_KERNEL_SIZE],
}

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let val = (self.state >> 32) as u32;
        (val as f32) / (u32::MAX as f32)
    }

    fn next_f32_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

pub fn build_ssao_kernel() -> SsaoUniform {
    let mut rng = SimpleRng::new(12345);
    let mut kernel = [[0.0; 4]; SSAO_MAX_KERNEL_SIZE];
    for (i, kernel_i) in kernel.iter_mut().enumerate() {
        let mut sample = Vec3::new(
            rng.next_f32_range(-1.0, 1.0),
            rng.next_f32_range(-1.0, 1.0),
            rng.next_f32_range(0.0, 1.0), // Hemisphere pointing towards +Z
        );
        sample = sample.normalize();

        let mut scale = i as f32 / SSAO_KERNEL_SIZE as f32;
        scale = 0.1 + scale * scale * (1.0 - 0.1);
        sample *= scale;

        *kernel_i = [sample.x, sample.y, sample.z, 0.0];
    }
    SsaoUniform {
        kernel_size: SSAO_KERNEL_SIZE,
        _padding: [0; 3],
        kernel,
    }
}

pub fn build_ssao_noise_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> Texture {
    let mut rng = SimpleRng::new(54321);
    let mut noise_data = [0u8; 16 * 4];
    for i in 0..16 {
        let noise = Vec3::new(
            rng.next_f32_range(-1.0, 1.0),
            rng.next_f32_range(-1.0, 1.0),
            0.0,
        )
        .normalize();

        noise_data[i * 4] = ((noise.x * 0.5 + 0.5) * 255.0) as u8;
        noise_data[i * 4 + 1] = ((noise.y * 0.5 + 0.5) * 255.0) as u8;
        noise_data[i * 4 + 2] = 0;
        noise_data[i * 4 + 3] = 255;
    }

    let size = wgpu::Extent3d {
        width: 4,
        height: 4,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("SSAO Noise Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &noise_data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * 4),
            rows_per_image: Some(4),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("SSAO Noise Sampler"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    Texture {
        texture,
        view,
        sampler,
        texture_size: size,
    }
}
