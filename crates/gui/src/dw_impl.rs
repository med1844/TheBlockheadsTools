// Implements traits for dynamic object types defined in lib
use super::gpu::{
    VoxelType,
    dw::{DwBlock, DwIcon, DwObj, DwSprite, ToDwObj},
};
use eframe::egui;
use std::{hash::Hash, ops::DerefMut};
use the_blockheads_tools_lib::game::{
    coord::BlockCoord,
    dynamic_object::{
        ArtificialLight, DynamicObject, InteractionObject, InteractionObjectType, LightDirection,
        UniqueID,
        chest::{Chest, ChestType},
        plant::{CarrotPlant, CornPlant, KelpPlant, NormalPlant, Plant, TomatoPlant},
        tree::{
            AppleTree, CactusTree, CherryTree, CoconutTree, CoffeeTree, GemTree, LimeTree,
            MangoTree, MapleTree, OrangeTree, PineTree, Tree, TreeFruit, TreeType,
        },
    },
    item::ItemType,
};

trait ToRow {
    fn to_row(&mut self, ui: &mut egui::Ui);

    fn add_row(&mut self, label: &str, ui: &mut egui::Ui) {
        ui.label(label);
        self.to_row(ui);
        ui.end_row();
    }
}

impl ToRow for [f32; 2] {
    fn to_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut self[0]).speed(0.1).prefix("X: "));
            ui.add(egui::DragValue::new(&mut self[1]).speed(0.1).prefix("Y: "));
        });
    }
}

impl ToRow for u64 {
    fn to_row(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::DragValue::new(self));
    }
}

impl ToRow for u32 {
    fn to_row(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::DragValue::new(self));
    }
}

impl ToRow for u16 {
    fn to_row(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::DragValue::new(self));
    }
}

impl ToRow for i32 {
    fn to_row(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::DragValue::new(self));
    }
}

impl ToRow for UniqueID {
    fn to_row(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("{:?}", self));
    }
}

impl ToRow for String {
    fn to_row(&mut self, ui: &mut egui::Ui) {
        ui.text_edit_singleline(self);
    }
}

impl<T: ToRow + Default> ToRow for Option<T> {
    fn to_row(&mut self, ui: &mut egui::Ui) {
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
                t.to_row(ui);
            } else {
                ui.weak("None");
            }
        });
    }
}

impl ToRow for f32 {
    fn to_row(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::DragValue::new(self));
    }
}

impl ToRow for f64 {
    fn to_row(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::DragValue::new(self));
    }
}

impl ToRow for bool {
    fn to_row(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(self, "");
    }
}

trait ToGrid {
    fn to_grid(&mut self, ui: &mut egui::Ui);
    fn add_grid<H: Hash>(&mut self, id: H, ui: &mut egui::Ui) {
        egui::Grid::new(id).num_columns(2).show(ui, |ui| {
            self.to_grid(ui);
        });
    }
}

impl ToGrid for DynamicObject {
    fn to_grid(&mut self, ui: &mut egui::Ui) {
        self.float_pos.add_row("floatPos", ui);
        self.pos_x.add_row("pos_x", ui);
        self.pos_y.add_row("pos_y", ui);
        self.unique_id.add_row("uniqueID", ui);
        self.owner_id.add_row("ownerID", ui);
    }
}

impl<T: Default + ToGrid> ToRow for Vec<T> {
    fn to_row(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            let mut to_remove = None;

            for (i, item) in self.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if ui.button("Del").on_hover_text("Remove item").clicked() {
                        to_remove = Some(i);
                    }

                    ui.collapsing(format!("Item #{}", i), |ui| {
                        item.add_grid(format!("item_grid_{}", i), ui);
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

impl<T: ToGrid> ToRow for T {
    fn to_row(&mut self, ui: &mut egui::Ui) {
        // Indent or frame the nested grid so it's visually distinct
        // from the parent grid rows.
        ui.indent("inner_grid_indent", |ui| {
            self.add_grid("inner_grid", ui);
        });
    }
}

impl ToGrid for TreeFruit {
    fn to_grid(&mut self, ui: &mut egui::Ui) {
        self.has_created_free_block_this_season
            .add_row("hasCreatedFreeBlockThisSeason", ui);
        self.pos_x.add_row("pos.x", ui);
        self.pos_y.add_row("pos.y", ui);
    }
}

impl ToGrid for Tree {
    fn to_grid(&mut self, ui: &mut egui::Ui) {
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
        self.tree_fruits.add_row("treeFruits", ui);
    }
}

pub(crate) trait InfoUi {
    fn info(&mut self, ui: &mut egui::Ui);
}

impl InfoUi for DynamicObject {
    fn info(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("DynamicObject");
            ui.separator();
            self.add_grid("dynamic_object_grid", ui);
        });
    }
}

impl InfoUi for Tree {
    fn info(&mut self, ui: &mut egui::Ui) {
        let obj = self.deref_mut();
        obj.info(ui);

        ui.vertical(|ui| {
            ui.heading("Tree");
            ui.separator();
            self.add_grid("tree_grid", ui);
        });
    }
}

impl ToDwObj for AppleTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::Apple))
    }
}

impl ToGrid for AppleTree {
    fn to_grid(&mut self, ui: &mut egui::Ui) {
        self.available_food.add_row("availableFood", ui);
    }
}

impl InfoUi for AppleTree {
    fn info(&mut self, ui: &mut egui::Ui) {
        let tree = self.deref_mut();
        tree.info(ui);

        ui.vertical(|ui| {
            ui.heading("AppleTree");
            ui.separator();
            self.add_grid("apple_tree_grid", ui);
        });
    }
}

impl ToDwObj for MapleTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::MapleSeed))
    }
}

impl InfoUi for MapleTree {
    fn info(&mut self, ui: &mut egui::Ui) {
        let tree = self.deref_mut();
        tree.info(ui);
    }
}

impl ToDwObj for MangoTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::Mango))
    }
}

impl InfoUi for MangoTree {
    fn info(&mut self, ui: &mut egui::Ui) {
        let tree = self.deref_mut();
        tree.info(ui);
    }
}

impl ToGrid for PineTree {
    fn to_grid(&mut self, ui: &mut egui::Ui) {
        self.available_food.add_row("availableFood", ui);
    }
}

impl InfoUi for PineTree {
    fn info(&mut self, ui: &mut egui::Ui) {
        let tree = self.deref_mut();
        tree.info(ui);

        ui.vertical(|ui| {
            ui.heading("PineTree");
            ui.separator();
            self.add_grid("pine_tree_grid", ui);
        });
    }
}

impl ToDwObj for PineTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::Pinecone))
    }
}

impl ToGrid for CactusTree {
    fn to_grid(&mut self, ui: &mut egui::Ui) {
        self.available_food.add_row("availableFood", ui);
        self.split_direction.add_row("splitDirection", ui);
        self.split_height_a.add_row("splitHeightA", ui);
        self.split_height_b.add_row("splitHeightB", ui);
    }
}

impl InfoUi for CactusTree {
    fn info(&mut self, ui: &mut egui::Ui) {
        let tree = self.deref_mut();
        tree.info(ui);

        ui.vertical(|ui| {
            ui.heading("CactusTree");
            ui.separator();
            self.add_grid("cactus_tree_grid", ui);
        });
    }
}

impl ToDwObj for CactusTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::PricklyPear))
    }
}

impl InfoUi for CoconutTree {
    fn info(&mut self, ui: &mut egui::Ui) {
        let tree = self.deref_mut();
        tree.info(ui);
    }
}

impl ToDwObj for CoconutTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::Coconut))
    }
}

impl InfoUi for OrangeTree {
    fn info(&mut self, ui: &mut egui::Ui) {
        let tree = self.deref_mut();
        tree.info(ui);
    }
}

impl ToDwObj for OrangeTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::Orange))
    }
}

impl InfoUi for CherryTree {
    fn info(&mut self, ui: &mut egui::Ui) {
        let tree = self.deref_mut();
        tree.info(ui);
    }
}

impl ToDwObj for CherryTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::Cherry))
    }
}

impl InfoUi for CoffeeTree {
    fn info(&mut self, ui: &mut egui::Ui) {
        let tree = self.deref_mut();
        tree.info(ui);
    }
}

impl ToDwObj for CoffeeTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::CoffeeCherry))
    }
}

impl ToGrid for Plant {
    fn to_grid(&mut self, ui: &mut egui::Ui) {
        self.save_time.add_row("saveTime", ui);
        self.season_offset.add_row("seasonOffset", ui);
        self.gather_progress.add_row("gatherProgress", ui);
        self.has_flowered_this_season
            .add_row("hasFloweredThisSeason", ui);
        self.flowering.add_row("flowering", ui);
        self.frozen.add_row("frozen", ui);
        self.age.add_row("age", ui);
        self.max_age.add_row("maxAge", ui);
        self.max_age_gene.add_row("maxAgeGene", ui);
        self.growth_rate.add_row("growthRate", ui);
        self.growth_rate_gene.add_row("growthRateGene", ui);
    }
}

impl InfoUi for Plant {
    fn info(&mut self, ui: &mut egui::Ui) {
        let obj = self.deref_mut();
        obj.info(ui);

        ui.vertical(|ui| {
            ui.heading("Plant");
            ui.separator();
            self.add_grid("plant_grid", ui);
        });
    }
}

impl ToRow for LightDirection {
    fn to_row(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("Select one")
            .selected_text(format!("{:?}", self))
            .show_ui(ui, |ui| {
                ui.selectable_value(self, Self::All, "All");
                ui.selectable_value(self, Self::Up, "Up");
                ui.selectable_value(self, Self::Down, "Down");
            });
    }
}

impl ToGrid for ArtificialLight {
    fn to_grid(&mut self, ui: &mut egui::Ui) {
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
    fn to_grid(&mut self, ui: &mut egui::Ui) {
        self.available_food.add_row("saveTime", ui);
        self.light_dict.add_row("lightDict", ui);
    }
}

impl InfoUi for NormalPlant {
    fn info(&mut self, ui: &mut egui::Ui) {
        let plant = self.deref_mut();
        plant.info(ui);

        ui.vertical(|ui| {
            ui.heading("NormalPlant");
            ui.separator();
            self.add_grid("normal_plant_grid", ui);
        });
    }
}

impl InfoUi for CornPlant {
    fn info(&mut self, ui: &mut egui::Ui) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui);
    }
}

impl ToDwObj for CornPlant {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Sprite(DwSprite::new_from_parts(
            if self.flowering { (19, 6) } else { (20, 6) },
            [0.5, 0.0],
            self.float_pos,
            [1.0, 2.0],
            2.0,
        ))
    }
}

impl InfoUi for CarrotPlant {
    fn info(&mut self, ui: &mut egui::Ui) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui);
    }
}

impl ToDwObj for CarrotPlant {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Sprite(DwSprite::new_from_parts(
            if self.flowering { (21, 6) } else { (22, 6) },
            [0.5, 0.0],
            self.float_pos,
            [1.0, 2.0],
            2.0,
        ))
    }
}

impl ToGrid for KelpPlant {
    fn to_grid(&mut self, ui: &mut egui::Ui) {
        self.growth_timer.add_row("growthTimer", ui);
        self.number_of_occupied_tiles_above
            .add_row("numberOfOccupiedTilesAbove", ui);
    }
}

impl InfoUi for KelpPlant {
    fn info(&mut self, ui: &mut egui::Ui) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui);

        ui.vertical(|ui| {
            ui.heading("KelpPlant");
            ui.separator();
            self.add_grid("kelp_plant_grid", ui);
        });
    }
}

impl ToDwObj for KelpPlant {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Sprite(DwSprite::new_from_parts(
            (25, 6),
            [0.5, 0.0],
            self.float_pos,
            [1.0, 2.0],
            2.0,
        ))
    }
}

impl InfoUi for LimeTree {
    fn info(&mut self, ui: &mut egui::Ui) {
        let tree = self.deref_mut();
        tree.info(ui);
    }
}

impl ToDwObj for LimeTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::Lime))
    }
}

impl ToRow for InteractionObjectType {
    fn to_row(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("Select one")
            .selected_text(format!("{:?}", self))
            .show_ui(ui, |ui| {
                ui.selectable_value(self, Self::InteractionObject, "InteractionObject");
                ui.selectable_value(self, Self::Workbench, "Workbench");
                ui.selectable_value(self, Self::Chest, "Chest");
                ui.selectable_value(self, Self::Bed, "Bed");
                ui.selectable_value(self, Self::Sign, "Sign");
                ui.selectable_value(self, Self::TradingPost, "TradingPost");
                ui.selectable_value(self, Self::TrainStation, "TrainStation");
                ui.selectable_value(self, Self::TradePortal, "TradePortal");
                ui.selectable_value(self, Self::OwnershipSign, "OwnershipSign");
                ui.selectable_value(self, Self::Mirror, "Mirror");
            });
    }
}

impl ToGrid for InteractionObject {
    fn to_grid(&mut self, ui: &mut egui::Ui) {
        self.interaction_object_type
            .add_row("interactionObjectType", ui);
        self.is_in_use.add_row("isInUse", ui);
        self.flipped.add_row("flipped", ui);
        self.paint_color.add_row("paintColor", ui);
    }
}

impl InfoUi for InteractionObject {
    fn info(&mut self, ui: &mut egui::Ui) {
        let obj = self.deref_mut();
        obj.info(ui);

        ui.vertical(|ui| {
            ui.heading("InteractionObject");
            ui.separator();
            self.add_grid("interaction_object_grid", ui);
        });
    }
}

impl ToGrid for Chest {
    fn to_grid(&mut self, ui: &mut egui::Ui) {
        self.save_time.add_row("saveTime", ui);
        // TODO find a proper way to display the items
    }
}

impl InfoUi for Chest {
    fn info(&mut self, ui: &mut egui::Ui) {
        let obj = self.deref_mut();
        obj.info(ui);

        ui.vertical(|ui| {
            ui.heading("Chest");
            ui.separator();
            self.add_grid("chest_grid", ui);
        });
    }
}

impl ToDwObj for Chest {
    fn to_dw_obj(&self) -> DwObj {
        // TODO: make this fallable
        let block_coord = BlockCoord::new(self.pos_x as u32, self.pos_y)
            .expect("dynamic object size out of world bound");
        match self.slots.chest_type() {
            ChestType::Standard => {
                DwObj::Block(DwBlock::new(block_coord, VoxelType::StandardChest))
            }
            ChestType::Safe => DwObj::Block(DwBlock::new(block_coord, VoxelType::Safe)),
            ChestType::Shelf => DwObj::Icon(DwIcon::new(self.float_pos, ItemType::Shelf)),
            ChestType::Gold => DwObj::Block(DwBlock::new(block_coord, VoxelType::GoldChest)),
            ChestType::Portal => DwObj::Block(DwBlock::new(block_coord, VoxelType::PortalChest)),
            ChestType::Cabinet => {
                DwObj::Icon(DwIcon::new(self.float_pos, ItemType::DisplayCabinet))
            }
            ChestType::Feeder => DwObj::Block(DwBlock::new(block_coord, VoxelType::FeederChest)),
        }
    }
}

impl ToRow for TreeType {
    fn to_row(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("Select one")
            .selected_text(format!("{:?}", self))
            .show_ui(ui, |ui| {
                ui.selectable_value(self, Self::Nothing, "Nothing");
                ui.selectable_value(self, Self::Apple, "Apple");
                ui.selectable_value(self, Self::Mango, "Mango");
                ui.selectable_value(self, Self::Maple, "Maple");
                ui.selectable_value(self, Self::Pine, "Pine");
                ui.selectable_value(self, Self::Cactus, "Cactus");
                ui.selectable_value(self, Self::Coconut, "Coconut");
                ui.selectable_value(self, Self::Orange, "Orange");
                ui.selectable_value(self, Self::Cherry, "Cherry");
                ui.selectable_value(self, Self::Coffee, "Coffee");
                ui.selectable_value(self, Self::Lime, "Lime");
                ui.selectable_value(self, Self::Amethyst, "Amethyst");
                ui.selectable_value(self, Self::Sapphire, "Sapphire");
                ui.selectable_value(self, Self::Emerald, "Emerald");
                ui.selectable_value(self, Self::Ruby, "Ruby");
                ui.selectable_value(self, Self::Diamond, "Diamond");
            });
    }
}

impl ToGrid for GemTree {
    fn to_grid(&mut self, ui: &mut egui::Ui) {
        self.gem_tree_type.add_row("gemTreeType", ui);
        self.fruit_year.add_row("fruitYear", ui);
    }
}

impl InfoUi for GemTree {
    fn info(&mut self, ui: &mut egui::Ui) {
        let tree = self.deref_mut();
        tree.info(ui);

        ui.vertical(|ui| {
            ui.heading("GemTree");
            ui.separator();
            self.add_grid("gem_tree_grid", ui);
        });
    }
}

impl ToDwObj for GemTree {
    fn to_dw_obj(&self) -> DwObj {
        let item_type = match self.gem_tree_type {
            TreeType::Amethyst => ItemType::Amethyst,
            TreeType::Sapphire => ItemType::Sapphire,
            TreeType::Emerald => ItemType::Emerald,
            TreeType::Ruby => ItemType::Ruby,
            TreeType::Diamond => ItemType::Diamond,
            _ => ItemType::Unknown,
        };
        DwObj::Icon(DwIcon::new(self.float_pos, item_type))
    }
}

impl InfoUi for TomatoPlant {
    fn info(&mut self, ui: &mut egui::Ui) {
        let normal_plant = self.deref_mut();
        normal_plant.info(ui);
    }
}

impl ToDwObj for TomatoPlant {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Sprite(DwSprite::new_from_parts(
            if self.flowering { (27, 22) } else { (26, 22) },
            [0.5, 0.0],
            self.float_pos,
            [1.0, 2.0],
            2.0,
        ))
    }
}
