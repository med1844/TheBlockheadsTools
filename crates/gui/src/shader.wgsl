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

struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>, // xyz
    screen_size: vec4<f32>, // x=width, y=height
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

fn get_voxel_type(global_voxel_coords: vec3<i32>) -> u32 {
    if global_voxel_coords.x < 0 || global_voxel_coords.x >= i32(WORLD_DIM_X) ||
       global_voxel_coords.y < 0 || global_voxel_coords.y >= i32(WORLD_DIM_Y) ||
       global_voxel_coords.z < 0 || global_voxel_coords.z >= i32(WORLD_DIM_Z) {
        return 0u;
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

// --- Function to sample the texture atlas ---
// This function takes the block type, hit face, and local UV coordinates
// to return the final sampled color from the texture atlas.
fn sample_texture(voxel_type: u32, hit_face_id: u32, uv_on_face: vec2<f32>) -> vec4<f32> {
    // Calculate the index into the texture_uv_atlas_indices array
    // The Rust side array is flattened: [block0_face0, block0_face1, ..., block1_face0, ...]
    let atlas_index_lookup = voxel_type * 6u + hit_face_id; // 6 faces per block type
    let tile_index = texture_uv_atlas_indices[atlas_index_lookup];

    // Convert the 1D tile_index to 2D tile coordinates (row, col)
    let tile_x = f32(tile_index % TILES_PER_ROW);
    let tile_y = f32(tile_index / TILES_PER_ROW);

    // Calculate the min UV coordinates for this specific tile in the atlas
    let uv_min_tile = vec2<f32>(tile_x * TILE_SIZE_UV, tile_y * TILE_SIZE_UV);

    // Scale and offset the uv_on_face (0-1 within the tile) to the atlas's UV space
    let final_atlas_uv = uv_min_tile + uv_on_face * TILE_SIZE_UV;

    // Sample the texture
    return textureSample(texture_atlas, texture_sampler, final_atlas_uv);
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Define three vertices that form a triangle covering the screen in NDC space (-1 to 1).
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0), // Bottom-left
        vec2<f32>( 3.0, -1.0), // Extends far right
        vec2<f32>(-1.0,  3.0)  // Extends far up
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) clip_position: vec4<f32>) -> @location(0) vec4<f32> {
    let frag_coord = clip_position;
    let screen_width = camera.screen_size.x;
    let screen_height = camera.screen_size.y;

    let ndc_coords = vec2<f32>(
        (frag_coord.x / screen_width) * 2.0 - 1.0,
        // (frag_coord.y / screen_height) * 2.0 - 1.0
        1.0 - (frag_coord.y / screen_height) * 2.0
    );

    let inv_view_proj = camera.inv_view_proj;
    let clip_pos_near = vec4<f32>(ndc_coords.x, ndc_coords.y, 0.0, 1.0);
    let clip_pos_far = vec4<f32>(ndc_coords.x, ndc_coords.y, 1.0, 1.0);

    let world_pos_near = inv_view_proj * clip_pos_near;
    let world_pos_far = inv_view_proj * clip_pos_far;

    // The ray origin is the camera's position in its own local space.
    let ray_origin_local = camera.camera_pos.xyz;
    
    // The direction is from the local origin towards the unprojected far point.
    let ray_dir_local = normalize((world_pos_far.xyz / world_pos_far.w) - ray_origin_local);

    // Now, translate the local-space ray into world-space for traversal.
    let ray_origin_world = ray_origin_local + vec3<f32>(camera.world_offset.xy, 0.0);
    let ray_dir_world = ray_dir_local; // Direction is unaffected by translation.

    var hit_color = vec4<f32>(0.0, 0.0, 0.0, 0.0); // Default background color is nothing

    let world_min_bound = vec3<f32>(0.0, 0.0, 0.0);
    let world_max_bound = vec3<f32>(
        f32(WORLD_DIM_X) * VOXEL_SIZE,
        f32(WORLD_DIM_Y) * VOXEL_SIZE,
        f32(WORLD_DIM_Z) * VOXEL_SIZE
    );

    // AABB Intersection that finds the entry normal
    let inv_dir = 1.0 / ray_dir_world;
    let t_bottom = (world_min_bound - ray_origin_world) * inv_dir;
    let t_top = (world_max_bound - ray_origin_world) * inv_dir;

    let t_min_v = min(t_bottom, t_top);
    let t_max_v = max(t_bottom, t_top);

    let t_min_intersect = max(t_min_v.x, max(t_min_v.y, t_min_v.z));
    let t_max_intersect = min(t_max_v.x, min(t_max_v.y, t_max_v.z));

    if (t_min_intersect > t_max_intersect) {
        return hit_color;
    }

    var initial_normal = vec3<i32>(0);
    if (t_min_v.x > t_min_v.y && t_min_v.x > t_min_v.z) {
        initial_normal.x = -i32(sign(ray_dir_world.x));
    } else if (t_min_v.y > t_min_v.z) {
        initial_normal.y = -i32(sign(ray_dir_world.y));
    } else {
        initial_normal.z = -i32(sign(ray_dir_world.z));
    }

    // --- DDA Initialization ---
    // The DDA starting voxel is found by pushing the ray slightly past the chunk boundary.
    var current_ray_pos = ray_origin_world + ray_dir_world * t_min_intersect;
    var current_voxel_coords = vec3<i32>(floor(current_ray_pos / VOXEL_SIZE));
    
    var step_dir = sign(ray_dir_world);
    if (ray_dir_world.x == 0.0) { step_dir.x = 0.0; }
    if (ray_dir_world.y == 0.0) { step_dir.y = 0.0; }
    if (ray_dir_world.z == 0.0) { step_dir.z = 0.0; }

    // THE FIX: Calculate the initial t_max_axis using the ray's true origin (ray_origin_world)
    // instead of the DDA's starting position (current_ray_pos). This makes t_max_axis an
    // absolute 't' value, consistent with t_min_intersect and t_hit.
    var t_max_axis: vec3<f32>;
    let next_voxel_boundary = (vec3<f32>(current_voxel_coords) + max(vec3<f32>(0.0), step_dir)) * VOXEL_SIZE;

    if (ray_dir_world.x == 0.0) { t_max_axis.x = 100000000.0; }
    else { t_max_axis.x = (next_voxel_boundary.x - ray_origin_world.x) / ray_dir_world.x; }

    if (ray_dir_world.y == 0.0) { t_max_axis.y = 100000000.0; }
    else { t_max_axis.y = (next_voxel_boundary.y - ray_origin_world.y) / ray_dir_world.y; }

    if (ray_dir_world.z == 0.0) { t_max_axis.z = 100000000.0; }
    else { t_max_axis.z = (next_voxel_boundary.z - ray_origin_world.z) / ray_dir_world.z; }

    // t_delta_axis remains correct as it's an origin-independent increment.
    var t_delta_axis = VOXEL_SIZE / abs(ray_dir_world);
    if (ray_dir_world.x == 0.0) { t_delta_axis.x = 100000000.0; }
    if (ray_dir_world.y == 0.0) { t_delta_axis.y = 100000000.0; }
    if (ray_dir_world.z == 0.0) { t_delta_axis.z = 100000000.0; }

    // --- DDA Loop with UV calculation ---
    var normal_of_entry_face = initial_normal;
    var t_hit = t_min_intersect;

    for (var i: u32 = 0u; i < MAX_VOXEL_TRAVERSAL_STEPS; i = i + 1u) {
        // Stop if the ray has exited the chunk's bounding box.
        if (t_hit > t_max_intersect) {
            break;
        }

        let voxel_type = get_voxel_type(current_voxel_coords);

        if (voxel_type != 0u) { // If not air, we hit a voxel!
            var hit_face_id: u32;

            if (normal_of_entry_face.x != 0) {
                hit_face_id = select(FACE_NX, FACE_PX, normal_of_entry_face.x > 0);
            } else if (normal_of_entry_face.y != 0) {
                hit_face_id = select(FACE_NY, FACE_PY, normal_of_entry_face.y > 0);
            } else {
                hit_face_id = select(FACE_NZ, FACE_PZ, normal_of_entry_face.z > 0);
            }

            // Calculate the exact world-space position of the hit.
            let hit_point = ray_origin_world + ray_dir_world * t_hit;

            // Calculate UV coordinates based on the hit point and face normal.
            var uv: vec2<f32>;
            let fractional_pos = hit_point / VOXEL_SIZE;

            if (normal_of_entry_face.x != 0) { // Hit an X face
                uv = vec2<f32>(fract(fractional_pos.z), 1.0 - fract(fractional_pos.y));
            } else if (normal_of_entry_face.y != 0) { // Hit a Y face
                uv = vec2<f32>(fract(fractional_pos.x), fract(fractional_pos.z));
            } else { // Hit a Z face
                uv = vec2<f32>(fract(fractional_pos.x), 1.0 - fract(fractional_pos.y));
            }
            uv = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0 - 1e-6));

            hit_color = sample_texture(voxel_type, hit_face_id, uv);
            let alpha = hit_color.w;

            let face_normal_f32 = vec3<f32>(normal_of_entry_face);
            let light_direction = normalize(vec3<f32>(-1.0, 1.0, 0.5)); // Example light direction
            let ambient_light = 0.2; // Base ambient light
            let diffuse_factor = max(dot(face_normal_f32, light_direction), 0.0);
            let final_light_factor = ambient_light + (1.0 - ambient_light) * diffuse_factor;

            hit_color *= final_light_factor;

            let min_depth_factor = 0.85; // map 0..1 into 0.85..1
            hit_color *= (hit_point.z / 3.0) * (1 - min_depth_factor) + min_depth_factor;
            hit_color.w = alpha;

            break; // Exit loop, we found our color.
        }

        // Advance to the next voxel boundary and update the normal and t_hit for the next iteration.
        if (t_max_axis.x < t_max_axis.y && t_max_axis.x < t_max_axis.z) {
            t_hit = t_max_axis.x;
            current_voxel_coords.x += i32(step_dir.x);
            t_max_axis.x += t_delta_axis.x;
            normal_of_entry_face = vec3<i32>(-i32(step_dir.x), 0, 0);
        } else if (t_max_axis.y < t_max_axis.z) {
            t_hit = t_max_axis.y;
            current_voxel_coords.y += i32(step_dir.y);
            t_max_axis.y += t_delta_axis.y;
            normal_of_entry_face = vec3<i32>(0, -i32(step_dir.y), 0);
        } else {
            t_hit = t_max_axis.z;
            current_voxel_coords.z += i32(step_dir.z);
            t_max_axis.z += t_delta_axis.z;
            normal_of_entry_face = vec3<i32>(0, 0, -i32(step_dir.z));
        }
    }

    return hit_color;
}
