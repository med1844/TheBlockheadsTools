//!include camera_uniform.wgsl

// vertex shader block
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_ssao_blur(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((in_vertex_index & 1u) << 2u);
    let y = f32((in_vertex_index & 2u) << 1u);
    out.uv = vec2<f32>(x * 0.5, 1.0 - y * 0.5);
    out.clip_position = vec4<f32>(x - 1.0, y - 1.0, 0.0, 1.0);
    return out;
}

@group(0) @binding(0) var ssao_texture: texture_2d<f32>;
@group(0) @binding(1) var ssao_sampler: sampler;
@group(0) @binding(2) var depth_texture: texture_2d<f32>;
@group(0) @binding(3) var depth_sampler: sampler;
@group(0) @binding(4) var<uniform> camera: CameraUniform;

fn get_linear_z(uv: vec2<f32>, depth: f32) -> f32 {
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, (1.0 - uv.y) * 2.0 - 1.0);
    let clip_pos = vec4<f32>(ndc, depth, 1.0);
    let local_pos_homo = camera.inv_view_proj * clip_pos;
    return local_pos_homo.z / local_pos_homo.w;
}

@fragment
fn fs_ssao_blur(in: VertexOutput) -> @location(0) f32 {
    let tex_dim = vec2<f32>(textureDimensions(ssao_texture, 0));
    let texel_size = 1.0 / tex_dim;
    var result: f32 = 0.0;

    // Hardcoded simple cross bilateral box-blur
    let blur_radius = 1; // Fixed radius for SSAO noise

    let center_depth = textureSampleLevel(depth_texture, depth_sampler, in.uv, 0.0).r;
    if (center_depth >= 1.0) {
        return 1.0; // Fast exit void pixels without blur bleeding
    }
    
    let center_z = get_linear_z(in.uv, center_depth);

    var weight_sum: f32 = 0.0;

    for (var x = -blur_radius; x <= blur_radius; x = x + 1) {
        for (var y = -blur_radius; y <= blur_radius; y = y + 1) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            let sample_uv = in.uv + offset;

            let sample_depth = textureSampleLevel(depth_texture, depth_sampler, sample_uv, 0.0).r;
            let sample_z = get_linear_z(sample_uv, sample_depth);

            // Edge preservation mapping
            // Only aggregate neighbors loosely matching the same z depth
            let depth_diff = abs(center_z - sample_z);
            let z_threshold = 1.0; // 1 block unit physical distance
            let weight = select(0.0, 1.0, depth_diff < z_threshold);

            result += textureSampleLevel(ssao_texture, ssao_sampler, sample_uv, 0.0).r * weight;
            weight_sum += weight;
        }
    }

    return result / max(weight_sum, 1.0);
}
