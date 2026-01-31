use crate::gpu::dw::DwChunkBuf;

use super::{
    gpu::{
        RgbaTexture, VoxelType,
        dw::{DwIconInstanceRaw, DwIconVertex},
    },
    image_type::ImageType,
};
use eframe::{
    egui_wgpu,
    wgpu::{self, util::DeviceExt},
};
use zune_png::PngDecoder;

// Raymarch voxel renderer
pub struct VoxelRenderer {
    voxel_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl VoxelRenderer {
    pub fn new(
        device: &wgpu::Device,
        voxel_buf: wgpu::Buffer,
        camera_buf: &wgpu::Buffer,
        selected_block_buf: &wgpu::Buffer,
        hover_on_block_buf: &wgpu::Buffer,
        texture: &RgbaTexture,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("voxel.wgsl").into()),
        });

        let uv_face_u32 = VoxelType::UV_AT_FACE
            .iter()
            .flat_map(|v| v.map(|v| v as u32))
            .collect::<Vec<_>>();
        let uv_at_face_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Texture UV Atlas Buffer"),
            contents: bytemuck::cast_slice(&uv_face_u32),
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
                // hover on block
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
            ],
            label: Some("bind_group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout], // Use the new layout
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
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back), // Prevent rendering the inside of the cube
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        Self {
            voxel_buf,
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
        items_texture: &RgbaTexture,
        tilemap_texture: &RgbaTexture,
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
                    resource: wgpu::BindingResource::TextureView(&tilemap_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&tilemap_texture.sampler),
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
            bind_group,
        }
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

    voxel: VoxelRenderer,
    dw: DwIconRenderer,
    grid: GridRenderer,
}

impl RenderResources {
    pub fn new(
        state: &egui_wgpu::RenderState,
        camera_buf: wgpu::Buffer,
        voxel_buf: wgpu::Buffer,
        selected_block_buf: wgpu::Buffer,
        hover_on_block_buf: wgpu::Buffer,
    ) -> Self {
        let device = &state.device;
        let queue = &state.queue;
        let target_format = state.target_format;

        let tile_map_texture = {
            let bytes = include_bytes!("../resources/TileMap.png");
            let mut decoder = PngDecoder::new(bytes);
            let img = decoder.decode().unwrap().u8().unwrap();
            RgbaTexture::new(img.as_slice(), (512, 512), device, queue)
        };
        let items_texture = {
            let bytes = include_bytes!("../resources/Items.png");
            let mut decoder = PngDecoder::new(bytes);
            let img = decoder.decode().unwrap().u8().unwrap();
            RgbaTexture::new(img.as_slice(), (512, 256), device, queue)
        };

        Self {
            voxel: VoxelRenderer::new(
                device,
                voxel_buf,
                &camera_buf,
                &selected_block_buf,
                &hover_on_block_buf,
                &tile_map_texture,
                target_format,
            ),
            dw: DwIconRenderer::new(
                device,
                &camera_buf,
                &items_texture,
                &tile_map_texture,
                target_format,
            ),
            grid: GridRenderer::new(device, &camera_buf, target_format),
            camera_buf,
            selected_block_buf,
            hover_on_block_buf,
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

    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        dw_buf: &[DwChunkBuf],
        show_grid: bool,
    ) {
        self.voxel.render(render_pass);
        self.dw.render(render_pass, dw_buf);
        if show_grid {
            self.grid.render(render_pass);
        }
    }
}
