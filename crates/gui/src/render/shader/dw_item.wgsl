//!include camera_uniform.wgsl

//!include id_uniform.wgsl

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var items_texture: texture_2d<f32>;
@group(0) @binding(2) var items_sampler: sampler;
@group(0) @binding(3) var tilemap_texture: texture_2d<f32>;
@group(0) @binding(4) var tilemap_sampler: sampler;
@group(0) @binding(5) var<uniform> hover_on_id: IdUniform;
@group(0) @binding(6) var<uniform> selected_id: IdUniform;

//!include render_settings.wgsl
@group(0) @binding(7) var<uniform> render_settings: RenderSettings;
@group(0) @binding(8) var<uniform> world_dim_x: u32;

//!include item.wgsl

struct DynObjVertexInput {
    @location(0) position: vec2<f32>,
};

struct DynObjInstanceInput {
    @location(1) instance_pos: vec2<f32>,
    @location(2) item_type_or_block: u32,
    @location(3) top_side: u32,
    @location(4) id: u32,
    @location(5) chunk: u32,
};

struct DynObjVSOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) item_type: u32,
    @location(2) @interpolate(flat) is_block: u32,
    @location(3) @interpolate(flat) top: u32,
    @location(4) @interpolate(flat) side: u32,
    @location(5) @interpolate(flat) id: u32,
    @location(6) @interpolate(flat) chunk: u32,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) id: vec2<u32>,
}

@vertex
fn vs_dynamic_object_item(model: DynObjVertexInput, instance: DynObjInstanceInput) -> DynObjVSOutput {
    var out: DynObjVSOutput;

    let world_pos = vec3<f32>(instance.instance_pos + model.position, 2.0);
    var pos_in_view = world_pos - camera.world_offset.xyz;

    if (render_settings.enable_cyclic == 1u) {
        let world_width = f32(world_dim_x);
        pos_in_view.x -= world_width * round(pos_in_view.x / world_width);
    }

    out.clip_position = camera.view_proj * vec4<f32>(pos_in_view, 1.0);

    // Pass model position to fragment shader for UV calculation
    // Remap from [-0.5, 0.5] to [0, 1]
    out.uv = model.position + 0.5;
    out.uv.y = 1 - out.uv.y;
    out.item_type = instance.item_type_or_block & 0xFFFF;
    out.is_block = (instance.item_type_or_block >> 16) & 1;
    out.top = instance.top_side & 0xFFFF;
    out.side = instance.top_side >> 16;
    out.id = instance.id;
    out.chunk = instance.chunk;

    return out;
}

@fragment
fn fs_dynamic_object_item(in: DynObjVSOutput) -> FragmentOutput {
    var color = sample_item_texture(in.item_type, in.is_block, in.top, in.side, in.uv);

    // Add outline
    let outline_width: f32 = 0.03; // Adjust as needed for desired thickness
    let dist_from_edge_x = min(in.uv.x, 1.0 - in.uv.x);
    let dist_from_edge_y = min(in.uv.y, 1.0 - in.uv.y);

    if (dist_from_edge_x < outline_width || dist_from_edge_y < outline_width) {
        let outline_alpha: f32 = 0.5;
        let outline_color: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, outline_alpha); // White with 0.5 alpha

        // Simple blend, overlaying the outline
        // You might want a more sophisticated blend mode depending on desired effect
        color = mix(color, outline_color, outline_color.a);
    }

    let hovered = hover_on_id.is_some != 0u
        && in.chunk == hover_on_id.chunk
        && in.id == hover_on_id.id;

    var final_color: vec3<f32> = color.rgb;
    if (hovered) {
        final_color = mix(final_color, vec3<f32>(1.0), 0.15);
    }

    let selected = selected_id.is_some != 0u
        && in.chunk == selected_id.chunk
        && in.id == selected_id.id;

    if (selected) {
        final_color = mix(final_color, vec3<f32>(1.0, 1.0, 0.0), 0.25);
    }

    var output: FragmentOutput;
    output.color = vec4<f32>(final_color, color.a);
    output.id = vec2<u32>(in.id, in.chunk);
    return output;
}
