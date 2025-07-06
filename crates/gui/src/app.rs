use super::{
    egui_tools::EguiRenderer,
    gpu::{Camera, CameraBuf, ChunkBuffers, RgbaTexture, VoxelBuf},
    input::Input,
    renderer::{DEPTH_FORMAT, VoxelRenderer},
};
use egui_wgpu::wgpu::SurfaceError;
use egui_wgpu::{ScreenDescriptor, wgpu};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use zune_png::PngDecoder;

pub struct AppState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub scale_factor: f32,
    pub egui_renderer: EguiRenderer,

    // input
    pub input: Input,

    // 3d rendering related
    pub camera_buf: CameraBuf,
    pub voxel_renderer: VoxelRenderer,
    pub depth_view: wgpu::TextureView,
    pub chunk_manager: ChunkBuffers,
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

        let voxel_buf = VoxelBuf::load_test_chunk(&device, &queue).unwrap();
        let chunk_manager = ChunkBuffers::new(&device, voxel_buf);

        let egui_renderer = EguiRenderer::new(
            &device,
            surface_config.format,
            Some(DEPTH_FORMAT),
            1,
            window,
        );

        let scale_factor = 1.0;

        let camera = Camera::default();
        let camera_buf = camera.into_buffered(&device);

        let tile_map_texture = {
            let bytes = include_bytes!("../resources/TileMap.png");
            let mut decoder = PngDecoder::new(bytes);
            let img = decoder.decode().unwrap().u8().unwrap();
            RgbaTexture::new(img.as_slice(), (512, 512), &device, &queue)
        };
        let voxel_renderer = VoxelRenderer::new(
            &device,
            &surface_config,
            &camera_buf.buf,
            &chunk_manager.voxel_buf.buf,
            &tile_map_texture,
        );

        Self {
            device,
            queue,
            surface_config,
            surface,
            scale_factor,
            egui_renderer,

            input: Input::new(),

            camera_buf,
            voxel_renderer,
            depth_view,
            chunk_manager,
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

    fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
        self.input.handle_input(window, event);
        self.camera_buf.handle_input(&self.input);
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

        state.camera_buf.update_uniforms(
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
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&state.voxel_renderer.pipeline);
            rpass.set_bind_group(0, &state.voxel_renderer.bind_group, &[]);
            rpass.set_vertex_buffer(0, state.chunk_manager.vertex_buf.slice(..));
            rpass.set_index_buffer(
                state.chunk_manager.index_buf.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            rpass.draw_indexed(0..state.chunk_manager.num_indices, 0, 0..1);
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
                        egui::DragValue::new(&mut state.camera_buf.camera.center.x)
                            .speed(0.1)
                            .prefix("Viewport Center X: "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut state.camera_buf.camera.center.y)
                            .speed(0.1)
                            .prefix("Viewport Center Y: "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut state.camera_buf.camera.distance)
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
        let response = self
            .state
            .as_mut()
            .unwrap()
            .egui_renderer
            .handle_input(window, &event);

        if !response.consumed {
            self.state.as_mut().unwrap().handle_input(window, &event);
        }

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
