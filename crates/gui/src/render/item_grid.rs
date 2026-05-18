use super::{
    super::gpu::dw::{DwItem, DwItemInstanceRaw},
    DwIconVertex, Texture,
};
use eframe::{
    egui,
    wgpu::{self, util::DeviceExt},
};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use the_blockheads_tools_lib::game::item::ItemType;

const SCALE: f32 = 2.0;
pub const COL_PX: f32 = 16.0 * SCALE;
pub const ROW_PX: f32 = 16.0 * SCALE;

pub const SELECTOR_COLS: u32 = 32;
pub const SELECTOR_ROWS: u32 = 16;

pub const SELECTOR_SIZE: egui::Vec2 =
    egui::Vec2::new(SELECTOR_COLS as f32 * COL_PX, SELECTOR_ROWS as f32 * ROW_PX);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ItemGridUniforms {
    pub hovered_index: u32,
    pub selected_index: u32,
    /// Top-left of the visible scroll viewport in grid-pixel space.
    pub viewport_origin: [f32; 2],
    /// Width and height of the visible scroll viewport in grid-pixels.
    pub viewport_size: [f32; 2],
    /// Width and height of a single item cell in grid-pixels.
    pub cell_size: [f32; 2],
}

pub struct ItemGridRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    uniform_buf: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    uniform_bind_groups: HashMap<egui::Id, wgpu::BindGroup>,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    item_instance_bufs: HashMap<egui::Id, (wgpu::Buffer, u32)>,

    // item selector contains fixed amount of instances
    selector_instance_buf: wgpu::Buffer,
    num_selector_instances: u32,
}

impl ItemGridRenderer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &wgpu::Device,
        items_texture: &Texture,
        albedo_texture: &Texture,
        target_format: wgpu::TextureFormat,
        render_settings_buf: &wgpu::Buffer,
        camera_buf: &wgpu::Buffer,
        destruct_texture: &Texture,
        reflect_texture: &Texture,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Item Grid Shader"),
            source: wgpu::ShaderSource::Wgsl(
                wgsl_macro::include_wgsl!("src/render/shader/item_grid.wgsl").into(),
            ),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
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
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // render_settings
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
                // camera
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
                // tile_destruct texture
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // tile_destruct sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // tile_reflect texture
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // tile_reflect sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("item_grid_bind_group_layout"),
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Item Grid Uniforms Buffer"),
            size: std::mem::size_of::<ItemGridUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&items_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&items_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&albedo_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&albedo_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: render_settings_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: camera_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&destruct_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&destruct_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(&reflect_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::Sampler(&reflect_texture.sampler),
                },
            ],
            label: Some("item_grid_bind_group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Item Grid Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Item Grid Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[DwIconVertex::desc(), DwItemInstanceRaw::desc()],
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
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Item Grid Vertex Buffer"),
            contents: bytemuck::cast_slice(super::DwIconVertex::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Item Grid Index Buffer"),
            contents: bytemuck::cast_slice(super::DwIconVertex::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instances: Vec<DwItemInstanceRaw> = ItemType::iter()
            .enumerate()
            .map(|(i, item_type)| {
                let col = (i as u32) % SELECTOR_COLS;
                let row = (i as u32) / SELECTOR_COLS;
                DwItem::from_item_type([col, row].map(|v| v as f32), item_type)
                    .grid_instance(i as u32)
            })
            .collect();

        let instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Item Selector Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let num_instances = instances.len() as u32;

        Self {
            pipeline,
            bind_group,
            vertex_buf,
            index_buf,
            uniform_buf,
            uniform_bind_group,

            uniform_bind_groups: HashMap::new(),
            uniform_bind_group_layout,
            item_instance_bufs: HashMap::new(),

            selector_instance_buf: instance_buf,
            num_selector_instances: num_instances,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        hovered_index: Option<u32>,
        selected_index: Option<u32>,
        viewport: egui::Rect,
        pixels_per_point: f32,
        id_instances: Option<(egui::Id, &[DwItemInstanceRaw])>,
    ) {
        let viewport_ppp = viewport * pixels_per_point;
        let uniforms = ItemGridUniforms {
            hovered_index: hovered_index.unwrap_or(u32::MAX),
            selected_index: selected_index.unwrap_or(u32::MAX),
            viewport_origin: viewport_ppp.min.into(),
            viewport_size: viewport_ppp.size().into(),
            cell_size: [COL_PX * pixels_per_point, ROW_PX * pixels_per_point],
        };
        if let Some((id, instances)) = id_instances {
            let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Item Grid Uniforms Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                }],
                label: Some("uniform_bind_group"),
            });
            self.uniform_bind_groups.insert(id, uniform_bind_group);

            let instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chest Instance Buffer"),
                contents: bytemuck::cast_slice(instances),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let num_instances = instances.len() as u32;
            self.item_instance_bufs
                .insert(id, (instance_buf, num_instances));
        } else {
            queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&[uniforms]));
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass<'_>, id: Option<&egui::Id>) {
        let (instance_buf, num_instances) = id
            .and_then(|id| self.item_instance_bufs.get(id).map(|(a, b)| (a, b)))
            .unwrap_or((&self.selector_instance_buf, &self.num_selector_instances));
        let uniform_bind_group = id
            .and_then(|id| self.uniform_bind_groups.get(id))
            .unwrap_or(&self.uniform_bind_group);
        if *num_instances > 0 {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_bind_group(1, uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            render_pass.set_vertex_buffer(1, instance_buf.slice(..));
            render_pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..DwIconVertex::INDICES.len() as u32, 0, 0..*num_instances);
        }
    }
}
