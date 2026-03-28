struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>, // xyz
    world_offset: vec4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var tilemap_texture: texture_2d<f32>;
@group(0) @binding(2) var tilemap_sampler: sampler;

struct VertexInput {
    @location(0) id: u32,
    @location(1) position: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let pos_in_view = model.position - camera.world_offset.xyz;
    out.clip_position = camera.view_proj * vec4<f32>(pos_in_view, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let color = textureSample(tilemap_texture, tilemap_sampler, in.tex_coords);

    if (color.a == 0.0) {
        discard;
    }

    let gamma = 2.2;
    let corrected_color = pow(color.rgb, vec3<f32>(1.0 / gamma));

    var output: FragmentOutput;
    let depth = in.clip_position.z;
    output.depth = depth;
    output.color = vec4<f32>(corrected_color, 1.0);
    return output;
}
