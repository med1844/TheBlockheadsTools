use super::{
    fps_counter::FpsCounter,
    gpu::{
        Camera, CameraUniform, GpuBlockCoord, GpuBlockCoordUniform,
        dw::{DwBuf, DwChunkBuf},
        voxel_util,
    },
    renderer::RenderResources,
};
use eframe::{egui, egui_wgpu, emath::Rect, wgpu};
use glam::Vec3Swizzles;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(target_arch = "wasm32")]
use std::sync::mpsc::{Receiver, Sender, channel};
use the_blockheads_tools_lib::{
    BhError, BhResult,
    game::{
        coord::{BlockCoord, ChunkCoord},
        db::world_db::WorldDb,
    },
};

struct Render3dCallback {
    camera_uniform: CameraUniform,
    dw_chunks: Vec<DwChunkBuf>,
    show_grid: bool,
    selected_block_coord_uniform: GpuBlockCoordUniform,
    hover_on_block_coord_uniform: GpuBlockCoordUniform,
}

impl egui_wgpu::CallbackTrait for Render3dCallback {
    fn prepare(
        &self,
        _device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut eframe::wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        let r: &RenderResources = callback_resources.get().unwrap();
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
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'static>,
        callback_resources: &egui_wgpu::CallbackResources,
    ) {
        let r: &RenderResources = callback_resources.get().unwrap();
        r.render(render_pass, &self.dw_chunks, self.show_grid);
    }
}

enum ReaderState {
    #[cfg(not(target_arch = "wasm32"))]
    Native { open_path: Option<PathBuf> },

    #[cfg(target_arch = "wasm32")]
    Wasm {
        sender: Sender<Vec<u8>>,
        receiver: Receiver<Vec<u8>>,
    },
}

// Helper function to unify sync and async file reading on native & wasm
struct FileReader {
    state: ReaderState,
}

impl FileReader {
    fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        let (tx, rx) = channel();

        Self {
            #[cfg(not(target_arch = "wasm32"))]
            state: ReaderState::Native { open_path: None },
            #[cfg(target_arch = "wasm32")]
            state: ReaderState::Wasm {
                sender: tx,
                receiver: rx,
            },
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn set_open_path(&mut self, new_open_path: Option<PathBuf>) {
        match &mut self.state {
            ReaderState::Native { open_path } => {
                *open_path = new_open_path;
            }
        }
    }

    // Platform-specific helpers
    #[cfg(target_arch = "wasm32")]
    fn get_sender(&self) -> Sender<Vec<u8>> {
        match &self.state {
            ReaderState::Wasm { sender, .. } => sender.clone(),
        }
    }

    fn read_file(&mut self) -> Option<BhResult<Vec<u8>>> {
        match &mut self.state {
            #[cfg(not(target_arch = "wasm32"))]
            ReaderState::Native { open_path } => open_path
                .take()
                .map(|path| std::fs::read(path).map_err(Into::into)),
            #[cfg(target_arch = "wasm32")]
            ReaderState::Wasm { receiver, .. } => receiver.try_recv().ok().map(Ok),
        }
    }
}

pub struct EditorApp {
    world_db: Option<WorldDb>,
    dw_buf: DwBuf,
    camera: Camera,

    show_info: bool,
    show_grid: bool,
    fps_counter: FpsCounter,

    selected_block_coord: GpuBlockCoord,
    hover_on_block_coord: GpuBlockCoord,

    file_reader: FileReader,
    load_err: Option<BhError>,
    save_err: Option<BhError>,

    world_viewport_rect: Rect,
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let state = cc
            .wgpu_render_state
            .as_ref()
            .expect("CreationContext should have RenderState");
        let device = &state.device;

        let camera = Camera::default();
        let selected_block_coord = GpuBlockCoord::default();
        let hover_on_block_coord = GpuBlockCoord::default();
        let voxel_buf = voxel_util::create_buffer(device, 512);
        let render_resources = RenderResources::new(
            state,
            camera.to_buf(device),
            voxel_buf,
            selected_block_coord.to_buf(device),
            hover_on_block_coord.to_buf(device),
        );

        state
            .renderer
            .write()
            .callback_resources
            .insert(render_resources);

        Self {
            world_db: None,
            dw_buf: DwBuf::new(),
            camera,

            show_info: false,
            show_grid: false,
            fps_counter: FpsCounter::new(2.0),

            selected_block_coord,
            hover_on_block_coord,

            file_reader: FileReader::new(),
            load_err: None,
            save_err: None,

            world_viewport_rect: Rect::from_x_y_ranges(0.0..=1920.0, 0.0..=1080.0),
        }
    }

    fn open_world_db(
        &mut self,
        data: Vec<u8>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        voxel_buffer: &wgpu::Buffer,
    ) -> BhResult<()> {
        let mut world_db = WorldDb::from_bytes(&data)?;
        let spawn_x = world_db.main.world_v2.start_portal_pos_x;
        let spawn_y = world_db.main.world_v2.start_portal_pos_y - 1;

        // By default we look at start portal
        *self.camera.world_offset_mut() = glam::Vec3::new(spawn_x as f32, spawn_y as f32, 5.0);

        let new_world_width_macro = world_db.main.world_v2.world_width_macro as usize;

        // TODO: if width_macro doesn't match the old one, create new buffer and update voxel renderer bind group
        voxel_util::set_chunks(
            queue,
            voxel_buffer,
            &mut world_db.chunks,
            new_world_width_macro,
        );

        self.dw_buf.clear();
        for chunk_y in 0..32 {
            for chunk_x in 0..world_db.main.world_v2.world_width_macro {
                let chunk_coord = ChunkCoord::new(chunk_x, chunk_y).unwrap();
                if !self.dw_buf.has_chunk(chunk_coord)
                    && let Some(chunk) = world_db.dw.chunk_at(chunk_coord)
                {
                    self.dw_buf.set_chunk(device, chunk_coord, chunk);
                }
            }
        }

        self.world_db = Some(world_db);

        Ok(())
    }

    #[allow(unused_variables)] // ctx is only used in async
    fn render_menu_bar(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))]
        let mut save_path = None;

        egui::MenuBar::new().ui(ui, |ui| {
            ui.toggle_value(&mut self.show_info, "Info");
            ui.toggle_value(&mut self.show_grid, "Grid");
            ui.separator();
            ui.menu_button("File", |ui| {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if ui.button("Open").clicked() {
                        self.file_reader
                            .set_open_path(rfd::FileDialog::new().pick_file());
                    }
                    if ui.button("Save As").clicked() {
                        save_path = rfd::FileDialog::new().pick_folder();
                    }
                }

                #[cfg(target_arch = "wasm32")]
                {
                    if ui.button("Open").clicked() {
                        let sender = self.file_reader.get_sender();
                        let task = rfd::AsyncFileDialog::new().pick_file();
                        let ctx = ui.ctx().clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            if let Some(file) = task.await {
                                let bytes = file.read().await;
                                let _ = sender.send(bytes);
                                ctx.request_repaint();
                            }
                        })
                    }
                    if ui.button("Save As").clicked() {
                        if let Some(world_db) = self.world_db.as_ref() {
                            let mut out_bytes = Vec::new();
                            match world_db.write_to(&mut out_bytes) {
                                Ok(()) => {
                                    let task = rfd::AsyncFileDialog::new()
                                        .set_file_name("data.mdb")
                                        .save_file();
                                    let ctx = ui.ctx().clone();
                                    wasm_bindgen_futures::spawn_local(async move {
                                        if let Some(file) = task.await {
                                            let _ = file.write(&out_bytes).await;
                                            ctx.request_repaint();
                                        }
                                    })
                                }
                                Err(e) => {
                                    self.save_err = Some(e);
                                }
                            }
                        }
                    }
                }
            });
        });

        if let Some(state) = frame.wgpu_render_state()
            && let Some(read_result) = self.file_reader.read_file()
            && let Err(e) = read_result.and_then(|bytes| {
                let read_renderer = state.renderer.read();
                let r = read_renderer
                    .callback_resources
                    .get::<RenderResources>()
                    .expect("should have render resources");
                self.open_world_db(bytes, &state.device, &state.queue, r.voxel_buf())
            })
        {
            self.load_err = Some(e);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(world_db) = self.world_db.as_ref()
                && let Some(save_path) = save_path
                && let Err(e) = world_db.to_path(save_path)
            {
                self.save_err = Some(e);
            }
        }
    }

    fn render_side_panel(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("fps: {:.1}", self.fps_counter.fps()));
        ui.separator();
        ui.add(
            egui::DragValue::new(&mut self.camera.world_offset_mut().x)
                .speed(0.1)
                .prefix("Viewport Center X: "),
        );
        ui.add(
            egui::DragValue::new(&mut self.camera.world_offset_mut().y)
                .speed(0.1)
                .prefix("Viewport Center Y: "),
        );
        ui.add(
            egui::DragValue::new(&mut self.camera.world_offset_mut().z)
                .speed(0.1)
                .range(Camera::MAX_BLOCK_Z..=Camera::MAX_Z)
                .prefix("Distance: "),
        );
    }

    fn update_camera_pos(&mut self, ui: &mut egui::Ui, response: &egui::Response) {
        let viewport_size = (
            self.world_viewport_rect.width(),
            self.world_viewport_rect.height(),
        );

        if response.dragged_by(egui::PointerButton::Primary)
            && let Some(cur_pos) = response.interact_pointer_pos()
        {
            let delta = response.drag_delta();
            let prev_pos = cur_pos - delta;

            self.camera
                .handle_drag(prev_pos.into(), cur_pos.into(), viewport_size);
        }

        if response.hovered()
            && let Some(pos) = response.hover_pos()
        {
            let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_y.abs() > 0.0 {
                self.camera.handle_zoom(
                    (pos - self.world_viewport_rect.min).into(),
                    viewport_size,
                    scroll_y,
                );
            }
        }
    }

    fn update_gpu_block_coords(&mut self, response: &egui::Response) {
        if response.clicked_by(egui::PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            let [x, y] = self
                .camera
                .mouse_at(
                    (pos - self.world_viewport_rect.min).into(),
                    self.world_viewport_rect.size().into(),
                )
                .floor()
                .to_array();
            self.selected_block_coord
                .toggle(BlockCoord::new(x as u32, y as u16).ok());
        }

        if response.hovered()
            && let Some(pos) = response.hover_pos()
        {
            let [x, y] = self
                .camera
                .mouse_at(
                    (pos - self.world_viewport_rect.min).into(),
                    self.world_viewport_rect.size().into(),
                )
                .floor()
                .to_array();
            self.hover_on_block_coord
                .update(BlockCoord::new(x as u32, y as u16).ok());
        }
    }

    fn render_3d_viewport(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let (rect, response) =
            ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());

        self.world_viewport_rect = rect;
        self.camera.set_aspect(rect.aspect_ratio());

        self.update_camera_pos(ui, &response);
        self.update_gpu_block_coords(&response);

        let [min_coords, max_coords] = self.camera.visible_world_region_2d(rect.size().into());
        let center = self.camera.world_offset().xy();
        const MAX_DIST: f32 = 48.0;

        let min_x = min_coords.x.max(center.x - MAX_DIST).max(0.0);
        let min_y = min_coords.y.max(center.y - MAX_DIST).max(0.0);
        let max_x = max_coords.x.min(center.x + MAX_DIST);
        let max_y = max_coords.y.min(center.y + MAX_DIST).min(1024.0);

        let chunk_min_x = (min_x / 32.0).floor() as u32;
        let chunk_min_y = (min_y / 32.0).floor() as u8;
        let chunk_max_x = (max_x / 32.0).ceil() as u32;
        let chunk_max_y = (max_y / 32.0).ceil() as u8;
        let mut dw_chunks = Vec::with_capacity(
            chunk_max_x.saturating_sub(chunk_min_x) as usize
                * chunk_max_y.saturating_sub(chunk_min_y) as usize,
        );
        for x in chunk_min_x..chunk_max_x {
            for y in chunk_min_y..chunk_max_y {
                if let Some(chunk_buf) = self.dw_buf.get_chunk(ChunkCoord::new(x, y).unwrap()) {
                    dw_chunks.push(chunk_buf.clone()); // chunk_buf is cheap to clone
                }
            }
        }

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            Render3dCallback {
                camera_uniform: self.camera.uniform(),
                dw_chunks,
                show_grid: self.show_grid,
                selected_block_coord_uniform: self.selected_block_coord.to_uniform(),
                hover_on_block_coord_uniform: self.hover_on_block_coord.to_uniform(),
            },
        ));
    }

    fn render_error_windows(&mut self, ctx: &egui::Context) {
        if let Some(e) = self.load_err.as_ref() {
            let mut open = true;
            egui::Window::new("Failed to load world")
                .open(&mut open)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Error message");
                    ui.label(e.to_string());
                });
            if !open {
                self.load_err = None;
            }
        }

        if let Some(e) = self.save_err.as_ref() {
            let mut open = true;
            egui::Window::new("Failed to save world")
                .open(&mut open)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Error message");
                    ui.label(e.to_string());
                });
            if !open {
                self.save_err = None;
            }
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.fps_counter.update(ctx.input(|i| i.time));

        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Top, "menu").show(ctx, |ui| {
            self.render_menu_bar(ui, frame);
        });

        egui::SidePanel::left("Info")
            .resizable(false)
            .show_animated(ctx, self.show_info, |ui| {
                self.render_side_panel(ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(0.0))
            .show(ctx, |ui| {
                self.render_3d_viewport(ui);
            });

        self.render_error_windows(ctx);
    }
}
