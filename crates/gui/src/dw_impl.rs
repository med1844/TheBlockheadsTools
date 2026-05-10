// Implements traits for dynamic object types defined in lib
use super::{
    gpu::{
        VoxelType,
        dw::{
            BuildDwMesh, BuildDwMeshError, CoordOutOfBoundSnafu, DwBlock, DwCapacity,
            DwChunkBufBuilder, DwFace, DwIcon, DwQuad, FaceDirection, InvalidItemTypeForDoorSnafu,
            InvalidItemTypeForTorchSnafu, InvalidWorkbenchLevelSnafu,
        },
    },
    image_type::ImageType,
};
use eframe::egui;
use snafu::ResultExt;
use std::{
    hash::Hash,
    ops::{BitOrAssign, DerefMut},
};
use the_blockheads_tools_lib::game::{
    coord::BlockCoord,
    dynamic_object::{
        ArtificialLight, DynamicObject, InteractionObject, InteractionObjectType, LightDirection,
        UniqueID,
        animal::{DodoBreed, Egg},
        chest::{Chest, ChestType},
        craft::{Door, Ladder, Torch, TorchConnectionType},
        plant::{CarrotPlant, CornPlant, KelpPlant, NormalPlant, Plant, TomatoPlant},
        train::TrainStation,
        tree::{
            AppleTree, CactusTree, CherryTree, CoconutTree, CoffeeTree, GemTree, LimeTree,
            MangoTree, MapleTree, OrangeTree, PineTree, Tree, TreeFruit, TreeType,
        },
        workbench::{Workbench, WorkbenchType},
    },
    item::ItemType,
};

const CUBE_NUM_FACES: usize = 6;

#[derive(Debug)]
pub enum ObjFlags {
    PosChangedTo { x: f32, y: f32 },
    RebuildMesh,
    NoChange,
}

impl Default for ObjFlags {
    fn default() -> Self {
        Self::NoChange
    }
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
    fn to_grid(&mut self, _: &mut egui::Ui, _: &mut ObjFlags) {}

    fn add_grid<H: Hash>(&mut self, id: H, ui: &mut egui::Ui, flags: &mut ObjFlags) {
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
                t.add_grid(id, ui, flags);
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
    fn to_grid(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags);
    fn add_grid<H: Hash>(&mut self, id: H, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        egui::Grid::new(id).num_columns(2).show(ui, |ui| {
            self.to_grid(ui, flags);
        });
    }
}

impl ToGrid for DynamicObject {
    fn to_grid(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let mut float_pos_changed = false;
        float_pos_changed |= self.float_pos[0].add_row("floatPos[0]", ui).changed();
        float_pos_changed |= self.float_pos[1].add_row("floatPos[1]", ui).changed();

        let mut int_pos_changed = false;
        int_pos_changed |= self.pos_x.add_row("pos_x", ui).changed();
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
            *flags |= ObjFlags::PosChangedTo {
                x: self.float_pos[0],
                y: self.float_pos[1],
            };
        }

        self.unique_id.add_row("uniqueID", ui);
        self.owner_id.add_row("ownerID", ui);
    }
}

impl<T: Default + ToGrid> ToGrid for Vec<T> {
    fn to_grid(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        ui.vertical(|ui| {
            let mut to_remove = None;

            for (i, item) in self.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if ui.button("Del").on_hover_text("Remove item").clicked() {
                        to_remove = Some(i);
                    }

                    ui.collapsing(format!("Item #{}", i), |ui| {
                        item.add_grid(format!("item_grid_{}", i), ui, flags);
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
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut ObjFlags) {
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
    flags: &mut ObjFlags,
) {
    ui.label(label);
    t.add_grid(id, ui, flags);
    ui.end_row();
}

impl ToGrid for Tree {
    fn to_grid(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
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
            flags,
        );
    }
}

pub(crate) trait InfoUi {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags);
}

impl InfoUi for DynamicObject {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        ui.vertical(|ui| {
            ui.heading("DynamicObject");
            ui.separator();
            self.add_grid("dynamic_object_grid", ui, flags);
        });
    }
}

impl InfoUi for Tree {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let obj = self.deref_mut();
        obj.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("Tree");
            ui.separator();
            self.add_grid("tree_grid", ui, flags);
        });
    }
}

impl BuildDwMesh for AppleTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_icon(DwIcon::new(self.float_pos, ItemType::Apple));
        Ok(())
    }
}

impl ToGrid for AppleTree {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut ObjFlags) {
        self.available_food.add_row("availableFood", ui);
    }
}

impl InfoUi for AppleTree {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let tree = self.deref_mut();
        tree.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("AppleTree");
            ui.separator();
            self.add_grid("apple_tree_grid", ui, flags);
        });
    }
}

impl BuildDwMesh for MapleTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_icon(DwIcon::new(self.float_pos, ItemType::MapleSeed));
        Ok(())
    }
}

impl InfoUi for MapleTree {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let tree = self.deref_mut();
        tree.info(ui, flags);
    }
}

impl BuildDwMesh for MangoTree {
    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_icon(DwIcon::new(self.float_pos, ItemType::Mango));
        Ok(())
    }
}

impl InfoUi for MangoTree {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let tree = self.deref_mut();
        tree.info(ui, flags);
    }
}

impl ToGrid for PineTree {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut ObjFlags) {
        self.available_food.add_row("availableFood", ui);
    }
}

impl InfoUi for PineTree {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let tree = self.deref_mut();
        tree.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("PineTree");
            ui.separator();
            self.add_grid("pine_tree_grid", ui, flags);
        });
    }
}

impl BuildDwMesh for PineTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_icon(DwIcon::new(self.float_pos, ItemType::Pinecone));
        Ok(())
    }
}

impl ToGrid for CactusTree {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut ObjFlags) {
        self.available_food.add_row("availableFood", ui);
        self.split_direction.add_row("splitDirection", ui);
        self.split_height_a.add_row("splitHeightA", ui);
        self.split_height_b.add_row("splitHeightB", ui);
    }
}

impl InfoUi for CactusTree {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let tree = self.deref_mut();
        tree.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("CactusTree");
            ui.separator();
            self.add_grid("cactus_tree_grid", ui, flags);
        });
    }
}

impl BuildDwMesh for CactusTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_icon(DwIcon::new(self.float_pos, ItemType::PricklyPear));
        Ok(())
    }
}

impl InfoUi for CoconutTree {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let tree = self.deref_mut();
        tree.info(ui, flags);
    }
}

impl BuildDwMesh for CoconutTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_icon(DwIcon::new(self.float_pos, ItemType::Coconut));
        Ok(())
    }
}

impl InfoUi for OrangeTree {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let tree = self.deref_mut();
        tree.info(ui, flags);
    }
}
impl BuildDwMesh for OrangeTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_icon(DwIcon::new(self.float_pos, ItemType::Orange));
        Ok(())
    }
}

impl InfoUi for CherryTree {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let tree = self.deref_mut();
        tree.info(ui, flags);
    }
}

impl BuildDwMesh for CherryTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_icon(DwIcon::new(self.float_pos, ItemType::Cherry));
        Ok(())
    }
}

impl InfoUi for CoffeeTree {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let tree = self.deref_mut();
        tree.info(ui, flags);
    }
}

impl BuildDwMesh for CoffeeTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_icon(DwIcon::new(self.float_pos, ItemType::CoffeeCherry));
        Ok(())
    }
}

impl ToGrid for Plant {
    fn to_grid(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        self.save_time.add_row("saveTime", ui);
        self.season_offset.add_row("seasonOffset", ui);
        self.gather_progress.add_row("gatherProgress", ui);
        self.has_flowered_this_season
            .add_row("hasFloweredThisSeason", ui);
        if self.flowering.add_row("flowering", ui).changed() {
            *flags |= ObjFlags::RebuildMesh;
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
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let obj = self.deref_mut();
        obj.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("Plant");
            ui.separator();
            self.add_grid("plant_grid", ui, flags);
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
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut ObjFlags) {
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
    fn to_grid(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        self.available_food.add_row("saveTime", ui);
        grid_as_row(
            &mut self.light_dict,
            "lightDict",
            "light_dict_grid",
            ui,
            flags,
        );
    }
}

impl InfoUi for NormalPlant {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let plant = self.deref_mut();
        plant.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("NormalPlant");
            ui.separator();
            self.add_grid("normal_plant_grid", ui, flags);
        });
    }
}

impl InfoUi for CornPlant {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui, flags);
    }
}

impl BuildDwMesh for CornPlant {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 0, quads: 1 });

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

impl ToRow for ItemType {
    fn to_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
        // ItemType contains TOO MANY types, might need dedicated selector.
        // TODO either add selector, limit door/ladder/etc type, or display raw id
        let item_type_str: &'static str = (*self).into();
        ui.label(item_type_str)
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
    fn to_grid(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        grid_as_row(
            &mut self.light_dict,
            "lightDict",
            "light_dict_grid",
            ui,
            flags,
        );
        if self.connection_type.add_row("connectionType", ui).changed() {
            *flags |= ObjFlags::RebuildMesh;
        }
        self.item_type.add_row("itemType", ui);
        self.data_a.add_row("dataA", ui);
        self.data_b.add_row("dataB", ui);
    }
}

impl InfoUi for Torch {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let obj = self.deref_mut();
        obj.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("Torch");
            ui.separator();
            self.add_grid("torch_grid", ui, flags);
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
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 0, quads: 1 });

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
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut ObjFlags) {
        self.paint_color.add_row("paintColor", ui);
        self.item_type.add_row("itemType", ui);
    }
}

impl InfoUi for Ladder {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let obj = self.deref_mut();
        obj.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("Ladder");
            ui.separator();
            self.add_grid("ladder_grid", ui, flags);
        });
    }
}

impl BuildDwMesh for Ladder {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 0, quads: 1 });

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
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut ObjFlags) {
        self.item_type.add_row("itemType", ui);
        self.blocked.add_row("blocked", ui);
        self.iron_place_client_id.add_row("ironPlaceClientId", ui);
    }
}

impl InfoUi for Door {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let obj = self.deref_mut();
        obj.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("Door");
            ui.separator();
            self.add_grid("door_grid", ui, flags);
        });
    }
}

impl BuildDwMesh for Door {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 0, quads: 2 });

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
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui, flags);
    }
}

impl BuildDwMesh for CarrotPlant {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 0, quads: 1 });

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
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut ObjFlags) {
        self.genes_dict.breed.add_row("breed", ui);
        self.hatch_timer.add_row("hatchTimer", ui);
        self.save_time.add_row("saveTime", ui);
    }
}

impl InfoUi for Egg {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let obj = self.deref_mut();
        obj.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("Egg");
            ui.separator();
            self.add_grid("egg_grid", ui, flags);
        });
    }
}

impl BuildDwMesh for Egg {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        // TODO add render egg with real breed textures
        builder.add_icon(DwIcon::new(self.float_pos, ItemType::DodoEgg));
        Ok(())
    }
}

impl ToGrid for KelpPlant {
    fn to_grid(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        self.growth_timer.add_row("growthTimer", ui);
        if self
            .number_of_occupied_tiles_above
            .add_row("numberOfOccupiedTilesAbove", ui)
            .changed()
        {
            *flags |= ObjFlags::RebuildMesh;
        }
    }
}

impl InfoUi for KelpPlant {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("KelpPlant");
            ui.separator();
            self.add_grid("kelp_plant_grid", ui, flags);
        });
    }
}

impl BuildDwMesh for KelpPlant {
    fn capacity(&self) -> DwCapacity {
        let len = self.number_of_occupied_tiles_above as usize;
        DwCapacity {
            icons: 0,
            quads: if len == 0 { 0 } else { len / 2 + 1 },
        }
    }

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        let mut len = self.number_of_occupied_tiles_above;
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
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let tree = self.deref_mut();
        tree.info(ui, flags);
    }
}

impl BuildDwMesh for LimeTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_icon(DwIcon::new(self.float_pos, ItemType::Lime));
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
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut ObjFlags) {
        self.interaction_object_type
            .add_row("interactionObjectType", ui);
        self.is_in_use.add_row("isInUse", ui);
        self.flipped.add_row("flipped", ui);
        self.paint_color.add_row("paintColor", ui);
    }
}

impl InfoUi for InteractionObject {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let obj = self.deref_mut();
        obj.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("InteractionObject");
            ui.separator();
            self.add_grid("interaction_object_grid", ui, flags);
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
    fn to_grid(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
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
            *flags |= ObjFlags::RebuildMesh;
        }
        self.save_time.add_row("saveTime", ui);
        self.selected_index.add_row("selectedIndex", ui);
        if self.workbench_type.add_row("workbenchType", ui).changed() {
            *flags |= ObjFlags::RebuildMesh;
        }
        self.x_scroll.add_row("xScroll", ui);
        grid_as_row(
            &mut self.light_dict,
            "lightDict",
            "light_dict_grid",
            ui,
            flags,
        );
    }
}

impl InfoUi for Workbench {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let obj = self.deref_mut();
        obj.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("Workbench");
            ui.separator();
            self.add_grid("workbench_grid", ui, flags);
        });
    }
}

impl BuildDwMesh for Workbench {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity {
        icons: 0,
        quads: CUBE_NUM_FACES,
    });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        let block_coord = BlockCoord::new(self.pos_x, self.pos_y).context(CoordOutOfBoundSnafu)?;
        match self.workbench_type {
            WorkbenchType::Undefined => {
                builder.add_icon(DwIcon::new(self.float_pos, ItemType::Unknown))
            }
            WorkbenchType::BasicPortal | WorkbenchType::PlacedPortal => builder.add_face(
                DwFace::new_sprite(ImageType::Portal0, [0.5, 0.0], self.float_pos, [1, 2], 2.0),
            ),
            WorkbenchType::Workbench => builder.add_block(DwBlock::new(
                block_coord,
                match self.level {
                    0 => VoxelType::WorkbenchLevel1,
                    1 => VoxelType::WorkbenchLevel2,
                    2 => VoxelType::WorkbenchLevel3,
                    3 => VoxelType::WorkbenchLevel4,
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
                    0 => VoxelType::TailorsBenchLevel1,
                    1 => VoxelType::TailorsBenchLevel2,
                    _ => InvalidWorkbenchLevelSnafu {
                        workbench_type: self.workbench_type,
                        level: self.level,
                        maximum: 2,
                    }
                    .fail()?,
                },
            )),
            WorkbenchType::Wood => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::WoodworkBench))
            }
            WorkbenchType::Tool => builder.add_block(DwBlock::new(
                block_coord,
                match self.level {
                    0 => VoxelType::ToolBenchLevel1,
                    1 => VoxelType::ToolBenchLevel2,
                    2 => VoxelType::ToolBenchLevel3,
                    3 => VoxelType::ToolBenchLevel4,
                    4 => VoxelType::ToolBenchLevel5,
                    5 => VoxelType::ToolBenchLevel6,
                    6 => VoxelType::ToolBenchLevel7,
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
                    0 => VoxelType::PressLevel1,
                    1 => VoxelType::PressLevel2,
                    _ => InvalidWorkbenchLevelSnafu {
                        workbench_type: self.workbench_type,
                        level: self.level,
                        maximum: 2,
                    }
                    .fail()?,
                },
            )),
            WorkbenchType::Kiln => builder.add_block(DwBlock::new(block_coord, VoxelType::Kiln)),
            WorkbenchType::Furnace => builder.add_block(DwBlock::new(
                block_coord,
                match self.level {
                    0 => VoxelType::FurnaceLevel1,
                    1 => VoxelType::FurnaceLevel2,
                    2 => VoxelType::FurnaceLevel3,
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
                    0 => VoxelType::CraftBenchLevel1,
                    1 => VoxelType::CraftBenchLevel2,
                    2 => VoxelType::CraftBenchLevel3,
                    3 => VoxelType::CraftBenchLevel4,
                    _ => InvalidWorkbenchLevelSnafu {
                        workbench_type: self.workbench_type,
                        level: self.level,
                        maximum: 4,
                    }
                    .fail()?,
                },
            )),
            WorkbenchType::Mix => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::MixingBench))
            }
            WorkbenchType::Dye => builder.add_block(DwBlock::new(block_coord, VoxelType::DyeBench)),
            WorkbenchType::Metalwork => builder.add_block(DwBlock::new(
                block_coord,
                match self.level {
                    0 => VoxelType::MetalworkBenchLevel1,
                    1 => VoxelType::MetalworkBenchLevel2,
                    _ => InvalidWorkbenchLevelSnafu {
                        workbench_type: self.workbench_type,
                        level: self.level,
                        maximum: 2,
                    }
                    .fail()?,
                },
            )),
            WorkbenchType::SteamGenerator => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::SteamGenerator))
            }
            WorkbenchType::ElectricKiln => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::ElectricKiln))
            }
            WorkbenchType::ElectricFurnace => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::ElectricFurnace))
            }
            WorkbenchType::ElectricMetalworkBench => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::ElectricMetalworkBench))
            }
            WorkbenchType::ElectricStove => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::ElectricStove))
            }
            WorkbenchType::SolarPanel => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::SolarPanel))
            }
            WorkbenchType::Flywheel => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::Flywheel))
            }
            WorkbenchType::ArmorBench => builder.add_block(DwBlock::new(
                block_coord,
                match self.level {
                    0 => VoxelType::ArmorBenchLevel1,
                    1 => VoxelType::ArmorBenchLevel2,
                    2 => VoxelType::ArmorBenchLevel3,
                    _ => InvalidWorkbenchLevelSnafu {
                        workbench_type: self.workbench_type,
                        level: self.level,
                        maximum: 3,
                    }
                    .fail()?,
                },
            )),
            WorkbenchType::TrainYard => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::TrainYard))
            }
            WorkbenchType::Easel => builder.add_icon(DwIcon::new(self.float_pos, ItemType::Easel)),
            WorkbenchType::Build => builder.add_block(DwBlock::new(
                block_coord,
                match self.level {
                    0 => VoxelType::BuildersBenchLevel1,
                    1 => VoxelType::BuildersBenchLevel2,
                    _ => InvalidWorkbenchLevelSnafu {
                        workbench_type: self.workbench_type,
                        level: self.level,
                        maximum: 2,
                    }
                    .fail()?,
                },
            )),
            WorkbenchType::Refinery => {
                builder.add_icon(DwIcon::new(self.float_pos, ItemType::Refinery))
            }
            WorkbenchType::ElectricPress => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::ElectricPress))
            }
            WorkbenchType::CompostBin => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::CompostBin))
            }
            WorkbenchType::Sluice => {
                builder.add_icon(DwIcon::new(self.float_pos, ItemType::ElectricSluice))
            }
            WorkbenchType::EggExtractor => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::EggExtractor))
            }
            WorkbenchType::PizzaOven => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::PizzaOven))
            }
        }
        Ok(())
    }
}

impl ToGrid for Chest {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut ObjFlags) {
        self.save_time.add_row("saveTime", ui);
        // TODO find a proper way to display the items
    }
}

impl InfoUi for Chest {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let obj = self.deref_mut();
        obj.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("Chest");
            ui.separator();
            self.add_grid("chest_grid", ui, flags);
        });
    }
}

impl BuildDwMesh for Chest {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity {
        icons: 0,
        quads: CUBE_NUM_FACES,
    });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        let block_coord = BlockCoord::new(self.pos_x, self.pos_y).context(CoordOutOfBoundSnafu)?;
        match self.slots.chest_type() {
            ChestType::Standard => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::StandardChest))
            }
            ChestType::Safe => builder.add_block(DwBlock::new(block_coord, VoxelType::Safe)),
            ChestType::Shelf => builder.add_icon(DwIcon::new(self.float_pos, ItemType::Shelf)),
            ChestType::Gold => builder.add_block(DwBlock::new(block_coord, VoxelType::GoldChest)),
            ChestType::Portal => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::PortalChest))
            }
            ChestType::Cabinet => {
                builder.add_icon(DwIcon::new(self.float_pos, ItemType::DisplayCabinet))
            }
            ChestType::Feeder => {
                builder.add_block(DwBlock::new(block_coord, VoxelType::FeederChest))
            }
        };
        Ok(())
    }
}

impl ToGrid for TrainStation {
    fn to_grid(&mut self, ui: &mut egui::Ui, _: &mut ObjFlags) {
        self.text.add_row("text", ui);
        self.save_time.add_row("saveTime", ui);
    }
}

impl InfoUi for TrainStation {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let obj = self.deref_mut();
        obj.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("TrainStation");
            ui.separator();
            self.add_grid("train_station_grid", ui, flags);
        });
    }
}

impl BuildDwMesh for TrainStation {
    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        builder.add_icon(DwIcon::new(self.float_pos, ItemType::TrainStation));
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
    fn to_grid(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        if self.gem_tree_type.add_row("gemTreeType", ui).changed() {
            *flags |= ObjFlags::RebuildMesh;
        }
        self.fruit_year.add_row("fruitYear", ui);
    }
}

impl InfoUi for GemTree {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let tree = self.deref_mut();
        tree.info(ui, flags);

        ui.vertical(|ui| {
            ui.heading("GemTree");
            ui.separator();
            self.add_grid("gem_tree_grid", ui, flags);
        });
    }
}

impl BuildDwMesh for GemTree {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 1, quads: 0 });

    fn build_dw_mesh(&self, builder: &mut DwChunkBufBuilder) -> Result<(), BuildDwMeshError> {
        let item_type = match self.gem_tree_type {
            TreeType::Amethyst => ItemType::Amethyst,
            TreeType::Sapphire => ItemType::Sapphire,
            TreeType::Emerald => ItemType::Emerald,
            TreeType::Ruby => ItemType::Ruby,
            TreeType::Diamond => ItemType::Diamond,
            _ => ItemType::Unknown,
        };
        builder.add_icon(DwIcon::new(self.float_pos, item_type));
        Ok(())
    }
}

impl InfoUi for TomatoPlant {
    fn info(&mut self, ui: &mut egui::Ui, flags: &mut ObjFlags) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui, flags);
    }
}

impl BuildDwMesh for TomatoPlant {
    const STATIC_CAPACITY: Option<DwCapacity> = Some(DwCapacity { icons: 0, quads: 1 });

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
