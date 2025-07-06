use crate::gpu::{AllChunk, UVMappedBlockType};
use crate::{egui_tools::EguiRenderer, gpu::Vertex};
use egui_wgpu::wgpu::{SurfaceError, util::DeviceExt};
use egui_wgpu::{ScreenDescriptor, wgpu};
use glam::{Mat4, Vec3};
use std::sync::Arc;
use the_blockheads_tools_lib::{
    BhResult, BlockContent, BlockCoord, BlockView, ChunkBlockCoord, WorldDb,
};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use zune_png::PngDecoder;

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

fn load_chunk() -> BhResult<AllChunk> {
    let world_db =
        WorldDb::from_path("../../test_data/saves/3d716d9bbf89c77ef5001e9cd227ec29/world_db")?;
    if let Some(mut world_db) = world_db {
        let x = world_db.main.world_v2.start_portal_pos_x;
        let y = world_db.main.world_v2.start_portal_pos_y;
        let start_portal_pos = BlockCoord::new(x, (y - 1) as u16)?;
        let mut blocks = Vec::with_capacity(32 * 32 * 3);
        let chunk = world_db.blocks.chunk_at(start_portal_pos).unwrap()?;
        for y in 0..32 {
            for x in 0..32 {
                let block = chunk.block_at(&ChunkBlockCoord::new(x, y)?);
                let fg_type: UVMappedBlockType = (
                    block.fg()?,
                    block.fg_content().unwrap_or(BlockContent::None),
                )
                    .try_into()?;
                let bg_type: UVMappedBlockType = (block.bg()?, BlockContent::None).try_into()?;
                blocks.push(bg_type);
                blocks.push(fg_type);
                blocks.push(fg_type);
            }
        }
        Ok(AllChunk {
            block_types: blocks,
        })
    } else {
        Err(the_blockheads_tools_lib::BhError::InvalidBlockIdError(0))
    }
}

struct ChunkMesh {
    vertices: [Vertex; 8],
}

impl ChunkMesh {
    const INDICES: &[u16] = &[
        0, 1, 2, 2, 3, 0, // front
        1, 5, 6, 6, 2, 1, // right
        5, 4, 7, 7, 6, 5, // back
        4, 0, 3, 3, 7, 4, // left
        3, 2, 6, 6, 7, 3, // top
        4, 5, 1, 1, 0, 4, // bottom
    ];

    fn new(offset: Vec3) -> Self {
        let [min_x, min_y, min_z] = offset.to_array();
        let max_x = min_x + 32.;
        let max_y = min_y + 32.;
        let max_z = min_z + 3.;
        Self {
            vertices: [
                // Front face
                Vertex {
                    pos: [min_x, min_y, max_z],
                },
                Vertex {
                    pos: [max_x, min_y, max_z],
                },
                Vertex {
                    pos: [max_x, max_y, max_z],
                },
                Vertex {
                    pos: [min_x, max_y, max_z],
                },
                // Back face
                Vertex {
                    pos: [min_x, min_y, min_z],
                },
                Vertex {
                    pos: [max_x, min_y, min_z],
                },
                Vertex {
                    pos: [max_x, max_y, min_z],
                },
                Vertex {
                    pos: [min_x, max_y, min_z],
                },
            ],
        }
    }

    fn to_vertex_index_buffer(&self, device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(Self::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buf, index_buf, Self::INDICES.len() as u32)
    }
}

// Define how to connect the vertices to form triangles.
pub struct Camera {
    pub center: glam::Vec2,
    pub distance: f32,
    pub up: Vec3,
    pub fovy: f32,   // Field of view in radians
    pub z_near: f32, // Near clipping plane
    pub z_far: f32,  // Far clipping plane
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            center: glam::Vec2::new(0.0, 0.0),
            distance: 5.0,
            up: Vec3::Y,
            fovy: 45.0_f32.to_radians(),
            z_near: 0.01,
            z_far: 10000.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],     // Combined view and projection matrix
    inv_view_proj: [[f32; 4]; 4], // Combined view and projection matrix
    camera_pos: [f32; 4],         // Camera's world position (vec3 + padding)
    screen_size: [f32; 4],        // Screen width and height (vec2 + padding)
}

impl Camera {
    fn eye(&self) -> Vec3 {
        self.center.extend(self.distance)
    }

    fn target(&self) -> Vec3 {
        self.center.extend(0.0)
    }

    fn to_uniform(&self, window_size: (f32, f32)) -> CameraUniform {
        let (width, height) = window_size;
        let aspect = width / height;
        let view = Mat4::look_at_rh(self.eye(), self.target(), self.up);
        let proj = Mat4::perspective_rh(self.fovy, aspect, self.z_near, self.z_far);
        let pv = proj * view;
        CameraUniform {
            view_proj: pv.to_cols_array_2d(),
            inv_view_proj: pv.inverse().to_cols_array_2d(),
            camera_pos: self.eye().extend(1.0).into(),
            screen_size: [width, height, 0.0, 0.0],
        }
    }
}

struct Scene {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    voxel_buf: wgpu::Buffer,
    num_indices: u32,
    pipeline: wgpu::RenderPipeline,
    camera_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    camera: Camera,
}

impl Scene {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let chunk = ChunkMesh::new(Vec3::ZERO);
        let (vertex_buf, index_buf, num_indices) = chunk.to_vertex_index_buffer(device);

        let all_chunks = load_chunk().unwrap();
        let voxel_buf = all_chunks.create_buffer_init(device);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let camera_uniform = Camera::default().to_uniform((800.0, 600.0));

        // Create a uniform buffer for the MVP matrix.
        let camera_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let tile_map_img_bytes = include_bytes!("../resources/TileMap.png");
        let mut decoder = PngDecoder::new(tile_map_img_bytes);
        let (tile_map_img_w, tile_map_img_h) = (512, 512);
        let tile_map_img = decoder.decode().unwrap().u8().unwrap();

        let tile_map_texture_size = wgpu::Extent3d {
            width: tile_map_img_w as u32,
            height: tile_map_img_h as u32,
            depth_or_array_layers: 1, // It's a 2D texture
        };

        let tile_map_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: tile_map_texture_size,
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
                texture: &tile_map_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            tile_map_img.as_slice(), // Get raw bytes of the image
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(tile_map_img_h as u32 * 4), // 4 bytes per pixel (RGBA)
                rows_per_image: Some(tile_map_img_w as u32),
            },
            tile_map_texture_size,
        );

        let tile_map_texture_view =
            tile_map_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 5. Create the Sampler (Nearest Neighbor)
        let tile_map_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler (Nearest Neighbor)"),
            address_mode_u: wgpu::AddressMode::ClampToEdge, // Or Repeat, depending on your needs
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest, // Nearest for pixelated look
            min_filter: wgpu::FilterMode::Nearest, // Nearest for pixelated look
            mipmap_filter: wgpu::FilterMode::Nearest, // Nearest for mipmaps (even if count is 1)
            ..Default::default()
        });

        let uv_at_face_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Texture UV Atlas Buffer"),
            contents: bytemuck::cast_slice(&UVMappedBlockType::UV_AT_FACE),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create a bind group layout and bind group to link the uniform buffer to the shader.
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
                    resource: wgpu::BindingResource::TextureView(&tile_map_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&tile_map_texture_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: uv_at_face_buf.as_entire_binding(),
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
                buffers: &[Vertex::buffer_layout()],
                // buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back), // Prevent rendering the inside of the cube
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            // depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            vertex_buf,
            index_buf,
            voxel_buf,
            num_indices,
            pipeline,
            camera_buf,
            bind_group,
            camera: Camera::default(),
        }
    }

    fn update_uniforms(&self, queue: &wgpu::Queue, width: u32, height: u32) {
        let camera = self.camera.to_uniform((width as f32, height as f32));

        queue.write_buffer(&self.camera_buf, 0, bytemuck::cast_slice(&[camera]));
    }
}

pub struct AppState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub scale_factor: f32,
    pub egui_renderer: EguiRenderer,
    depth_view: wgpu::TextureView,
    scene: Scene,
}

impl AppState {
    async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        window: &Window,
        width: u32,
        height: u32,
    ) -> Self {
        let power_pref = wgpu::PowerPreference::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_pref,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let features = wgpu::Features::empty();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: Default::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let selected_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let swapchain_format = *swapchain_capabilities
            .formats
            .iter()
            .find(|d| **d == selected_format)
            .expect("failed to select proper surface texture format!");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let depth_view = Self::create_depth_view(&device, width, height);
        let scene = Scene::new(&device, &queue, &surface_config);

        let egui_renderer = EguiRenderer::new(
            &device,
            surface_config.format,
            Some(DEPTH_FORMAT),
            1,
            window,
        );

        let scale_factor = 1.0;

        Self {
            device,
            queue,
            surface,
            surface_config,
            egui_renderer,
            scale_factor,
            depth_view,
            scene,
        }
    }

    fn create_depth_view(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("Depth Texture"),
            view_formats: &[DEPTH_FORMAT],
        });
        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
        self.depth_view = Self::create_depth_view(&self.device, width, height);
    }
}

pub struct App {
    instance: wgpu::Instance,
    state: Option<AppState>,
    window: Option<Arc<Window>>,
}

impl App {
    pub fn new() -> Self {
        let instance = egui_wgpu::wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        Self {
            instance,
            state: None,
            window: None,
        }
    }

    async fn set_window(&mut self, window: Window) {
        let window = Arc::new(window);
        let initial_size = window.inner_size();

        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("Failed to create surface!");

        let state = AppState::new(
            &self.instance,
            surface,
            &window,
            initial_size.width,
            initial_size.height,
        )
        .await;

        self.window.get_or_insert(window);
        self.state.get_or_insert(state);
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.state.as_mut().unwrap().resize_surface(width, height);
        }
    }

    fn handle_redraw(&mut self) {
        if self
            .window
            .as_ref()
            .and_then(|w| w.is_minimized())
            .unwrap_or(false)
        {
            return;
        }

        let state = self.state.as_mut().unwrap();

        // Update the rotation based on elapsed time.
        state.scene.update_uniforms(
            &state.queue,
            state.surface_config.width,
            state.surface_config.height,
        );

        let surface_texture_result = state.surface.get_current_texture();
        let surface_texture = match surface_texture_result {
            Ok(texture) => texture,
            Err(SurfaceError::Outdated) => {
                state
                    .surface
                    .configure(&state.device, &state.surface_config);
                state
                    .surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture after resize")
            }
            Err(e) => {
                eprintln!("Failed to acquire next swap chain texture: {:?}", e);
                return;
            }
        };

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // 3D Scene Pass
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("3D Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &state.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                // depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&state.scene.pipeline);
            rpass.set_bind_group(0, &state.scene.bind_group, &[]);
            rpass.set_vertex_buffer(0, state.scene.vertex_buf.slice(..));
            rpass.set_index_buffer(state.scene.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..state.scene.num_indices, 0, 0..1);
            // rpass.draw(0..3, 0..1);
        }

        // egui Pass
        let window = self.window.as_ref().unwrap();
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [state.surface_config.width, state.surface_config.height],
            pixels_per_point: window.scale_factor() as f32 * state.scale_factor,
        };

        {
            state.egui_renderer.begin_frame(window);

            egui::Window::new("winit + egui + wgpu says hello!")
                .resizable(true)
                .vscroll(true)
                .show(state.egui_renderer.context(), |ui| {
                    ui.label("Label!");
                    if ui.button("Button!").clicked() {
                        println!("boom!")
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "Pixels per point: {}",
                            state.egui_renderer.context().pixels_per_point()
                        ));
                        if ui.button("-").clicked() {
                            state.scale_factor = (state.scale_factor - 0.1).max(0.3);
                        }
                        if ui.button("+").clicked() {
                            state.scale_factor = (state.scale_factor + 0.1).min(3.0);
                        }
                    });

                    ui.add(
                        egui::DragValue::new(&mut state.scene.camera.center.x)
                            .speed(0.1)
                            .prefix("Viewport Center X: "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut state.scene.camera.center.y)
                            .speed(0.1)
                            .prefix("Viewport Center Y: "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut state.scene.camera.distance)
                            .speed(0.1)
                            .prefix("Distance: "),
                    );
                });

            let depth_stencil_attachment = Some(wgpu::RenderPassDepthStencilAttachment {
                view: &state.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            });

            state.egui_renderer.end_frame_and_draw(
                &state.device,
                &state.queue,
                &mut encoder,
                window,
                &surface_view,
                screen_descriptor,
                depth_stencil_attachment,
            );
        }

        state.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Egui + WGPU")
                .with_inner_size(PhysicalSize::new(1360, 768));
            let window = event_loop.create_window(window_attributes).unwrap();
            pollster::block_on(self.set_window(window));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if self.state.is_none() {
            return;
        }

        let window = self.window.as_ref().unwrap();
        self.state
            .as_mut()
            .unwrap()
            .egui_renderer
            .handle_input(window, &event);

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.handle_redraw();
            }
            WindowEvent::Resized(new_size) => {
                self.handle_resized(new_size.width, new_size.height);
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            self.window.as_ref().unwrap().request_redraw();
        }
    }
}
