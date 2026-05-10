//!include camera_uniform.wgsl

//!include id_uniform.wgsl

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var tilemap_texture: texture_2d<f32>;
@group(0) @binding(2) var tilemap_sampler: sampler;

struct VertexInput {
    @location(0) @interpolate(flat) id: u32,
    @location(1) @interpolate(flat) chunk: u32,
    @location(2) position: vec3<f32>,
    @location(3) normal: vec3<f32>,
    @location(4) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) @interpolate(flat) id: u32,
    @location(3) @interpolate(flat) chunk: u32,
    @location(4) world_pos: vec3<f32>,
};

//!include render_settings.wgsl

@group(0) @binding(3) var<uniform> render_settings: RenderSettings;
@group(0) @binding(4) var<uniform> world_dim_x: u32;

struct FragmentOutput {
    @location(0) uv: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) id: vec2<u32>,
    @location(3) translucency: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos_in_view = model.position - camera.world_offset.xyz;
    if (render_settings.enable_cyclic == 1u) {
        let world_width = f32(world_dim_x);
        pos_in_view.x -= world_width * round(pos_in_view.x / world_width);
    }
    out.clip_position = camera.view_proj * vec4<f32>(pos_in_view, 1.0);
    out.tex_coords = model.tex_coords;
    out.id = model.id;
    out.chunk = model.chunk;
    out.normal = model.normal;
    out.world_pos = vec3<f32>(camera.world_offset.x + pos_in_view.x, model.position.y, model.position.z);
    return out;
}

fn calculate_translucent_lighting(base_color: vec3<f32>, face_normal: vec3<f32>, view_dir: vec3<f32>) -> vec3<f32> {
    let light_direction = normalize(render_settings.light_dir);
    let ambient_light = render_settings.ambient_light;
    let diffuse_factor = max(dot(face_normal, light_direction), 0.0);
    let final_light_factor = ambient_light + (1.0 - ambient_light) * diffuse_factor;

    // Specular is ignored for meshes for now
    return base_color * final_light_factor;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let color = textureSample(tilemap_texture, tilemap_sampler, in.tex_coords);

    if (color.a == 0.0) {
        discard;
    }

    var output: FragmentOutput;
    output.id = vec2<u32>(in.id, in.chunk);

    if (color.a < 1.0) {
        // Translucent mesh pixel
        let view_dir = normalize(camera.camera_pos.xyz - (in.world_pos - camera.world_offset.xyz));
        let lit_rgb = calculate_translucent_lighting(color.rgb, normalize(in.normal), view_dir);
        output.translucency = vec4<f32>(lit_rgb, color.a);

        // Clear other targets
        output.uv = vec4<f32>(0.0);
        output.normal = vec4<f32>(0.0, 0.0, 1.0, 0.0);
        output.depth = 1.0; // Don't write to depth
    } else {
        // Opaque mesh pixel
        output.depth = in.clip_position.z;
        output.uv = vec4<f32>(in.tex_coords, 0.0, 1.0);
        output.normal = vec4<f32>(in.normal, 1.0);
        output.translucency = vec4<f32>(0.0);
    }

    return output;
}
