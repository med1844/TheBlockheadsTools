use super::{
    dw_impl::{DwUiContext, InfoUi, ObjFlags, block_content_type_drop_menu, block_type_drop_menu},
    fps_counter::FpsCounter,
    gpu::{
        Camera, GpuCoord, RenderSettings,
        dw::{DwBuf, DwChunkObjId, DwChunkObjIdUniform, ObjectType},
        voxel_util,
    },
    render::{GeometryBuffer, Render3dCallback, RenderResources},
    util::{FastRem, Pow2},
};
use eframe::{egui, egui_wgpu, emath::Rect, wgpu};
use glam::Vec3Swizzles;
use snafu::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(target_arch = "wasm32")]
use std::sync::mpsc::{Receiver, Sender, channel};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use the_blockheads_tools_lib::{
    DynArch,
    game::{
        block::{Block, BlockContentType, BlockMut, BlockType, BlockViewMut, NUM_BYTES_PER_BLOCK},
        chunk::{Chunk, Chunks},
        coord::{BlockCoord, ChunkBlockCoord, ChunkCoord},
        db::world_db::{WorldDb, WorldDbError},
        dynamic_object::{AnyDynamicObject, DynamicObjectType},
        dynamic_world::{ChunkDynamicObjects, DynamicWorld},
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

    hover_on_dyn_obj_id: Arc<Mutex<Option<(DwChunkObjId, ChunkCoord)>>>,

    selected_block_gpu: GpuCoord<BlockCoord>,
    hover_on_block_gpu: GpuCoord<BlockCoord>,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            hover: None,
            select: None,
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

    fn set_select(&mut self, select: Option<InteractionTarget>) {
        self.select = select;
        if let Some(InteractionTarget::Block(block_coord)) = self.select {
            self.selected_block_gpu.update(Some(block_coord));
        } else {
            self.selected_block_gpu.update(None);
        }
    }

    fn copy_hover_to_select(&mut self) {
        self.set_select(match self.select == self.hover {
            true => None,
            false => self.hover,
        });
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

#[derive(Default)]
struct ChunkCache {
    // We can easily add eviction support here if memory usage becomes a concern.
    decompressed_chunks: HashMap<ChunkCoord, Chunk>,
    voxel_outdated: HashSet<ChunkCoord>,
    mesh_outdated: HashSet<ChunkCoord>,
}

impl ChunkCache {
    fn new() -> Self {
        Self::default()
    }

    fn mark_voxel_outdated<I: Into<ChunkCoord>>(&mut self, chunk_coord: I) {
        self.voxel_outdated.insert(chunk_coord.into());
    }

    fn mark_mesh_outdated<I: Into<ChunkCoord>>(&mut self, chunk_coord: I) {
        self.mesh_outdated.insert(chunk_coord.into());
    }

    fn get_chunk_mut<'a>(
        &'a mut self,
        chunk_coord: ChunkCoord,
        world_db: &WorldDb,
    ) -> Option<&'a mut Chunk> {
        if let std::collections::hash_map::Entry::Vacant(e) =
            self.decompressed_chunks.entry(chunk_coord)
        {
            let compressed_chunk = world_db.chunks.chunk_at(chunk_coord)?;
            let chunk = compressed_chunk.decompress().ok()?;
            e.insert(chunk);
        }
        self.decompressed_chunks.get_mut(&chunk_coord)
    }

    fn get_or_create_chunk_mut<'a>(
        &'a mut self,
        chunk_coord: ChunkCoord,
        world_db: &WorldDb,
    ) -> &'a mut Chunk {
        self.decompressed_chunks
            .entry(chunk_coord)
            .or_insert_with(|| {
                world_db
                    .chunks
                    .chunk_at(chunk_coord)
                    .and_then(|c| c.decompress().ok())
                    .unwrap_or_else(Chunk::new_empty)
            })
    }

    fn flush(&mut self, world_db: &mut WorldDb) {
        for (coord, chunk) in self.decompressed_chunks.drain() {
            if let Ok(compressed) = chunk.compress() {
                world_db.chunks.set_chunk_at(coord, compressed);
            }
        }
    }

    fn update_voxels(&mut self, queue: &wgpu::Queue, voxel_buf: &wgpu::Buffer) {
        for coord in self.voxel_outdated.drain() {
            if let Some(chunk) = self.decompressed_chunks.get(&coord) {
                voxel_util::set_chunk(queue, voxel_buf, coord, chunk);
            }
        }
    }

    fn update_meshes(&mut self, device: &wgpu::Device, dw: &mut DynamicWorld, dw_buf: &mut DwBuf) {
        for coord in self.mesh_outdated.drain() {
            if let Some(dw_chunk) = dw.chunk_at(coord) {
                dw_buf.set_chunk(device, coord, dw_chunk);
            }
        }
    }
}

#[derive(Debug, Snafu)]
pub enum EditorAppError {
    #[snafu(display("Failed to open world_db"))]
    OpenWorldDb { source: WorldDbError },
    #[snafu(display("Failed to save world_db"))]
    SaveWorldDb { source: WorldDbError },
    #[snafu(display("Failed to read world_db bytes"))]
    ReadWorldDbBytes { source: std::io::Error },
}

type Result<T> = std::result::Result<T, EditorAppError>;

// Different mode has different interaction logic
#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum Mode {
    // drag: move around
    // hover: hover on block / dw object
    // left click: select block / dw object
    // scroll: zoom in / out
    #[default]
    View,

    // drag: draw blocks
    // hover: hover on block / dw object
    // left click: select block / dw object, if none then draw block A / add dw object
    // middle click: sample block / dw object as reference
    // right click: draw block B / delete dw_object
    // scroll: zoom in / out
    Pen,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum PenDrawTarget {
    #[default]
    Block,
    DwObj,
}

#[derive(Debug, Clone)]
struct BlockData([u8; NUM_BYTES_PER_BLOCK]);

impl BlockData {
    fn view_mut(&mut self) -> BlockViewMut<'_> {
        BlockViewMut::new(&mut self.0)
    }

    fn new(fg: BlockType, bg: BlockType, content: BlockContentType) -> Self {
        let mut data = Self([0; NUM_BYTES_PER_BLOCK]);
        let mut view_mut = data.view_mut();
        view_mut.set_fg(fg);
        view_mut.set_bg(bg);
        view_mut.set_content(content);
        data
    }

    fn from_bytes(bytes: [u8; NUM_BYTES_PER_BLOCK]) -> Self {
        Self(bytes)
    }

    fn as_bytes(&self) -> &[u8; NUM_BYTES_PER_BLOCK] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
enum PrimaryTarget {
    #[default]
    A,
    B,
}

#[derive(Debug, Clone)]
struct PenModeSettings {
    block_a: BlockData,
    block_b: BlockData,
    // primary => binds to left click, not primary => binds to right click
    primary_block: PrimaryTarget,
    dyn_obj: AnyDynamicObject,
    dyn_obj_snap: bool,
    target: PenDrawTarget,
}

impl Default for PenModeSettings {
    fn default() -> Self {
        Self {
            block_a: BlockData::new(
                BlockType::Stone,
                BlockType::Wood,
                BlockContentType::TitaniumOre,
            ),
            block_b: BlockData::new(BlockType::Air, BlockType::Air, BlockContentType::Nothing),
            primary_block: PrimaryTarget::default(),
            dyn_obj: AnyDynamicObject::default(),
            dyn_obj_snap: true,
            target: PenDrawTarget::default(),
        }
    }
}

pub struct EditorApp {
    world_db: Option<WorldDb>,
    chunk_cache: ChunkCache,
    dw_buf: DwBuf,
    camera: Camera,

    show_settings: bool,
    fps_counter: FpsCounter,
    mode: Mode,
    pen_mode_settings: PenModeSettings,

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
        let interaction_state = InteractionState::default();
        let render_resources = RenderResources::new(
            state,
            camera.create_buffer(device),
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
            chunk_cache: ChunkCache::new(),
            dw_buf: DwBuf::new(),
            camera,

            show_settings: false,
            fps_counter: FpsCounter::new(2.0),
            mode: Mode::default(),
            pen_mode_settings: PenModeSettings::default(),

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
        if new_world_width_macro != old_world_width_macro {
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
        for chunk_y in 0..Chunks::NUM_CHUNK_PER_COL {
            for chunk_x in 0..world_db.main.world_v2.world_width_macro {
                let chunk_coord = ChunkCoord::new(chunk_x, chunk_y as u8).unwrap();
                if !self.dw_buf.has_chunk(chunk_coord)
                    && let Some(chunk) = world_db.dw.chunk_at(chunk_coord)
                {
                    self.dw_buf.set_chunk(device, chunk_coord, chunk);
                }
            }
        }
        self.dw_buf
            .set_blockheads(device, &world_db.main.blockheads);

        self.world_db = Some(world_db);

        Ok(())
    }

    fn render_menu_bar(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        self.file_reader
                            .set_open_path(rfd::FileDialog::new().pick_file());
                    }

                    #[cfg(target_arch = "wasm32")]
                    {
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
                }
                let mut arch = None;
                ui.menu_button("Save As", |ui| {
                    if ui
                        .button("32-bit")
                        .on_hover_text("Save as 32 bit LMDB file, usually for client.")
                        .clicked()
                    {
                        arch = Some(DynArch::Arch32);
                    }
                    if ui
                        .button("64-bit")
                        .on_hover_text("Save as 64 bit LMDB file, usually for server.")
                        .clicked()
                    {
                        arch = Some(DynArch::Arch64);
                    }
                });
                if let Some(arch) = arch
                        // TODO if world_db is empty we should return as error
                        && let Some(world_db) = self.world_db.as_mut()
                {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if let Some(save_path) = rfd::FileDialog::new().pick_folder() {
                            self.chunk_cache.flush(world_db);
                            if let Err(e) =
                                world_db.to_path(save_path, arch).context(SaveWorldDbSnafu)
                            {
                                // TODO allow user select arch
                                self.save_err = Some(e);
                            }
                        }
                    }

                    #[cfg(target_arch = "wasm32")]
                    {
                        self.chunk_cache.flush(world_db);
                        let mut out_bytes = Vec::new();
                        match world_db
                            .write_to(&mut out_bytes, arch)
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

            ui.separator();
            ui.toggle_value(&mut self.render_settings.show_grid, "Grid");
            ui.toggle_value(&mut self.show_settings, "Settings");

            ui.separator();
            if ui
                .toggle_value(&mut (self.mode == Mode::View), "View")
                .changed()
            {
                self.mode = Mode::View;
            }
            if ui
                .toggle_value(&mut (self.mode == Mode::Pen), "Pen")
                .changed()
            {
                self.mode = Mode::Pen;
            }

            ui.separator();
            ui.label("Viewport center:");
            ui.add(
                egui::DragValue::new(&mut self.camera.world_offset_mut().x)
                    .speed(0.1)
                    .prefix("X: "),
            );
            ui.add(
                egui::DragValue::new(&mut self.camera.world_offset_mut().y)
                    .speed(0.1)
                    .prefix("Y: "),
            );
            ui.add(
                egui::DragValue::new(&mut self.camera.world_offset_mut().z)
                    .speed(0.1)
                    .range(Camera::MAX_BLOCK_Z..=Camera::MAX_Z)
                    .prefix("Dist: "),
            );
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

    fn render_block_info(ui: &mut egui::Ui, mut block: BlockViewMut<'_>) -> bool {
        let mut update_voxel = false;
        egui::Grid::new("my_grid")
            .num_columns(3)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("foreground");
                ui.push_id("fg", |ui| {
                    update_voxel |= ui.add(egui::DragValue::new(block.fg_raw_mut())).changed();
                    match block.fg() {
                        Ok(mut fg_type) => {
                            if block_type_drop_menu(ui, &mut fg_type).changed() {
                                block.set_fg(fg_type);
                                update_voxel = true;
                            }
                        }
                        Err(e) => {
                            ui.weak(e.to_string());
                        }
                    };
                });
                ui.end_row();

                ui.label("background");
                ui.push_id("bg", |ui| {
                    update_voxel |= ui.add(egui::DragValue::new(block.bg_raw_mut())).changed();
                    match block.bg() {
                        Ok(mut bg_type) => {
                            if block_type_drop_menu(ui, &mut bg_type).changed() {
                                block.set_bg(bg_type);
                                update_voxel = true;
                            }
                        }
                        Err(e) => {
                            ui.weak(e.to_string());
                        }
                    };
                });
                ui.end_row();

                ui.label("content");
                ui.push_id("ct", |ui| {
                    update_voxel |= ui
                        .add(egui::DragValue::new(block.content_raw_mut()))
                        .changed();
                    match block.content() {
                        Ok(mut content_type) => {
                            if block_content_type_drop_menu(ui, &mut content_type).changed() {
                                block.set_content(content_type);
                                update_voxel = true;
                            }
                        }
                        Err(e) => {
                            ui.weak(e.to_string());
                        }
                    };
                });
                ui.end_row();

                ui.label("height")
                    .on_hover_text("The height of water/snow; not rendered here");
                ui.add(egui::DragValue::new(block.height_mut()));
                ui.end_row();

                ui.label("damage")
                    .on_hover_text("The gathering progress value");
                ui.add(egui::DragValue::new(block.damage_mut()));
                ui.end_row();

                ui.label("visibility")
                    .on_hover_text("The black fog that covers the undiscovered area");
                ui.add(egui::DragValue::new(block.visibility_mut()));
                ui.end_row();

                ui.label("brightness")
                    .on_hover_text("Blocks in cave have near-zero brightness");
                ui.add(egui::DragValue::new(block.brightness_mut()));
                ui.end_row();
            });

        update_voxel
    }

    fn render_settings_window(&mut self, ui: &mut egui::Ui) {
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
            &mut self.render_settings.render_dw_item,
            "Render Dynamic Object Icons",
        );
        ui.checkbox(
            &mut self.render_settings.render_dw_mesh,
            "Render Dynamic Object Meshs",
        );
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
            egui::Slider::new(&mut self.render_settings.ambient_reflect, 0.0..=1.0)
                .text("Ambient Reflection"),
        );
        ui.add(
            egui::Slider::new(&mut self.render_settings.min_depth_factor, 0.0..=1.0)
                .text("Min Depth Factor"),
        );
    }

    fn update_camera_pos(&mut self, ui: &mut egui::Ui, response: &egui::Response) {
        let viewport_size = (
            self.world_viewport_rect.width(),
            self.world_viewport_rect.height(),
        );

        if ((matches!(self.mode, Mode::View) && response.dragged_by(egui::PointerButton::Primary))
            || (matches!(self.mode, Mode::Pen) && response.dragged_by(egui::PointerButton::Middle)))
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
            // scroll_y measured on platforms
            #[cfg(target_arch = "wasm32")]
            const SCROLL_Y: f32 = 90.909;

            #[cfg(not(target_arch = "wasm32"))]
            const SCROLL_Y: f32 = 40.0;

            const TARGET_SCROLL_Y: f32 = 40.0;
            const SCROLL_MULTIPLIER: f32 = TARGET_SCROLL_Y / SCROLL_Y;

            let scroll_y = ui.input(|i| i.smooth_scroll_delta.y) * SCROLL_MULTIPLIER;
            if scroll_y.abs() > 0.0 {
                self.camera.handle_zoom(
                    (pos - self.world_viewport_rect.min).into(),
                    viewport_size,
                    scroll_y,
                );
            }
        }

        // Wrap camera X cyclically when cyclic mode is enabled and a world is loaded
        if self.render_settings.enable_cyclic
            && let Some(world_db) = self.world_db.as_ref()
        {
            let world_block_width =
                (world_db.main.world_v2.world_width_macro as f32) * Chunk::NUM_BLOCK_PER_ROW as f32;
            if world_block_width > 0.0 {
                let x = &mut self.camera.world_offset_mut().x;
                *x = x.rem_euclid(world_block_width);
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
                if let Some((id, chunk_coord)) = {
                    *self
                        .interaction_state
                        .hover_on_dyn_obj_id
                        .lock()
                        .expect("should lock")
                } {
                    self.interaction_state.set_hover_on_obj(chunk_coord, id);
                } else {
                    self.interaction_state.set_hover_on_block(block_coord);
                }
            } else {
                self.interaction_state.clear_hover();
            }
        }

        if matches!(self.mode, Mode::View) && response.clicked_by(egui::PointerButton::Primary) {
            self.interaction_state.copy_hover_to_select();
        }
    }

    fn update_mouse_pos(&mut self, response: &egui::Response) {
        self.mouse_pos = response.hover_pos().map(|pos| {
            ((pos - self.world_viewport_rect.min) * response.ctx.pixels_per_point()).into()
        });
    }

    fn handle_pen_mode_input(&mut self, response: &egui::Response) {
        if matches!(self.mode, Mode::Pen)
            && let Some(world_db) = self.world_db.as_mut()
        {
            match self.pen_mode_settings.target {
                PenDrawTarget::Block => {
                    let (primary_block_data, secondary_block_data) =
                        match self.pen_mode_settings.primary_block {
                            PrimaryTarget::A => (
                                &mut self.pen_mode_settings.block_a,
                                &self.pen_mode_settings.block_b,
                            ),
                            PrimaryTarget::B => (
                                &mut self.pen_mode_settings.block_b,
                                &self.pen_mode_settings.block_a,
                            ),
                        };

                    if let Some(InteractionTarget::Block(block_coord)) =
                        self.interaction_state.hover
                    {
                        if response.clicked_by(egui::PointerButton::Primary)
                            || response.drag_started_by(egui::PointerButton::Primary)
                            || response.dragged_by(egui::PointerButton::Primary)
                        {
                            let chunk = self
                                .chunk_cache
                                .get_or_create_chunk_mut(block_coord.into(), world_db);
                            let chunk_block_coord: ChunkBlockCoord = block_coord.into();
                            let mut chunk_view_mut = chunk.view_mut();
                            let mut block = chunk_view_mut.block_at_mut(chunk_block_coord);
                            *block.as_mut_bytes() = *primary_block_data.as_bytes();
                            self.chunk_cache.mark_voxel_outdated(block_coord);
                        }
                        if response.clicked_by(egui::PointerButton::Middle)
                            && let Some(chunk) =
                                self.chunk_cache.get_chunk_mut(block_coord.into(), world_db)
                        {
                            let chunk_view = chunk.view();
                            let chunk_block_coord: ChunkBlockCoord = block_coord.into();
                            let block_view = chunk_view.block_at(chunk_block_coord);
                            *primary_block_data = BlockData::from_bytes(*block_view.as_bytes());
                        }
                        if response.clicked_by(egui::PointerButton::Secondary)
                            || response.drag_started_by(egui::PointerButton::Secondary)
                            || response.dragged_by(egui::PointerButton::Secondary)
                        {
                            let chunk = self
                                .chunk_cache
                                .get_or_create_chunk_mut(block_coord.into(), world_db);
                            let chunk_block_coord: ChunkBlockCoord = block_coord.into();
                            let mut chunk_view_mut = chunk.view_mut();
                            let mut block = chunk_view_mut.block_at_mut(chunk_block_coord);
                            *block.as_mut_bytes() = *secondary_block_data.as_bytes();
                            self.chunk_cache.mark_voxel_outdated(block_coord);
                        }
                    }
                }
                PenDrawTarget::DwObj => {
                    let dw = &mut world_db.dw;
                    if response.clicked_by(egui::PointerButton::Primary) {
                        match self.interaction_state.hover {
                            Some(InteractionTarget::DynamicObject { .. }) => {
                                self.interaction_state.copy_hover_to_select();
                            }
                            Some(InteractionTarget::Block(_)) | None => {
                                if let Some(pos) = response.hover_pos()
                                    && let [x, y] = self
                                        .camera
                                        .mouse_at(
                                            (pos - self.world_viewport_rect.min).into(),
                                            self.world_viewport_rect.size().into(),
                                        )
                                        .to_array()
                                    && let Ok(block_coord) =
                                        BlockCoord::new(x.floor() as u32, y.floor() as u16)
                                {
                                    let entry = dw.entry(block_coord);
                                    let chunk_dyn_objs =
                                        entry.or_insert_with(ChunkDynamicObjects::default);
                                    let mut dyn_obj = self.pen_mode_settings.dyn_obj.clone();
                                    match self.pen_mode_settings.dyn_obj_snap {
                                        true => dyn_obj.set_pos((block_coord.x(), block_coord.y())),
                                        false => dyn_obj.set_float_pos([x, y]),
                                    }
                                    dyn_obj.set_unique_id(
                                        world_db.main.dynamic_world_v2.new_unique_id(),
                                    );
                                    // TODO: it's not clear if we should reuse AnyDynamicObject for both item and pen mode
                                    // e.g. no item can ever hold "gatherBlock" object
                                    // potentially need a dedicated type, for now we stick to AnyDynamicObject
                                    chunk_dyn_objs.insert(dyn_obj);
                                    self.chunk_cache.mark_mesh_outdated(block_coord);
                                }
                            }
                        }
                    }
                    if let Some(InteractionTarget::DynamicObject { chunk_coord, id }) =
                        self.interaction_state.hover
                        && let ObjectType::DynamicObject(dyn_obj_ty) = id.obj_type
                    {
                        if response.clicked_by(egui::PointerButton::Middle)
                            && let Some(chunk_dyn_objs) = dw.chunk_at(chunk_coord)
                            && let Some(any_dyn_obj_ref) = chunk_dyn_objs.get(dyn_obj_ty, id.index)
                        {
                            self.pen_mode_settings.dyn_obj = any_dyn_obj_ref.to_owned();
                        }
                        if response.clicked_by(egui::PointerButton::Secondary)
                            && let Some(chunk_dyn_objs) = dw.chunk_at_mut(chunk_coord)
                        {
                            chunk_dyn_objs.remove(dyn_obj_ty, id.index);
                            self.chunk_cache.mark_mesh_outdated(chunk_coord);
                            self.interaction_state.clear_hover();

                            if let Some(InteractionTarget::DynamicObject {
                                chunk_coord: selected_chunk_coord,
                                id: selected_id,
                            }) = self.interaction_state.select
                                && selected_chunk_coord == chunk_coord
                            {
                                match id.index.cmp(&selected_id.index) {
                                    std::cmp::Ordering::Less => {
                                        self.interaction_state.set_select(Some(
                                            InteractionTarget::DynamicObject {
                                                chunk_coord,
                                                id: DwChunkObjId {
                                                    obj_type: ObjectType::DynamicObject(dyn_obj_ty),
                                                    index: selected_id.index - 1,
                                                },
                                            },
                                        ));
                                    }
                                    std::cmp::Ordering::Equal => {
                                        self.interaction_state.set_select(None);
                                    }
                                    std::cmp::Ordering::Greater => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn update_outdated_voxel_mesh(&mut self, frame: &eframe::Frame) {
        if let Some(state) = frame.wgpu_render_state() {
            let read_renderer = state.renderer.read();
            let r = read_renderer
                .callback_resources
                .get::<RenderResources>()
                .expect("should have render resources");
            self.chunk_cache.update_voxels(&state.queue, r.voxel_buf());
            if let Some(world_db) = self.world_db.as_mut() {
                self.chunk_cache
                    .update_meshes(&state.device, &mut world_db.dw, &mut self.dw_buf);
            }
        }
    }

    fn render_3d_viewport(&mut self, ui: &mut egui::Ui, frame: &eframe::Frame) {
        let available_size = ui.available_size();
        let (rect, response) =
            ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());

        self.world_viewport_rect = rect;
        self.camera.set_aspect(rect.aspect_ratio());

        self.update_mouse_pos(&response);
        self.update_interaction_state(&response);
        self.update_camera_pos(ui, &response);
        self.handle_pen_mode_input(&response);

        self.update_outdated_voxel_mesh(frame);

        let [min_coords, max_coords] = self.camera.visible_world_region_2d(rect.size().into());
        let center = self.camera.world_offset().xy();
        const MAX_DIST: f32 = 48.0;

        let min_x = min_coords.x.max(center.x - MAX_DIST);
        let min_y = min_coords.y.max(center.y - MAX_DIST).max(0.0);
        let max_x = max_coords.x.min(center.x + MAX_DIST);
        let max_y = max_coords.y.min(center.y + MAX_DIST).min(1024.0);

        let chunk_width_f32 = Chunk::NUM_BLOCK_PER_ROW as f32;

        let chunk_min_y = (min_y / chunk_width_f32).floor() as u8;
        let chunk_max_y = (max_y / chunk_width_f32).ceil() as u8;

        let world_width_macro = self
            .world_db
            .as_ref()
            .and_then(|db| Pow2::new(db.main.world_v2.world_width_macro));
        let is_cyclic = self.render_settings.enable_cyclic;

        enum ChunkXIter {
            Cyclic {
                start: u32,
                i: u32,
                end: u32,
                width: Pow2<u32>,
            },
            Linear {
                i: u32,
                end: u32,
            },
        }

        impl Iterator for ChunkXIter {
            type Item = u32;

            fn next(&mut self) -> Option<Self::Item> {
                let (val, current) = match self {
                    ChunkXIter::Cyclic {
                        start,
                        end,
                        i,
                        width,
                    } => ((*i < *end).then_some((*start + *i).fast_rem(*width)), i),
                    ChunkXIter::Linear { i: current, end } => {
                        ((*current < *end).then_some(*current), current)
                    }
                };
                *current += 1;
                val
            }
        }

        let x_iter = if is_cyclic && let Some(width) = world_width_macro {
            let span_x = max_x - min_x;
            let world_block_width = (*width * Chunk::NUM_BLOCK_PER_ROW as u32) as f32;

            let start_x_wrapped = min_x.rem_euclid(world_block_width);
            let start_chunk_x = (start_x_wrapped / chunk_width_f32).floor() as u32;
            let num_chunks_x =
                ((start_x_wrapped % chunk_width_f32 + span_x) / chunk_width_f32).ceil() as u32;

            ChunkXIter::Cyclic {
                start: start_chunk_x,
                end: num_chunks_x,
                i: 0,
                width,
            }
        } else {
            let min_x = min_x.max(0.0);
            let chunk_min_x = (min_x / chunk_width_f32).floor() as u32;
            let mut chunk_max_x = (max_x / chunk_width_f32).ceil() as u32;

            if let Some(width) = world_width_macro {
                chunk_max_x = chunk_max_x.min(*width);
            }

            ChunkXIter::Linear {
                i: chunk_min_x,
                end: chunk_max_x,
            }
        };

        let mut dw_chunks = Vec::new();
        for x in x_iter {
            for y in chunk_min_y..chunk_max_y {
                if let Some(chunk_buf) = self.dw_buf.get_chunk(ChunkCoord::new(x, y).unwrap()) {
                    dw_chunks.push(chunk_buf.clone()); // chunk_buf is cheap to clone
                }
            }
        }
        let blockhead_instances = self.dw_buf.get_blockhead_instances();

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            Render3dCallback {
                camera_uniform: self.camera.uniform(),
                dw_chunks,
                blockhead_instances,
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
                    let e = snafu::Report::from_error(e);
                    ui.heading("Error message");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(format!("{}", e));
                    });
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

    fn render_selected_block_info_window(&mut self, egui_ctx: &egui::Context) {
        let mut open = true;
        if let Some(InteractionTarget::Block(selected_block_coord)) = self.interaction_state.select
            && let Some(world_db) = self.world_db.as_ref()
            && let chunk_coord = selected_block_coord.into()
            && let Some(selected_chunk) = self.chunk_cache.get_chunk_mut(chunk_coord, world_db)
        {
            let mut update_voxel = false;
            egui::Window::new(format!("Selected Block in Chunk {}", chunk_coord))
                .id("selected_dynamic_obj_info".into())
                .open(&mut open)
                .show(egui_ctx, |ui| {
                    let mut chunk_view_mut = selected_chunk.view_mut();
                    let block_view_mut = chunk_view_mut.block_at_mut(selected_block_coord);
                    ui.separator();
                    ui.heading(format!(
                        "Block {}, {}",
                        selected_block_coord.x(),
                        selected_block_coord.y(),
                    ));
                    update_voxel = Self::render_block_info(ui, block_view_mut);
                });
            if update_voxel {
                self.chunk_cache.mark_voxel_outdated(chunk_coord);
            }
        }
        if !open {
            self.interaction_state.set_select(None);
        }
    }

    fn render_selected_dyn_obj_info_window(
        &mut self,
        egui_ctx: &egui::Context,
        frame: &mut eframe::Frame,
    ) {
        if let Some(InteractionTarget::DynamicObject { chunk_coord, id }) =
            self.interaction_state.select
            && let Some(world_db) = self.world_db.as_mut()
        {
            let dw = &mut world_db.dw;

            fn draw_window<T: InfoUi>(
                title: &str,
                t: Option<&mut T>,
                ctx: (&egui::Context, DwUiContext),
            ) -> (ObjFlags, bool) {
                let (egui_ctx, mut context) = ctx;
                let mut open = true;
                if let Some(t) = t {
                    egui::Window::new(title)
                        .id("selected_dynamic_obj_info".into())
                        .open(&mut open)
                        .show(egui_ctx, |ui| {
                            egui::ScrollArea::both().show(ui, |ui| {
                                t.info(ui, &mut context);
                            });
                        });
                }
                (context.flags, open)
            }

            fn draw_dyn_obj_window(
                dyn_obj_type: DynamicObjectType,
                index: usize,
                dw_chunk: &mut ChunkDynamicObjects,
                ctx: (&egui::Context, DwUiContext),
            ) -> (ObjFlags, bool) {
                use DynamicObjectType::*;
                let title: &'static str = dyn_obj_type.into();
                match dyn_obj_type {
                    AppleTree => draw_window(title, dw_chunk.apple_tree.get_mut(index), ctx),
                    MapleTree => draw_window(title, dw_chunk.maple_tree.get_mut(index), ctx),
                    MangoTree => draw_window(title, dw_chunk.mango_tree.get_mut(index), ctx),
                    PineTree => draw_window(title, dw_chunk.pine_tree.get_mut(index), ctx),
                    CactusTree => draw_window(title, dw_chunk.cactus_tree.get_mut(index), ctx),
                    CoconutTree => draw_window(title, dw_chunk.coconut_tree.get_mut(index), ctx),
                    OrangeTree => draw_window(title, dw_chunk.orange_tree.get_mut(index), ctx),
                    CherryTree => draw_window(title, dw_chunk.cherry_tree.get_mut(index), ctx),
                    CoffeeTree => draw_window(title, dw_chunk.coffee_tree.get_mut(index), ctx),
                    FlaxPlant => draw_window(title, dw_chunk.flax_plant.get_mut(index), ctx),
                    SunflowerPlant => {
                        draw_window(title, dw_chunk.sunflower_plant.get_mut(index), ctx)
                    }
                    CornPlant => draw_window(title, dw_chunk.corn_plant.get_mut(index), ctx),
                    // Dodo => {}
                    // DroppedItem => {}
                    // Fire => {}
                    Torch => draw_window(title, dw_chunk.torch.get_mut(index), ctx),
                    // GlowBlock => {}
                    Ladder => draw_window(title, dw_chunk.ladder.get_mut(index), ctx),
                    Door => draw_window(title, dw_chunk.door.get_mut(index), ctx),
                    // ArtificialLight => {}
                    // Bed => {}
                    // DropBear => {}
                    // GatherBlock => {}
                    CarrotPlant => draw_window(title, dw_chunk.carrot_plant.get_mut(index), ctx),
                    // Donkey => {}
                    Egg => draw_window(title, dw_chunk.egg.get_mut(index), ctx),
                    Window => draw_window(title, dw_chunk.window.get_mut(index), ctx),
                    // Boat => {}
                    ChilliPlant => draw_window(title, dw_chunk.chilli_plant.get_mut(index), ctx),
                    KelpPlant => draw_window(title, dw_chunk.kelp_plant.get_mut(index), ctx),
                    // ClownFish => {}
                    // Shark => {}
                    LimeTree => draw_window(title, dw_chunk.lime_tree.get_mut(index), ctx),
                    Wire => draw_window(title, dw_chunk.wire.get_mut(index), ctx),
                    // CaveTroll => {}
                    // Rail => {}
                    // HandCar => {}
                    // SteamLocomotive => {}
                    // FreightCar => {}
                    // PassengerCar => {}
                    Workbench => draw_window(title, dw_chunk.workbench.get_mut(index), ctx),
                    Chest => draw_window(title, dw_chunk.chest.get_mut(index), ctx),
                    Sign => draw_window(title, dw_chunk.sign.get_mut(index), ctx),
                    // TradingPost => {}
                    TrainStation => draw_window(title, dw_chunk.train_station.get_mut(index), ctx),
                    // TradePortal => {}
                    // Scorpion => {}
                    // Painting => {}
                    // Column => {}
                    // Stairs => {}
                    // ElevatorMotor => {}
                    // ElevatorShaft => {}
                    GemTree => draw_window(title, dw_chunk.gem_tree.get_mut(index), ctx),
                    VinePlant => draw_window(title, dw_chunk.vine_plant.get_mut(index), ctx),
                    TulipPlant => draw_window(title, dw_chunk.tulip_plant.get_mut(index), ctx),
                    // OwnershipSign => {}
                    WheatPlant => draw_window(title, dw_chunk.wheat_plant.get_mut(index), ctx),
                    TomatoPlant => draw_window(title, dw_chunk.tomato_plant.get_mut(index), ctx),
                    // Yak => {}
                    // Mirror => {}
                    _ => (ObjFlags::default(), true),
                }
            }

            let context = DwUiContext::new(
                world_db.main.world_v2.world_width_macro * Chunk::NUM_BLOCK_PER_ROW as u32,
                self.render_settings.enable_cyclic,
                ObjFlags::default(),
            );
            let ctx = (egui_ctx, context);
            let draw_result = match id.obj_type {
                ObjectType::DynamicObject(dyn_obj_type) => dw
                    .chunk_at_mut(chunk_coord)
                    .map(|dw_chunk| draw_dyn_obj_window(dyn_obj_type, id.index, dw_chunk, ctx)),
                ObjectType::Blockhead => Some(draw_window(
                    "Blockhead",
                    world_db.main.blockheads.get_mut(id.index),
                    ctx,
                )),
            };
            if let Some((flags, open)) = draw_result {
                if !open {
                    self.interaction_state.select = None;
                }

                let mut move_dyn_obj =
                    |x: f32,
                     y: f32,
                     chunk_coord: ChunkCoord,
                     dyn_obj_ty: DynamicObjectType,
                     index: usize,
                     dw: &mut DynamicWorld| {
                        if let Ok(dst_coord) = BlockCoord::new(x as u32, y as u16)
                            && let (dst_chunk_coord, _) = dst_coord.decompose()
                            && dst_chunk_coord != chunk_coord
                        {
                            let new_i =
                                dw.move_element(chunk_coord, dst_chunk_coord, dyn_obj_ty, index);
                            if let Some(new_i) = new_i {
                                self.interaction_state.select =
                                    Some(InteractionTarget::DynamicObject {
                                        chunk_coord: dst_chunk_coord,
                                        id: DwChunkObjId::from_dyn_obj(dyn_obj_ty, new_i),
                                    });
                            }
                            self.chunk_cache.mark_mesh_outdated(dst_chunk_coord);
                        }
                    };
                // handle obj move & dst chunk re-render
                if let Some((x, y)) = flags.pos_changed_to {
                    match id.obj_type {
                        ObjectType::DynamicObject(dyn_obj_ty) => {
                            move_dyn_obj(x, y, chunk_coord, dyn_obj_ty, id.index, dw);
                        }
                        ObjectType::Blockhead => {
                            if let Some(blockhead) = world_db.main.blockheads.get_mut(id.index) {
                                blockhead.float_pos = [x, y];
                            }
                            if let Some(state) = frame.wgpu_render_state() {
                                self.dw_buf
                                    .set_blockheads(&state.device, &world_db.main.blockheads);
                            }
                        }
                    }
                }
                if flags.rebuild_mesh {
                    self.chunk_cache.mark_mesh_outdated(chunk_coord);
                }
            }
        }
    }

    // helper function for the pen mode
    fn render_block_slot(
        ui: &mut egui::Ui,
        title: &str,
        id: &str,
        primary_block: &mut PrimaryTarget,
        target: PrimaryTarget,
        block_view_mut: BlockViewMut,
    ) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(title);
                ui.selectable_value(
                    primary_block,
                    target,
                    if *primary_block == target {
                        "Primary"
                    } else {
                        "Secondary"
                    },
                );
            });
            ui.push_id(id, |ui| {
                Self::render_block_info(ui, block_view_mut);
            });
        });
    }

    fn render_pen_mode_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("Pen Mode Settings")
            .id("pen_mode_settings_window".into())
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::splat(-10.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.pen_mode_settings.target,
                        PenDrawTarget::Block,
                        "Block",
                    );
                    ui.selectable_value(
                        &mut self.pen_mode_settings.target,
                        PenDrawTarget::DwObj,
                        "Dynamic Object",
                    );
                });

                ui.separator();

                let primary_block = &mut self.pen_mode_settings.primary_block;
                match self.pen_mode_settings.target {
                    PenDrawTarget::Block => {
                        ui.horizontal(|ui| {
                            Self::render_block_slot(
                                ui,
                                "Block A",
                                "block_a",
                                primary_block,
                                PrimaryTarget::A,
                                self.pen_mode_settings.block_a.view_mut(),
                            );
                            ui.separator();
                            Self::render_block_slot(
                                ui,
                                "Block B",
                                "block_b",
                                primary_block,
                                PrimaryTarget::B,
                                self.pen_mode_settings.block_b.view_mut(),
                            );
                        });
                    }
                    PenDrawTarget::DwObj => {
                        ui.checkbox(&mut self.pen_mode_settings.dyn_obj_snap, "Snap to Blocks");

                        self.pen_mode_settings
                            .dyn_obj
                            .info(ui, &mut DwUiContext::default());
                    }
                };
            });
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.try_read_id(frame);
        self.fps_counter.update(ctx.input(|i| i.time));

        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Top, "menu").show(ctx, |ui| {
            self.render_menu_bar(ui, frame);
        });

        let mut show_settings = self.show_settings;
        egui::Window::new("Settings")
            .open(&mut show_settings)
            .show(ctx, |ui| {
                self.render_settings_window(ui);
            });
        self.show_settings = show_settings;

        self.render_error_windows(ctx);
        self.render_selected_block_info_window(ctx);
        self.render_selected_dyn_obj_info_window(ctx, frame);
        if self.mode == Mode::Pen {
            self.render_pen_mode_window(ctx);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(0.0))
            .show(ctx, |ui| {
                self.render_3d_viewport(ui, frame);
            });
    }
}
