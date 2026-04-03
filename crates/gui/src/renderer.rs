use super::{
    gpu::{
        CameraUniform, GpuBlockCoordUniform, Texture, VoxelType,
        dw::{DwChunkBuf, DwChunkObjId, DwIconInstanceRaw, DwIconVertex, DwVertex},
    },
    image_type::ImageType,
};
use eframe::{
    egui::{self},
    egui_wgpu,
    wgpu::{self, util::DeviceExt},
};
use std::sync::{Arc, Mutex};
use zune_png::PngDecoder;

const ID_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Uint;

pub(crate) enum ResizeOutcome {
    Unchanged,
    Resized,
}

pub(crate) struct GeometryBuffer {
    size: (u32, u32),
    albedo: Texture,
    // transparent voxels needs depth texture of meshes to be obscured correctly during ray marching
    mesh_depth: Texture,
    voxel_depth: Texture,
    dyn_obj_id: Texture,
}

impl GeometryBuffer {
    pub const DEFAULT_WIDTH: u32 = 1920;
    pub const DEFAULT_HEIGHT: u32 = 1080;

    pub fn new(size: (u32, u32), device: &wgpu::Device) -> Self {
        let color_texture = Texture::new(
            size,
            device,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::TextureFormat::Bgra8Unorm,
        );
        let voxel_depth = Texture::new(
            size,
            device,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::TextureFormat::Depth32Float,
        );
        let depth_texture = Texture::new(
            size,
            device,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::TextureFormat::Depth32Float,
        );
        let dyn_obj_id_texture = Texture::new(
            size,
            device,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            wgpu::TextureFormat::R32Uint,
        );
        Self {
            size,
            albedo: color_texture,
            voxel_depth,
            mesh_depth: depth_texture,
            dyn_obj_id: dyn_obj_id_texture,
        }
    }

    pub fn default(device: &wgpu::Device) -> Self {
        Self::new((Self::DEFAULT_WIDTH, Self::DEFAULT_HEIGHT), device)
    }

    pub fn resize(&mut self, size: (u32, u32), device: &wgpu::Device) -> ResizeOutcome {
        if self.size != size {
            *self = Self::new(size, device);
            ResizeOutcome::Resized
        } else {
            ResizeOutcome::Unchanged
        }
    }
}

// Raymarch voxel renderer
pub struct VoxelRenderer {
    voxel_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    depth_bind_group_layout: wgpu::BindGroupLayout,
    depth_bind_group: wgpu::BindGroup,
}

impl VoxelRenderer {
    pub fn new(
        device: &wgpu::Device,
        voxel_buf: wgpu::Buffer,
        camera_buf: &wgpu::Buffer,
        selected_block_buf: &wgpu::Buffer,
        hover_on_block_buf: &wgpu::Buffer,
        texture: &Texture,
        target_format: wgpu::TextureFormat,
        g_buffer: &GeometryBuffer,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxel Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("voxel.wgsl").into()),
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

        let is_transparent_u32 = VoxelType::transparency()
            .into_iter()
            .map(|b| b as u32)
            .collect::<Vec<_>>();
        let is_transparent_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Is Transparent Buffer"),
            contents: bytemuck::cast_slice(&is_transparent_u32),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create a bind group layout and bind group to link the uniform buffer to the shader.
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT, // Only fragment shader needs this
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true }, // Read-only storage
                        has_dynamic_offset: false, // No dynamic offsets for this example
                        min_binding_size: None,    // No minimum size requirement
                    },
                    count: None, // Not an array of buffers
                },
                // Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
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
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // block id + face id -> texture uv offset
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true }, // Or Uniform for smaller data
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // selected block
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
                // is_transparent array
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("bind_group_layout"),
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
                    resource: voxel_buf.as_entire_binding(), // Bind the whole voxel buffer
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: uv_at_face_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: selected_block_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: hover_on_block_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: is_transparent_buf.as_entire_binding(),
                },
            ],
            label: Some("bind_group"),
        });

        let depth_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Voxel Depth Bind Group Layout"),
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
                ],
            });

        let depth_bind_group =
            Self::create_depth_bind_group(device, &depth_bind_group_layout, g_buffer);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, &depth_bind_group_layout],
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
                    Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: ID_TEXTURE_FORMAT,
                        blend: None,
                        write_mask: wgpu::ColorWrites::empty(),
                    }),
                ],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back), // Prevent rendering the inside of the cube
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
            voxel_buf,
            pipeline,
            bind_group,
            depth_bind_group_layout,
            depth_bind_group,
        }
    }

    fn create_depth_bind_group(
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
            ],
            label: Some("Voxel Depth Bind Group"),
        })
    }

    pub fn resize(&mut self, g_buffer: &GeometryBuffer, device: &wgpu::Device) {
        self.depth_bind_group =
            Self::create_depth_bind_group(device, &self.depth_bind_group_layout, g_buffer);
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, &self.depth_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

pub struct DwIconRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl DwIconRenderer {
    const ITEM_IMAGE_TYPE: &[[ImageType; 6]] = {
        use ImageType::*;
        &[
            [MinedStone; 6], // Stone
                             // Kiln
                             // Brick
                             // Limestone
                             // MinedLimestone
                             // Marble
                             // MinedMarble
                             // Furnace
                             // WoodworkBench
                             // TaylorsBench
                             // Press
                             // Sandstone
                             // MinedSandstone
                             // RedMarble
                             // MinedRedMarble
                             // WovenFlaxMat
                             // YellowFlaxMat
                             // RedFlaxMat
                             // Glass
                             // Chest
                             // DeprecatedFood
                             // GoldBlock
                             // DeprecatedMango
                             // Rock
                             // Dirt
                             // Wood
                             // WorkBench
                             // Sand
                             // ToolBench
                             // LapisLazuli
                             // MinedLapisLazuli
                             // CraftBench
                             // MixingBench
                             // ReinforcedPlatform
                             // DeprecatedStonePickaxe
                             // DeprecatedCopperIngot
                             // Ice
                             // DyeBench
                             // Compost
                             // Basalt
                             // MinedBasalt
                             // Safe
                             // CopperBlock
                             // TinBlock
                             // BronzeBlock
                             // IronBlock
                             // SteelBlock
                             // MetalworkBench
                             // GoldenChest
                             // DeprecatedBronzeMachete
                             // PortalChest
                             // BlackSand
                             // BlackGlass
                             // SteamGenerator
                             // ElectricKiln
                             // ElectricFurnace
                             // ElectricMetalworkBench
                             // ElectricStove
                             // SolarPanel
                             // Flywheel
                             // ArmorBench
                             // TrainYard
                             // BuildersBench
                             // ElevatorShaft
                             // ElectricElevatorMotor
                             // PlatiumBlock
                             // CarbonFiberBlock
                             // TitaniumBlock
                             // DeprecatedIronSword
                             // ElectricPress
                             // Gravel
                             // CompostBin
                             // EggExtractor
                             // PizzaOven
                             // AmethystBlock
                             // SapphireBlock
                             // EmeraldBlock
                             // RubyBlock
                             // DiamondBlock
                             // Plaster
                             // FeederChest
                             // LuminousPlaster
        ]
    };

    pub fn new(
        device: &wgpu::Device,
        camera_buf: &wgpu::Buffer,
        items_texture: &Texture,
        tile_map_texture: &Texture,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Dynamic Object Icon Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("dw_icon.wgsl").into()),
        });

        let block_image_types = Self::ITEM_IMAGE_TYPE
            .iter()
            .flat_map(|v| v.map(|v| v as u32))
            .collect::<Vec<_>>();

        let uv_at_face_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Texture UV Atlas Buffer"),
            contents: bytemuck::cast_slice(&block_image_types),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dynamic Object Icon Vertex Buffer"),
            contents: bytemuck::cast_slice(DwIconVertex::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dynamic Object Icon Index Buffer"),
            contents: bytemuck::cast_slice(DwIconVertex::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Camera Uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Items Texture
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
                // Items Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // TileMap Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // TileMap Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Voxel UV Atlas
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("dynamic_object_icon_bind_group_layout"),
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
                    resource: wgpu::BindingResource::TextureView(&items_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&items_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&tile_map_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&tile_map_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: uv_at_face_buf.as_entire_binding(),
                },
            ],
            label: Some("dynamic_object_icon_bind_group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Dynamic Object Icon Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Dynamic Object Icon Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_dynamic_object_icon"),
                buffers: &[DwIconVertex::desc(), DwIconInstanceRaw::desc()],
                compilation_options: Default::default(),
            },
            // ... fragment and other states remain the same
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_dynamic_object_icon"),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: ID_TEXTURE_FORMAT,
                        blend: None,
                        write_mask: wgpu::ColorWrites::empty(),
                    }),
                ],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
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
            vertex_buf,
            index_buf,
            bind_group,
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass<'_>, dw_buf: &[DwChunkBuf]) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        for dw_chunk_buf in dw_buf {
            render_pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            render_pass.set_vertex_buffer(1, dw_chunk_buf.instance_buf.slice(..));
            render_pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(
                0..DwIconVertex::INDICES.len() as u32,
                0,
                0..dw_chunk_buf.num_instances,
            );
        }
    }
}

pub struct DwSpriteRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl DwSpriteRenderer {
    pub fn new(
        device: &wgpu::Device,
        camera_buf: &wgpu::Buffer,
        tile_map_texture: &Texture,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("DW Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("dw_sprite.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("dw_bind_group_layout"),
            entries: &[
                // Camera
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Tilemap Texture
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
                // Tilemap Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
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
                    resource: wgpu::BindingResource::TextureView(&tile_map_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&tile_map_texture.sampler),
                },
            ],
            label: Some("dw_bind_group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("DW Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("DW Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[DwVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: ID_TEXTURE_FORMAT,
                        blend: None,
                        write_mask: wgpu::ColorWrites::RED,
                    }),
                ],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
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
        }
    }

    pub fn render(&self, rpass: &mut wgpu::RenderPass<'_>, dw_buf: &[DwChunkBuf]) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        for dw_chunk_buf in dw_buf {
            rpass.set_vertex_buffer(0, dw_chunk_buf.vertex_buf.slice(..));
            rpass.set_index_buffer(dw_chunk_buf.index_buf.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..dw_chunk_buf.num_indices, 0, 0..1);
        }
    }
}

pub struct GridRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl GridRenderer {
    pub fn new(
        device: &wgpu::Device,
        camera_buf: &wgpu::Buffer,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Grid Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("grid.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Grid Bind Group Layout"),
            entries: &[
                // Camera Uniform
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
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Grid Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grid Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_grid"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_grid"),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: ID_TEXTURE_FORMAT,
                        blend: None,
                        write_mask: wgpu::ColorWrites::empty(),
                    }),
                ],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
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
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

pub struct BlitRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl BlitRenderer {
    pub fn new(
        device: &wgpu::Device,
        g_buffer: &GeometryBuffer,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blit Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("blit.wgsl").into()),
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blit Bind Group Layout"),
            entries: &[
                // Color Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Color Texture sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = Self::create_bind_group(&bind_group_layout, g_buffer, device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blit Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout], // Use the new layout
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blit Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_blit"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_blit"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        Self {
            pipeline,
            bind_group_layout,
            bind_group,
        }
    }

    fn create_bind_group(
        bind_group_layout: &wgpu::BindGroupLayout,
        g_buffer: &GeometryBuffer,
        device: &wgpu::Device,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&g_buffer.albedo.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&g_buffer.albedo.sampler),
                },
            ],
            label: Some("Blit Bind Group"),
        })
    }

    pub fn resize(&mut self, g_buffer: &GeometryBuffer, device: &wgpu::Device) {
        self.bind_group = Self::create_bind_group(&self.bind_group_layout, g_buffer, device);
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

pub struct RenderResources {
    camera_buf: wgpu::Buffer,
    selected_block_buf: wgpu::Buffer,
    hover_on_block_buf: wgpu::Buffer,

    g_buffer: GeometryBuffer,

    // Stores the object ID of the pixel under cursor from g_buffer.dyn_obj_id
    staging_buffer: wgpu::Buffer,
    hover_on_dyn_obj_id: Arc<Mutex<Option<DwChunkObjId>>>,

    voxel: VoxelRenderer,
    dw_icon: DwIconRenderer,
    dw_sprite: DwSpriteRenderer,
    grid: GridRenderer,
    blit: BlitRenderer,
}

impl RenderResources {
    const STAGING_BUFFER_SIZE: u64 = std::mem::size_of::<u32>() as u64; // only read single pixel

    pub fn new(
        state: &egui_wgpu::RenderState,
        camera_buf: wgpu::Buffer,
        voxel_buf: wgpu::Buffer,
        selected_block_buf: wgpu::Buffer,
        hover_on_block_buf: wgpu::Buffer,
        hover_on_dyn_obj_id: Arc<Mutex<Option<DwChunkObjId>>>,
    ) -> Self {
        let device = &state.device;
        let queue = &state.queue;
        let target_format = state.target_format;

        let tile_map_texture = {
            let bytes = include_bytes!("../resources/TileMap.png");
            let mut decoder = PngDecoder::new(bytes);
            let img = decoder.decode().unwrap().u8().unwrap();
            Texture::from_img(img.as_slice(), (512, 512), device, queue)
        };
        let items_texture = {
            let bytes = include_bytes!("../resources/Items.png");
            let mut decoder = PngDecoder::new(bytes);
            let img = decoder.decode().unwrap().u8().unwrap();
            Texture::from_img(img.as_slice(), (512, 256), device, queue)
        };
        let g_buffer = GeometryBuffer::default(device);
        let staging_buffer = device.create_buffer(&wgpu::wgt::BufferDescriptor {
            label: Some("staging buffer"),
            size: Self::STAGING_BUFFER_SIZE,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let blit = BlitRenderer::new(device, &g_buffer, target_format);

        Self {
            voxel: VoxelRenderer::new(
                device,
                voxel_buf,
                &camera_buf,
                &selected_block_buf,
                &hover_on_block_buf,
                &tile_map_texture,
                target_format,
                &g_buffer,
            ),
            dw_icon: DwIconRenderer::new(
                device,
                &camera_buf,
                &items_texture,
                &tile_map_texture,
                target_format,
            ),
            dw_sprite: DwSpriteRenderer::new(device, &camera_buf, &tile_map_texture, target_format),
            grid: GridRenderer::new(device, &camera_buf, target_format),
            blit,

            camera_buf,
            selected_block_buf,
            hover_on_block_buf,
            g_buffer,

            staging_buffer,
            hover_on_dyn_obj_id,
        }
    }

    pub fn camera_buf(&self) -> &wgpu::Buffer {
        &self.camera_buf
    }

    pub fn selected_block_buf(&self) -> &wgpu::Buffer {
        &self.selected_block_buf
    }

    pub fn hover_on_block_buf(&self) -> &wgpu::Buffer {
        &self.hover_on_block_buf
    }

    pub fn voxel_buf(&self) -> &wgpu::Buffer {
        &self.voxel.voxel_buf
    }

    pub fn resize(&mut self, size: (u32, u32), device: &wgpu::Device) {
        if let ResizeOutcome::Resized = self.g_buffer.resize(size, device) {
            self.blit.resize(&self.g_buffer, device);
            self.voxel.resize(&self.g_buffer, device);
        }
    }

    pub fn render_mesh_pass(&self, render_pass: &mut wgpu::RenderPass<'_>, dw_buf: &[DwChunkBuf]) {
        self.dw_sprite.render(render_pass, dw_buf);
    }

    pub fn render_voxel_pass(&self, render_pass: &mut wgpu::RenderPass<'_>, show_grid: bool) {
        self.voxel.render(render_pass);
        // self.dw_icon.render(render_pass, dw_buf);
        if show_grid {
            self.grid.render(render_pass);
        }
    }

    pub fn blit(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        self.blit.render(render_pass);
    }

    pub fn read_hover_id(
        &self,
        pos: (u32, u32),
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let (x, y) = pos;
        let (sx, sy) = self.g_buffer.size;
        if x < sx && y < sy {
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfoBase {
                    texture: &self.g_buffer.dyn_obj_id.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x, y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfoBase {
                    buffer: &self.staging_buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: None,
                        rows_per_image: None,
                    },
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );

            // On WASM, device.poll is no-op, and the callback never finishes before queue.submit.
            // This will cause rendering issue. Thus reading from GPU is disabled on WASM.
            #[cfg(not(target_arch = "wasm32"))]
            {
                let bounds = 0..RenderResources::STAGING_BUFFER_SIZE;
                let staging_buffer = self.staging_buffer.clone();
                let hover_on_dyn_obj_id = self.hover_on_dyn_obj_id.clone();
                self.staging_buffer
                    .map_async(wgpu::MapMode::Read, bounds.clone(), move |result| {
                        if result.is_ok() {
                            let buffer_view = staging_buffer.get_mapped_range(bounds);
                            let raw_id = u32::from_ne_bytes(
                                buffer_view[..]
                                    .try_into()
                                    .expect("array size should be exactly 4"),
                            );
                            let mut guard = hover_on_dyn_obj_id.lock().expect("should lock mutex");
                            *guard = DwChunkObjId::try_from_u32(raw_id);
                        }
                        staging_buffer.unmap();
                    });
                device
                    .poll(wgpu::wgt::PollType::wait_indefinitely())
                    .expect("should poll");
            }
        }
    }
}

pub struct Render3dCallback {
    pub camera_uniform: CameraUniform,
    pub dw_chunks: Vec<DwChunkBuf>,
    pub show_grid: bool,
    pub selected_block_coord_uniform: GpuBlockCoordUniform,
    pub hover_on_block_coord_uniform: GpuBlockCoordUniform,
    pub mouse_physical_pos: Option<(f32, f32)>,
    pub world_viewport_rect: egui::Rect,
}

impl egui_wgpu::CallbackTrait for Render3dCallback {
    fn prepare(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        egui_encoder: &mut eframe::wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        let r: &mut RenderResources = callback_resources.get_mut().unwrap();
        let (vp_w, vp_h) = (self.world_viewport_rect.size() * screen_descriptor.pixels_per_point)
            .round()
            .into();
        r.resize((vp_w as u32, vp_h as u32), device);
        queue.write_buffer(
            r.camera_buf(),
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        queue.write_buffer(
            r.selected_block_buf(),
            0,
            bytemuck::cast_slice(&[self.selected_block_coord_uniform]),
        );
        queue.write_buffer(
            r.hover_on_block_buf(),
            0,
            bytemuck::cast_slice(&[self.hover_on_block_coord_uniform]),
        );
        {
            let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mesh pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.albedo.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.dyn_obj_id.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &r.g_buffer.mesh_depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            r.render_mesh_pass(&mut render_pass, &self.dw_chunks);
        }

        {
            let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("voxel pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.albedo.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &r.g_buffer.dyn_obj_id.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &r.g_buffer.voxel_depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            r.render_voxel_pass(&mut render_pass, self.show_grid);
        }

        if let Some((x, y)) = self.mouse_physical_pos {
            let x = x.round() as u32;
            let y = y.round() as u32;
            r.read_hover_id((x, y), device, egui_encoder);
        }

        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'static>,
        callback_resources: &egui_wgpu::CallbackResources,
    ) {
        let r: &RenderResources = callback_resources.get().unwrap();
        r.blit(render_pass);
    }
}
