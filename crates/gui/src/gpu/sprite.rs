use super::dw::{DwChunkObjId, DwVertex};

pub struct Sprite {
    pub min: [f32; 2],
    pub max: [f32; 2],
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
    pub z: f32,
}

impl Sprite {
    pub const TILE_SIZE: f32 = 16.0 / 512.0;

    pub(crate) fn new_from_parts(
        uv_top_left: (u8, u8),
        local_center_pos: [f32; 2],
        global_center_pos: [f32; 2],
        sprite_size: [f32; 2],
        z: f32,
    ) -> Self {
        let (u_tile, v_tile) = uv_top_left;
        let [local_center_x_offset, local_center_y_offset] = local_center_pos;
        let [sprite_width, sprite_height] = sprite_size;
        let [global_center_x, global_center_y] = global_center_pos;

        let u_min = u_tile as f32 * Sprite::TILE_SIZE;
        let v_min = v_tile as f32 * Sprite::TILE_SIZE;
        let u_max = (u_tile as f32 + sprite_width) * Sprite::TILE_SIZE;
        let v_max = (v_tile as f32 + sprite_height) * Sprite::TILE_SIZE;

        let min_x = global_center_x - local_center_x_offset;
        let min_y = global_center_y - local_center_y_offset;

        let max_x = min_x + sprite_width;
        let max_y = min_y + sprite_height;

        Sprite {
            min: [min_x, min_y],
            max: [max_x, max_y],
            uv_min: [u_min, v_min],
            uv_max: [u_max, v_max],
            z,
        }
    }

    pub fn to_vertices(&self, id: DwChunkObjId) -> ([DwVertex; 4], [u32; 6]) {
        let [min_x, min_y] = self.min;
        let [max_x, max_y] = self.max;
        let [u_min, v_min] = self.uv_min;
        let [u_max, v_max] = self.uv_max;
        (
            [
                DwVertex {
                    id,
                    position: [min_x, min_y, self.z],
                    tex_coords: [u_min, v_max],
                }, // Bottom-left
                DwVertex {
                    id,
                    position: [max_x, min_y, self.z],
                    tex_coords: [u_max, v_max],
                }, // Bottom-right
                DwVertex {
                    id,
                    position: [max_x, max_y, self.z],
                    tex_coords: [u_max, v_min],
                }, // Top-right
                DwVertex {
                    id,
                    position: [min_x, max_y, self.z],
                    tex_coords: [u_min, v_min],
                }, // Top-left
            ],
            [0, 1, 2, 0, 2, 3],
        )
    }
}

pub trait ToSprite {
    fn to_sprite(&self) -> Option<Sprite>;
}
