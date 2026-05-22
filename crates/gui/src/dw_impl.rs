// Implements traits for dynamic object types defined in lib
use super::{
    gpu::{
        BlockUv,
        dw::{
            BuildDwMesh, BuildDwMeshError, CoordOutOfBoundSnafu, DwBlock, DwCapacity,
            DwChunkBufBuilder, DwFace, DwItem, DwItemInstanceRaw, DwQuad, FaceDirection,
            InvalidItemTypeForDoorSnafu, InvalidItemTypeForTorchSnafu, InvalidWorkbenchLevelSnafu,
        },
    },
    image_type::ImageType,
    render::{
        ItemGridCallback, ItemGridInstances,
        item_grid::{COL_PX, ITEM_SELECTOR_COLS, ITEM_SELECTOR_ROWS, ITEM_SELECTOR_SIZE, ROW_PX},
    },
};
use eframe::{egui, egui_wgpu};
use snafu::ResultExt;
use std::{
    hash::Hash,
    ops::{BitOrAssign, DerefMut},
};
use strum::IntoEnumIterator;
use the_blockheads_tools_lib::game::{
    block::{BlockContentType, BlockType},
    coord::BlockCoord,
    dynamic_object::{
        AnyDynamicObject, ArtificialLight, DynamicObject, InteractionObject, InteractionObjectType,
        LightDirection, UniqueID,
        animal::{DodoBreed, Egg},
        chest::{Chest, ChestSlots, ChestType},
        craft::{
            Door, Ladder, Sign, SignConnectionType, Torch, TorchConnectionType, Wire,
            WireConfiguration, WireSolidConfiguration,
        },
        plant::{
            CarrotPlant, ChilliPlant, CornPlant, FlaxPlant, KelpPlant, NormalPlant, Plant,
            SunflowerPlant, TomatoPlant, TulipPlant, VinePlant, WheatPlant,
        },
        train::TrainStation,
        tree::{
            AppleTree, CactusTree, CherryTree, CoconutTree, CoffeeTree, GemTree, LimeTree,
            MangoTree, MapleTree, OrangeTree, PineTree, Tree, TreeFruit, TreeType,
        },
        workbench::{Workbench, WorkbenchType},
    },
    item::{Item, ItemType, Slot},
};

const CUBE_NUM_FACES: usize = 6;

#[derive(Debug, Default, PartialEq)]
pub enum ObjFlags {
    PosChangedTo {
        x: f32,
        y: f32,
    },
    RebuildMesh,
    #[default]
    NoChange,
}

impl BitOrAssign for ObjFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (lhs @ Self::PosChangedTo { .. }, rhs @ Self::PosChangedTo { .. })
            | (lhs @ Self::RebuildMesh, rhs @ Self::PosChangedTo { .. })
            | (lhs @ Self::NoChange, rhs) => *lhs = rhs,
            (Self::PosChangedTo { .. }, Self::RebuildMesh)
            | (Self::RebuildMesh, Self::RebuildMesh)
            | (_, Self::NoChange) => {}
        }
    }
}

// helper mod to keep read-only fields private
mod context {
    use super::ObjFlags;

    pub struct DwUiContext {
        world_width: u32,
        cyclic: bool,
        pub flags: ObjFlags,
    }

    impl DwUiContext {
        pub fn new(world_width: u32, cyclic: bool, flags: ObjFlags) -> Self {
            Self {
                world_width,
                cyclic,
                flags,
            }
        }

        pub fn world_width(&self) -> u32 {
            self.world_width
        }

        pub fn cyclic(&self) -> bool {
            self.cyclic
        }
    }
}
pub use context::DwUiContext;

trait ToRow {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response;

    fn add_row(&mut self, label: &str, ui: &mut egui::Ui) -> egui::Response {
        ui.label(label);
        let response = self.to_row(ui);
        ui.end_row();
        response
    }
}

impl ToRow for u64 {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.add(egui::DragValue::new(self))
    }
}

impl ToRow for u32 {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.add(egui::DragValue::new(self))
    }
}

impl ToRow for u16 {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.add(egui::DragValue::new(self))
    }
}

impl ToRow for u8 {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.add(egui::DragValue::new(self))
    }
}

impl ToRow for i32 {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.add(egui::DragValue::new(self))
    }
}

impl ToRow for UniqueID {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(format!("{:?}", self))
    }
}

impl ToRow for String {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.text_edit_singleline(self)
    }
}

impl ToRow for &'static str {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(*self)
    }
}

impl<T: ToRow + Default> ToRow for Option<T> {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut is_some = self.is_some();

        ui.horizontal(|ui| {
            // Checkbox to toggle if the Option is None or Some
            if ui.checkbox(&mut is_some, "").changed() {
                if is_some {
                    *self = Some(T::default());
                } else {
                    *self = None;
                }
            }

            if let Some(t) = self {
                t.to_row(ui)
            } else {
                ui.weak("None")
            }
        })
        .inner
    }
}

impl<T: ToGrid + Default> ToGrid for Option<T> {
    fn to_grid(&mut self, _: &mut egui::Ui, _: &mut DwUiContext) {}

    fn add_grid<H: Hash>(&mut self, id: H, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let mut is_some = self.is_some();

        ui.vertical(|ui| {
            // Checkbox to toggle if the Option is None or Some
            if ui.checkbox(&mut is_some, "").changed() {
                if is_some {
                    *self = Some(T::default());
                } else {
                    *self = None;
                }
            }

            if let Some(t) = self {
                t.add_grid(id, ui, context);
            } else {
                ui.weak("None");
            }
        });
    }
}

const FLOAT_SPEED: f32 = 0.5;

impl ToRow for f32 {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.add(egui::DragValue::new(self).speed(FLOAT_SPEED))
    }
}

impl ToRow for f64 {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.add(egui::DragValue::new(self).speed(FLOAT_SPEED))
    }
}

impl ToRow for bool {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.checkbox(self, "")
    }
}

trait ToGrid {
    fn to_grid(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext);
    fn add_grid<H: Hash>(&mut self, id: H, ui: &mut egui::Ui, context: &mut DwUiContext) {
        egui::Grid::new(id).num_columns(2).show(ui, |ui| {
            self.to_grid(ui, context);
        });
    }
}

impl ToGrid for DynamicObject {
    fn to_grid(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let mut float_pos_changed = false;
        float_pos_changed |= self.float_pos[0].add_row("floatPos[0]", ui).changed();
        if float_pos_changed {
            let world_width = context.world_width() as f32;
            if context.cyclic() {
                self.float_pos[0] = self.float_pos[0].rem_euclid(world_width);
            } else {
                self.float_pos[0] = self.float_pos[0].max(0.0).min(world_width - 1e-3);
            }
        }
        float_pos_changed |= self.float_pos[1].add_row("floatPos[1]", ui).changed();

        let mut int_pos_changed = false;
        int_pos_changed |= self.pos_x.add_row("pos_x", ui).changed();
        if int_pos_changed {
            if context.cyclic() {
                self.pos_x = self.pos_x.rem_euclid(context.world_width());
            } else {
                self.pos_x = self.pos_x.min(context.world_width() - 1);
            }
        }
        int_pos_changed |= self.pos_y.add_row("pos_y", ui).changed();

        match (float_pos_changed, int_pos_changed) {
            // float takes precedence
            (true, true) | (true, false) => {
                self.pos_x = self.float_pos[0] as u32;
                self.pos_y = self.float_pos[1] as u16;
            }
            (false, true) => {
                self.float_pos[0] = self.pos_x as f32;
                self.float_pos[1] = self.pos_y as f32;
            }
            (false, false) => {}
        }
        if float_pos_changed | int_pos_changed {
            context.flags |= ObjFlags::PosChangedTo {
                x: self.float_pos[0],
                y: self.float_pos[1],
            };
        }

        self.unique_id.add_row("uniqueID", ui);
        self.owner_id.add_row("ownerID", ui);
    }
}

impl<T: Default + ToGrid> ToGrid for Vec<T> {
    fn to_grid(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        ui.vertical(|ui| {
            let mut to_remove = None;

            for (i, item) in self.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if ui.button("Del").on_hover_text("Remove item").clicked() {
                        to_remove = Some(i);
                    }

                    ui.collapsing(format!("Item #{}", i), |ui| {
                        item.add_grid(format!("item_grid_{}", i), ui, context);
                    });
                });
            }

            if ui.button("Add").clicked() {
                self.push(T::default());
            }

            // Perform deletion after the loop to avoid borrow checker issues
            if let Some(idx) = to_remove {
                self.remove(idx);
            }
        });
    }
}

impl ToGrid for TreeFruit {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut DwUiContext) {
        self.has_created_free_block_this_season
            .add_row("hasCreatedFreeBlockThisSeason", ui);
        self.pos_x.add_row("pos.x", ui);
        self.pos_y.add_row("pos.y", ui);
    }
}

fn grid_as_row<T: ToGrid, H: Hash>(
    t: &mut T,
    label: &str,
    id: H,
    ui: &mut egui::Ui,
    context: &mut DwUiContext,
) {
    ui.label(label);
    t.add_grid(id, ui, context);
    ui.end_row();
}

impl ToGrid for Tree {
    fn to_grid(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        self.age.add_row("age", ui);
        self.dead.add_row("dead", ui);
        self.time_died.add_row("timeDied", ui);
        self.remove_check_count.add_row("removeCheckCount", ui);
        self.growth_counter.add_row("growthCounter", ui);
        self.growth_rate.add_row("growthRate", ui);
        self.growth_rate_gene.add_row("growthRateGene", ui);
        self.height.add_row("height", ui);
        self.max_age.add_row("maxAge", ui);
        self.max_height.add_row("maxHeight", ui);
        self.max_height_gene.add_row("maxHeightGene", ui);
        self.max_height_reached.add_row("maxHeightReached", ui);
        self.save_time.add_row("saveTime", ui);
        self.tree_season_offset.add_row("treeSeasonOffset", ui);

        grid_as_row(
            &mut self.tree_fruits,
            "treeFruits",
            "tree_fruits_grid",
            ui,
            context,
        );
    }
}

pub(crate) trait InfoUi {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext);
}

impl InfoUi for DynamicObject {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        ui.vertical(|ui| {
            ui.heading("DynamicObject");
            ui.separator();
            self.add_grid("dynamic_object_grid", ui, context);
        });
    }
}

impl InfoUi for Tree {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let obj = self.deref_mut();
        obj.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("Tree");
            ui.separator();
            self.add_grid("tree_grid", ui, context);
        });
    }
}

impl BuildDwMesh for AppleTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_item(DwItem::from_item_type(self.float_pos, ItemType::Apple));
        Ok(())
    }
}

impl ToGrid for AppleTree {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut DwUiContext) {
        self.available_food.add_row("availableFood", ui);
    }
}

impl InfoUi for AppleTree {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let tree = self.deref_mut();
        tree.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("AppleTree");
            ui.separator();
            self.add_grid("apple_tree_grid", ui, context);
        });
    }
}

impl BuildDwMesh for MapleTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_item(DwItem::from_item_type(self.float_pos, ItemType::MapleSeed));
        Ok(())
    }
}

impl InfoUi for MapleTree {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let tree = self.deref_mut();
        tree.info(ui, context);
    }
}

impl BuildDwMesh for MangoTree {
    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_item(DwItem::from_item_type(self.float_pos, ItemType::Mango));
        Ok(())
    }
}

impl InfoUi for MangoTree {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let tree = self.deref_mut();
        tree.info(ui, context);
    }
}

impl ToGrid for PineTree {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut DwUiContext) {
        self.available_food.add_row("availableFood", ui);
    }
}

impl InfoUi for PineTree {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let tree = self.deref_mut();
        tree.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("PineTree");
            ui.separator();
            self.add_grid("pine_tree_grid", ui, context);
        });
    }
}

impl BuildDwMesh for PineTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_item(DwItem::from_item_type(self.float_pos, ItemType::Pinecone));
        Ok(())
    }
}

impl ToGrid for CactusTree {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut DwUiContext) {
        self.available_food.add_row("availableFood", ui);
        self.split_direction.add_row("splitDirection", ui);
        self.split_height_a.add_row("splitHeightA", ui);
        self.split_height_b.add_row("splitHeightB", ui);
    }
}

impl InfoUi for CactusTree {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let tree = self.deref_mut();
        tree.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("CactusTree");
            ui.separator();
            self.add_grid("cactus_tree_grid", ui, context);
        });
    }
}

impl BuildDwMesh for CactusTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_item(DwItem::from_item_type(
            self.float_pos,
            ItemType::PricklyPear,
        ));
        Ok(())
    }
}

impl InfoUi for CoconutTree {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let tree = self.deref_mut();
        tree.info(ui, context);
    }
}

impl BuildDwMesh for CoconutTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_item(DwItem::from_item_type(self.float_pos, ItemType::Coconut));
        Ok(())
    }
}

impl InfoUi for OrangeTree {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let tree = self.deref_mut();
        tree.info(ui, context);
    }
}
impl BuildDwMesh for OrangeTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_item(DwItem::from_item_type(self.float_pos, ItemType::Orange));
        Ok(())
    }
}

impl InfoUi for CherryTree {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let tree = self.deref_mut();
        tree.info(ui, context);
    }
}

impl BuildDwMesh for CherryTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_item(DwItem::from_item_type(self.float_pos, ItemType::Cherry));
        Ok(())
    }
}

impl InfoUi for CoffeeTree {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let tree = self.deref_mut();
        tree.info(ui, context);
    }
}

impl BuildDwMesh for CoffeeTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_item(DwItem::from_item_type(
            self.float_pos,
            ItemType::CoffeeCherry,
        ));
        Ok(())
    }
}

impl ToGrid for Plant {
    fn to_grid(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        self.save_time.add_row("saveTime", ui);
        self.season_offset.add_row("seasonOffset", ui);
        self.gather_progress.add_row("gatherProgress", ui);
        self.has_flowered_this_season
            .add_row("hasFloweredThisSeason", ui);
        if self.flowering.add_row("flowering", ui).changed() {
            context.flags |= ObjFlags::RebuildMesh;
        }
        self.frozen.add_row("frozen", ui);
        self.age.add_row("age", ui);
        self.max_age.add_row("maxAge", ui);
        self.max_age_gene.add_row("maxAgeGene", ui);
        self.growth_rate.add_row("growthRate", ui);
        self.growth_rate_gene.add_row("growthRateGene", ui);
    }
}

impl InfoUi for Plant {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let obj = self.deref_mut();
        obj.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("Plant");
            ui.separator();
            self.add_grid("plant_grid", ui, context);
        });
    }
}

fn wrap_combo_box_resp(r: egui::InnerResponse<Option<egui::Response>>) -> egui::Response {
    let mut resp = r.response;

    // we can't merge response from different layers, only certain properties
    if let Some(inner) = r.inner
        && inner.changed()
    {
        resp.mark_changed();
    }
    resp
}

impl ToRow for LightDirection {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        wrap_combo_box_resp(
            egui::ComboBox::from_id_salt("light_direction_combo_box")
                .selected_text(format!("{:?}", self))
                .show_ui(ui, |ui| {
                    ui.selectable_value(self, Self::All, "All")
                        | ui.selectable_value(self, Self::Up, "Up")
                        | ui.selectable_value(self, Self::Down, "Down")
                }),
        )
    }
}

impl ToGrid for ArtificialLight {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut DwUiContext) {
        self.max_red.add_row("maxRed", ui);
        self.max_green.add_row("maxGreen", ui);
        self.max_blue.add_row("maxBlue", ui);
        self.max_heat.add_row("maxHeat", ui);
        self.radius.add_row("radius", ui);
        self.contribution_grid_origin_x
            .add_row("contributionGridOrigin.x", ui);
        self.contribution_grid_origin_y
            .add_row("contributionGridOrigin.y", ui);
        self.light_direction.add_row("lightDirection", ui);
    }
}

impl ToGrid for NormalPlant {
    fn to_grid(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        self.available_food.add_row("saveTime", ui);
        grid_as_row(
            &mut self.light_dict,
            "lightDict",
            "light_dict_grid",
            ui,
            context,
        );
    }
}

impl InfoUi for NormalPlant {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let plant = self.deref_mut();
        plant.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("NormalPlant");
            ui.separator();
            self.add_grid("normal_plant_grid", ui, context);
        });
    }
}

impl InfoUi for FlaxPlant {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui, context);
    }
}

impl BuildDwMesh for FlaxPlant {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 0, quads: 1 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_face(DwFace::new_sprite(
            if self.flowering {
                ImageType::FlaxPlantFlower
            } else {
                ImageType::FlaxPlant
            },
            [0.5, 0.0],
            self.float_pos,
            [1, 2],
            2.0,
        ));
        Ok(())
    }
}

impl InfoUi for SunflowerPlant {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui, context);
    }
}

impl BuildDwMesh for SunflowerPlant {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 0, quads: 1 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_face(DwFace::new_sprite(
            if self.flowering {
                ImageType::SunflowerPlantFlower
            } else {
                ImageType::SunflowerPlant
            },
            [0.5, 0.0],
            self.float_pos,
            [1, 2],
            2.0,
        ));
        Ok(())
    }
}

impl InfoUi for CornPlant {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui, context);
    }
}

impl BuildDwMesh for CornPlant {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 0, quads: 1 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_face(DwFace::new_sprite(
            if self.flowering {
                ImageType::CornPlantFlower
            } else {
                ImageType::CornPlant
            },
            [0.5, 0.0],
            self.float_pos,
            [1, 2],
            2.0,
        ));
        Ok(())
    }
}

struct ItemGridResult {
    hovered_idx: Option<usize>,
    viewport: egui::Rect,
}

fn handle_item_grid_drag(
    ui: &mut egui::Ui,
    scroll_id: egui::Id,
    max_size: egui::Vec2,
    rect: egui::Rect,
    num_col: usize,
    num_row: usize,
    response: &egui::Response,
) -> ItemGridResult {
    let mut scroll_offset: egui::Vec2 =
        ui.data_mut(|d| d.get_temp(scroll_id).unwrap_or(egui::Vec2::ZERO));
    let mut drag_delta = response.drag_delta();
    if drag_delta != egui::Vec2::ZERO {
        let max_offset = max_size - rect.size();

        // 1. Calculate the minimum allowed drag delta.
        // If we're already out of bounds (scroll_offset > max_offset), this floors at 0.0.
        // If we're in bounds, this floors at the exact distance to the max_offset.
        let min_delta = (scroll_offset - max_offset).min(egui::Vec2::ZERO);

        // 2. Cap the drag_delta so it never pushes us further out of bounds.
        drag_delta = drag_delta.max(min_delta);
        scroll_offset = (scroll_offset - drag_delta).max(egui::Vec2::ZERO);
        ui.data_mut(|d| d.insert_temp(scroll_id, scroll_offset));
    }
    let viewport = egui::Rect::from_min_size(scroll_offset.to_pos2(), rect.size());

    let mut hovered_idx = None;
    if let Some(pos) = response.hover_pos() {
        let rel = pos + scroll_offset - rect.min;
        let col = (rel.x / COL_PX).floor() as usize;
        let row = (rel.y / ROW_PX).floor() as usize;

        if col < num_col && row < num_row {
            hovered_idx = Some(row * num_col + col);
        }
    }

    ItemGridResult {
        hovered_idx,
        viewport,
    }
}

impl ToRow for ItemType {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let item_type_str: &'static str = (*self).into();

        let mut resp = ui.button(item_type_str);
        let window_id = resp.id.with("item_type_window");

        let mut is_open = ui.data_mut(|d| d.get_temp(window_id).unwrap_or(false));

        if resp.clicked() {
            is_open = !is_open;
            ui.data_mut(|d| d.insert_temp(window_id, is_open));
        }

        if is_open {
            let mut keep_open = true;
            let scroll_id = window_id.with("scroll_offset");
            let hovered_idx_id = window_id.with("hovered_item");

            egui::Window::new("Select ItemType")
                .id(window_id)
                .open(&mut keep_open)
                .resizable(true)
                .default_width(ITEM_SELECTOR_SIZE.x)
                .default_height(ITEM_SELECTOR_SIZE.y + 17.65625) // default label height is 14.65625, spacing is 3
                .show(ui.ctx(), |ui| {
                    // ui.available_size() will return all space available within the window.
                    // It won't save space for ui.label if we put it after the gpu-rendered part.
                    // The window height will, as the result, increase indefinitely.
                    //
                    // Either we subtract the size of ui.label from ui.available_size()
                    // then use ui.with_layout(Layout::bottom_up), or we accept 1 frame latency
                    // by storing the hovered_item and use it in next frame.
                    //
                    // You don't know the size of ui.label until you add it. There's no way to solve
                    // this circular dependency with egui.
                    //
                    // For now we accept 1 frame lag and some extra memory usage.
                    let mut hovered_idx: Option<usize> =
                        ui.data_mut(|d| d.get_temp::<Option<usize>>(hovered_idx_id).flatten());
                    let text_h = ui
                        .label(
                            if let Some(idx) = hovered_idx.take()
                                && let Some(item_type) = ItemType::iter().nth(idx)
                            {
                                let name: &'static str = item_type.into();
                                name
                            } else {
                                ""
                            },
                        )
                        .rect
                        .height();

                    let (rect, response) = ui.allocate_exact_size(
                        ui.available_size().min(ITEM_SELECTOR_SIZE),
                        egui::Sense::all(),
                    );
                    let rect = rect.intersect(ui.ctx().content_rect());

                    let ItemGridResult {
                        hovered_idx,
                        viewport,
                    } = handle_item_grid_drag(
                        ui,
                        scroll_id,
                        ITEM_SELECTOR_SIZE,
                        rect,
                        ITEM_SELECTOR_COLS as usize,
                        ITEM_SELECTOR_ROWS as usize,
                        &response,
                    );
                    ui.data_mut(|d| d.insert_temp(hovered_idx_id, hovered_idx));

                    if response.clicked()
                        && let Some(idx) = hovered_idx
                        && let Ok(new_item_type) = ItemType::from_idx(idx)
                        && *self != new_item_type
                    {
                        *self = new_item_type;
                        resp.mark_changed();
                    }

                    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                        rect,
                        ItemGridCallback {
                            hovered_index: hovered_idx.map(|i| i as u32),
                            selected_index: Some(self.to_idx() as u32),
                            viewport,
                            pixels_per_point: ui.pixels_per_point(),
                            id: ui.id(),
                            instances: ItemGridInstances::Items,
                        },
                    ));

                    ui.set_max_height(text_h + rect.height());
                });

            if !keep_open {
                ui.data_mut(|d| d.insert_temp(window_id, false));
            }
        }

        resp
    }
}

impl ToRow for TorchConnectionType {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        wrap_combo_box_resp(
            egui::ComboBox::from_id_salt("torch_connection_type_combo_box")
                .selected_text(format!("{:?}", self))
                .show_ui(ui, |ui| {
                    ui.selectable_value(self, Self::Bg, "Background")
                        | ui.selectable_value(self, Self::Left, "Left")
                        | ui.selectable_value(self, Self::Ground, "Ground")
                        | ui.selectable_value(self, Self::Right, "Right")
                        | ui.selectable_value(self, Self::Mg, "Middleground")
                }),
        )
    }
}

impl ToGrid for Torch {
    fn to_grid(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        grid_as_row(
            &mut self.light_dict,
            "lightDict",
            "light_dict_grid",
            ui,
            context,
        );
        if self.connection_type.add_row("connectionType", ui).changed() {
            context.flags |= ObjFlags::RebuildMesh;
        }
        self.item_type.add_row("itemType", ui);
        self.data_a.add_row("dataA", ui);
        self.data_b.add_row("dataB", ui);
    }
}

impl InfoUi for Torch {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let obj = self.deref_mut();
        obj.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("Torch");
            ui.separator();
            self.add_grid("torch_grid", ui, context);
        });
    }
}

struct FrontTorchFace {
    bottom_left: [f32; 3],
    uv_min_max: [[f32; 2]; 2],
}

impl FrontTorchFace {
    const THETA: f32 = 0.3;
}

impl DwQuad for FrontTorchFace {
    fn quad(&self) -> [[f32; 3]; 4] {
        let [x, y, z] = self.bottom_left;
        let [w, h] = [1.0; 2];
        [
            [x, y, z],
            [x + w, y, z],
            [x + w, y + h * Self::THETA.cos(), z + h * Self::THETA.sin()],
            [x, y + h * Self::THETA.cos(), z + h * Self::THETA.sin()],
        ]
    }

    fn normal(&self) -> [f32; 3] {
        [0.0, -Self::THETA.sin(), Self::THETA.cos()]
    }

    fn uv_min_max(&self) -> [[f32; 2]; 2] {
        self.uv_min_max
    }
}

struct RotatedTorchFace {
    bottom_left: [f32; 3],
    uv_min_max: [[f32; 2]; 2],
    theta: f32,
}

impl RotatedTorchFace {
    const THETA: f32 = 0.6;
}

impl DwQuad for RotatedTorchFace {
    fn quad(&self) -> [[f32; 3]; 4] {
        let [b, l, z] = self.bottom_left;
        let [w, h] = [1.0; 2];
        let [w_2, h_2] = [w / 2.0, h / 2.0];
        let rotation_mat = glam::Mat2::from_angle(self.theta);
        [[-w_2, -h_2], [w_2, -h_2], [w_2, h_2], [-w_2, h_2]].map(|[x, y]| {
            // rotated radius
            let [rw_2, rh_2] = rotation_mat.mul_vec2(glam::Vec2::new(x, y)).to_array();
            [b + rw_2, l + rh_2, z]
        })
    }

    fn normal(&self) -> [f32; 3] {
        [0.0, 0.0, 1.0]
    }

    fn uv_min_max(&self) -> [[f32; 2]; 2] {
        self.uv_min_max
    }
}

impl BuildDwMesh for Torch {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 0, quads: 1 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        let image_type = match self.item_type {
            ItemType::Torch => ImageType::BasicTorch0,
            ItemType::IceTorch => ImageType::IceTorch0,
            ItemType::OilLantern => ImageType::ClayLantern0,
            ItemType::SteelLantern => ImageType::SteelLantern0,
            ItemType::SteelDownlight => ImageType::SteelDownlight,
            ItemType::SteelUplight => ImageType::SteelUplight,
            _ => InvalidItemTypeForTorchSnafu {
                item_type: self.item_type,
                torch: self.clone(),
            }
            .fail()?,
        };
        let [x, y] = self.float_pos;
        match self.connection_type {
            TorchConnectionType::Bg => {
                builder.add_quad(FrontTorchFace {
                    bottom_left: [x - 0.5, y, 1.0],
                    uv_min_max: image_type.uv_min_max(1, 1),
                });
            }
            TorchConnectionType::Left => {
                builder.add_quad(RotatedTorchFace {
                    bottom_left: [
                        x - (1.0 - RotatedTorchFace::THETA.sin()) / 2.0,
                        y + 0.5,
                        2.0,
                    ],
                    uv_min_max: image_type.uv_min_max(1, 1),
                    theta: -RotatedTorchFace::THETA,
                });
            }
            TorchConnectionType::Ground => {
                builder.add_face(DwFace::new_sprite(
                    image_type,
                    [0.5, 0.0],
                    self.float_pos,
                    [1, 1],
                    2.0,
                ));
            }
            TorchConnectionType::Right => {
                builder.add_quad(RotatedTorchFace {
                    bottom_left: [
                        x + (1.0 - RotatedTorchFace::THETA.sin()) / 2.0,
                        y + 0.5,
                        2.0,
                    ],
                    uv_min_max: image_type.uv_min_max(1, 1),
                    theta: RotatedTorchFace::THETA,
                });
            }
            TorchConnectionType::Mg => {
                builder.add_quad(FrontTorchFace {
                    bottom_left: [x - 0.5, y, 2.0],
                    uv_min_max: image_type.uv_min_max(1, 1),
                });
            }
        }
        Ok(())
    }
}

impl ToGrid for Ladder {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut DwUiContext) {
        self.paint_color.add_row("paintColor", ui);
        self.item_type.add_row("itemType", ui);
    }
}

impl InfoUi for Ladder {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let obj = self.deref_mut();
        obj.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("Ladder");
            ui.separator();
            self.add_grid("ladder_grid", ui, context);
        });
    }
}

impl BuildDwMesh for Ladder {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 0, quads: 1 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_face(DwFace::new_sprite(
            ImageType::Ladder,
            [0.5, 0.0],
            self.float_pos,
            [1, 1],
            2.0,
        ));
        Ok(())
    }
}

impl ToGrid for Door {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut DwUiContext) {
        self.item_type.add_row("itemType", ui);
        self.blocked.add_row("blocked", ui);
        self.iron_place_client_id.add_row("ironPlaceClientId", ui);
    }
}

impl InfoUi for Door {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let obj = self.deref_mut();
        obj.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("Door");
            ui.separator();
            self.add_grid("door_grid", ui, context);
        });
    }
}

impl BuildDwMesh for Door {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 0, quads: 2 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        let [x, y] = self.float_pos;
        let z = 2.0;
        let image_type = match self.item_type {
            ItemType::Door => ImageType::Door,
            ItemType::IronDoor => ImageType::IronDoor,
            ItemType::Trapdoor => ImageType::DoorTop,
            ItemType::IronTrapdoor => ImageType::IronDoorTop,
            _ => InvalidItemTypeForDoorSnafu {
                item_type: self.item_type,
                door: self.clone(),
            }
            .fail()?,
        };
        match self.item_type {
            ItemType::Door | ItemType::IronDoor => {
                builder.add_face(DwFace::from_tile_map(
                    image_type,
                    FaceDirection::Left,
                    [x - 0.5, y, z],
                    [1, 2],
                ));
                builder.add_face(DwFace::from_tile_map(
                    image_type,
                    FaceDirection::Right,
                    [x + 0.5, y, z],
                    [1, 2],
                ));
            }
            ItemType::Trapdoor | ItemType::IronTrapdoor => {
                builder.add_face(DwFace::from_tile_map(
                    image_type,
                    FaceDirection::Up,
                    [x - 0.5, y + 1.0, z + 1.0],
                    [1, 1],
                ));
                builder.add_face(DwFace::from_tile_map(
                    image_type,
                    FaceDirection::Down,
                    [x - 0.5, y, z + 1.0],
                    [1, 1],
                ));
            }
            // SAFETY: previous match have failed this arm
            _ => unreachable!(),
        }
        Ok(())
    }
}

impl InfoUi for CarrotPlant {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui, context);
    }
}

impl BuildDwMesh for CarrotPlant {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 0, quads: 1 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_face(DwFace::new_sprite(
            if self.flowering {
                ImageType::CarrotFlower
            } else {
                ImageType::CarrotPlant
            },
            [0.5, 0.0],
            self.float_pos,
            [1, 2],
            2.0,
        ));
        Ok(())
    }
}

impl ToRow for DodoBreed {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        wrap_combo_box_resp(
            egui::ComboBox::from_id_salt("dodo_breed_combo_box")
                .selected_text(format!("{:?}", self))
                .show_ui(ui, |ui| {
                    ui.selectable_value(self, Self::Standard, "Standard")
                        | ui.selectable_value(self, Self::Stone, "Stone")
                        | ui.selectable_value(self, Self::Limestone, "Limestone")
                        | ui.selectable_value(self, Self::Sandstone, "Sandstone")
                        | ui.selectable_value(self, Self::Marble, "Marble")
                        | ui.selectable_value(self, Self::RedMarble, "RedMarble")
                        | ui.selectable_value(self, Self::Lapis, "Lapis")
                        | ui.selectable_value(self, Self::Dirt, "Dirt")
                        | ui.selectable_value(self, Self::Compost, "Compost")
                        | ui.selectable_value(self, Self::Wood, "Wood")
                        | ui.selectable_value(self, Self::Gravel, "Gravel")
                        | ui.selectable_value(self, Self::Sand, "Sand")
                        | ui.selectable_value(self, Self::BlackSand, "BlackSand")
                        | ui.selectable_value(self, Self::Glass, "Glass")
                        | ui.selectable_value(self, Self::BlackGlass, "BlackGlass")
                        | ui.selectable_value(self, Self::Clay, "Clay")
                        | ui.selectable_value(self, Self::RedBrick, "RedBrick")
                        | ui.selectable_value(self, Self::Flint, "Flint")
                        | ui.selectable_value(self, Self::Coal, "Coal")
                        | ui.selectable_value(self, Self::Oil, "Oil")
                        | ui.selectable_value(self, Self::Fuel, "Fuel")
                        | ui.selectable_value(self, Self::Copper, "Copper")
                        | ui.selectable_value(self, Self::Tin, "Tin")
                        | ui.selectable_value(self, Self::Iron, "Iron")
                        | ui.selectable_value(self, Self::Gold, "Gold")
                        | ui.selectable_value(self, Self::Titanium, "Titanium")
                        | ui.selectable_value(self, Self::Platinum, "Platinum")
                        | ui.selectable_value(self, Self::Amethyst, "Amethyst")
                        | ui.selectable_value(self, Self::Sapphire, "Sapphire")
                        | ui.selectable_value(self, Self::Emerald, "Emerald")
                        | ui.selectable_value(self, Self::Ruby, "Ruby")
                        | ui.selectable_value(self, Self::Diamond, "Diamond")
                        | ui.selectable_value(self, Self::Rainbow, "Rainbow")
                }),
        )
    }
}

impl ToGrid for Egg {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut DwUiContext) {
        self.genes_dict.breed.add_row("breed", ui);
        self.hatch_timer.add_row("hatchTimer", ui);
        self.save_time.add_row("saveTime", ui);
    }
}

impl InfoUi for Egg {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let obj = self.deref_mut();
        obj.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("Egg");
            ui.separator();
            self.add_grid("egg_grid", ui, context);
        });
    }
}

impl BuildDwMesh for Egg {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        // TODO add render egg with real breed textures
        builder.add_item(DwItem::from_item_type(self.float_pos, ItemType::DodoEgg));
        Ok(())
    }
}

impl InfoUi for ChilliPlant {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui, context);
    }
}

impl BuildDwMesh for ChilliPlant {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 0, quads: 1 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_face(DwFace::new_sprite(
            if self.flowering {
                ImageType::ChilliPlantFlower
            } else {
                ImageType::ChilliPlant
            },
            [0.5, 0.0],
            self.float_pos,
            [1, 2],
            2.0,
        ));
        Ok(())
    }
}

impl ToGrid for KelpPlant {
    fn to_grid(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        self.growth_timer.add_row("growthTimer", ui);
        if self
            .number_of_occupied_tiles_above
            .add_row("numberOfOccupiedTilesAbove", ui)
            .changed()
        {
            context.flags |= ObjFlags::RebuildMesh;
        }
    }
}

impl InfoUi for KelpPlant {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("KelpPlant");
            ui.separator();
            self.add_grid("kelp_plant_grid", ui, context);
        });
    }
}

impl BuildDwMesh for KelpPlant {
    fn capacity(&self) -> DwCapacity {
        let len = self.number_of_occupied_tiles_above as usize + 1;
        DwCapacity {
            items: 0,
            quads: if len == 0 { 0 } else { len / 2 + 1 },
        }
    }

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        let mut len = self.number_of_occupied_tiles_above + 1;
        let [x, mut y] = self.float_pos;
        while len >= 3 {
            builder.add_face(DwFace::new_sprite(
                ImageType::KelpPlant,
                [0.5, 0.0],
                [x, y],
                [1, 2],
                2.0,
            ));
            y += 2.0;
            len -= 2;
        }
        match len {
            1 => {
                builder.add_face(DwFace::new_sprite(
                    ImageType::KelpPlantOddLenTop,
                    [0.5, 0.0],
                    [x, y],
                    [1, 1],
                    2.0,
                ));
            }
            2 => {
                builder.add_face(DwFace::new_sprite(
                    ImageType::KelpPlant,
                    [0.5, 0.0],
                    [x, y],
                    [1, 1],
                    2.0,
                ));
                builder.add_face(DwFace::new_sprite(
                    ImageType::KelpPlantEvenLenTop,
                    [0.5, 0.0],
                    [x, y + 1.0],
                    [1, 1],
                    2.0,
                ));
            }
            _ => {}
        }
        Ok(())
    }
}

impl InfoUi for LimeTree {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let tree = self.deref_mut();
        tree.info(ui, context);
    }
}

impl BuildDwMesh for LimeTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_item(DwItem::from_item_type(self.float_pos, ItemType::Lime));
        Ok(())
    }
}

impl ToRow for WireConfiguration {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        wrap_combo_box_resp(
            egui::ComboBox::from_id_salt("wire_configuration_combo_box")
                .selected_text(format!("{:?}", self))
                .show_ui(ui, |ui| {
                    ui.selectable_value(self, Self::Undefined, "Undefined")
                        | ui.selectable_value(self, Self::AllConnections, "AllConnections")
                        | ui.selectable_value(self, Self::NoConnections, "NoConnections")
                        | ui.selectable_value(self, Self::AboveBelowOnly, "AboveBelowOnly")
                        | ui.selectable_value(self, Self::AboveBelowLeft, "AboveBelowLeft")
                        | ui.selectable_value(self, Self::AboveBelowRight, "AboveBelowRight")
                        | ui.selectable_value(self, Self::LeftRightOnly, "LeftRightOnly")
                        | ui.selectable_value(self, Self::LeftRightUp, "LeftRightUp")
                        | ui.selectable_value(self, Self::LeftRightDown, "LeftRightDown")
                        | ui.selectable_value(self, Self::LeftDown, "LeftDown")
                        | ui.selectable_value(self, Self::LeftUp, "LeftUp")
                        | ui.selectable_value(self, Self::RightDown, "RightDown")
                        | ui.selectable_value(self, Self::RightUp, "RightUp")
                }),
        )
    }
}

impl ToRow for WireSolidConfiguration {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        wrap_combo_box_resp(
            egui::ComboBox::from_id_salt("wire_solid_configuration_combo_box")
                .selected_text(format!("{:?}", self))
                .show_ui(ui, |ui| {
                    ui.selectable_value(self, Self::Undefined, "Undefined")
                        | ui.selectable_value(self, Self::NotSolid, "NotSolid")
                        | ui.selectable_value(self, Self::AllConnections, "AllConnections")
                        | ui.selectable_value(self, Self::ThisTileOnly, "ThisTileOnly")
                        | ui.selectable_value(self, Self::AboveBelowOnly, "AboveBelowOnly")
                        | ui.selectable_value(self, Self::AboveBelowLeft, "AboveBelowLeft")
                        | ui.selectable_value(self, Self::AboveBelowRight, "AboveBelowRight")
                        | ui.selectable_value(self, Self::LeftRightOnly, "LeftRightOnly")
                        | ui.selectable_value(self, Self::LeftOnly, "LeftOnly")
                        | ui.selectable_value(self, Self::LeftRightUp, "LeftRightUp")
                        | ui.selectable_value(self, Self::LeftRightDown, "LeftRightDown")
                        | ui.selectable_value(self, Self::LeftDown, "LeftDown")
                        | ui.selectable_value(self, Self::LeftUp, "LeftUp")
                        | ui.selectable_value(self, Self::RightDown, "RightDown")
                        | ui.selectable_value(self, Self::RightUp, "RightUp")
                        | ui.selectable_value(self, Self::RightOnly, "RightOnly")
                        | ui.selectable_value(self, Self::UpOnly, "UpOnly")
                        | ui.selectable_value(self, Self::DownOnly, "DownOnly")
                }),
        )
    }
}

impl ToGrid for Wire {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut DwUiContext) {
        self.item_type.add_row("itemType", ui);
        self.configuration.add_row("configuration", ui);
        self.solid_configuration.add_row("solidConfiguration", ui);
    }
}

impl InfoUi for Wire {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let obj = self.deref_mut();
        obj.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("Wire");
            ui.separator();
            self.add_grid("wire_grid", ui, context);
        });
    }
}

impl BuildDwMesh for Wire {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_item(DwItem::from_item_type(self.float_pos, ItemType::CopperWire));
        Ok(())
    }
}

impl ToRow for InteractionObjectType {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        wrap_combo_box_resp(
            egui::ComboBox::from_id_salt("interaction_object_type_combo_box")
                .selected_text(format!("{:?}", self))
                .show_ui(ui, |ui| {
                    ui.selectable_value(self, Self::InteractionObject, "InteractionObject")
                        | ui.selectable_value(self, Self::Workbench, "Workbench")
                        | ui.selectable_value(self, Self::Chest, "Chest")
                        | ui.selectable_value(self, Self::Bed, "Bed")
                        | ui.selectable_value(self, Self::Sign, "Sign")
                        | ui.selectable_value(self, Self::TradingPost, "TradingPost")
                        | ui.selectable_value(self, Self::TrainStation, "TrainStation")
                        | ui.selectable_value(self, Self::TradePortal, "TradePortal")
                        | ui.selectable_value(self, Self::OwnershipSign, "OwnershipSign")
                        | ui.selectable_value(self, Self::Mirror, "Mirror")
                }),
        )
    }
}

impl ToGrid for InteractionObject {
    fn to_grid(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        self.interaction_object_type
            .add_row("interactionObjectType", ui);
        self.is_in_use.add_row("isInUse", ui);
        if self.flipped.add_row("flipped", ui).changed() {
            context.flags |= ObjFlags::RebuildMesh;
        }
        self.paint_color.add_row("paintColor", ui);
    }
}

impl InfoUi for InteractionObject {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let obj = self.deref_mut();
        obj.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("InteractionObject");
            ui.separator();
            self.add_grid("interaction_object_grid", ui, context);
        });
    }
}

impl ToRow for WorkbenchType {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        wrap_combo_box_resp(
            egui::ComboBox::from_id_salt("workbench_type_combo_box")
                .selected_text(format!("{:?}", self))
                .show_ui(ui, |ui| {
                    ui.selectable_value(self, Self::Undefined, "Undefined")
                        | ui.selectable_value(self, Self::BasicPortal, "BasicPortal")
                        | ui.selectable_value(self, Self::Workbench, "Workbench")
                        | ui.selectable_value(self, Self::Campfire, "Campfire")
                        | ui.selectable_value(self, Self::Weave, "Weave")
                        | ui.selectable_value(self, Self::Wood, "Wood")
                        | ui.selectable_value(self, Self::Tool, "Tool")
                        | ui.selectable_value(self, Self::Press, "Press")
                        | ui.selectable_value(self, Self::Kiln, "Kiln")
                        | ui.selectable_value(self, Self::Furnace, "Furnace")
                        | ui.selectable_value(self, Self::Craft, "Craft")
                        | ui.selectable_value(self, Self::Mix, "Mix")
                        | ui.selectable_value(self, Self::Dye, "Dye")
                        | ui.selectable_value(self, Self::PlacedPortal, "PlacedPortal")
                        | ui.selectable_value(self, Self::Metalwork, "Metalwork")
                        | ui.selectable_value(self, Self::SteamGenerator, "SteamGenerator")
                        | ui.selectable_value(self, Self::ElectricKiln, "ElectricKiln")
                        | ui.selectable_value(self, Self::ElectricFurnace, "ElectricFurnace")
                        | ui.selectable_value(
                            self,
                            Self::ElectricMetalworkBench,
                            "ElectricMetalworkBench",
                        )
                        | ui.selectable_value(self, Self::ElectricStove, "ElectricStove")
                        | ui.selectable_value(self, Self::SolarPanel, "SolarPanel")
                        | ui.selectable_value(self, Self::Flywheel, "Flywheel")
                        | ui.selectable_value(self, Self::ArmorBench, "ArmorBench")
                        | ui.selectable_value(self, Self::TrainYard, "TrainYard")
                        | ui.selectable_value(self, Self::Easel, "Easel")
                        | ui.selectable_value(self, Self::Build, "Build")
                        | ui.selectable_value(self, Self::Refinery, "Refinery")
                        | ui.selectable_value(self, Self::ElectricPress, "ElectricPress")
                        | ui.selectable_value(self, Self::CompostBin, "CompostBin")
                        | ui.selectable_value(self, Self::Sluice, "Sluice")
                        | ui.selectable_value(self, Self::EggExtractor, "EggExtractor")
                        | ui.selectable_value(self, Self::PizzaOven, "PizzaOven")
                }),
        )
    }
}

impl ToGrid for Workbench {
    fn to_grid(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        self.available_electricity
            .add_row("availableElectricity", ui);
        self.craft_progress_count.add_row("craftProgressCount", ui);
        self.fire_spread_timer.add_row("fireSpreadTimer", ui);
        self.fuel_fraction.add_row("fuelFraction", ui);
        self.has_fuel.add_row("hasFuel", ui);
        self.hurry_cost.add_row("hurryCost", ui);
        self.hurry_seconds.add_row("hurrySeconds", ui);
        self.hurry_timer.add_row("hurryTimer", ui);
        self.hurrying.add_row("hurrying", ui);
        self.last_world_time.add_row("lastWorldTime", ui);
        if self.level.add_row("level", ui).changed() {
            context.flags |= ObjFlags::RebuildMesh;
        }
        self.save_time.add_row("saveTime", ui);
        self.selected_index.add_row("selectedIndex", ui);
        if self.workbench_type.add_row("workbenchType", ui).changed() {
            context.flags |= ObjFlags::RebuildMesh;
        }
        self.x_scroll.add_row("xScroll", ui);
        grid_as_row(
            &mut self.light_dict,
            "lightDict",
            "light_dict_grid",
            ui,
            context,
        );
    }
}

impl InfoUi for Workbench {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let obj = self.deref_mut();
        obj.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("Workbench");
            ui.separator();
            self.add_grid("workbench_grid", ui, context);
        });
    }
}

impl BuildDwMesh for Workbench {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity {
        items: 0,
        quads: CUBE_NUM_FACES,
    });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        use BlockUv::*;
        use ImageType::*;
        let block_coord = BlockCoord::new(self.pos_x, self.pos_y).context(CoordOutOfBoundSnafu)?;
        match self.workbench_type {
            WorkbenchType::Undefined => {
                builder.add_item(DwItem::from_item_type(self.float_pos, ItemType::Unknown))
            }
            WorkbenchType::BasicPortal | WorkbenchType::PlacedPortal => builder.add_face(
                DwFace::new_sprite(ImageType::Portal0, [0.5, 0.0], self.float_pos, [1, 2], 2.0),
            ),
            WorkbenchType::Workbench => builder.add_block(DwBlock::new(
                block_coord,
                match self.level {
                    0 => TopSide {
                        top: WorkbenchLevel1Top,
                        side: WorkbenchLevel1,
                    },
                    1 => TopSide {
                        top: WorkbenchLevel2Top,
                        side: WorkbenchLevel2,
                    },
                    2 => TopSide {
                        top: WorkbenchLevel3Top,
                        side: WorkbenchLevel3,
                    },
                    3 => TopSide {
                        top: WorkbenchLevel4Top,
                        side: WorkbenchLevel4,
                    },
                    _ => InvalidWorkbenchLevelSnafu {
                        workbench_type: self.workbench_type,
                        level: self.level,
                        maximum: 4,
                    }
                    .fail()?,
                },
            )),
            WorkbenchType::Campfire => builder.add_face(DwFace::new_sprite(
                ImageType::Campfire0,
                [0.5, 0.0],
                self.float_pos,
                [1, 1],
                2.0,
            )),
            WorkbenchType::Weave => builder.add_block(DwBlock::new(
                block_coord,
                match self.level {
                    0 => TopSide {
                        top: WorkbenchWeave1Top,
                        side: WorkbenchWeave1,
                    },
                    1 => TopSide {
                        top: WorkbenchWeave2Top,
                        side: WorkbenchWeave2,
                    },
                    _ => InvalidWorkbenchLevelSnafu {
                        workbench_type: self.workbench_type,
                        level: self.level,
                        maximum: 2,
                    }
                    .fail()?,
                },
            )),
            WorkbenchType::Wood => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: WorkbenchWood1Top,
                    side: WorkbenchWood1,
                },
            )),
            WorkbenchType::Tool => builder.add_block(DwBlock::new(
                block_coord,
                match self.level {
                    0 => TopSide {
                        top: WorkbenchTool1Top,
                        side: WorkbenchTool1,
                    },
                    1 => TopSide {
                        top: WorkbenchTool2Top,
                        side: WorkbenchTool2,
                    },
                    2 => TopSide {
                        top: WorkbenchTool3Top,
                        side: WorkbenchTool3,
                    },
                    3 => TopSide {
                        top: WorkbenchTool4Top,
                        side: WorkbenchTool4,
                    },
                    4 => TopSide {
                        top: WorkbenchTool5Top,
                        side: WorkbenchTool5,
                    },
                    5 => TopSide {
                        top: WorkbenchTool6Top,
                        side: WorkbenchTool6,
                    },
                    6 => TopSide {
                        top: WorkbenchTool7Top,
                        side: WorkbenchTool7,
                    },
                    _ => InvalidWorkbenchLevelSnafu {
                        workbench_type: self.workbench_type,
                        level: self.level,
                        maximum: 7,
                    }
                    .fail()?,
                },
            )),
            WorkbenchType::Press => builder.add_block(DwBlock::new(
                block_coord,
                match self.level {
                    0 => TopSide {
                        top: WorkbenchPress1Top,
                        side: WorkbenchPress1,
                    },
                    1 => TopSide {
                        top: WorkbenchPress2Top,
                        side: WorkbenchPress2,
                    },
                    _ => InvalidWorkbenchLevelSnafu {
                        workbench_type: self.workbench_type,
                        level: self.level,
                        maximum: 2,
                    }
                    .fail()?,
                },
            )),
            WorkbenchType::Kiln => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: KilnTop,
                    side: Kiln,
                },
            )),
            WorkbenchType::Furnace => builder.add_block(DwBlock::new(
                block_coord,
                match self.level {
                    0 => TopSide {
                        top: Furnace1Top,
                        side: Furnace1,
                    },
                    1 => TopSide {
                        top: Furnace2Top,
                        side: Furnace2,
                    },
                    2 => TopSide {
                        top: Furnace3Top,
                        side: Furnace3,
                    },
                    _ => InvalidWorkbenchLevelSnafu {
                        workbench_type: self.workbench_type,
                        level: self.level,
                        maximum: 3,
                    }
                    .fail()?,
                },
            )),
            WorkbenchType::Craft => builder.add_block(DwBlock::new(
                block_coord,
                match self.level {
                    0 => TopSide {
                        top: CraftBenchLevel1Top,
                        side: CraftBenchLevel1,
                    },
                    1 => TopSide {
                        top: CraftBenchLevel2Top,
                        side: CraftBenchLevel2,
                    },
                    2 => TopSide {
                        top: CraftBenchLevel3Top,
                        side: CraftBenchLevel3,
                    },
                    3 => TopSide {
                        top: CraftBenchLevel4Top,
                        side: CraftBenchLevel4,
                    },
                    _ => InvalidWorkbenchLevelSnafu {
                        workbench_type: self.workbench_type,
                        level: self.level,
                        maximum: 4,
                    }
                    .fail()?,
                },
            )),
            WorkbenchType::Mix => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: MixBenchLevel1Top,
                    side: MixBenchLevel1,
                },
            )),
            WorkbenchType::Dye => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: DyeBenchLevel1Top,
                    side: DyeBenchLevel1,
                },
            )),
            WorkbenchType::Metalwork => builder.add_block(DwBlock::new(
                block_coord,
                match self.level {
                    0 => TopSide {
                        top: MetalworkBenchLevel1Top,
                        side: MetalworkBenchLevel1,
                    },
                    1 => TopSide {
                        top: MetalworkBenchLevel2Top,
                        side: MetalworkBenchLevel2,
                    },
                    _ => InvalidWorkbenchLevelSnafu {
                        workbench_type: self.workbench_type,
                        level: self.level,
                        maximum: 2,
                    }
                    .fail()?,
                },
            )),
            WorkbenchType::SteamGenerator => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: SteamGeneratorTop,
                    side: SteamGenerator,
                },
            )),
            WorkbenchType::ElectricKiln => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: ElectricKilnTop,
                    side: ElectricKiln,
                },
            )),
            WorkbenchType::ElectricFurnace => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: ElectricFurnaceTop,
                    side: ElectricFurnace,
                },
            )),
            WorkbenchType::ElectricMetalworkBench => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: ElectricMetalworkBenchTop,
                    side: ElectricMetalworkBench,
                },
            )),
            WorkbenchType::ElectricStove => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: ElectricStoveTop,
                    side: ElectricStove,
                },
            )),
            WorkbenchType::SolarPanel => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: SolarPanelTop,
                    side: SolarPanel,
                },
            )),
            WorkbenchType::Flywheel => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: FlywheelTop,
                    side: Flywheel,
                },
            )),
            WorkbenchType::ArmorBench => builder.add_block(DwBlock::new(
                block_coord,
                match self.level {
                    0 => TopSide {
                        top: ArmorBenchLevel1Top,
                        side: ArmorBenchLevel1,
                    },
                    1 => TopSide {
                        top: ArmorBenchLevel2Top,
                        side: ArmorBenchLevel2,
                    },
                    2 => TopSide {
                        top: ArmorBenchLevel3Top,
                        side: ArmorBenchLevel3,
                    },
                    _ => InvalidWorkbenchLevelSnafu {
                        workbench_type: self.workbench_type,
                        level: self.level,
                        maximum: 3,
                    }
                    .fail()?,
                },
            )),
            WorkbenchType::TrainYard => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: TrainYardTop,
                    side: TrainYard,
                },
            )),
            WorkbenchType::Easel => {
                builder.add_item(DwItem::from_item_type(self.float_pos, ItemType::Easel))
            }
            WorkbenchType::Build => builder.add_block(DwBlock::new(
                block_coord,
                match self.level {
                    0 => TopSide {
                        top: BuildersBenchLevel1Top,
                        side: BuildersBenchLevel1,
                    },
                    1 => TopSide {
                        top: BuildersBenchLevel2Top,
                        side: BuildersBenchLevel2,
                    },
                    _ => InvalidWorkbenchLevelSnafu {
                        workbench_type: self.workbench_type,
                        level: self.level,
                        maximum: 2,
                    }
                    .fail()?,
                },
            )),
            WorkbenchType::Refinery => {
                builder.add_item(DwItem::from_item_type(self.float_pos, ItemType::Refinery))
            }
            WorkbenchType::ElectricPress => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: ElectricPressTop,
                    side: ElectricPress,
                },
            )),
            WorkbenchType::CompostBin => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: CompostBinTop,
                    side: CompostBin,
                },
            )),
            WorkbenchType::Sluice => builder.add_item(DwItem::from_item_type(
                self.float_pos,
                ItemType::ElectricSluice,
            )),
            WorkbenchType::EggExtractor => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: EggExtractorTop,
                    side: EggExtractor,
                },
            )),
            WorkbenchType::PizzaOven => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: PizzaOvenTop,
                    side: PizzaOven,
                },
            )),
        }
        Ok(())
    }
}

fn slots_to_item_instances<'a, I: Iterator<Item = &'a Slot>>(
    slots: I,
    num_col: usize,
) -> Vec<DwItemInstanceRaw> {
    slots
        .enumerate()
        .map(|(i, slot)| {
            let col = (i % num_col) as u32;
            let row = (i / num_col) as u32;

            if let Some(first_item) = slot.first()
                && let Ok(dw_item) = DwItem::from_item([col, row].map(|v| v as f32), first_item)
            {
                dw_item.grid_instance(i as u32)
            } else {
                DwItemInstanceRaw::empty(col, row, i as u32)
            }
        })
        .collect()
}

fn toggle_selected_index(
    ui: &mut egui::Ui,
    selected_idx_id: egui::Id,
    hovered_idx: Option<usize>,
    response: &egui::Response,
) -> Option<usize> {
    let mut selected_idx: Option<usize> =
        ui.data_mut(|d| d.get_temp::<Option<usize>>(selected_idx_id).flatten());
    if response.clicked() {
        // toggle
        if selected_idx == hovered_idx {
            selected_idx = None;
        } else {
            selected_idx = hovered_idx;
        }
    }
    ui.data_mut(|d| d.insert_temp(selected_idx_id, selected_idx));
    selected_idx
}

impl ToGrid for Item {
    fn to_grid(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        if self.type_id.add_row("typeId", ui).changed() {
            context.flags |= ObjFlags::RebuildMesh;
        }
        let item_type = self.item_type();
        match item_type {
            Ok(mut item_type) => {
                if item_type.add_row("itemType", ui).changed() {
                    self.set_item_type(item_type);
                    context.flags |= ObjFlags::RebuildMesh;
                }
            }
            Err(e) => {
                ui.weak(e.to_string());
                ui.end_row();
            }
        }
        self.data_a.add_row("dataA", ui);
        self.data_b.add_row("dataB", ui);
        self.selected_sub_item_index
            .add_row("selectedSubItemIndex", ui);

        ui.label("subItems");
        if let Some(slots) = self.sub_items.as_mut() {
            ui.vertical(|ui| {
                let id = ui.id().with("chest_item_rows");
                let scroll_id = id.with("scroll_offset");
                let selected_idx_id = id.with("selected_idx");

                let (num_col, num_row) = (Self::MAX_SUB_ITEMS, 1usize);
                let size = egui::Vec2::new(COL_PX * num_col as f32, ROW_PX * num_row as f32);
                let (rect, response) = ui.allocate_exact_size(size, egui::Sense::all());
                let rect = rect.intersect(ui.ctx().content_rect());
                let ItemGridResult {
                    hovered_idx,
                    viewport,
                } = handle_item_grid_drag(ui, scroll_id, size, rect, num_col, num_row, &response);

                let selected_idx =
                    toggle_selected_index(ui, selected_idx_id, hovered_idx, &response);

                let instances = slots_to_item_instances(slots.iter(), num_col);

                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                    rect,
                    ItemGridCallback {
                        hovered_index: hovered_idx.map(|i| i as u32),
                        selected_index: selected_idx.map(|i| i as u32),
                        viewport,
                        pixels_per_point: ui.pixels_per_point(),
                        id: ui.id(),
                        instances: ItemGridInstances::Custom { instances },
                    },
                ));

                if let Some(idx) = selected_idx
                    && let Some(slot) = slots.get_mut(idx)
                {
                    let items = slot.deref_mut();
                    items.add_grid(id.with("selected_slot_items_grid"), ui, context);
                }
            });
        } else {
            ui.weak("No subItems");
        }
        ui.end_row();

        ui.label("dynamicObject");
        ui.vertical(|ui| {
            let current_variant_name = match &self.dynamic_object {
                None => "None",
                Some(obj) => obj.into(),
            };

            egui::ComboBox::from_id_salt("dyn_obj_combo")
                .selected_text(current_variant_name)
                .show_ui(ui, |ui| {
                    let mut changed = false;

                    let mut selectable_variant =
                        |ui: &mut egui::Ui,
                         name: &str,
                         is_selected: bool,
                         new_variant: fn() -> Option<AnyDynamicObject>,
                         new_item_type: Option<ItemType>,
                         item: &mut Item| {
                            if ui.selectable_label(is_selected, name).clicked() {
                                item.dynamic_object = new_variant();
                                if let Some(new_item_type) = new_item_type {
                                    item.set_item_type(new_item_type);
                                }
                                changed = true;
                            }
                        };

                    selectable_variant(
                        ui,
                        "None",
                        self.dynamic_object.is_none(),
                        || None,
                        None,
                        self,
                    );
                    selectable_variant(
                        ui,
                        "Ladder",
                        matches!(self.dynamic_object, Some(AnyDynamicObject::Ladder(_))),
                        || Some(AnyDynamicObject::Ladder(Box::default())),
                        Some(ItemType::Ladder),
                        self,
                    );
                    selectable_variant(
                        ui,
                        "Door",
                        matches!(self.dynamic_object, Some(AnyDynamicObject::Door(_))),
                        || Some(AnyDynamicObject::Door(Box::default())),
                        Some(ItemType::Door),
                        self,
                    );
                    selectable_variant(
                        ui,
                        "Bed",
                        matches!(self.dynamic_object, Some(AnyDynamicObject::Bed(_))),
                        || Some(AnyDynamicObject::Bed(Box::default())),
                        Some(ItemType::Bed),
                        self,
                    );
                    selectable_variant(
                        ui,
                        "Egg",
                        matches!(self.dynamic_object, Some(AnyDynamicObject::Egg(_))),
                        || Some(AnyDynamicObject::Egg(Box::default())),
                        Some(ItemType::DodoEgg),
                        self,
                    );
                    selectable_variant(
                        ui,
                        "Workbench",
                        matches!(self.dynamic_object, Some(AnyDynamicObject::Workbench(_))),
                        || Some(AnyDynamicObject::Workbench(Box::default())),
                        Some(ItemType::WorkBench),
                        self,
                    );
                    selectable_variant(
                        ui,
                        "Chest",
                        matches!(self.dynamic_object, Some(AnyDynamicObject::Chest(_))),
                        || Some(AnyDynamicObject::Chest(Box::default())),
                        Some(ItemType::Chest),
                        self,
                    );
                    selectable_variant(
                        ui,
                        "Sign",
                        matches!(self.dynamic_object, Some(AnyDynamicObject::Sign(_))),
                        || Some(AnyDynamicObject::Sign(Box::default())),
                        Some(ItemType::Sign),
                        self,
                    );
                    selectable_variant(
                        ui,
                        "TrainStation",
                        matches!(self.dynamic_object, Some(AnyDynamicObject::TrainStation(_))),
                        || Some(AnyDynamicObject::TrainStation(Box::default())),
                        Some(ItemType::TrainStation),
                        self,
                    );

                    if changed {
                        context.flags |= ObjFlags::RebuildMesh;
                    }
                });

            if let Some(dyn_obj) = &mut self.dynamic_object {
                ui.horizontal(|ui| match dyn_obj {
                    AnyDynamicObject::Ladder(ladder) => {
                        ladder.add_grid("ladder_grid", ui, context);
                    }
                    AnyDynamicObject::Door(door) => {
                        door.add_grid("door_grid", ui, context);
                    }
                    AnyDynamicObject::Bed(bed) => {
                        bed.add_grid("bed_grid", ui, context);
                    }
                    AnyDynamicObject::Egg(egg) => {
                        egg.add_grid("egg_grid", ui, context);
                    }
                    AnyDynamicObject::Workbench(workbench) => {
                        workbench.add_grid("workbench_grid", ui, context);
                    }
                    AnyDynamicObject::Chest(chest) => {
                        chest.add_grid("chest_grid", ui, context);
                    }
                    AnyDynamicObject::Sign(sign) => {
                        sign.add_grid("sign_grid", ui, context);
                    }
                    AnyDynamicObject::TrainStation(train_station) => {
                        train_station.add_grid("train_station_grid", ui, context);
                    }
                });
                ui.end_row();
            }
        });
        ui.end_row();
    }
}

impl ToGrid for Chest {
    fn to_grid(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        self.save_time.add_row("saveTime", ui);
        ui.label("slots");
        ui.vertical(|ui| {
            let unique_id = self.unique_id;
            if let Some((num_col, num_row, slots)) = match &mut self.slots {
                ChestSlots::Standard(slots)
                | ChestSlots::Safe(slots)
                | ChestSlots::Gold(slots)
                | ChestSlots::Feeder(slots) => Some((4, 4, slots.as_mut_slice())),
                ChestSlots::Shelf { slots, .. } | ChestSlots::Cabinet { slots, .. } => {
                    Some((2, 2, slots.as_mut_slice()))
                }
                ChestSlots::Portal => None,
            } {
                let id = ui.id().with("chest_item_rows");
                let scroll_id = id.with("scroll_offset");
                let selected_idx_id = id.with("selected_idx");

                let size = egui::Vec2::new(COL_PX * num_col as f32, ROW_PX * num_row as f32);
                let (rect, response) = ui.allocate_exact_size(size, egui::Sense::all());
                let rect = rect.intersect(ui.ctx().content_rect());
                let ItemGridResult {
                    hovered_idx,
                    viewport,
                } = handle_item_grid_drag(ui, scroll_id, size, rect, num_col, num_row, &response);

                let selected_idx =
                    toggle_selected_index(ui, selected_idx_id, hovered_idx, &response);

                let instances = slots_to_item_instances(slots.iter(), num_col);

                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                    rect,
                    ItemGridCallback {
                        hovered_index: hovered_idx.map(|i| i as u32),
                        selected_index: selected_idx.map(|i| i as u32),
                        viewport,
                        pixels_per_point: ui.pixels_per_point(),
                        id: ui.id(),
                        instances: ItemGridInstances::Custom { instances },
                    },
                ));

                if let Some(idx) = selected_idx
                    && let Some(slot) = slots.get_mut(idx)
                {
                    let items = slot.deref_mut();
                    items.add_grid(
                        format!("selected_slot_items_grid_{:?}", unique_id),
                        ui,
                        context,
                    );
                }
            } else {
                ui.weak("Portal chest has no owned slots");
            }
        });
        ui.end_row();
    }
}

impl InfoUi for Chest {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let obj = self.deref_mut();
        obj.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("Chest");
            ui.separator();
            self.add_grid("chest_grid", ui, context);
        });
    }
}

impl BuildDwMesh for Chest {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity {
        items: 0,
        quads: CUBE_NUM_FACES,
    });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        use BlockUv::*;
        use ImageType::*;
        let block_coord = BlockCoord::new(self.pos_x, self.pos_y).context(CoordOutOfBoundSnafu)?;
        match self.slots.chest_type() {
            ChestType::Standard => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: ChestTop,
                    side: Chest,
                },
            )),
            ChestType::Safe => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: SafeTop,
                    side: Safe,
                },
            )),
            ChestType::Shelf => {
                builder.add_item(DwItem::from_item_type(self.float_pos, ItemType::Shelf))
            }
            ChestType::Gold => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: ChestGoldTop,
                    side: ChestGold,
                },
            )),
            ChestType::Portal => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: ChestPortalTop,
                    side: ChestPortal,
                },
            )),
            ChestType::Cabinet => builder.add_item(DwItem::from_item_type(
                self.float_pos,
                ItemType::DisplayCabinet,
            )),
            ChestType::Feeder => builder.add_block(DwBlock::new(
                block_coord,
                TopSide {
                    top: ChestFeederTop,
                    side: ChestFeeder,
                },
            )),
        };
        Ok(())
    }
}

impl ToRow for SignConnectionType {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        wrap_combo_box_resp(
            egui::ComboBox::from_id_salt("sign_connection_type_combo_box")
                .selected_text(format!("{:?}", self))
                .show_ui(ui, |ui| {
                    ui.selectable_value(self, Self::None, "None")
                        | ui.selectable_value(self, Self::GroundDouble, "GroundDouble")
                        | ui.selectable_value(self, Self::GroundSingle, "GroundSingle")
                        | ui.selectable_value(self, Self::Front, "Front")
                        | ui.selectable_value(self, Self::Side, "Side")
                        | ui.selectable_value(self, Self::Up, "Up")
                }),
        )
    }
}

impl ToGrid for Sign {
    fn to_grid(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        self.text.add_row("text", ui);
        if self.connection_type.add_row("connectionType", ui).changed() {
            context.flags |= ObjFlags::RebuildMesh;
        }
        self.offset_type.add_row("offsetType", ui);
        self.save_time.add_row("saveTime", ui);
    }
}

impl InfoUi for Sign {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let obj = self.deref_mut();
        obj.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("Sign");
            ui.separator();
            self.add_grid("sign_grid", ui, context);
        });
    }
}

impl BuildDwMesh for Sign {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 0, quads: 1 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        let z = 3.0;
        match self.connection_type {
            SignConnectionType::None => {}
            SignConnectionType::GroundDouble => {
                builder.add_face(DwFace::new_sprite(
                    ImageType::SignGroundDouble,
                    [1.0, 0.0],
                    self.float_pos,
                    [2, 1],
                    z,
                ));
            }
            SignConnectionType::GroundSingle => {
                builder.add_face(DwFace::new_sprite(
                    ImageType::SignGroundSingle,
                    [1.0, 0.0],
                    self.float_pos,
                    [2, 1],
                    z,
                ));
            }
            SignConnectionType::Front => {
                builder.add_face(DwFace::new_sprite(
                    ImageType::Sign,
                    [1.0, 0.0],
                    self.float_pos,
                    [2, 1],
                    z,
                ));
            }
            SignConnectionType::Side => {
                let mut face = DwFace::new_sprite(
                    ImageType::SignHang,
                    [if self.flipped { 1.5 } else { 0.5 }, 0.0],
                    self.float_pos,
                    [2, 2],
                    z,
                );
                if !self.flipped {
                    face.mirror_uv_h();
                }
                builder.add_face(face);
            }
            SignConnectionType::Up => {
                let mut face = DwFace::new_sprite(
                    ImageType::SignHang,
                    [if self.flipped { 1.5 } else { 0.5 }, 0.0],
                    self.float_pos,
                    [2, 1],
                    z,
                );
                if !self.flipped {
                    face.mirror_uv_h();
                }
                builder.add_face(face);
            }
        }
        Ok(())
    }
}

impl ToGrid for TrainStation {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut DwUiContext) {
        self.text.add_row("text", ui);
        self.save_time.add_row("saveTime", ui);
    }
}

impl InfoUi for TrainStation {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let obj = self.deref_mut();
        obj.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("TrainStation");
            ui.separator();
            self.add_grid("train_station_grid", ui, context);
        });
    }
}

impl BuildDwMesh for TrainStation {
    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_item(DwItem::from_item_type(
            self.float_pos,
            ItemType::TrainStation,
        ));
        Ok(())
    }
}

impl ToRow for TreeType {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        wrap_combo_box_resp(
            egui::ComboBox::from_id_salt("tree_type_combo_box")
                .selected_text(format!("{:?}", self))
                .show_ui(ui, |ui| {
                    ui.selectable_value(self, Self::Nothing, "Nothing")
                        | ui.selectable_value(self, Self::Apple, "Apple")
                        | ui.selectable_value(self, Self::Mango, "Mango")
                        | ui.selectable_value(self, Self::Maple, "Maple")
                        | ui.selectable_value(self, Self::Pine, "Pine")
                        | ui.selectable_value(self, Self::Cactus, "Cactus")
                        | ui.selectable_value(self, Self::Coconut, "Coconut")
                        | ui.selectable_value(self, Self::Orange, "Orange")
                        | ui.selectable_value(self, Self::Cherry, "Cherry")
                        | ui.selectable_value(self, Self::Coffee, "Coffee")
                        | ui.selectable_value(self, Self::Lime, "Lime")
                        | ui.selectable_value(self, Self::Amethyst, "Amethyst")
                        | ui.selectable_value(self, Self::Sapphire, "Sapphire")
                        | ui.selectable_value(self, Self::Emerald, "Emerald")
                        | ui.selectable_value(self, Self::Ruby, "Ruby")
                        | ui.selectable_value(self, Self::Diamond, "Diamond")
                }),
        )
    }
}

impl ToGrid for GemTree {
    fn to_grid(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        if self.gem_tree_type.add_row("gemTreeType", ui).changed() {
            context.flags |= ObjFlags::RebuildMesh;
        }
        self.fruit_year.add_row("fruitYear", ui);
    }
}

impl InfoUi for GemTree {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let tree = self.deref_mut();
        tree.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("GemTree");
            ui.separator();
            self.add_grid("gem_tree_grid", ui, context);
        });
    }
}

impl BuildDwMesh for GemTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        let item_type = match self.gem_tree_type {
            TreeType::Amethyst => ItemType::Amethyst,
            TreeType::Sapphire => ItemType::Sapphire,
            TreeType::Emerald => ItemType::Emerald,
            TreeType::Ruby => ItemType::Ruby,
            TreeType::Diamond => ItemType::Diamond,
            _ => ItemType::Unknown,
        };
        builder.add_item(DwItem::from_item_type(self.float_pos, item_type));
        Ok(())
    }
}

impl ToGrid for VinePlant {
    fn to_grid(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        self.growth_timer.add_row("growthTimer", ui);
        if self
            .number_of_occupied_tiles_below
            .add_row("numberOfOccupiedTilesBelow", ui)
            .changed()
        {
            context.flags |= ObjFlags::RebuildMesh;
        }
    }
}

impl InfoUi for VinePlant {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let tree = self.deref_mut();
        tree.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("GemTree");
            ui.separator();
            self.add_grid("gem_tree_grid", ui, context);
        });
    }
}

impl BuildDwMesh for VinePlant {
    fn capacity(&self) -> DwCapacity {
        let len = self.number_of_occupied_tiles_below as usize + 1;
        DwCapacity {
            items: 0,
            quads: if len <= 1 { len } else { (len + 3) / 2 },
        }
    }

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        let len = self.number_of_occupied_tiles_below + 1;
        let [x, mut y] = self.float_pos;
        match len {
            0 => {}
            1 => {
                builder.add_face(DwFace::new_sprite(
                    ImageType::VinePlantBottom,
                    [0.5, 0.0],
                    [x, y],
                    [1, 1],
                    3.0,
                ));
            }
            mut len => {
                builder.add_face(DwFace::new_sprite(
                    ImageType::VinePlantTop,
                    [0.5, 0.0],
                    [x, y],
                    [1, 1],
                    3.0,
                ));
                y -= 1.0;
                len -= 1;
                while len >= 3 {
                    builder.add_face(DwFace::new_sprite(
                        ImageType::VinePlant,
                        [0.5, 1.0],
                        [x, y],
                        [1, 2],
                        3.0,
                    ));
                    y -= 2.0;
                    len -= 2;
                }
                if len == 2 {
                    builder.add_face(DwFace::new_sprite(
                        ImageType::VinePlantOddLen,
                        [0.5, 0.0],
                        [x, y],
                        [1, 1],
                        3.0,
                    ));
                    y -= 1.0;
                }
                builder.add_face(DwFace::new_sprite(
                    ImageType::VinePlantBottom,
                    [0.5, 0.0],
                    [x, y],
                    [1, 1],
                    3.0,
                ));
            }
        }

        Ok(())
    }
}

impl ToGrid for TulipPlant {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut DwUiContext) {
        self.color_genes.add_row("colorGenes", ui);
        self.mate_color_genes.add_row("mateColorGenes", ui);
        self.mix_genes.add_row("mixGenes", ui);
    }
}

impl InfoUi for TulipPlant {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let tree = self.deref_mut();
        tree.info(ui, context);

        ui.vertical(|ui| {
            ui.heading("TulipPlant");
            ui.separator();
            self.add_grid("tulip_plant_grid", ui, context);
        });
    }
}

impl BuildDwMesh for TulipPlant {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 0, quads: 1 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_face(DwFace::new_sprite(
            if self.flowering {
                ImageType::TulipPlantMature
            } else {
                ImageType::TulipPlantSprout
            },
            [0.5, 0.0],
            self.float_pos,
            [1, 1],
            2.0,
        ));
        Ok(())
    }
}

impl InfoUi for WheatPlant {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui, context);
    }
}

impl BuildDwMesh for WheatPlant {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 0, quads: 1 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_face(DwFace::new_sprite(
            if self.flowering {
                ImageType::WheatPlantFlower
            } else {
                ImageType::WheatPlant
            },
            [0.5, 0.0],
            self.float_pos,
            [1, 2],
            2.0,
        ));
        Ok(())
    }
}

impl InfoUi for TomatoPlant {
    fn info(&mut self, ui: &mut egui::Ui, context: &mut DwUiContext) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui, context);
    }
}

impl BuildDwMesh for TomatoPlant {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { items: 0, quads: 1 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_face(DwFace::new_sprite(
            if self.flowering {
                ImageType::TomatoPlantFlower
            } else {
                ImageType::TomatoPlant
            },
            [0.5, 0.0],
            self.float_pos,
            [1, 2],
            2.0,
        ));
        Ok(())
    }
}

pub(crate) fn block_type_drop_menu(ui: &mut egui::Ui, b: &mut BlockType) -> egui::Response {
    use BlockType::*;
    wrap_combo_box_resp(
        egui::ComboBox::from_id_salt(ui.id().with("block_type_combo_box"))
            .selected_text(format!("{:?}", b))
            .show_ui(ui, |ui| {
                ui.selectable_value(b, Stone, "Stone")
                    | ui.selectable_value(b, Air, "Air")
                    | ui.selectable_value(b, Water, "Water")
                    | ui.selectable_value(b, Ice, "Ice")
                    | ui.selectable_value(b, Snow, "Snow")
                    | ui.selectable_value(b, Dirt, "Dirt")
                    | ui.selectable_value(b, DesertSand, "DesertSand")
                    | ui.selectable_value(b, BeachSand, "BeachSand")
                    | ui.selectable_value(b, Wood, "Wood")
                    | ui.selectable_value(b, MinedStone, "MinedStone")
                    | ui.selectable_value(b, RedBrick, "RedBrick")
                    | ui.selectable_value(b, Limestone, "Limestone")
                    | ui.selectable_value(b, MinedLimestone, "MinedLimestone")
                    | ui.selectable_value(b, Marble, "Marble")
                    | ui.selectable_value(b, MinedMarble, "MinedMarble")
                    | ui.selectable_value(b, TimeCrystal, "TimeCrystal")
                    | ui.selectable_value(b, SandStone, "SandStone")
                    | ui.selectable_value(b, MinedSandStone, "MinedSandStone")
                    | ui.selectable_value(b, RedMarble, "RedMarble")
                    | ui.selectable_value(b, MinedRedMarble, "MinedRedMarble")
                    | ui.selectable_value(b, Glass, "Glass")
                    | ui.selectable_value(b, SpawnPortalBase, "SpawnPortalBase")
                    | ui.selectable_value(b, GoldBlock, "GoldBlock")
                    | ui.selectable_value(b, GrassDirt, "GrassDirt")
                    | ui.selectable_value(b, SnowDirt, "SnowDirt")
                    | ui.selectable_value(b, LapisLazuli, "LapisLazuli")
                    | ui.selectable_value(b, MinedLapisLazuli, "MinedLapisLazuli")
                    | ui.selectable_value(b, Lava, "Lava")
                    | ui.selectable_value(b, ReinforcedPlatform, "ReinforcedPlatform")
                    | ui.selectable_value(b, SpawnPortalBaseAmethyst, "SpawnPortalBaseAmethyst")
                    | ui.selectable_value(b, SpawnPortalBaseSapphire, "SpawnPortalBaseSapphire")
                    | ui.selectable_value(b, SpawnPortalBaseEmerald, "SpawnPortalBaseEmerald")
                    | ui.selectable_value(b, SpawnPortalBaseRuby, "SpawnPortalBaseRuby")
                    | ui.selectable_value(b, SpawnPortalBaseDiamond, "SpawnPortalBaseDiamond")
                    | ui.selectable_value(b, NorthPole, "NorthPole")
                    | ui.selectable_value(b, SouthPole, "SouthPole")
                    | ui.selectable_value(b, WestPole, "WestPole")
                    | ui.selectable_value(b, EastPole, "EastPole")
                    | ui.selectable_value(b, PortalBase, "PortalBase")
                    | ui.selectable_value(b, PortalBaseAmethyst, "PortalBaseAmethyst")
                    | ui.selectable_value(b, PortalBaseSapphire, "PortalBaseSapphire")
                    | ui.selectable_value(b, PortalBaseEmerald, "PortalBaseEmerald")
                    | ui.selectable_value(b, PortalBaseRuby, "PortalBaseRuby")
                    | ui.selectable_value(b, PortalBaseDiamond, "PortalBaseDiamond")
                    | ui.selectable_value(b, Compost, "Compost")
                    | ui.selectable_value(b, GrassCompost, "GrassCompost")
                    | ui.selectable_value(b, SnowCompost, "SnowCompost")
                    | ui.selectable_value(b, Basalt, "Basalt")
                    | ui.selectable_value(b, MinedBasalt, "MinedBasalt")
                    | ui.selectable_value(b, CopperBlock, "CopperBlock")
                    | ui.selectable_value(b, TinBlock, "TinBlock")
                    | ui.selectable_value(b, BronzeBlock, "BronzeBlock")
                    | ui.selectable_value(b, IronBlock, "IronBlock")
                    | ui.selectable_value(b, SteelBlock, "SteelBlock")
                    | ui.selectable_value(b, BlackSand, "BlackSand")
                    | ui.selectable_value(b, BlackGlass, "BlackGlass")
                    | ui.selectable_value(b, TradePortalBase, "TradePortalBase")
                    | ui.selectable_value(b, TradePortalBaseAmethyst, "TradePortalBaseAmethyst")
                    | ui.selectable_value(b, TradePortalBaseSapphire, "TradePortalBaseSapphire")
                    | ui.selectable_value(b, TradePortalBaseEmerald, "TradePortalBaseEmerald")
                    | ui.selectable_value(b, TradePortalBaseRuby, "TradePortalBaseRuby")
                    | ui.selectable_value(b, TradePortalBaseDiamond, "TradePortalBaseDiamond")
                    | ui.selectable_value(b, PlatinumBlock, "PlatinumBlock")
                    | ui.selectable_value(b, TitaniumBlock, "TitaniumBlock")
                    | ui.selectable_value(b, CarbonFiberBlock, "CarbonFiberBlock")
                    | ui.selectable_value(b, Gravel, "Gravel")
                    | ui.selectable_value(b, AmethystBlock, "AmethystBlock")
                    | ui.selectable_value(b, SapphireBlock, "SapphireBlock")
                    | ui.selectable_value(b, EmeraldBlock, "EmeraldBlock")
                    | ui.selectable_value(b, RubyBlock, "RubyBlock")
                    | ui.selectable_value(b, DiamondBlock, "DiamondBlock")
                    | ui.selectable_value(b, Plaster, "Plaster")
                    | ui.selectable_value(b, LuminousPlaster, "LuminousPlaster")
            }),
    )
}

pub(crate) fn block_content_type_drop_menu(
    ui: &mut egui::Ui,
    b: &mut BlockContentType,
) -> egui::Response {
    use BlockContentType::*;
    wrap_combo_box_resp(
        egui::ComboBox::from_id_salt(ui.id().with("block_content_type_combo_box"))
            .selected_text(format!("{:?}", b))
            .show_ui(ui, |ui| {
                ui.selectable_value(b, Nothing, "Nothing")
                    | ui.selectable_value(b, Flint, "Flint")
                    | ui.selectable_value(b, Clay, "Clay")
                    | ui.selectable_value(b, AppleTreeLeaf, "AppleTreeLeaf")
                    | ui.selectable_value(b, AppleTreeTrunk, "AppleTreeTrunk")
                    | ui.selectable_value(b, AppleTreeTrunkLeaf, "AppleTreeTrunkLeaf")
                    | ui.selectable_value(b, PineTreeLeaf, "PineTreeLeaf")
                    | ui.selectable_value(b, PineTreeTrunk, "PineTreeTrunk")
                    | ui.selectable_value(b, PineTreeTrunkLeaf, "PineTreeTrunkLeaf")
                    | ui.selectable_value(b, MapleTreeLeaf, "MapleTreeLeaf")
                    | ui.selectable_value(b, MapleTreeTrunk, "MapleTreeTrunk")
                    | ui.selectable_value(b, MapleTreeTrunkLeaf, "MapleTreeTrunkLeaf")
                    | ui.selectable_value(b, MangoTreeLeaf, "MangoTreeLeaf")
                    | ui.selectable_value(b, MangoTreeTrunk, "MangoTreeTrunk")
                    | ui.selectable_value(b, MangoTreeTrunkLeaf, "MangoTreeTrunkLeaf")
                    | ui.selectable_value(b, CoconutTreeLeaf, "CoconutTreeLeaf")
                    | ui.selectable_value(b, CoconutTreeTrunk, "CoconutTreeTrunk")
                    | ui.selectable_value(b, OrangeTreeLeaf, "OrangeTreeLeaf")
                    | ui.selectable_value(b, OrangeTreeTrunk, "OrangeTreeTrunk")
                    | ui.selectable_value(b, OrangeTreeTrunkLeaf, "OrangeTreeTrunkLeaf")
                    | ui.selectable_value(b, CherryTreeLeaf, "CherryTreeLeaf")
                    | ui.selectable_value(b, CherryTreeTrunk, "CherryTreeTrunk")
                    | ui.selectable_value(b, CherryTreeTrunkLeaf, "CherryTreeTrunkLeaf")
                    | ui.selectable_value(b, CoffeeTreeLeaf, "CoffeeTreeLeaf")
                    | ui.selectable_value(b, CoffeeTreeTrunk, "CoffeeTreeTrunk")
                    | ui.selectable_value(b, CoffeeTreeTrunkLeaf, "CoffeeTreeTrunkLeaf")
                    | ui.selectable_value(b, DeadPineTreeTrunk, "DeadPineTreeTrunk")
                    | ui.selectable_value(b, DeadPineTreeLeaf, "DeadPineTreeLeaf")
                    | ui.selectable_value(b, DeadOrangeTreeLeaf, "DeadOrangeTreeLeaf")
                    | ui.selectable_value(b, DeadOrangeTreeTrunk, "DeadOrangeTreeTrunk")
                    | ui.selectable_value(b, DeadCherryTreeLeaf, "DeadCherryTreeLeaf")
                    | ui.selectable_value(b, DeadCherryTreeTrunk, "DeadCherryTreeTrunk")
                    | ui.selectable_value(b, Cactus, "Cactus")
                    | ui.selectable_value(b, DeadCactus, "DeadCactus")
                    | ui.selectable_value(b, Workbench, "Workbench")
                    | ui.selectable_value(b, WorkbenchSprite, "WorkbenchSprite")
                    | ui.selectable_value(b, Sprite, "Sprite")
                    | ui.selectable_value(b, CopperOre, "CopperOre")
                    | ui.selectable_value(b, TinOre, "TinOre")
                    | ui.selectable_value(b, IronOre, "IronOre")
                    | ui.selectable_value(b, Oil, "Oil")
                    | ui.selectable_value(b, Coal, "Coal")
                    | ui.selectable_value(b, GoldNuggets, "GoldNuggets")
                    | ui.selectable_value(b, LimeTreeLeaf, "LimeTreeLeaf")
                    | ui.selectable_value(b, LimeTreeTrunk, "LimeTreeTrunk")
                    | ui.selectable_value(b, LimeTreeTrunkLeaf, "LimeTreeTrunkLeaf")
                    | ui.selectable_value(b, DeadLimeTreeLeaf, "DeadLimeTreeLeaf")
                    | ui.selectable_value(b, DeadLimeTreeTrunk, "DeadLimeTreeTrunk")
                    | ui.selectable_value(b, GoldChest, "GoldChest")
                    | ui.selectable_value(b, PlatinumOre, "PlatinumOre")
                    | ui.selectable_value(b, TitaniumOre, "TitaniumOre")
                    | ui.selectable_value(b, AmethystTreeTrunk, "AmethystTreeTrunk")
                    | ui.selectable_value(b, AmethystTreeLeaf, "AmethystTreeLeaf")
                    | ui.selectable_value(b, AmethystTreeTrunkLeaf, "AmethystTreeTrunkLeaf")
                    | ui.selectable_value(b, SapphireTreeTrunk, "SapphireTreeTrunk")
                    | ui.selectable_value(b, SapphireTreeLeaf, "SapphireTreeLeaf")
                    | ui.selectable_value(b, SapphireTreeTrunkLeaf, "SapphireTreeTrunkLeaf")
                    | ui.selectable_value(b, EmeraldTreeTrunk, "EmeraldTreeTrunk")
                    | ui.selectable_value(b, EmeraldTreeLeaf, "EmeraldTreeLeaf")
                    | ui.selectable_value(b, EmeraldTreeTrunkLeaf, "EmeraldTreeTrunkLeaf")
                    | ui.selectable_value(b, RubyTreeTrunk, "RubyTreeTrunk")
                    | ui.selectable_value(b, RubyTreeLeaf, "RubyTreeLeaf")
                    | ui.selectable_value(b, RubyTreeTrunkLeaf, "RubyTreeTrunkLeaf")
                    | ui.selectable_value(b, DiamondTreeTrunk, "DiamondTreeTrunk")
                    | ui.selectable_value(b, DiamondTreeLeaf, "DiamondTreeLeaf")
                    | ui.selectable_value(b, DiamondTreeTrunkLeaf, "DiamondTreeTrunkLeaf")
            }),
    )
}
