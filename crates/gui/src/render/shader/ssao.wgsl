//!include camera_uniform.wgsl

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var voxel_normal_texture: texture_2d<f32>;
@group(0) @binding(2) var voxel_normal_sampler: sampler;
@group(0) @binding(3) var voxel_depth_texture: texture_depth_2d;
@group(0) @binding(4) var voxel_depth_sampler: sampler;

struct SsaoUniform {
    kernel_size: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
    kernel: array<vec4<f32>, 256>, // SSAO_MAX_KERNEL_SIZE
};

@group(0) @binding(5) var<uniform> ssao_data: SsaoUniform;
@group(0) @binding(6) var noise_texture: texture_2d<f32>;
@group(0) @binding(7) var noise_sampler: sampler;
@group(0) @binding(8) var mesh_depth_texture: texture_depth_2d;
@group(0) @binding(9) var mesh_depth_sampler: sampler;
@group(0) @binding(10) var mesh_normal_texture: texture_2d<f32>;
@group(0) @binding(11) var mesh_normal_sampler: sampler;
@group(0) @binding(12) var mesh_uv_texture: texture_2d<f32>;
@group(0) @binding(13) var mesh_uv_sampler: sampler;
@group(0) @binding(14) var voxel_uv_texture: texture_2d<f32>;
@group(0) @binding(15) var voxel_uv_sampler: sampler;
@group(0) @binding(16) var flags_texture: texture_2d<u32>;

//!include surface.wgsl

// We will tile the 4x4 noise texture over the screen
const RADIUS: f32 = 0.5; // Sampling radius in world space units
const BIAS: f32 = 0.025;  // To avoid acne

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_ssao(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
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

// Function to recover local-space position from depth buffer
fn reconstruct_local_pos(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    // Convert UV to NDC
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, (1.0 - uv.y) * 2.0 - 1.0);
    let clip_pos = vec4<f32>(ndc, depth, 1.0);

    // Note: camera.inv_view_proj transforms NDC straight to Local Space.
    let local_pos_homo = camera.inv_view_proj * clip_pos;
    let local_pos = local_pos_homo.xyz / local_pos_homo.w;

    return local_pos;
}

// Function to project local-space coordinates backwards into screen UV space
fn project_to_uv(local_pos: vec3<f32>) -> vec2<f32> {
    var offset_clip = camera.view_proj * vec4<f32>(local_pos, 1.0);
    offset_clip = offset_clip / offset_clip.w;
    return vec2<f32>(offset_clip.x * 0.5 + 0.5, 1.0 - (offset_clip.y * 0.5 + 0.5));
}

// Function to fetch and map the randomized tangential rotation scale correctly
fn get_random_vec(uv: vec2<f32>) -> vec3<f32> {
    let screen_dim = vec2<f32>(textureDimensions(voxel_depth_texture, 0));
    let noise_dim = vec2<f32>(textureDimensions(noise_texture, 0));
    let noise_uv = uv * (screen_dim / noise_dim);
    let noise_vec_raw = textureSampleLevel(noise_texture, noise_sampler, noise_uv, 0.0).xy;
    return normalize(vec3<f32>(noise_vec_raw * 2.0 - 1.0, 0.0));
}

// Function to construct a TBN rotation matrix for hemisphere alignment
fn get_tbn_matrix(normal: vec3<f32>, random_vec: vec3<f32>) -> mat3x3<f32> {
    let tangent = normalize(random_vec - normal * dot(random_vec, normal));
    let bitangent = cross(normal, tangent);
    return mat3x3<f32>(tangent, bitangent, normal);
}

// Function to calculate occlusion dynamically by mapping the offset depth against physical bounds
fn evaluate_sample_occlusion(local_pos: vec3<f32>, sample_local_pos: vec3<f32>) -> f32 {
    let offset_uv = project_to_uv(sample_local_pos);

    // Verify screen bounds
    if (offset_uv.x < 0.0 || offset_uv.x > 1.0 || offset_uv.y < 0.0 || offset_uv.y > 1.0) {
        return 0.0;
    }

    let screen_size = vec2<f32>(textureDimensions(voxel_depth_texture, 0));
    let sample_depth = get_surface_depth_only(offset_uv, screen_size);
    
    if (sample_depth >= 1.0) {
        return 0.0;
    }

    let actual_sample_local_pos = reconstruct_local_pos(offset_uv, sample_depth);

    let range_check = smoothstep(0.0, 1.0, RADIUS / abs(local_pos.z - actual_sample_local_pos.z));
    let dist_to_sample = length(actual_sample_local_pos - camera.camera_pos.xyz);
    let dist_to_target = length(sample_local_pos - camera.camera_pos.xyz);

    if (dist_to_sample < dist_to_target - BIAS) {
        return 1.0 * range_check;
    }

    return 0.0;
}


@fragment
fn fs_ssao(in: VertexOutput) -> @location(0) f32 {
    let screen_size = vec2<f32>(textureDimensions(voxel_depth_texture, 0));
    let surf = get_surface(in.uv, screen_size);

    if (!surf.has_opaque || surf.depth >= 1.0) {
        return 1.0; // Sky is not occluded
    }

    let local_pos = reconstruct_local_pos(in.uv, surf.depth);
    let normal = normalize(surf.normal);

    let random_vec = get_random_vec(in.uv);

    let tbn = get_tbn_matrix(normal, random_vec);

    var occlusion: f32 = 0.0;

    for (var i = 0u; i < ssao_data.kernel_size; i = i + 1u) {
        let sample_dir = tbn * ssao_data.kernel[i].xyz;
        let sample_local_pos = local_pos + sample_dir * RADIUS;

        occlusion += evaluate_sample_occlusion(local_pos, sample_local_pos);
    }

    occlusion = 1.0 - (occlusion / f32(ssao_data.kernel_size));
    return occlusion;
}
