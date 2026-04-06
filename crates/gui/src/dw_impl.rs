// Implements traits for dynamic object types defined in lib
use super::gpu::dw::{DwIcon, DwObj, DwSprite, ToDwObj};
use the_blockheads_tools_lib::game::{
    dynamic_object::{
        plant::{CarrotPlant, CornPlant, KelpPlant, TomatoPlant},
        tree::{
            AppleTree, CactusTree, CherryTree, CoconutTree, CoffeeTree, LimeTree, MangoTree,
            MapleTree, OrangeTree, PineTree,
        },
    },
    item::ItemType,
};

impl ToDwObj for AppleTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::Apple))
    }
}

impl ToDwObj for MapleTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::MapleSeed))
    }
}

impl ToDwObj for MangoTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::Mango))
    }
}

impl ToDwObj for PineTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::Pinecone))
    }
}

impl ToDwObj for CactusTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::PricklyPear))
    }
}

impl ToDwObj for CoconutTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::Coconut))
    }
}

impl ToDwObj for OrangeTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::Orange))
    }
}

impl ToDwObj for CherryTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::Cherry))
    }
}

impl ToDwObj for CoffeeTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::CoffeeCherry))
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

impl ToDwObj for LimeTree {
    fn to_dw_obj(&self) -> DwObj {
        DwObj::Icon(DwIcon::new(self.float_pos, ItemType::Lime))
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
