@group(0) @binding(0) var color_texture: texture_2d<f32>;
@group(0) @binding(1) var color_sampler: sampler;
@group(0) @binding(2) var translucency_texture: texture_2d<f32>;
@group(0) @binding(3) var translucency_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_blit(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
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
fn fs_blit(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(color_texture, color_sampler, in.uv);
    let trans_color = textureSample(translucency_texture, translucency_sampler, in.uv);
    
    let final_rgb = trans_color.rgb + base_color.rgb * (1.0 - trans_color.a);
    let final_a = trans_color.a + base_color.a * (1.0 - trans_color.a);
    
    return vec4<f32>(final_rgb, final_a);
}
