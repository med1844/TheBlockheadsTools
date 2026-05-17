// --- Item Texture Atlas Constants ---
const ITEMS_ATLAS_DIM_PX: vec2<f32> = vec2<f32>(512.0, 256.0);
const ITEMS_TILE_DIM_PX: f32 = 16.0;
const ITEMS_TILES_PER_ROW: u32 = 32u;
const ITEMS_TILE_SIZE_UV: vec2<f32> = vec2<f32>(
    ITEMS_TILE_DIM_PX / ITEMS_ATLAS_DIM_PX.x,
    ITEMS_TILE_DIM_PX / ITEMS_ATLAS_DIM_PX.y
);

// --- Voxel/Block Texture Atlas Constants (for blocks rendered as items) ---
const VOXEL_ATLAS_DIM_PX: f32 = 512.0;
const VOXEL_TILE_DIM_PX: f32 = 16.0;
const VOXEL_TILES_PER_ROW: u32 = 32u;
const VOXEL_TILE_SIZE_UV: f32 = VOXEL_TILE_DIM_PX / VOXEL_ATLAS_DIM_PX;

// --- 2.5D Block Item Rendering Logic ---
// This function simulates an orthographic view of a cube.
fn render_block_item(uv: vec2<f32>, top: u32, side: u32) -> vec4<f32> {
    // Define a fixed orthographic view projection for the icon
    let view_dir = normalize(vec3<f32>(0.5, 0.25, 0.75));
    let up = vec3<f32>(0.0, 1.0, 0.0);
    let right = normalize(cross(view_dir, up));
    let ortho_up = normalize(cross(right, view_dir));

    let block_scale: f32 = 0.5;

    // Convert fragment's UV coords to a ray origin
    let ray_origin = vec3<f32>(
        (uv.x - 0.5) * right.x + (uv.y - 0.5) * ortho_up.x,
        (uv.x - 0.5) * right.y + (uv.y - 0.5) * ortho_up.y,
        (uv.x - 0.5) * right.z + (uv.y - 0.5) * ortho_up.z
    ) / block_scale - view_dir * 2.0;

    let ray_dir = view_dir;

    // Intersect ray with an axis-aligned unit cube centered at (0,0,0)
    let inv_dir = 1.0 / ray_dir;
    let t1 = (vec3<f32>(-0.5) - ray_origin) * inv_dir;
    let t2 = (vec3<f32>(0.5) - ray_origin) * inv_dir;
    let tmin = max(max(min(t1.x, t2.x), min(t1.y, t2.y)), min(t1.z, t2.z));
    let tmax = min(min(max(t1.x, t2.x), max(t1.y, t2.y)), max(t1.z, t2.z));

    if (tmin > tmax) {
        return vec4<f32>(0.0);
    }

    let hit_pos = ray_origin + tmin * ray_dir;
    let abs_hit_pos = abs(hit_pos);

    var tile_index: u32;
    var face_uv: vec2<f32>;
    var face_normal: vec3<f32>; // Added to store the face normal

    // Determine which face was hit and calculate its UVs
    // We will only hit 3 of them but let's just keep the code as-is
    if (abs_hit_pos.y > abs_hit_pos.x && abs_hit_pos.y > abs_hit_pos.z) {
        // Top or Bottom face (PY or NY)
        tile_index = top; // fixed perspective, impossible to hit bottom
        face_uv = hit_pos.xz + 0.5;
        face_normal = select(vec3<f32>(0.0, -1.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), hit_pos.y > 0.0); // Set normal
    } else if (abs_hit_pos.x > abs_hit_pos.z) {
        // Front or Back face (PX or NX)
        tile_index = side;
        face_uv = vec2<f32>(hit_pos.z + 0.5, 0.5 - hit_pos.y);
        face_normal = select(vec3<f32>(-1.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), hit_pos.x > 0.0); // Set normal
    } else {
        // Right or Left face (PZ or NZ, but we map to our own indices)
        tile_index = side;
        face_uv = vec2<f32>(0.5 - hit_pos.x, 0.5 - hit_pos.y);
        face_normal = select(vec3<f32>(0.0, 0.0, -1.0), vec3<f32>(0.0, 0.0, 1.0), hit_pos.z > 0.0); // Set normal
    }
    face_uv.y = 1.0 - face_uv.y;

    // Now sample the tilemap using the determined face
    let tile_x = f32(tile_index % VOXEL_TILES_PER_ROW);
    let tile_y = f32(tile_index / VOXEL_TILES_PER_ROW);
    let uv_min_tile = vec2<f32>(tile_x * VOXEL_TILE_SIZE_UV, tile_y * VOXEL_TILE_SIZE_UV);

    let clenched_uv = face_uv * 0.998 + 0.001; // inset by 0.1% to avoid atlas bleeding
    let final_atlas_uv = uv_min_tile + clenched_uv * VOXEL_TILE_SIZE_UV;
    var surface_color = textureSampleLevel(tilemap_texture, tilemap_sampler, final_atlas_uv, 0.0);

    // Apply lighting
    if (surface_color.a > 0.0) {
        var perturbed_normal = face_normal;
        if (render_settings.enable_destruct != 0u) {
            let destruct_color = textureSampleLevel(tile_destruct, tile_destruct_sampler, final_atlas_uv, 0.0).rgb;
            perturbed_normal = perturb_normal(face_normal, destruct_color);
        }
        let icon_light_dir = vec3<f32>(0.0, -1.0, -0.5);
        let reflect_val = textureSampleLevel(tile_reflect, tile_reflect_sampler, final_atlas_uv, 0.0).r;
        let lit = calculate_lighting(
            icon_light_dir,
            vec3<f32>(0.0), // no world position for icons
            perturbed_normal,
            surface_color.rgb,
            reflect_val,
            1.0, // no SSAO for item
            1.0, // no depth darkening for icons
        );
        surface_color = vec4<f32>(lit, surface_color.a);
    }
    return surface_color;
}

fn sample_item_texture(item_type: u32, is_block: u32, top: u32, side: u32, uv: vec2<f32>) -> vec4<f32> {
    var color: vec4<f32>;

    if (is_block != 0u) {
        // It's a block, render it as a 2.5D icon
        color = render_block_item(uv, top, side);
    } else {
        // It's a regular item, use the Items texture
        let tile_index = item_type;
        let tile_x = f32(tile_index % ITEMS_TILES_PER_ROW);
        let tile_y = f32(tile_index / ITEMS_TILES_PER_ROW);
        let uv_min_tile = vec2<f32>(tile_x * ITEMS_TILE_SIZE_UV.x, tile_y * ITEMS_TILE_SIZE_UV.y);

        let clenched_uv = uv * 0.998 + 0.001; // inset by 0.1% to avoid atlas bleeding
        let final_atlas_uv = uv_min_tile + clenched_uv * ITEMS_TILE_SIZE_UV;
        color = textureSampleLevel(items_texture, items_sampler, final_atlas_uv, 0.0);
    }
    return color;
}
