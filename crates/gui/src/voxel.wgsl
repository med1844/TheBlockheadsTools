// --- Global Constants (must match Rust constants) ---
const CHUNK_DIM_X: u32 = 32u;
const CHUNK_DIM_Y: u32 = 32u;
const CHUNK_DIM_Z: u32 = 3u;
const CHUNK_VOXEL_COUNT: u32 = CHUNK_DIM_X * CHUNK_DIM_Y * CHUNK_DIM_Z;
const VOXEL_SIZE: f32 = 1.0;

// World dimensions in chunks
const WORLD_CHUNKS_X: u32 = 512u;
const WORLD_CHUNKS_Y: u32 = 32u;

// The total number of voxels along each axis of the world
const WORLD_DIM_X: u32 = CHUNK_DIM_X * WORLD_CHUNKS_X;
const WORLD_DIM_Y: u32 = CHUNK_DIM_Y * WORLD_CHUNKS_Y;
const WORLD_DIM_Z: u32 = CHUNK_DIM_Z; // World is flat, only one chunk deep in Z

// Iterate finite amount of steps or GPU on fire
const MAX_VOXEL_TRAVERSAL_STEPS: u32 = 10u;

// Face IDs are not used for coloring anymore but are good for reference
const FACE_PX: u32 = 0u;
const FACE_NX: u32 = 1u;
const FACE_PY: u32 = 2u;
const FACE_NY: u32 = 3u;
const FACE_PZ: u32 = 4u;
const FACE_NZ: u32 = 5u;

// --- Texture Atlas Constants ---
const TEXTURE_ATLAS_DIM_PX: f32 = 512.0; // Total width/height of the atlas in pixels
const TILE_DIM_PX: f32 = 16.0;           // Width/height of a single tile in pixels
const TILES_PER_ROW: u32 = 32u;          // Number of tiles per row (512 / 16 = 32)
const TILE_SIZE_UV: f32 = TILE_DIM_PX / TEXTURE_ATLAS_DIM_PX; // Normalized UV size of one tile (16/512 = 0.03125)

const AIR_TYPE: u32 = 2u;

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
    inv_direction: vec3<f32>,
}

struct BoundingBoxIntersection {
    hit: bool,
    t_min: f32,
    t_max: f32,
}

struct DDAState {
    current_voxel: vec3<i32>,
    step_dir: vec3<f32>,
    t_max: vec3<f32>,
    t_delta: vec3<f32>,
    face_normal: vec3<i32>,
}

struct VoxelSurface {
    voxel_type: u32,
    hit_point: vec3<f32>,
    normal: vec3<i32>,
    distance: f32,
}

struct TraversalResult {
    hit_solid: bool,
    solid_surface: VoxelSurface,
    accumulated_transparent_color: vec4<f32>,
    hit_selected_block: bool,
    hit_hovered_block: bool,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>, // xyz
    world_offset: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<storage, read> voxel_data: array<u32>;

@group(0) @binding(2)
var texture_atlas: texture_2d<f32>;

@group(0) @binding(3)
var texture_sampler: sampler;

@group(0) @binding(4)
var<storage, read> texture_uv_atlas_indices: array<u32>; // UV atlas storing single u16 tile indices

@group(0) @binding(5)
var<uniform> selected_block: vec4<u32>;

@group(0) @binding(6)
var<uniform> hover_on_block: vec4<u32>;

@group(0) @binding(7)
var<storage, read> is_transparent_buffer: array<u32>;

@group(1) @binding(0) var mesh_depth_texture: texture_depth_2d;
@group(1) @binding(1) var mesh_depth_sampler: sampler;

fn get_voxel_type(global_voxel_coords: vec3<i32>) -> u32 {
    if global_voxel_coords.x < 0 || global_voxel_coords.x >= i32(WORLD_DIM_X) ||
       global_voxel_coords.y < 0 || global_voxel_coords.y >= i32(WORLD_DIM_Y) ||
       global_voxel_coords.z < 0 || global_voxel_coords.z >= i32(WORLD_DIM_Z) {
        return AIR_TYPE; // everything outside of the world is air
    }

    let chunk_coord_x = u32(global_voxel_coords.x) / CHUNK_DIM_X;
    let chunk_coord_y = u32(global_voxel_coords.y) / CHUNK_DIM_Y;

    let local_voxel_coord_x = u32(global_voxel_coords.x) % CHUNK_DIM_X;
    let local_voxel_coord_y = u32(global_voxel_coords.y) % CHUNK_DIM_Y;
    let local_voxel_coord_z = u32(global_voxel_coords.z);

    let chunk_offset = (chunk_coord_x * WORLD_CHUNKS_Y + chunk_coord_y) * CHUNK_VOXEL_COUNT;

    let local_voxel_index = local_voxel_coord_z +
                            local_voxel_coord_x * CHUNK_DIM_Z +
                            local_voxel_coord_y * CHUNK_DIM_Z * CHUNK_DIM_X;

    let final_index = chunk_offset + local_voxel_index;

    let data = voxel_data[final_index >> 1];
    if ((final_index & 1) != 0) {
        return data >> 16;
    } else {
        return data & 0xFFFFu;
    }
}

fn sample_texture_by_face(voxel_type: u32, hit_face_id: u32, uv_on_face: vec2<f32>) -> vec4<f32> {
    let atlas_index_lookup = voxel_type * 6u + hit_face_id;
    let tile_index = texture_uv_atlas_indices[atlas_index_lookup];
    let tile_x = f32(tile_index % TILES_PER_ROW);
    let tile_y = f32(tile_index / TILES_PER_ROW);
    let uv_min_tile = vec2<f32>(tile_x * TILE_SIZE_UV, tile_y * TILE_SIZE_UV);
    let final_atlas_uv = uv_min_tile + uv_on_face * TILE_SIZE_UV;
    return textureSampleLevel(texture_atlas, texture_sampler, final_atlas_uv, 0.0);
}

fn blend_colors(current_color: vec4<f32>, new_color: vec4<f32>) -> vec4<f32> {
    let blend_factor = (1.0 - current_color.a);
    let new_accumulated_rgb = current_color.rgb + new_color.rgb * new_color.a * blend_factor;
    let new_accumulated_alpha = current_color.a + new_color.a * blend_factor;
    return vec4<f32>(new_accumulated_rgb, new_accumulated_alpha);
}

fn sample_surface_texture(surface: VoxelSurface) -> vec4<f32> {
    var hit_face_id: u32;
    if (surface.normal.x != 0) {
        hit_face_id = select(FACE_NX, FACE_PX, surface.normal.x > 0);
    } else if (surface.normal.y != 0) {
        hit_face_id = select(FACE_NY, FACE_PY, surface.normal.y > 0);
    } else {
        hit_face_id = select(FACE_NZ, FACE_PZ, surface.normal.z > 0);
    }

    var uv: vec2<f32>;
    let fractional_pos = surface.hit_point / VOXEL_SIZE;
    if (surface.normal.x != 0) { // Hit an X face
        uv = vec2<f32>(fract(fractional_pos.z), 1.0 - fract(fractional_pos.y));
    } else if (surface.normal.y != 0) { // Hit a Y face
        uv = vec2<f32>(fract(fractional_pos.x), fract(fractional_pos.z));
    } else { // Hit a Z face
        uv = vec2<f32>(fract(fractional_pos.x), 1.0 - fract(fractional_pos.y));
    }
    uv = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0 - 1e-6));

    return sample_texture_by_face(surface.voxel_type, hit_face_id, uv);
}

fn calculate_lighting(surface: VoxelSurface, base_color: vec4<f32>) -> vec3<f32> {
    let face_normal_f32 = vec3<f32>(surface.normal);
    let light_direction = normalize(vec3<f32>(-1.0, 1.0, 0.5));
    let ambient_light = 0.2;
    let diffuse_factor = max(dot(face_normal_f32, light_direction), 0.0);
    let final_light_factor = ambient_light + (1.0 - ambient_light) * diffuse_factor;

    let lit_rgb = base_color.rgb * final_light_factor;

    let min_depth_factor = 0.85;
    let depth_multiplier = (surface.hit_point.z / 3.0) * (1.0 - min_depth_factor) + min_depth_factor;
    return lit_rgb * depth_multiplier;
}

fn apply_block_highlights(base_color: vec4<f32>, is_selected: bool, is_hovered: bool) -> vec4<f32> {
    var final_color = base_color;

    if (is_selected) {
        let highlight_color = vec4<f32>(1.0, 1.0, 1.0, 0.1);
        let final_rgb = mix(final_color.rgb, highlight_color.rgb, highlight_color.a);
        let final_a = highlight_color.a + final_color.a * (1.0 - highlight_color.a);
        final_color = vec4<f32>(final_rgb, final_a);
    }

    if (is_hovered) {
        let highlight_color = vec4<f32>(0.0, 0.0, 1.0, 0.1);
        let final_rgb = mix(final_color.rgb, highlight_color.rgb, highlight_color.a);
        let final_a = highlight_color.a + final_color.a * (1.0 - highlight_color.a);
        final_color = vec4<f32>(final_rgb, final_a);
    }

    return final_color;
}

fn calculate_depth(hit_distance: f32, ray: Ray) -> f32 {
    let hit_point_world = ray.origin + ray.direction * hit_distance;
    let hit_point_local = hit_point_world - camera.world_offset.xyz;
    let clip_pos = camera.view_proj * vec4<f32>(hit_point_local, 1.0);
    return clip_pos.z / clip_pos.w;
}

// helper function to render a surface and blend its color.
fn render_and_blend(
    voxel_type: u32,
    hit_point: vec3<f32>,
    face_normal: vec3<i32>,
    accumulated_color: ptr<function, vec4<f32>>
) {
    // Skip rendering surfaces of air blocks.
    if (voxel_type == AIR_TYPE) {
        return;
    }

    let surface = VoxelSurface(voxel_type, hit_point, face_normal, 0.0);
    let surface_color = sample_surface_texture(surface);

    if (surface_color.a > 0.0) {
        let lit_rgb = calculate_lighting(surface, surface_color);
        let new_color_to_blend = vec4<f32>(lit_rgb, surface_color.a);
        *accumulated_color = blend_colors(*accumulated_color, new_color_to_blend);
    }
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0)
    );
    var out: VertexOutput;
    out.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    out.uv = positions[vertex_index] * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

fn create_camera_ray(ndc: vec2<f32>) -> Ray {
    let clip_pos_far = vec4<f32>(ndc.x, ndc.y, 1.0, 1.0);
    let world_pos_far = camera.inv_view_proj * clip_pos_far;

    let ray_origin_local = camera.camera_pos.xyz;
    let ray_dir_local = normalize((world_pos_far.xyz / world_pos_far.w) - ray_origin_local);
    let ray_origin_world = ray_origin_local + camera.world_offset.xyz;

    let inv_direction = 1.0 / ray_dir_local;

    return Ray(ray_origin_world, ray_dir_local, inv_direction);
}

fn intersect_world_bounds(ray: Ray) -> BoundingBoxIntersection {
    let world_min_bound = vec3<f32>(0.0, 0.0, 0.0);
    let world_max_bound = vec3<f32>(f32(WORLD_DIM_X), f32(WORLD_DIM_Y), f32(WORLD_DIM_Z)) * VOXEL_SIZE;

    let t_bottom = (world_min_bound - ray.origin) * ray.inv_direction;
    let t_top = (world_max_bound - ray.origin) * ray.inv_direction;
    let t_min_v = min(t_bottom, t_top);
    let t_max_v = max(t_bottom, t_top);

    let t_min_intersect = max(t_min_v.x, max(t_min_v.y, t_min_v.z));
    let t_max_intersect = min(t_max_v.x, min(t_max_v.y, t_max_v.z));

    return BoundingBoxIntersection(t_min_intersect <= t_max_intersect, t_min_intersect, t_max_intersect);
}

fn initialize_dda(ray: Ray, start_t: f32) -> DDAState {
    var state: DDAState;

    let current_ray_pos = ray.origin + ray.direction * start_t;
    state.current_voxel = vec3<i32>(floor(current_ray_pos / VOXEL_SIZE));
    state.current_voxel = clamp(state.current_voxel, vec3<i32>(0), vec3<i32>(i32(WORLD_DIM_X)-1, i32(WORLD_DIM_Y)-1, i32(WORLD_DIM_Z)-1));

    state.step_dir = sign(ray.direction);
    if (ray.direction.x == 0.0) { state.step_dir.x = 0.0; }
    if (ray.direction.y == 0.0) { state.step_dir.y = 0.0; }
    if (ray.direction.z == 0.0) { state.step_dir.z = 0.0; }

    let next_voxel_boundary = (vec3<f32>(state.current_voxel) + max(vec3<f32>(0.0), state.step_dir)) * VOXEL_SIZE;

    if (ray.direction.x == 0.0) { state.t_max.x = 1e9; } else { state.t_max.x = (next_voxel_boundary.x - ray.origin.x) * ray.inv_direction.x; }
    if (ray.direction.y == 0.0) { state.t_max.y = 1e9; } else { state.t_max.y = (next_voxel_boundary.y - ray.origin.y) * ray.inv_direction.y; }
    if (ray.direction.z == 0.0) { state.t_max.z = 1e9; } else { state.t_max.z = (next_voxel_boundary.z - ray.origin.z) * ray.inv_direction.z; }

    state.t_delta = VOXEL_SIZE * abs(ray.inv_direction);
    if (ray.direction.x == 0.0) { state.t_delta.x = 1e9; }
    if (ray.direction.y == 0.0) { state.t_delta.y = 1e9; }
    if (ray.direction.z == 0.0) { state.t_delta.z = 1e9; }

    let world_min_bound = vec3<f32>(0.0, 0.0, 0.0);
    let world_max_bound = vec3<f32>(f32(WORLD_DIM_X), f32(WORLD_DIM_Y), f32(WORLD_DIM_Z)) * VOXEL_SIZE;
    let t_bottom = (world_min_bound - ray.origin) * ray.inv_direction;
    let t_top = (world_max_bound - ray.origin) * ray.inv_direction;
    let t_min_v = min(t_bottom, t_top);

    state.face_normal = vec3<i32>(0);
    if (t_min_v.x > t_min_v.y && t_min_v.x > t_min_v.z) { state.face_normal.x = -i32(sign(ray.direction.x)); }
    else if (t_min_v.y > t_min_v.z) { state.face_normal.y = -i32(sign(ray.direction.y)); }
    else { state.face_normal.z = -i32(sign(ray.direction.z)); }

    return state;
}

fn step_dda(dda: ptr<function, DDAState>) -> f32 {
    var next_t: f32;
    if ((*dda).t_max.x < (*dda).t_max.y && (*dda).t_max.x < (*dda).t_max.z) {
        next_t = (*dda).t_max.x;
        (*dda).current_voxel.x += i32((*dda).step_dir.x);
        (*dda).t_max.x += (*dda).t_delta.x;
        (*dda).face_normal = vec3<i32>(-i32((*dda).step_dir.x), 0, 0);
    } else if ((*dda).t_max.y < (*dda).t_max.z) {
        next_t = (*dda).t_max.y;
        (*dda).current_voxel.y += i32((*dda).step_dir.y);
        (*dda).t_max.y += (*dda).t_delta.y;
        (*dda).face_normal = vec3<i32>(0, -i32((*dda).step_dir.y), 0);
    } else {
        next_t = (*dda).t_max.z;
        (*dda).current_voxel.z += i32((*dda).step_dir.z);
        (*dda).t_max.z += (*dda).t_delta.z;
        (*dda).face_normal = vec3<i32>(0, 0, -i32((*dda).step_dir.z));
    }
    return next_t;
}

fn is_transparent(voxel_type: u32) -> bool {
    if (voxel_type >= arrayLength(&is_transparent_buffer)) {
        return false;
    }
    return is_transparent_buffer[voxel_type] != 0u;
}

fn traverse_world(ray: Ray, bounds: BoundingBoxIntersection) -> TraversalResult {
    var hit_solid = false;
    var solid_surface = VoxelSurface(0u, vec3<f32>(0.0), vec3<i32>(0), 0.0);
    var accumulated_color = vec4<f32>(0.0);
    var hit_selected_block = false;
    var hit_hovered_block = false;

    var dda = initialize_dda(ray, bounds.t_min);
    var t_hit = bounds.t_min;
    var prev_voxel_type = AIR_TYPE;

    for (var i: u32 = 0u; i < MAX_VOXEL_TRAVERSAL_STEPS; i = i + 1u) {
        if (t_hit > bounds.t_max || hit_solid) {
            break;
        }

        hit_selected_block |= selected_block.x == 1 && all(vec3<u32>(dda.current_voxel).xy == selected_block.yz);
        hit_hovered_block |= hover_on_block.x == 1 && all(vec3<u32>(dda.current_voxel).xy == hover_on_block.yz);

        let current_voxel_type = get_voxel_type(dda.current_voxel);
        if (current_voxel_type != prev_voxel_type) {
            let hit_point = ray.origin + ray.direction * t_hit;

            if (is_transparent(prev_voxel_type)) {
                // Render the back-face of the transparent block we are EXITING
                render_and_blend(prev_voxel_type, hit_point, dda.face_normal, &accumulated_color);
            }

            if (is_transparent(current_voxel_type)) {
                // Render the front-face of the transparent block we are ENTERING
                render_and_blend(current_voxel_type, hit_point, dda.face_normal, &accumulated_color);
            } else if (current_voxel_type != AIR_TYPE) {
                // We reached a solid block
                hit_solid = true;
                solid_surface = VoxelSurface(current_voxel_type, hit_point, dda.face_normal, t_hit);
                break;
            }
        }

        prev_voxel_type = current_voxel_type;

        t_hit = step_dda(&dda);
    }

    // After the loop, if the ray exited the world from a transparent block, render its final exit surface.
    if (!hit_solid && t_hit >= bounds.t_max && prev_voxel_type != AIR_TYPE && is_transparent(prev_voxel_type)) {
        let hit_point = ray.origin + ray.direction * bounds.t_max;
        let clamped_hit_point = clamp(hit_point, vec3<f32>(0.0), vec3<f32>(f32(WORLD_DIM_X), f32(WORLD_DIM_Y), f32(WORLD_DIM_Z)));
        render_and_blend(prev_voxel_type, clamped_hit_point, dda.face_normal, &accumulated_color);
    }

    return TraversalResult(
        hit_solid,
        solid_surface,
        accumulated_color,
        hit_selected_block,
        hit_hovered_block
    );
}

fn get_t_mesh(uv: vec2<f32>, raw_depth: f32, ray: Ray) -> f32 {
    if (raw_depth >= 1.0) {
        return 1e9;
    }
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, (1.0 - uv.y) * 2.0 - 1.0);
    let clip_pos = vec4<f32>(ndc, raw_depth, 1.0);
    let pos_homo = camera.inv_view_proj * clip_pos;
    let pos_local = pos_homo.xyz / pos_homo.w;
    let pos_world = pos_local + camera.world_offset.xyz;
    return length(pos_world - ray.origin);
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let raw_mesh_depth = textureSampleLevel(mesh_depth_texture, mesh_depth_sampler, in.uv, 0);

    let ndc_coords = vec2<f32>(in.uv.x * 2.0 - 1.0, (1.0 - in.uv.y) * 2.0 - 1.0);
    let ray = create_camera_ray(ndc_coords);
    var bounds_intersect = intersect_world_bounds(ray);

    let t_mesh = get_t_mesh(in.uv, raw_mesh_depth, ray);
    bounds_intersect.t_max = min(bounds_intersect.t_max, t_mesh);

    if (!bounds_intersect.hit || bounds_intersect.t_min > bounds_intersect.t_max) {
        var output: FragmentOutput;
        output.color = vec4<f32>(0.0);
        output.depth = 1.0;
        return output;
    }

    let traversal = traverse_world(ray, bounds_intersect);

    var final_transparent_color = traversal.accumulated_transparent_color;
    final_transparent_color = apply_block_highlights(final_transparent_color, traversal.hit_selected_block, traversal.hit_hovered_block);

    // eframe uses Bgra8Unorm so we have to manually do gamma correction
    let gamma = 2.2;
    let corrected_translucency = pow(final_transparent_color.rgb, vec3<f32>(1.0 / gamma));

    var output: FragmentOutput;

    if (traversal.hit_solid) {
        let base_color = sample_surface_texture(traversal.solid_surface);
        let lit_color = calculate_lighting(traversal.solid_surface, base_color);
        let corrected_solid = pow(lit_color, vec3<f32>(1.0 / gamma));

        let final_rgb = corrected_translucency + corrected_solid * (1.0 - final_transparent_color.a);

        output.color = vec4<f32>(final_rgb, 1.0);
        output.depth = calculate_depth(traversal.solid_surface.distance, ray);
    } else {
        output.color = vec4<f32>(corrected_translucency, final_transparent_color.a);
        output.depth = 1.0;
    }

    return output;
}
