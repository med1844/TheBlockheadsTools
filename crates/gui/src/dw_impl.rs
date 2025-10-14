// Implements traits for dynamic object types defined in lib
use super::gpu::dw::{DwIconInstanceRaw, ToIconInstance};
use the_blockheads_tools_lib::game::{
    dw::dynamic_object::{CarrotPlant, CornPlant, TomatoPlant},
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

impl ToIconInstance for CarrotPlant {
    fn to_icon_instance(&self) -> DwIconInstanceRaw {
        DwIconInstanceRaw {
            position: self.float_pos,
            item_type: ItemType::Carrot as u32,
        }
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
