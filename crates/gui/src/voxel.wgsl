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

// Tracks the first fully-opaque surface hit by the ray (alpha = 1.0).
// Used to write depth and normal_spec outputs.
struct DepthHit {
    is_set:   bool,
    surface:  VoxelSurface,
    normal:   vec3<f32>,
    specular: f32,
}

struct TraversalResult {
    depth_hit:          DepthHit,
    /// Fully-opaque (alpha = 1.0) solid surface color — goes to albedo target.
    solid_color:        vec4<f32>,
    /// Accumulated semi-transparent (0 < alpha < 1.0) colors — goes to translucency target.
    translucency_color: vec4<f32>,
    hit_selected_block: bool,
    hit_hovered_block:  bool,
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

struct RenderSettings {
    light_dir: vec3<f32>,
    enable_reflect: u32,
    enable_destruct: u32,
    enable_ssao: u32,
    ambient_light: f32,
    shininess: f32,
    specular_intensity: f32,
    min_depth_factor: f32,
    _padding0: u32,
    _padding1: u32,
};

@group(0) @binding(8)
var<uniform> render_settings: RenderSettings;

@group(0) @binding(9) var texture_reflect: texture_2d<f32>;
@group(0) @binding(10) var sampler_reflect: sampler;

@group(0) @binding(11) var texture_destruct: texture_2d<f32>;
@group(0) @binding(12) var sampler_destruct: sampler;

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

fn get_atlas_uv(voxel_type: u32, hit_face_id: u32, uv_on_face: vec2<f32>) -> vec2<f32> {
    let atlas_index_lookup = voxel_type * 6u + hit_face_id;
    let tile_index = texture_uv_atlas_indices[atlas_index_lookup];
    let tile_x = f32(tile_index % TILES_PER_ROW);
    let tile_y = f32(tile_index / TILES_PER_ROW);
    let uv_min_tile = vec2<f32>(tile_x * TILE_SIZE_UV, tile_y * TILE_SIZE_UV);
    return uv_min_tile + uv_on_face * TILE_SIZE_UV;
}

fn sample_texture_by_face(voxel_type: u32, hit_face_id: u32, uv_on_face: vec2<f32>) -> vec4<f32> {
    let final_atlas_uv = get_atlas_uv(voxel_type, hit_face_id, uv_on_face);
    return textureSampleLevel(texture_atlas, texture_sampler, final_atlas_uv, 0.0);
}

fn sample_reflect_texture(voxel_type: u32, hit_face_id: u32, uv_on_face: vec2<f32>) -> vec4<f32> {
    let final_atlas_uv = get_atlas_uv(voxel_type, hit_face_id, uv_on_face);
    return textureSampleLevel(texture_reflect, sampler_reflect, final_atlas_uv, 0.0);
}

fn sample_destruct_texture(voxel_type: u32, hit_face_id: u32, uv_on_face: vec2<f32>) -> vec4<f32> {
    let final_atlas_uv = get_atlas_uv(voxel_type, hit_face_id, uv_on_face);
    return textureSampleLevel(texture_destruct, sampler_destruct, final_atlas_uv, 0.0);
}

fn blend_colors(current_color: vec4<f32>, new_color: vec4<f32>) -> vec4<f32> {
    let blend_factor = (1.0 - current_color.a);
    let new_accumulated_rgb = current_color.rgb + new_color.rgb * new_color.a * blend_factor;
    let new_accumulated_alpha = current_color.a + new_color.a * blend_factor;
    return vec4<f32>(new_accumulated_rgb, new_accumulated_alpha);
}

struct MaterialData {
    base_color: vec4<f32>,
    reflect_intensity: f32,
    normal: vec3<f32>,
}

fn sample_material(surface: VoxelSurface) -> MaterialData {
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

    let base_color = sample_texture_by_face(surface.voxel_type, hit_face_id, uv);
    var perturbed_normal = vec3<f32>(surface.normal);

    if (render_settings.enable_destruct == 1u) {
        let destruct_color = sample_destruct_texture(surface.voxel_type, hit_face_id, uv);

        var tangent: vec3<f32>;
        var bitangent: vec3<f32>;
        if (surface.normal.x != 0) {
            tangent = vec3<f32>(0.0, 0.0, 1.0);
            bitangent = vec3<f32>(0.0, -1.0, 0.0);
        } else if (surface.normal.y != 0) {
            tangent = vec3<f32>(1.0, 0.0, 0.0);
            bitangent = vec3<f32>(0.0, 0.0, 1.0);
        } else {
            tangent = vec3<f32>(1.0, 0.0, 0.0);
            bitangent = vec3<f32>(0.0, -1.0, 0.0);
        }

        // In the original game, normal perturbation is calculated as:
        // vec3((-destruct.r + 0.5) * 0.5, (destruct.g - 0.5) * 0.5, 0.0)
        // It hardcodes Z to 0.0 (ignoring the noisy blue channel) and flips X.
        // It also applies it universally without any alpha masking.
        // We replicate that mathematical behavior here but map it properly into our 3D Tangent Space.
        let local_normal = vec3<f32>(
            (destruct_color.r - 0.5) * 0.5,
            (destruct_color.g - 0.5) * 0.5,
            1.0 // Z acts as the unperturbed outward strength base
        );

        perturbed_normal = normalize(
            tangent * local_normal.x +
            bitangent * local_normal.y +
            vec3<f32>(surface.normal) * local_normal.z
        );
    }

    var reflect_intensity = 0.0;
    if (render_settings.enable_reflect == 1u) {
        let reflect_color = sample_reflect_texture(surface.voxel_type, hit_face_id, uv);
        reflect_intensity = reflect_color.r * reflect_color.a * base_color.a;
    }
    return MaterialData(base_color, reflect_intensity, perturbed_normal);
}

struct LightingOutput {
    lit_color: vec3<f32>,
    specular_scalar: f32,
}

fn calculate_lighting(surface: VoxelSurface, material: MaterialData, ray_dir: vec3<f32>) -> LightingOutput {
    let face_normal_f32 = material.normal;
    let light_direction = normalize(render_settings.light_dir);
    let ambient_light = render_settings.ambient_light;
    let diffuse_factor = max(dot(face_normal_f32, light_direction), 0.0);

    // NOTE: The original game used a heavily modified diffuse calculation to boost lighting on reflective surfaces:
    // diffuse = max(((lightDP + 0.2 * (2.0 - reflect))), 0.2) * (1.0 + reflect * 0.2);
    // Here we use standard linear interpolation for a cleaner, universal material look.
    let final_light_factor = ambient_light + (1.0 - ambient_light) * diffuse_factor;

    let view_dir = -ray_dir;
    let half_vector = normalize(light_direction + view_dir);
    let spec_angle = max(dot(face_normal_f32, half_vector), 0.0);
    let shininess = render_settings.shininess;

    // NOTE: The original game FAKED specular physically disconnected from the camera view angle,
    // using purely the diffuse dot product to drive the shine based on the destruct mask:
    // specular = reflect * 2.0 * (pow(diffuse, 8.0 * reflect) * 0.15 + diffuse * 0.2);
    // We are deliberately preserving a physically-accurate Blinn-Phong specular view angle here.
    let specular_factor = pow(spec_angle, shininess);
    let spec_scalar = specular_factor * material.reflect_intensity * render_settings.specular_intensity;

    let lit_rgb = material.base_color.rgb * final_light_factor;

    let min_depth_factor = render_settings.min_depth_factor;
    let depth_multiplier = (surface.hit_point.z / 3.0) * (1.0 - min_depth_factor) + min_depth_factor;

    var out_lighting: LightingOutput;
    out_lighting.lit_color = lit_rgb * depth_multiplier;
    out_lighting.specular_scalar = spec_scalar;

    return out_lighting;
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

// Renders a voxel surface, routes its lit color into either solid_color (alpha=1.0)
// or translucency_color (0 < alpha < 1.0), and records depth_hit on first opaque hit.
fn render_and_blend(
    voxel_type: u32,
    hit_point:  vec3<f32>,
    face_normal: vec3<i32>,
    t_hit:      f32,
    ray_dir:    vec3<f32>,
    solid_color:        ptr<function, vec4<f32>>,
    translucency_color: ptr<function, vec4<f32>>,
    depth_hit:          ptr<function, DepthHit>,
) {
    if (voxel_type == AIR_TYPE) {
        return;
    }

    let surface  = VoxelSurface(voxel_type, hit_point, face_normal, t_hit);
    let material = sample_material(surface);

    if (material.base_color.a > 0.0) {
        let lighting = calculate_lighting(surface, material, ray_dir);
        let lit = vec4<f32>(lighting.lit_color, material.base_color.a);

        if (material.base_color.a >= 1.0) {
            // Fully opaque — goes to the solid albedo buffer
            *solid_color = blend_colors(*solid_color, lit);

            if (!(*depth_hit).is_set) {
                (*depth_hit).surface  = surface;
                (*depth_hit).normal   = material.normal;
                (*depth_hit).specular = lighting.specular_scalar;
                (*depth_hit).is_set   = true;
            }
        } else {
            // Semi-transparent — goes to the translucency buffer
            *translucency_color = blend_colors(*translucency_color, lit);
        }
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
    @location(0) albedo:       vec4<f32>,
    @location(1) normal_spec:  vec4<f32>,
    @builtin(frag_depth) depth: f32,
    // @location(2) is dyn_obj_id — written by dw_sprite, not voxel
    @location(3) translucency: vec4<f32>,
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

fn traverse_world(ray: Ray, bounds: BoundingBoxIntersection) -> TraversalResult {
    var depth_hit: DepthHit;
    var solid_color        = vec4<f32>(0.0);
    var translucency_color = vec4<f32>(0.0);
    var hit_selected_block = false;
    var hit_hovered_block = false;

    var dda = initialize_dda(ray, bounds.t_min);
    var t_hit = bounds.t_min;
    var prev_voxel_type = AIR_TYPE;

    for (var i: u32 = 0u; i < MAX_VOXEL_TRAVERSAL_STEPS; i = i + 1u) {
        // Stop once the solid surface is hit (solid_color is opaque)
        if (t_hit > bounds.t_max || solid_color.a >= 1.0) {
            break;
        }

        hit_selected_block |= selected_block.x == 1 && all(vec3<u32>(dda.current_voxel).xy == selected_block.yz);
        hit_hovered_block |= hover_on_block.x == 1 && all(vec3<u32>(dda.current_voxel).xy == hover_on_block.yz);

        let current_voxel_type = get_voxel_type(dda.current_voxel);
        if (current_voxel_type != prev_voxel_type) {
            let hit_point = ray.origin + ray.direction * t_hit;

            // Render exit face of the block we are leaving
            if (prev_voxel_type != AIR_TYPE) {
                render_and_blend(
                    prev_voxel_type, hit_point, dda.face_normal, t_hit, ray.direction,
                    &solid_color, &translucency_color, &depth_hit
                );
            }

            // Render entry face of the block we are entering
            if (current_voxel_type != AIR_TYPE) {
                render_and_blend(
                    current_voxel_type, hit_point, dda.face_normal, t_hit, ray.direction,
                    &solid_color, &translucency_color, &depth_hit
                );
            }
        }

        prev_voxel_type = current_voxel_type;
        t_hit = step_dda(&dda);
    }

    // After the loop, render exit face if we left the world from within a non-air block
    if (solid_color.a < 1.0 && t_hit >= bounds.t_max && prev_voxel_type != AIR_TYPE) {
        let hit_point = ray.origin + ray.direction * bounds.t_max;
        let clamped_hit_point = clamp(hit_point, vec3<f32>(0.0), vec3<f32>(f32(WORLD_DIM_X), f32(WORLD_DIM_Y), f32(WORLD_DIM_Z)));
        render_and_blend(
            prev_voxel_type, clamped_hit_point, dda.face_normal, bounds.t_max, ray.direction,
            &solid_color, &translucency_color, &depth_hit
        );
    }

    return TraversalResult(depth_hit, solid_color, translucency_color, hit_selected_block, hit_hovered_block);
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
        output.albedo       = vec4<f32>(0.0);
        output.translucency = vec4<f32>(0.0);
        output.depth        = 1.0;
        return output;
    }

    let traversal = traverse_world(ray, bounds_intersect);

    // Apply block highlights to solid surface only
    var solid = traversal.solid_color;
    solid = apply_block_highlights(solid, traversal.hit_selected_block, traversal.hit_hovered_block);

    // eframe uses Bgra8Unorm so we have to manually do gamma correction
    let gamma = 2.2;
    let corrected_solid        = pow(solid.rgb, vec3<f32>(1.0 / gamma));
    let corrected_translucency = pow(traversal.translucency_color.rgb, vec3<f32>(1.0 / gamma));

    var output: FragmentOutput;
    output.translucency = vec4<f32>(corrected_translucency, traversal.translucency_color.a);

    if (traversal.depth_hit.is_set) {
        output.albedo      = vec4<f32>(corrected_solid, 1.0);
        output.normal_spec = vec4<f32>(traversal.depth_hit.normal, traversal.depth_hit.specular);
        output.depth       = calculate_depth(traversal.depth_hit.surface.distance, ray);
    } else {
        output.albedo      = vec4<f32>(0.0);
        output.normal_spec = vec4<f32>(0.0, 0.0, 1.0, 0.0);
        output.depth       = 1.0;
    }

    return output;
}
