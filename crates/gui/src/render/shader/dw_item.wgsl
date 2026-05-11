//!include camera_uniform.wgsl

//!include id_uniform.wgsl

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var items_texture: texture_2d<f32>;
@group(0) @binding(2) var items_sampler: sampler;
@group(0) @binding(3) var tilemap_texture: texture_2d<f32>;
@group(0) @binding(4) var tilemap_sampler: sampler;
@group(0) @binding(5) var<uniform> hover_on_id: IdUniform;
@group(0) @binding(6) var<uniform> selected_id: IdUniform;

//!include render_settings.wgsl
@group(0) @binding(7) var<uniform> render_settings: RenderSettings;
@group(0) @binding(8) var<uniform> world_dim_x: u32;

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


struct DynObjVertexInput {
    @location(0) position: vec2<f32>,
};

struct DynObjInstanceInput {
    @location(1) instance_pos: vec2<f32>,
    @location(2) item_type_or_block: u32,
    @location(3) top_side: u32,
    @location(4) id: u32,
    @location(5) chunk: u32,
};

struct DynObjVSOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) item_type: u32,
    @location(2) @interpolate(flat) is_block: u32,
    @location(3) @interpolate(flat) top: u32,
    @location(4) @interpolate(flat) side: u32,
    @location(5) @interpolate(flat) id: u32,
    @location(6) @interpolate(flat) chunk: u32,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) id: vec2<u32>,
}

@vertex
fn vs_dynamic_object_item(model: DynObjVertexInput, instance: DynObjInstanceInput) -> DynObjVSOutput {
    var out: DynObjVSOutput;

    let world_pos = vec3<f32>(instance.instance_pos + model.position, 2.0);
    var pos_in_view = world_pos - camera.world_offset.xyz;

    if (render_settings.enable_cyclic == 1u) {
        let world_width = f32(world_dim_x);
        pos_in_view.x -= world_width * round(pos_in_view.x / world_width);
    }

    out.clip_position = camera.view_proj * vec4<f32>(pos_in_view, 1.0);

    // Pass model position to fragment shader for UV calculation
    // Remap from [-0.5, 0.5] to [0, 1]
    out.uv = model.position + 0.5;
    out.uv.y = 1 - out.uv.y;
    out.item_type = instance.item_type_or_block & 0xFFFF;
    out.is_block = (instance.item_type_or_block >> 16) & 1;
    out.top = instance.top_side & 0xFFFF;
    out.side = instance.top_side >> 16;
    out.id = instance.id;
    out.chunk = instance.chunk;

    return out;
}

// --- 2.5D Block Item Rendering Logic ---
// This function simulates an orthographic view of a cube.
fn render_block_item(uv: vec2<f32>, top: u32, side: u32) -> vec4<f32> {
    // Define a fixed orthographic view projection for the icon
    let view_dir = normalize(vec3<f32>(0.5, 0.25, 0.75));
    let up = vec3<f32>(0.0, 1.0, 0.0);
    let right = normalize(cross(view_dir, up));
    let ortho_up = normalize(cross(right, view_dir));

    let block_scale: f32 = 0.6;

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
        discard;
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

    // Apply lighting similar to voxel.wgsl
    if (surface_color.a > 0.0) {
        let light_direction = normalize(vec3<f32>(0.0, -1.0, -0.5));
        let ambient_light = 0.05;
        let diffuse_factor = max(dot(face_normal, light_direction), 0.0);
        let final_light_factor = ambient_light + (1.0 - ambient_light) * diffuse_factor;
        let final_rgb = surface_color.rgb * final_light_factor; // Apply lighting
        surface_color = vec4<f32>(final_rgb, surface_color.a);
    }
    return surface_color;
}

@fragment
fn fs_dynamic_object_item(in: DynObjVSOutput) -> FragmentOutput {
    var color: vec4<f32>;

    if (in.is_block != 0) {
        // It's a block, render it as a 2.5D icon
        color = render_block_item(in.uv, in.top, in.side);
    } else {
        // It's a regular item, use the Items texture
        let tile_index = in.item_type;
        let tile_x = f32(tile_index % ITEMS_TILES_PER_ROW);
        let tile_y = f32(tile_index / ITEMS_TILES_PER_ROW);
        let uv_min_tile = vec2<f32>(tile_x * ITEMS_TILE_SIZE_UV.x, tile_y * ITEMS_TILE_SIZE_UV.y);

        let clenched_uv = in.uv * 0.998 + 0.001; // inset by 0.1% to avoid atlas bleeding
        let final_atlas_uv = uv_min_tile + clenched_uv * ITEMS_TILE_SIZE_UV;
        color = textureSampleLevel(items_texture, items_sampler, final_atlas_uv, 0.0);
    }

    // Add outline
    let outline_width: f32 = 0.03; // Adjust as needed for desired thickness
    let dist_from_edge_x = min(in.uv.x, 1.0 - in.uv.x);
    let dist_from_edge_y = min(in.uv.y, 1.0 - in.uv.y);

    if (dist_from_edge_x < outline_width || dist_from_edge_y < outline_width) {
        let outline_alpha: f32 = 0.5;
        let outline_color: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, outline_alpha); // White with 0.5 alpha

        // Simple blend, overlaying the outline
        // You might want a more sophisticated blend mode depending on desired effect
        color = mix(color, outline_color, outline_color.a);
    }

    let hovered = hover_on_id.is_some != 0u
        && in.chunk == hover_on_id.chunk
        && in.id == hover_on_id.id;

    var final_color: vec3<f32> = color.rgb;
    if (hovered) {
        final_color = mix(final_color, vec3<f32>(1.0), 0.15);
    }

    let selected = selected_id.is_some != 0u
        && in.chunk == selected_id.chunk
        && in.id == selected_id.id;

    if (selected) {
        final_color = mix(final_color, vec3<f32>(1.0, 1.0, 0.0), 0.25);
    }

    var output: FragmentOutput;
    output.color = vec4<f32>(final_color, color.a);
    output.id = vec2<u32>(in.id, in.chunk);
    return output;
}
