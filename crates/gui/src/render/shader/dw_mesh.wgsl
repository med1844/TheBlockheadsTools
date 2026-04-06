struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>, // xyz
    world_offset: vec4<f32>,
};

struct IdUniform {
    is_some: u32,
    id: u32,
    x: u32,
    y: u32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var tilemap_texture: texture_2d<f32>;
@group(0) @binding(2) var tilemap_sampler: sampler;
@group(0) @binding(3) var<uniform> hover_on_id: IdUniform;
@group(0) @binding(4) var<uniform> selected_id: IdUniform;

struct VertexInput {
    @location(0) @interpolate(flat) id: u32,
    @location(1) @interpolate(flat) chunk_x: u32,
    @location(2) @interpolate(flat) chunk_y: u32,
    @location(3) position: vec3<f32>,
    @location(4) normal: vec3<f32>,
    @location(5) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) @interpolate(flat) id: u32,
    @location(3) @interpolate(flat) chunk_x: u32,
    @location(4) @interpolate(flat) chunk_y: u32,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) normal_spec: vec4<f32>,
    @location(2) id: u32,
    @builtin(frag_depth) depth: f32,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let pos_in_view = model.position - camera.world_offset.xyz;
    out.clip_position = camera.view_proj * vec4<f32>(pos_in_view, 1.0);
    out.tex_coords = model.tex_coords;
    out.id = model.id;
    out.chunk_x = model.chunk_x;
    out.chunk_y = model.chunk_y;
    out.normal = model.normal;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let color = textureSample(tilemap_texture, tilemap_sampler, in.tex_coords);

    if (color.a == 0.0) {
        discard;
    }

    // Highlight when both chunk coord and object id match the hovered target.
    let hovered = hover_on_id.is_some != 0u
        && in.chunk_x == hover_on_id.x
        && in.chunk_y == hover_on_id.y
        && in.id == hover_on_id.id;

    var final_color: vec3<f32> = color.rgb;
    if (hovered) {
        // Brighten and tint towards white to indicate hover.
        final_color = mix(final_color, vec3<f32>(1.0), 0.25);
    }

    let selected = selected_id.is_some != 0u
        && in.chunk_x == selected_id.x
        && in.chunk_y == selected_id.y
        && in.id == selected_id.id;

    if (selected) {
        final_color = mix(final_color, vec3<f32>(1.0, 1.0, 0.0), 0.15);
    }

    var output: FragmentOutput;
    let depth = in.clip_position.z;
    output.depth = depth;
    output.color = vec4<f32>(final_color, color.a);
    output.normal_spec = vec4<f32>(in.normal, 0.0);
    output.id = in.id;
    return output;
}
