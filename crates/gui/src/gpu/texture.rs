use egui_wgpu::wgpu;

pub struct RgbaTexture {
    // pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl RgbaTexture {
    pub fn new(img: &[u8], size: (u32, u32), device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let (w, h) = size;

        let texture_size = wgpu::Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1, // It's a 2D texture
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1, // No mipmaps for nearest neighbor
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb, // Standard format for images
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Texture Atlas"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            img,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(h as u32 * 4), // 4 bytes per pixel (RGBA)
                rows_per_image: Some(w as u32),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler (Nearest Neighbor)"),
            address_mode_u: wgpu::AddressMode::ClampToEdge, // Or Repeat, depending on your needs
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest, // Nearest for pixelated look
            min_filter: wgpu::FilterMode::Nearest, // Nearest for pixelated look
            mipmap_filter: wgpu::FilterMode::Nearest, // Nearest for mipmaps (even if count is 1)
            ..Default::default()
        });

        Self {
            // texture,
            view,
            sampler,
        }
    }
}
