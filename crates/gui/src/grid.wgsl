struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>, // xyz
    world_offset: vec4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

// --- World Constants (must match voxel.wgsl) ---
const WORLD_CHUNKS_X: u32 = 512u;
const WORLD_CHUNKS_Y: u32 = 32u;
const CHUNK_DIM_X: u32 = 32u;
const CHUNK_DIM_Y: u32 = 32u;
const WORLD_DIM_X_F32: f32 = f32(CHUNK_DIM_X * WORLD_CHUNKS_X); // 16384.0
const WORLD_DIM_Y_F32: f32 = f32(CHUNK_DIM_Y * WORLD_CHUNKS_Y); // 1024.0

// --- Grid Constants ---
const GRID_Z: f32 = 3.0;
const CHUNK_SIZE: f32 = 32.0;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_grid(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
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

@fragment
fn fs_grid(in: VertexOutput) -> @location(0) vec4<f32> {
    // --- 1. Generate World-Space Ray (same as voxel.wgsl) ---
    let frag_coord = in.uv;
    let ndc_coords = vec2<f32>(frag_coord.x*2.0-1.0, (1.0-frag_coord.y)*2.0 - 1.0);

    let inv_view_proj = camera.inv_view_proj;
    let clip_pos_near = vec4<f32>(ndc_coords.x, ndc_coords.y, 0.0, 1.0);
    let clip_pos_far = vec4<f32>(ndc_coords.x, ndc_coords.y, 1.0, 1.0);
    let world_pos_near = inv_view_proj * clip_pos_near;
    let world_pos_far = inv_view_proj * clip_pos_far;

    let ray_origin_local = camera.camera_pos.xyz;
    let ray_dir_local = normalize((world_pos_far.xyz/world_pos_far.w) - ray_origin_local);
    let ray_origin_world = ray_origin_local + camera.world_offset.xyz;
    let ray_dir_world = ray_dir_local;

    // --- 2. Ray-Plane Intersection ---
    // Find intersection with plane world.z = GRID_Z

    // Discard if ray is parallel to the plane
    if (abs(ray_dir_world.z) < 1e-6) {
        discard;
    }

    let t = (GRID_Z - ray_origin_world.z) / ray_dir_world.z;

    // Discard if intersection is behind the camera
    if (t < 0.0) {
        discard;
    }

    let world_pos = ray_origin_world + ray_dir_world * t;

    // --- 3. Check World Boundaries ---
    if (world_pos.x < 0.0 || world_pos.x > WORLD_DIM_X_F32 ||
        world_pos.y < 0.0 || world_pos.y > WORLD_DIM_Y_F32) {
        discard;
    }

    // --- 4. Calculate Anti-Aliased Grid Lines ---

    // Get the width of one pixel in world coordinates
    let line_width_vec = fwidth(world_pos.xy);
    let line_width = (line_width_vec.x + line_width_vec.y) * 0.5 * 1.0; // 1.0 pixel wide

    // `d` = distance from the "previous" grid line
    let d = world_pos.xy % CHUNK_SIZE;
    // `dist` = distance to the *nearest* grid line
    let dist = min(d, CHUNK_SIZE - d);

    // `grid_line` will be 0.0 on the line, and increase as we move away
    let grid_line = dist / line_width;

    // Find the closest line (horizontal or vertical)
    let line = min(grid_line.x, grid_line.y);

    // `alpha` will be 1.0 on the line, fading to 0.0 over 1 pixel
    let alpha = 1.0 - min(line, 1.0);

    let final_alpha = alpha * 0.1;

    if (final_alpha < 1e-6) {
        discard;
    }

    return vec4<f32>(1.0, 1.0, 1.0, final_alpha);
}
