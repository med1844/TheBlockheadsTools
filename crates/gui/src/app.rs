use super::{
    dw_impl::InfoUi,
    fps_counter::FpsCounter,
    gpu::{
        Camera, GpuCoord, RenderSettings,
        dw::{DwBuf, DwChunkObjId, DwChunkObjIdUniform},
        voxel_util,
    },
    render::{GeometryBuffer, Render3dCallback, RenderResources, voxel::VoxelRenderer},
};
use eframe::{egui, egui_wgpu, emath::Rect, wgpu};
use glam::Vec3Swizzles;
use snafu::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(target_arch = "wasm32")]
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use the_blockheads_tools_lib::{
    self as lib, DynArch,
    game::{
        block::{Block, BlockView},
        chunk::Chunk,
        coord::{BlockCoord, ChunkCoord},
        db::world_db::{WorldDb, WorldDbError},
    },
};

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

    fn read_file(&mut self) -> Option<std::io::Result<Vec<u8>>> {
        match &mut self.state {
            #[cfg(not(target_arch = "wasm32"))]
            ReaderState::Native { open_path } => open_path.take().map(std::fs::read),
            #[cfg(target_arch = "wasm32")]
            ReaderState::Wasm { receiver, .. } => receiver.try_recv().ok().map(Ok),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InteractionTarget {
    Block(BlockCoord),
    DynamicObject {
        chunk_coord: ChunkCoord,
        id: DwChunkObjId,
    },
}

struct InteractionState {
    hover: Option<InteractionTarget>,
    select: Option<InteractionTarget>,

    selected_block_chunk: Option<Chunk>,
    hover_on_dyn_obj_id: Arc<Mutex<Option<DwChunkObjId>>>,

    selected_block_gpu: GpuCoord<BlockCoord>,
    hover_on_block_gpu: GpuCoord<BlockCoord>,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            hover: None,
            select: None,
            selected_block_chunk: None,
            hover_on_dyn_obj_id: Arc::new(Mutex::new(None)),
            selected_block_gpu: GpuCoord::default(),
            hover_on_block_gpu: GpuCoord::default(),
        }
    }
}

impl InteractionState {
    fn set_hover_on_obj(&mut self, chunk_coord: ChunkCoord, id: DwChunkObjId) {
        self.hover = Some(InteractionTarget::DynamicObject { chunk_coord, id });
        self.hover_on_block_gpu.update(None);
    }

    fn set_hover_on_block(&mut self, block_coord: BlockCoord) {
        self.hover = Some(InteractionTarget::Block(block_coord));
        self.hover_on_block_gpu.update(Some(block_coord));
    }

    fn clear_hover(&mut self) {
        self.hover = None;
        self.hover_on_block_gpu.update(None);
    }

    fn copy_hover_to_select(&mut self, world_db: Option<&WorldDb>) {
        self.select = match self.select == self.hover {
            true => None,
            false => self.hover,
        };
        if let Some(InteractionTarget::Block(block_coord)) = self.select
            && let Some(world_db) = world_db
            && let Some(compressed_chunk) = world_db.chunks.chunk_at(block_coord)
            && let Ok(chunk) = compressed_chunk.decompress()
        {
            self.selected_block_gpu.update(Some(block_coord));
            self.selected_block_chunk = Some(chunk);
        } else {
            self.selected_block_gpu.update(None);
            self.selected_block_chunk = None;
        }
    }

    fn hover_on_id_uniform(&self) -> DwChunkObjIdUniform {
        match self.hover {
            Some(InteractionTarget::DynamicObject { id, chunk_coord }) => {
                Some((id, chunk_coord)).into()
            }
            _ => None.into(),
        }
    }

    fn selected_id_uniform(&self) -> DwChunkObjIdUniform {
        match self.select {
            Some(InteractionTarget::DynamicObject { id, chunk_coord }) => {
                Some((id, chunk_coord)).into()
            }
            _ => None.into(),
        }
    }
}

#[derive(Debug, Snafu)]
pub enum EditorAppError {
    #[snafu(display("Failed to open world_db: {source}"))]
    OpenWorldDb { source: WorldDbError },
    #[snafu(display("Failed to save world_db: {source}"))]
    SaveWorldDb { source: WorldDbError },
    #[snafu(display("Failed to read world_db bytes: {source}"))]
    ReadWorldDbBytes { source: std::io::Error },
}

type Result<T> = std::result::Result<T, EditorAppError>;

pub struct EditorApp {
    world_db: Option<WorldDb>,
    dw_buf: DwBuf,
    camera: Camera,

    show_info: bool,
    show_grid: bool,
    fps_counter: FpsCounter,

    interaction_state: InteractionState,

    file_reader: FileReader,
    load_err: Option<EditorAppError>,
    save_err: Option<EditorAppError>,

    world_viewport_rect: Rect,
    mouse_pos: Option<(f32, f32)>,
    render_settings: RenderSettings,
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let state = cc
            .wgpu_render_state
            .as_ref()
            .expect("CreationContext should have RenderState");
        let device = &state.device;

        let camera = Camera::default();
        let voxel_buf =
            voxel_util::create_buffer(device, VoxelRenderer::DEFAULT_WORLD_WIDTH_CHUNK as usize);
        let interaction_state = InteractionState::default();
        let render_resources = RenderResources::new(
            state,
            camera.create_buffer(device),
            voxel_buf,
            interaction_state
                .selected_block_gpu
                .uniform()
                .create_buffer(device),
            interaction_state
                .hover_on_block_gpu
                .uniform()
                .create_buffer(device),
            interaction_state.hover_on_dyn_obj_id.clone(),
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

            interaction_state,

            file_reader: FileReader::new(),
            load_err: None,
            save_err: None,

            world_viewport_rect: Rect::from_x_y_ranges(
                0.0..=GeometryBuffer::DEFAULT_WIDTH as f32,
                0.0..=GeometryBuffer::DEFAULT_HEIGHT as f32,
            ),
            mouse_pos: None,
            render_settings: RenderSettings::default(),
        }
    }

    fn open_world_db(
        &mut self,
        data: Vec<u8>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_resources: &mut RenderResources,
    ) -> Result<()> {
        let mut world_db = WorldDb::from_bytes(&data).context(OpenWorldDbSnafu)?;
        let spawn_x = world_db.main.world_v2.start_portal_pos_x;
        let spawn_y = world_db.main.world_v2.start_portal_pos_y - 1;

        let new_world_width_macro = world_db.main.world_v2.world_width_macro;
        let world_dim_x = new_world_width_macro * Chunk::NUM_BLOCK_PER_ROW as u32;
        let world_block_width = world_dim_x as f32;

        // By default we look at start portal, wrapped into [0, world_width)
        let wrapped_spawn_x = (spawn_x as f32).rem_euclid(world_block_width);
        *self.camera.world_offset_mut() = glam::Vec3::new(wrapped_spawn_x, spawn_y as f32, 5.0);

        // Only allocate a new voxel buffer when the new world is wider than the current one.
        // Reusing the existing (larger) buffer avoids a VRAM allocation on every world load.
        let old_world_width_macro = self
            .world_db
            .as_ref()
            .map(|db| db.main.world_v2.world_width_macro)
            .unwrap_or(0);
        if new_world_width_macro > old_world_width_macro {
            render_resources.replace_voxel_buf(
                device,
                queue,
                voxel_util::create_buffer(device, new_world_width_macro as usize),
                world_dim_x,
            );
        }
        voxel_util::set_chunks(
            queue,
            render_resources.voxel_buf(),
            &mut world_db.chunks,
            new_world_width_macro as usize,
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

    fn render_menu_bar(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
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
                        // TODO if world_db is empty we should return as error
                        if let Some(world_db) = self.world_db.as_ref()
                            && let Some(save_path) = rfd::FileDialog::new().pick_folder()
                            && let Err(e) = world_db
                                .to_path(save_path, DynArch::Arch64)
                                .context(SaveWorldDbSnafu)
                        {
                            // TODO allow user select arch
                            self.save_err = Some(e);
                        }
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
                    if ui.button("Export").clicked()
                        && let Some(world_db) = self.world_db.as_ref()
                    {
                        let mut out_bytes = Vec::new();
                        match world_db
                            .write_to(&mut out_bytes, DynArch::Arch64)
                            .context(SaveWorldDbSnafu)
                        {
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
            });
        });

        if let Some(state) = frame.wgpu_render_state()
            && let Some(read_result) = self.file_reader.read_file()
            && let Err(e) = read_result
                .context(ReadWorldDbBytesSnafu)
                .and_then(|bytes| {
                    let mut write_renderer = state.renderer.write();
                    let r = write_renderer
                        .callback_resources
                        .get_mut::<RenderResources>()
                        .expect("should have render resources");
                    self.open_world_db(bytes, &state.device, &state.queue, r)
                })
        {
            self.load_err = Some(e);
        }
    }

    fn render_selected_block_info(ui: &mut egui::Ui, block: BlockView<'_>) {
        let add_row = |ui: &mut egui::Ui, key: &str, value: &str| {
            ui.label(key);
            ui.label(egui::RichText::new(value).family(egui::FontFamily::Monospace));
            ui.end_row();
        };
        add_row(
            ui,
            "Foreground",
            &block.fg().map_or_else(
                |_| block.fg_raw().to_string(),
                |block_type| block_type.as_str().to_string(),
            ),
        );
        add_row(
            ui,
            "Background",
            &block.bg().map_or_else(
                |_| block.bg_raw().to_string(),
                |block_type| block_type.as_str().to_string(),
            ),
        );
        add_row(
            ui,
            "Content",
            &block.content().map_or_else(
                |_| block.content_raw().to_string(),
                |content_type| content_type.as_str().to_string(),
            ),
        );
    }

    fn render_side_panel(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("fps: {:.1}", self.fps_counter.fps()));
        ui.separator();
        ui.add(
            egui::DragValue::new(&mut self.render_settings.light_dir.x)
                .speed(0.1)
                .prefix("Light Dir X: "),
        );
        ui.add(
            egui::DragValue::new(&mut self.render_settings.light_dir.y)
                .speed(0.1)
                .prefix("Light Dir Y: "),
        );
        ui.add(
            egui::DragValue::new(&mut self.render_settings.light_dir.z)
                .speed(0.1)
                .prefix("Light Dir Z: "),
        );
        ui.checkbox(&mut self.render_settings.enable_reflect, "Enable Reflect");
        ui.checkbox(&mut self.render_settings.enable_destruct, "Enable Destruct");
        ui.checkbox(&mut self.render_settings.enable_ssao, "Enable SSAO");
        ui.checkbox(
            &mut self.render_settings.enable_cyclic,
            "Enable Cyclic World",
        );

        ui.separator();
        ui.add(
            egui::Slider::new(&mut self.render_settings.ambient_light, 0.0..=1.0)
                .text("Ambient Light"),
        );
        ui.add(
            egui::Slider::new(&mut self.render_settings.shininess, 1.0..=256.0).text("Shininess"),
        );
        ui.add(
            egui::Slider::new(&mut self.render_settings.specular_intensity, 0.0..=5.0)
                .text("Specular Intensity"),
        );
        ui.add(
            egui::Slider::new(&mut self.render_settings.min_depth_factor, 0.0..=1.0)
                .text("Min Depth Factor"),
        );

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
        if let Some(selected_chunk) = self.interaction_state.selected_block_chunk.as_ref()
            && let Some(InteractionTarget::Block(selected_block_coord)) =
                self.interaction_state.select
        {
            let block = selected_chunk.view().block_at(selected_block_coord);
            ui.separator();
            ui.heading(format!(
                "Block {}, {}",
                selected_block_coord.x(),
                selected_block_coord.y(),
            ));
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    Self::render_selected_block_info(ui, block);
                });
        }
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

        // Wrap camera X cyclically when cyclic mode is enabled and a world is loaded
        if self.render_settings.enable_cyclic {
            if let Some(world_db) = self.world_db.as_ref() {
                let world_block_width = (world_db.main.world_v2.world_width_macro as f32)
                    * Chunk::NUM_BLOCK_PER_ROW as f32;
                if world_block_width > 0.0 {
                    let x = &mut self.camera.world_offset_mut().x;
                    *x = x.rem_euclid(world_block_width);
                }
            }
        }
    }

    fn update_interaction_state(&mut self, response: &egui::Response) {
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
            let hover_on_block_coord = BlockCoord::new(x as u32, y as u16).ok();

            if let Some(block_coord) = hover_on_block_coord {
                if let Some(id) = {
                    self.interaction_state
                        .hover_on_dyn_obj_id
                        .lock()
                        .expect("should lock")
                        .clone()
                } {
                    let chunk_coord = block_coord.into();
                    self.interaction_state.set_hover_on_obj(chunk_coord, id);
                } else {
                    self.interaction_state.set_hover_on_block(block_coord);
                }
            } else {
                self.interaction_state.clear_hover();
            }
        }

        if response.clicked_by(egui::PointerButton::Primary) {
            self.interaction_state
                .copy_hover_to_select(self.world_db.as_ref());
        }
    }

    fn update_mouse_pos(&mut self, response: &egui::Response) {
        self.mouse_pos = response.hover_pos().map(|pos| {
            ((pos - self.world_viewport_rect.min) * response.ctx.pixels_per_point()).into()
        });
    }

    fn render_3d_viewport(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let (rect, response) =
            ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());

        self.world_viewport_rect = rect;
        self.camera.set_aspect(rect.aspect_ratio());

        self.update_mouse_pos(&response);
        self.update_interaction_state(&response);
        self.update_camera_pos(ui, &response);

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
                selected_block_coord_uniform: self.interaction_state.selected_block_gpu.uniform(),
                hover_on_block_coord_uniform: self.interaction_state.hover_on_block_gpu.uniform(),
                hover_on_id_uniform: self.interaction_state.hover_on_id_uniform(),
                selected_id_uniform: self.interaction_state.selected_id_uniform(),
                mouse_physical_pos: self.mouse_pos,
                world_viewport_rect: self.world_viewport_rect,
                render_settings: self.render_settings,
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

    fn try_read_id(&self, frame: &eframe::Frame) {
        if let Some(state) = frame.wgpu_render_state() {
            let read_renderer = state.renderer.read();
            if let Some(r) = read_renderer.callback_resources.get::<RenderResources>() {
                r.try_read_id();
            }
        }
    }

    fn render_selected_dyn_obj_info_window(&mut self, ctx: &egui::Context) {
        if let Some(InteractionTarget::DynamicObject { chunk_coord, id }) =
            self.interaction_state.select
            && let Some(world_db) = self.world_db.as_mut()
            && let Some(dw_chunk) = world_db.dw.chunk_at_mut(chunk_coord)
        {
            use lib::game::dynamic_object::DynamicObjectType::*;
            fn draw_window<T: InfoUi>(title: &str, t: Option<&mut T>, ctx: &egui::Context) {
                if let Some(t) = t {
                    egui::Window::new(title).show(ctx, |ui| {
                        t.info(ui);
                    });
                }
            }

            let title: &'static str = id.obj_type.into();
            match id.obj_type {
                AppleTree => draw_window(title, dw_chunk.apple_tree.get_mut(id.index), ctx),
                MapleTree => draw_window(title, dw_chunk.maple_tree.get_mut(id.index), ctx),
                MangoTree => draw_window(title, dw_chunk.mango_tree.get_mut(id.index), ctx),
                PineTree => draw_window(title, dw_chunk.pine_tree.get_mut(id.index), ctx),
                CactusTree => draw_window(title, dw_chunk.cactus_tree.get_mut(id.index), ctx),
                CoconutTree => draw_window(title, dw_chunk.coconut_tree.get_mut(id.index), ctx),
                OrangeTree => draw_window(title, dw_chunk.orange_tree.get_mut(id.index), ctx),
                CherryTree => draw_window(title, dw_chunk.cherry_tree.get_mut(id.index), ctx),
                CoffeeTree => draw_window(title, dw_chunk.coffee_tree.get_mut(id.index), ctx),
                FlaxPlant => {}
                SunflowerPlant => {}
                CornPlant => draw_window(title, dw_chunk.corn_plant.get_mut(id.index), ctx),
                Dodo => {}
                DroppedItem => {}
                Fire => {}
                Torch => {}
                GlowBlock => {}
                Ladder => {}
                Door => {}
                ArtificialLight => {}
                Bed => {}
                DropBear => {}
                GatherBlock => {}
                CarrotPlant => draw_window(title, dw_chunk.carrot_plant.get_mut(id.index), ctx),
                Donkey => {}
                Egg => {}
                Window => {}
                Boat => {}
                ChilliPlant => {}
                KelpPlant => draw_window(title, dw_chunk.kelp_plant.get_mut(id.index), ctx),
                ClownFish => {}
                Shark => {}
                LimeTree => draw_window(title, dw_chunk.lime_tree.get_mut(id.index), ctx),
                Wire => {}
                CaveTroll => {}
                Rail => {}
                HandCar => {}
                SteamLocomotive => {}
                FreightCar => {}
                PassengerCar => {}
                Workbench => {}
                Chest => draw_window(title, dw_chunk.chest.get_mut(id.index), ctx),
                Sign => {}
                TradingPost => {}
                TrainStation => {}
                TradePortal => {}
                Scorpion => {}
                Painting => {}
                Column => {}
                Stairs => {}
                ElevatorMotor => {}
                ElevatorShaft => {}
                GemTree => draw_window(title, dw_chunk.gem_tree.get_mut(id.index), ctx),
                VinePlant => {}
                TulipPlant => {}
                OwnershipSign => {}
                WheatPlant => {}
                TomatoPlant => draw_window(title, dw_chunk.tomato_plant.get_mut(id.index), ctx),
                Yak => {}
                Mirror => {}
            }
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.try_read_id(frame);
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
        self.render_selected_dyn_obj_info_window(ctx);
    }
}
