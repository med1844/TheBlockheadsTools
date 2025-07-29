// Implements traits for dynamic object types defined in lib
use super::gpu::{
    Sprite, ToSprite,
    dw::{DwIconInstanceRaw, ToIconInstance},
};
use the_blockheads_tools_lib::game::{dw::dynamic_object::TomatoPlant, item::ItemType};

impl ToIconInstance for TomatoPlant {
    fn to_icon_instance(&self) -> DwIconInstanceRaw {
        DwIconInstanceRaw {
            position: self.float_pos,
            item_type: ItemType::Tomato as u32,
        }
    }
}

impl ToSprite for TomatoPlant {
    fn to_sprite(&self) -> Sprite {
        let [center_x, center_y] = self.0.obj.float_pos;
        let min = [center_x - 0.5, center_y];
        let max = [center_x + 0.5, center_y + 2.0];
        let (x, y) = if self.flowering { (27, 22) } else { (26, 22) };
        let u_min = x as f32 * Sprite::TILE_SIZE;
        let u_max = (x + 1) as f32 * Sprite::TILE_SIZE;
        let v_min = y as f32 * Sprite::TILE_SIZE;
        let v_max = (y + 2) as f32 * Sprite::TILE_SIZE;
        Sprite {
            min,
            max,
            uv_min: [u_min, v_min],
            uv_max: [u_max, v_max],
            z: 2.0,
        }
    }
}
