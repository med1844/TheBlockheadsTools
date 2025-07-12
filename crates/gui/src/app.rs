use super::{
    egui_tools::EguiRenderer,
    fps_counter::FpsCounter,
    gpu::{Camera, CameraBuf, RgbaTexture, VoxelBuf},
    input::{EventResponse, Input},
    renderer::{DEPTH_FORMAT, VoxelRenderer},
};
use egui_wgpu::wgpu::SurfaceError;
use egui_wgpu::{ScreenDescriptor, wgpu};
use std::{path::Path, sync::Arc};
use the_blockheads_tools_lib::{BhResult, BlockCoord, ChunkCoord, WorldDb};
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
    pub voxel_buf: VoxelBuf,
    pub depth_view: wgpu::TextureView,

    // game save
    world_db: Option<WorldDb>,

    // inspections
    selected_block: Option<BlockCoord>,

    // utils
    fps_counter: FpsCounter,
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

        // When we load other worlds, rebuild voxel buf. By default we go 512.
        let voxel_buf = VoxelBuf::new(&device, 512);

        let egui_renderer = EguiRenderer::new(
            &device,
            surface_config.format,
            Some(DEPTH_FORMAT),
            1,
            window,
        );

        let scale_factor = 1.5;

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
            &voxel_buf.buf,
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
            voxel_buf,
            depth_view,

            world_db: None,

            selected_block: None,

            fps_counter: FpsCounter::new(2.0),
        }
    }

    fn open_world_db<P: AsRef<Path>>(&mut self, path: P) -> BhResult<()> {
        let mut world_db = WorldDb::from_path(path)?;
        let x = world_db.main.world_v2.start_portal_pos_x;
        let y = world_db.main.world_v2.start_portal_pos_y - 1;

        // By default we look at start portal
        self.camera_buf.camera.world_offset = glam::Vec3::new(x as f32, y as f32, 5.0);

        let new_world_width_macro = world_db.main.world_v2.world_width_macro as usize;

        // Resize voxel buffer if needed, otherwise, clear up the voxel buffer.
        if self.voxel_buf.world_width_macro == new_world_width_macro {
            self.voxel_buf.clear(&self.queue);
        } else {
            self.voxel_buf = VoxelBuf::new(&self.device, new_world_width_macro);
        }

        // Fill up all chunks for now. This could be dispatched to another thread later.
        for chunk_y in 0..32 {
            for chunk_x in 0..world_db.main.world_v2.world_width_macro {
                let chunk_coord = ChunkCoord::new(chunk_x, chunk_y).unwrap();
                if !self.voxel_buf.has_chunk(chunk_coord) {
                    if let Some(Ok(chunk)) = world_db.blocks.chunk_at(chunk_coord) {
                        let _ = self.voxel_buf.set_chunk(&self.queue, chunk_coord, chunk);
                    }
                }
            }
        }

        self.world_db = Some(world_db);

        Ok(())
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

    fn handle_input(&mut self, window: &Window, event: &WindowEvent) -> EventResponse {
        let response = self.input.handle_input(window, event);
        if response.click {
            let [x, y] = self.camera_buf.mouse_at(&self.input).floor().to_array();
            let new_coord = BlockCoord::new(x as u64, y as u16).ok();
            if self.selected_block == new_coord {
                self.selected_block = None;
            } else {
                self.selected_block = new_coord;
            }
        }
        self.camera_buf.handle_input(&self.input)
    }

    fn selected_block_info(&mut self) -> Option<(BlockCoord, String)> {
        let world_db = self.world_db.as_mut()?;
        let selected_block_coord = self.selected_block.as_ref()?;
        let block = world_db
            .blocks
            .block_at(selected_block_coord.clone())?
            .ok()?;
        Some((
            selected_block_coord.clone(),
            block.to_hex_string_single_allocation(),
        ))
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
            rpass.draw(0..3, 0..1);
        }

        state.fps_counter.update();

        // egui Pass
        let window = self.window.as_ref().unwrap();
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [state.surface_config.width, state.surface_config.height],
            pixels_per_point: window.scale_factor() as f32 * state.scale_factor,
        };

        {
            state.egui_renderer.begin_frame(window);

            let selected_block_info = state.selected_block_info();
            let mut opened_path = None;

            egui::TopBottomPanel::new(egui::panel::TopBottomSide::Top, "ok").show(
                state.egui_renderer.context(),
                |ui| {
                    egui::menu::bar(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("Open").clicked() {
                                opened_path = rfd::FileDialog::new().pick_folder();
                            }
                        })
                    })
                },
            );

            if let Some(path) = opened_path {
                state.open_world_db(path).unwrap();
            }

            egui::Window::new("Info")
                .resizable(true)
                .vscroll(true)
                .show(state.egui_renderer.context(), |ui| {
                    ui.label(format!("fps: {:.1}", state.fps_counter.fps()));
                    ui.separator();

                    if let Some((coord, bytes)) = selected_block_info {
                        ui.label(format!(
                            "Selected block: x = {}, y = {}",
                            coord.x(),
                            coord.y()
                        ));
                        ui.code(bytes);
                        ui.separator();
                    }

                    ui.add(
                        egui::DragValue::new(&mut state.camera_buf.camera.world_offset.x)
                            .speed(0.1)
                            .prefix("Viewport Center X: "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut state.camera_buf.camera.world_offset.y)
                            .speed(0.1)
                            .prefix("Viewport Center Y: "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut state.camera_buf.camera.world_offset.z)
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
            let _ = self.state.as_mut().unwrap().handle_input(window, &event);
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
