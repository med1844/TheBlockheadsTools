//!include render_settings.wgsl

//!include camera_uniform.wgsl

@group(0) @binding(0) var uv_texture:           texture_2d<f32>;
@group(0) @binding(1) var uv_sampler:           sampler;
@group(0) @binding(2) var normal_texture:       texture_2d<f32>;
@group(0) @binding(3) var normal_sampler:       sampler;
@group(0) @binding(4) var ssao_texture:         texture_2d<f32>;
@group(0) @binding(5) var ssao_sampler:         sampler;
@group(0) @binding(6) var translucency_texture: texture_2d<f32>;
@group(0) @binding(7) var translucency_sampler: sampler;
@group(0) @binding(8) var overlay_texture:      texture_2d<f32>;
@group(0) @binding(9) var overlay_sampler:      sampler;
@group(0) @binding(10) var flags_texture:        texture_2d<u32>;
@group(0) @binding(11) var voxel_depth_texture:  texture_depth_2d;

@group(1) @binding(0) var<uniform> render_settings: RenderSettings;
@group(1) @binding(1) var<uniform> camera:      CameraUniform;
@group(1) @binding(2) var tile_map:             texture_2d<f32>;
@group(1) @binding(3) var tile_map_sampler:     sampler;
@group(1) @binding(4) var tile_reflect:         texture_2d<f32>;
@group(1) @binding(5) var tile_reflect_sampler: sampler;
@group(1) @binding(6) var tile_destruct:        texture_2d<f32>;
@group(1) @binding(7) var tile_destruct_sampler: sampler;

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

//!include lighting.wgsl

fn calculate_solid_color(in: VertexOutput, raw_depth: f32) -> vec4<f32> {
    let raw_uv = textureSampleLevel(uv_texture, uv_sampler, in.uv, 0.0).rg;
    let normal_data = textureSampleLevel(normal_texture, normal_sampler, in.uv, 0.0).rgb;
    let ssao_raw = textureSampleLevel(ssao_texture, ssao_sampler, in.uv, 0.0).r;

    // Reconstruct world position
    let ndc = vec2<f32>(in.uv.x * 2.0 - 1.0, (1.0 - in.uv.y) * 2.0 - 1.0);
    let clip_pos = vec4<f32>(ndc, raw_depth, 1.0);
    let world_pos_homo = camera.inv_view_proj * clip_pos;
    let world_pos = world_pos_homo.xyz / world_pos_homo.w;

    // Sample material
    let albedo_color = textureSampleLevel(tile_map, tile_map_sampler, raw_uv, 0.0);
    let reflect_val = textureSampleLevel(tile_reflect, tile_reflect_sampler, raw_uv, 0.0).r;

    // Normal perturbation
    var normal = normalize(normal_data);
    if (render_settings.enable_destruct != 0u) {
        let destruct_color = textureSampleLevel(tile_destruct, tile_destruct_sampler, raw_uv, 0.0).rgb;
        normal = perturb_normal(normal, destruct_color);
    }

    // SSAO
    var occlusion = 1.0;
    if (render_settings.enable_ssao != 0u) {
        occlusion = ssao_raw;
    }

    // Lighting
    var solid_rgb = calculate_lighting(render_settings.light_dir, world_pos, normal, albedo_color.rgb, reflect_val, occlusion, raw_depth);
    return vec4<f32>(solid_rgb, 1.0);
}

fn blend_alpha(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    let rgb = mix(a.rgb, b.rgb, b.a);
    return vec4<f32>(rgb, a.a + b.a * (1.0 - a.a));
}

@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let screen_size = vec2<f32>(textureDimensions(uv_texture));
    let pixel_coords = vec2<i32>(in.uv * screen_size);
    let raw_depth = textureLoad(voxel_depth_texture, pixel_coords, 0);

    var solid_color = vec4<f32>(0.0);
    if (raw_depth < 1.0) {
        solid_color = calculate_solid_color(in, raw_depth);
    }

    // Translucency blend
    let translucent = textureSample(translucency_texture, translucency_sampler, in.uv);
    let composed = blend_alpha(solid_color, translucent);

    // Flags (highlights)
    var highlight = vec4<f32>(0.0);
    let flags = textureLoad(flags_texture, pixel_coords, 0).r;
    if ((flags & 1u) != 0u) { // Hovered
        highlight = vec4<f32>(1.0, 1.0, 1.0, 0.25);
    }
    if ((flags & 2u) != 0u) { // Selected
        highlight = vec4<f32>(1.0, 1.0, 0.0, 0.3);
    }

    // Overlay blend
    let overlay = textureSample(overlay_texture, overlay_sampler, in.uv);
    let blended = blend_alpha(composed, blend_alpha(overlay, highlight));

    // Gamma correction
    let gamma = 2.2;
    let corrected = pow(blended.rgb, vec3<f32>(1.0 / gamma));

    return vec4<f32>(corrected, blended.a);
}
