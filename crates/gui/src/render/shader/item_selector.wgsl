@group(0) @binding(0) var items_texture: texture_2d<f32>;
@group(0) @binding(1) var items_sampler: sampler;
@group(0) @binding(2) var tilemap_texture: texture_2d<f32>;
@group(0) @binding(3) var tilemap_sampler: sampler;

struct ItemSelectorUniforms {
    hovered_index: u32,
    selected_index: u32,
    // Top-left corner of the visible viewport in grid-pixel space.
    viewport_origin: vec2<f32>,
    // Width and height of the visible viewport in grid-pixels.
    viewport_size: vec2<f32>,
    // Width and height of a single item cell in grid-pixels.
    cell_size: vec2<f32>,
}
@group(0) @binding(4) var<uniform> uniforms:              ItemSelectorUniforms;
//!include render_settings.wgsl
@group(0) @binding(5) var<uniform> render_settings:       RenderSettings;
//!include camera_uniform.wgsl
@group(0) @binding(6) var<uniform> camera:                CameraUniform;
@group(0) @binding(7) var          tile_destruct:         texture_2d<f32>;
@group(0) @binding(8) var          tile_destruct_sampler: sampler;
@group(0) @binding(9) var          tile_reflect:          texture_2d<f32>;
@group(0) @binding(10) var         tile_reflect_sampler:  sampler;

//!include lighting.wgsl
//!include item.wgsl

struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct InstanceInput {
    // position stores [col, row] as f32.
    @location(1) instance_pos: vec2<f32>,
    // item_type in low 16 bits, is_block flag in bit 16. The rest are padding.
    @location(2) item_type_or_block: u32,
    // top tile index in low 16 bits, side tile index in high 16 bits.
    @location(3) top_side: u32,
    // locations 4 (id) and 5 (chunk) exist in the buffer but are unused here.
}

struct VSOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) item_type: u32,
    @location(2) @interpolate(flat) is_block: u32,
    @location(3) @interpolate(flat) top: u32,
    @location(4) @interpolate(flat) side: u32,
}

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VSOutput {
    var out: VSOutput;
    let pixel_pos = (instance.instance_pos + model.position + 0.5) * uniforms.cell_size;

    // Translate by the viewport origin (scroll offset) so that pixel_pos
    // becomes relative to the top-left of the visible area.
    let relative = pixel_pos - uniforms.viewport_origin;

    // Map the visible area [0, viewport_size] -> NDC [-1, 1].
    // Y is flipped because screen-space Y grows downward but NDC Y grows upward.
    let ndc_x = (relative.x / uniforms.viewport_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (relative.y / uniforms.viewport_size.y) * 2.0;

    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);

    out.uv = model.position + 0.5;

    // Unpack item_type_or_block and top_side.
    out.item_type = instance.item_type_or_block & 0xFFFF;
    out.is_block  = (instance.item_type_or_block >> 16) & 1u;
    out.top       = instance.top_side & 0xFFFF;
    out.side      = instance.top_side >> 16;

    return out;
}

@fragment
fn fs_main(in: VSOutput) -> @location(0) vec4<f32> {
    var color = sample_item_texture(in.item_type, in.is_block, in.top, in.side, in.uv);

    let outline_width: f32 = 0.03;
    let dist_from_edge_x = min(in.uv.x, 1.0 - in.uv.x);
    let dist_from_edge_y = min(in.uv.y, 1.0 - in.uv.y);

    if (dist_from_edge_x < outline_width || dist_from_edge_y < outline_width) {
        if (in.item_type == uniforms.selected_index) {
            let outline_color = vec4<f32>(1.0, 1.0, 0.0, 1.0);
            color = mix(color, outline_color, outline_color.a);
        } else if (in.item_type == uniforms.hovered_index) {
            let outline_color = vec4<f32>(1.0, 1.0, 1.0, 0.5);
            color = mix(color, outline_color, outline_color.a);
        }
    }

    // Gamma correction
    let gamma = 2.2;
    let corrected = pow(color.rgb, vec3<f32>(1.0 / gamma));

    return vec4<f32>(corrected, color.a);
}
