//!include render_settings.wgsl

//!include camera_uniform.wgsl

@group(0) @binding(0) var mesh_uv_texture:           texture_2d<f32>;
@group(0) @binding(1) var mesh_uv_sampler:           sampler;
@group(0) @binding(2) var mesh_normal_texture:       texture_2d<f32>;
@group(0) @binding(3) var mesh_normal_sampler:       sampler;
@group(0) @binding(4) var voxel_uv_texture:          texture_2d<f32>;
@group(0) @binding(5) var voxel_uv_sampler:          sampler;
@group(0) @binding(6) var voxel_normal_texture:      texture_2d<f32>;
@group(0) @binding(7) var voxel_normal_sampler:      sampler;
@group(0) @binding(8) var mesh_translucency_texture: texture_2d<f32>;
@group(0) @binding(9) var mesh_translucency_sampler: sampler;
@group(0) @binding(10) var voxel_translucency_texture: texture_2d<f32>;
@group(0) @binding(11) var voxel_translucency_sampler: sampler;
@group(0) @binding(12) var voxel_translucent_depth_texture: texture_2d<f32>;
@group(0) @binding(13) var voxel_translucent_depth_sampler: sampler;
@group(0) @binding(14) var ssao_texture:         texture_2d<f32>;
@group(0) @binding(15) var ssao_sampler:         sampler;
@group(0) @binding(16) var overlay_texture:      texture_2d<f32>;
@group(0) @binding(17) var overlay_sampler:      sampler;
@group(0) @binding(18) var flags_texture:        texture_2d<u32>;
@group(0) @binding(19) var mesh_depth_texture:   texture_depth_2d;
@group(0) @binding(20) var voxel_depth_texture:  texture_depth_2d;

//!include surface.wgsl

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

fn calculate_deferred_light(uv_data: vec2<f32>, normal_data: vec4<f32>, raw_depth: f32, in_uv: vec2<f32>, ssao_raw: f32) -> vec4<f32> {
    // Reconstruct world position
    let ndc = vec2<f32>(in_uv.x * 2.0 - 1.0, (1.0 - in_uv.y) * 2.0 - 1.0);
    let clip_pos = vec4<f32>(ndc, raw_depth, 1.0);
    let world_pos_homo = camera.inv_view_proj * clip_pos;
    let world_pos = world_pos_homo.xyz / world_pos_homo.w;

    // Sample material
    let albedo_color = textureSampleLevel(tile_map, tile_map_sampler, uv_data, 0.0);
    let reflect_val = textureSampleLevel(tile_reflect, tile_reflect_sampler, uv_data, 0.0).r;

    // Normal perturbation
    var normal = normalize(normal_data.rgb);
    if (render_settings.enable_destruct != 0u) {
        let destruct_color = textureSampleLevel(tile_destruct, tile_destruct_sampler, uv_data, 0.0).rgb;
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

// Blends a premultiplied src color over an un-premultiplied dst color
fn blend_premultiplied(dst: vec4<f32>, src_premult: vec4<f32>) -> vec4<f32> {
    let rgb = dst.rgb * (1.0 - src_premult.a) + src_premult.rgb;
    let a = src_premult.a + dst.a * (1.0 - src_premult.a);
    return vec4<f32>(rgb, a);
}

fn to_premultiplied(color: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(color.rgb * color.a, color.a);
}

@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let screen_size = vec2<f32>(textureDimensions(mesh_uv_texture));
    let pixel_coords = vec2<i32>(in.uv * screen_size);
    let surf = get_surface(in.uv, screen_size);

    let m_uv_data = textureLoad(mesh_uv_texture, pixel_coords, 0).rg;
    let v_uv_data = textureLoad(voxel_uv_texture, pixel_coords, 0).rg;
    let ssao_raw = textureLoad(ssao_texture, pixel_coords, 0).r;

    var solid_color = vec4<f32>(0.0);

    if (surf.has_opaque) {
        if (surf.is_mesh) {
            solid_color = calculate_deferred_light(m_uv_data, vec4<f32>(surf.normal, 1.0), surf.depth, in.uv, ssao_raw);
        } else if (surf.is_voxel) {
            solid_color = calculate_deferred_light(v_uv_data, vec4<f32>(surf.normal, 1.0), surf.depth, in.uv, ssao_raw);
        }
    }

    // Translucency blend
    let m_translucent = textureLoad(mesh_translucency_texture, pixel_coords, 0);
    let v_translucent = textureLoad(voxel_translucency_texture, pixel_coords, 0);
    let m_depth = textureLoad(mesh_depth_texture, pixel_coords, 0);
    let v_trans_d = textureLoad(voxel_translucent_depth_texture, pixel_coords, 0).r;

    var composed = solid_color;

    let is_m_trans = m_translucent.a > 0.0;
    let is_v_trans = v_translucent.a > 0.0;

    let flags = textureLoad(flags_texture, pixel_coords, 0).r;
    // transparent mesh pixel can get occluded by opaque pixels in other objects
    // voxel won't because we are already doing ray marching
    let is_m_opaque = (flags & (1u << 2u)) != 0u;

    if (is_m_trans && is_v_trans) {
        if (m_depth > v_trans_d) {
            // Mesh is further than voxel. Blend mesh first, then voxel.
            if (!is_m_opaque && m_depth < surf.depth) { composed = blend_premultiplied(composed, m_translucent); }
            if (v_trans_d < surf.depth) { composed = blend_premultiplied(composed, v_translucent); }
        } else {
            // Voxel is further than mesh. Blend voxel first, then mesh.
            if (v_trans_d < surf.depth) { composed = blend_premultiplied(composed, v_translucent); }
            if (!is_m_opaque && m_depth < surf.depth) { composed = blend_premultiplied(composed, m_translucent); }
        }
    } else if (is_m_trans) {
        if (!is_m_opaque && m_depth < surf.depth + 1e-3) {
            composed = blend_premultiplied(composed, m_translucent);
        }
    } else if (is_v_trans) {
        if (v_trans_d < surf.depth) {
            composed = blend_premultiplied(composed, v_translucent);
        }
    }

    // Flags (highlights)
    var highlight = vec4<f32>(0.0);
    if ((flags & 1u) != 0u) { // Hovered
        highlight = to_premultiplied(vec4<f32>(1.0, 1.0, 1.0, 0.25));
    }
    if ((flags & 2u) != 0u) { // Selected
        highlight = to_premultiplied(vec4<f32>(1.0, 1.0, 0.0, 0.3));
    }

    // Overlay blend
    let overlay = textureSample(overlay_texture, overlay_sampler, in.uv); // premultiplied by pipeline
    let blended = blend_premultiplied(blend_premultiplied(composed, overlay), highlight);

    // Gamma correction
    let gamma = 2.2;
    let corrected = pow(blended.rgb, vec3<f32>(1.0 / gamma));

    return vec4<f32>(corrected, blended.a);
}
