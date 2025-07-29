use super::dw::DwSpriteVertex;

pub struct Sprite {
    pub min: [f32; 2],
    pub max: [f32; 2],
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
    pub z: f32,
}

impl Sprite {
    pub const TILE_SIZE: f32 = 16.0 / 512.0;

    pub fn to_vertices(&self) -> ([DwSpriteVertex; 4], [u32; 6]) {
        let [min_x, min_y] = self.min;
        let [max_x, max_y] = self.max;
        let [u_min, v_min] = self.uv_min;
        let [u_max, v_max] = self.uv_max;
        (
            [
                DwSpriteVertex {
                    position: [min_x, min_y, self.z],
                    tex_coords: [u_min, v_max],
                }, // Bottom-left
                DwSpriteVertex {
                    position: [max_x, min_y, self.z],
                    tex_coords: [u_max, v_max],
                }, // Bottom-right
                DwSpriteVertex {
                    position: [max_x, max_y, self.z],
                    tex_coords: [u_max, v_min],
                }, // Top-right
                DwSpriteVertex {
                    position: [min_x, max_y, self.z],
                    tex_coords: [u_min, v_min],
                }, // Top-left
            ],
            [0, 1, 2, 0, 2, 3],
        )
    }
}

pub trait ToSprite {
    fn to_sprite(&self) -> Sprite;
}
