// Implements traits for dynamic object types defined in lib
use super::gpu::{
    dw::{DwIconInstanceRaw, ToIconInstance},
    sprite::{Sprite, ToSprite},
};
use the_blockheads_tools_lib::game::{
    dynamic_object::plant::{CarrotPlant, CornPlant, KelpPlant, TomatoPlant},
    item::ItemType,
};

impl ToIconInstance for CornPlant {
    fn to_icon_instance(&self) -> DwIconInstanceRaw {
        DwIconInstanceRaw {
            position: self.float_pos,
            item_type: ItemType::Corn as u32,
        }
    }
}

impl ToSprite for CornPlant {
    fn to_sprite(&self) -> Option<Sprite> {
        Some(Sprite::new_from_parts(
            if self.flowering { (19, 6) } else { (20, 6) },
            [0.5, 0.0],
            self.float_pos,
            [1.0, 2.0],
            2.0,
        ))
    }
}

impl ToIconInstance for CarrotPlant {
    fn to_icon_instance(&self) -> DwIconInstanceRaw {
        DwIconInstanceRaw {
            position: self.float_pos,
            item_type: ItemType::Carrot as u32,
        }
    }
}

impl ToSprite for CarrotPlant {
    fn to_sprite(&self) -> Option<Sprite> {
        Some(Sprite::new_from_parts(
            if self.flowering { (21, 6) } else { (22, 6) },
            [0.5, 0.0],
            self.float_pos,
            [1.0, 2.0],
            2.0,
        ))
    }
}

impl ToIconInstance for KelpPlant {
    fn to_icon_instance(&self) -> DwIconInstanceRaw {
        DwIconInstanceRaw {
            position: self.float_pos,
            item_type: ItemType::Kelp as u32,
        }
    }
}

impl ToSprite for KelpPlant {
    fn to_sprite(&self) -> Option<Sprite> {
        Some(Sprite::new_from_parts(
            (25, 6),
            [0.5, 0.0],
            self.float_pos,
            [1.0, 2.0],
            2.0,
        ))
    }
}

impl ToIconInstance for TomatoPlant {
    fn to_icon_instance(&self) -> DwIconInstanceRaw {
        DwIconInstanceRaw {
            position: self.float_pos,
            item_type: ItemType::Tomato as u32,
        }
    }
}

impl ToSprite for TomatoPlant {
    fn to_sprite(&self) -> Option<Sprite> {
        Some(Sprite::new_from_parts(
            if self.flowering { (27, 22) } else { (26, 22) },
            [0.5, 0.0],
            self.float_pos,
            [1.0, 2.0],
            2.0,
        ))
    }
}
