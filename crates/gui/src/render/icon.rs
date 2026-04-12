use super::{DwChunkBuf, DwIconInstanceRaw, DwIconVertex, ID_TEXTURE_FORMAT, ImageType, Texture};
use eframe::wgpu::{self, util::DeviceExt};

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
        hover_on_id_buf: &wgpu::Buffer,
        selected_id_buf: &wgpu::Buffer,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Dynamic Object Icon Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/dw_icon.wgsl").into()),
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
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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
                // Hover on id
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
                // Selected id
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
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
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: hover_on_id_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: selected_id_buf.as_entire_binding(),
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
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: ID_TEXTURE_FORMAT,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
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
            vertex_buf,
            index_buf,
            bind_group,
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass<'_>, dw_buf: &[DwChunkBuf]) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        for dw_chunk_buf in dw_buf {
            if dw_chunk_buf.num_instances > 0 {
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
}
