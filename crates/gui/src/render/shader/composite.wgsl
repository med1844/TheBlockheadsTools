// Solid voxel albedo (fully-opaque pixels only)
@group(0) @binding(0) var albedo_texture:  texture_2d<f32>;
@group(0) @binding(1) var albedo_sampler:  sampler;

// Normal + specular scalar (from depth_hit surface)
@group(0) @binding(2) var normal_spec_texture: texture_2d<f32>;
@group(0) @binding(3) var normal_spec_sampler: sampler;

// SSAO occlusion factor (blurred)
@group(0) @binding(4) var ssao_texture:   texture_2d<f32>;
@group(0) @binding(5) var ssao_sampler:   sampler;

@group(0) @binding(6) var<uniform> render_settings: RenderSettings;

// Semi-transparent voxel colors (water, leaves, glass …)
// Written to a separate target so SSAO never touches them.
@group(0) @binding(7) var translucency_texture: texture_2d<f32>;
@group(0) @binding(8) var translucency_sampler: sampler;

struct RenderSettings {
    light_dir:          vec3<f32>,
    enable_reflect:     u32,
    enable_destruct:    u32,
    enable_ssao:        u32,
    ambient_light:      f32,
    shininess:          f32,
    specular_intensity: f32,
    min_depth_factor:   f32,
    _padding0:          u32,
    _padding1:          u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_composite(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
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
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo      = textureSample(albedo_texture,      albedo_sampler,      in.uv);
    let normal_spec = textureSample(normal_spec_texture, normal_spec_sampler, in.uv);
    let ssao_val    = textureSample(ssao_texture,        ssao_sampler,        in.uv).r;
    let translucent = textureSample(translucency_texture, translucency_sampler, in.uv);

    // --- Step 1: apply SSAO + specular to the solid albedo only ---
    var occlusion = 1.0;
    if (render_settings.enable_ssao != 0u) {
        occlusion = ssao_val;
    }
    let specular_color = vec3<f32>(1.0) * normal_spec.a;
    let occluded_solid = albedo.rgb * occlusion + specular_color;

    // --- Step 2: blend translucency on top (over-operator) ---
    // translucent pixels were never occluded, so SSAO does not darken water/glass/leaves.
    let final_rgb = occluded_solid * (1.0 - translucent.a) + translucent.rgb * translucent.a;
    let final_a   = albedo.a + translucent.a * (1.0 - albedo.a);

    return vec4<f32>(final_rgb, final_a);
}
