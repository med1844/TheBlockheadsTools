use super::{super::gpu::voxel_util, GeometryBuffer, Texture, VoxelType};
use eframe::wgpu::{self, util::DeviceExt};

// Raymarch voxel renderer
pub struct VoxelRenderer {
    pub voxel_buf: wgpu::Buffer,
    pub world_dim_x_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    buffer_bind_group_layout: wgpu::BindGroupLayout,
    buffer_bind_group: wgpu::BindGroup,
    voxel_bind_group_layout: wgpu::BindGroupLayout,
    voxel_bind_group: wgpu::BindGroup,
}

impl VoxelRenderer {
    // Default world width in macro-chunks, used to pre-allocate the voxel buffer
    // before a real world is loaded.
    pub const DEFAULT_WORLD_WIDTH_CHUNK: u32 = 512;

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &wgpu::Device,
        camera_buf: &wgpu::Buffer,
        world_dim_x_buf: wgpu::Buffer,
        selected_block_buf: &wgpu::Buffer,
        hover_on_block_buf: &wgpu::Buffer,
        g_buffer: &GeometryBuffer,
        render_settings_buf: &wgpu::Buffer,
        albedo_texture: &Texture,
        reflect_texture: &Texture,
        destruct_texture: &Texture,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxel Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/voxel.wgsl").into()),
        });

        let uv_face_u32 = VoxelType::uv_at_face()
            .iter()
            .flat_map(|v| v.map(|v| v as u32))
            .collect::<Vec<_>>();
        let uv_at_face_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Texture UV Atlas Buffer"),
            contents: bytemuck::cast_slice(&uv_face_u32),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // --- Group 0: static uniforms + textures ---
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Texture sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // block id + face id -> texture uv offset
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // selected block
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // hover_on_block
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // render_settings uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // reflect texture
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // destruct texture
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // world_width_macro uniform (u32 = num chunks wide)
                wgpu::BindGroupLayoutEntry {
                    binding: 11,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Voxel Static Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&albedo_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&albedo_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: uv_at_face_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: selected_block_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: hover_on_block_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: render_settings_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(&reflect_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::Sampler(&reflect_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(&destruct_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: wgpu::BindingResource::Sampler(&destruct_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: world_dim_x_buf.as_entire_binding(),
                },
            ],
            label: Some("Voxel Static Bind Group"),
        });

        // --- Group 1: g-buffer textures (rebuilt on viewport resize) ---
        let buffer_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Voxel Buffer Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let buffer_bind_group =
            Self::create_buffer_bind_group(device, &buffer_bind_group_layout, g_buffer);

        // --- Group 2: per-world voxel data (rebuilt on world load with different width) ---
        let voxel_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Voxel Data Bind Group Layout"),
                entries: &[
                    // voxel storage buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let voxel_buf = voxel_util::create_buffer(device, Self::DEFAULT_WORLD_WIDTH_CHUNK as usize);
        let voxel_bind_group =
            Self::create_voxel_bind_group(device, &voxel_bind_group_layout, &voxel_buf);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &bind_group_layout,
                &buffer_bind_group_layout,
                &voxel_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[
                    // slot 0: uv. only r and g is filled. set a=1.0 to overwrite mesh output.
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba16Float,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    // slot 1: normal. set a=1.0 to overwrite mesh output, a=0.0 to keep it
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba16Float,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    // slot 2: translucency - semi-transparent voxels (alpha < 1.0)
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    // slot 3: flags
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::R8Uint,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
            buffer_bind_group_layout,
            buffer_bind_group,
            voxel_bind_group_layout,
            voxel_bind_group,
            voxel_buf,
            world_dim_x_buf,
        }
    }

    fn create_buffer_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        g_buffer: &GeometryBuffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&g_buffer.mesh_depth.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&g_buffer.mesh_depth.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&g_buffer.mesh_flags.view),
                },
            ],
            label: Some("Voxel Buffer Bind Group"),
        })
    }

    fn create_voxel_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        voxel_buf: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: voxel_buf.as_entire_binding(),
            }],
            label: Some("Voxel Data Bind Group"),
        })
    }

    pub fn resize(&mut self, g_buffer: &GeometryBuffer, device: &wgpu::Device) {
        self.buffer_bind_group =
            Self::create_buffer_bind_group(device, &self.buffer_bind_group_layout, g_buffer);
    }

    pub fn replace_voxel_buf(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        new_voxel_buf: wgpu::Buffer,
        world_dim_x: u32,
    ) {
        queue.write_buffer(
            &self.world_dim_x_buf,
            0,
            bytemuck::cast_slice(&[world_dim_x]),
        );
        self.voxel_buf = new_voxel_buf;
        self.voxel_bind_group =
            Self::create_voxel_bind_group(device, &self.voxel_bind_group_layout, &self.voxel_buf);
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, &self.buffer_bind_group, &[]);
        render_pass.set_bind_group(2, &self.voxel_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
