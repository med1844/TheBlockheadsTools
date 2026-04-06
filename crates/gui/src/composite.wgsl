@group(0) @binding(0) var color_texture: texture_2d<f32>;
@group(0) @binding(1) var color_sampler: sampler;

@group(0) @binding(2) var normal_spec_texture: texture_2d<f32>;
@group(0) @binding(3) var normal_spec_sampler: sampler;

@group(0) @binding(4) var ssao_texture: texture_2d<f32>;
@group(0) @binding(5) var ssao_sampler: sampler;

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

@group(0) @binding(6) var<uniform> render_settings: RenderSettings;

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
    let base_color = textureSample(color_texture, color_sampler, in.uv);
    let normal_spec = textureSample(normal_spec_texture, normal_spec_sampler, in.uv);
    let ssao_val = textureSample(ssao_texture, ssao_sampler, in.uv).r;

    var occlusion = 1.0;
    if (render_settings.enable_ssao != 0u) {
        occlusion = ssao_val;
    }

    // Unpack the specular highlight scalar from the normal map's alpha channel
    let specular_color = vec3<f32>(1.0) * normal_spec.a;
    let final_color_rgb = (base_color.rgb * occlusion) + specular_color;

    return vec4<f32>(final_color_rgb, base_color.a);
}
