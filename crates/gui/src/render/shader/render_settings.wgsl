struct RenderSettings {
    light_dir: vec3<f32>,
    enable_reflect: u32,
    enable_destruct: u32,
    enable_ssao: u32,
    enable_cyclic: u32,
    ambient_light: f32,
    shininess: f32,
    specular_intensity: f32,
    min_depth_factor: f32,
    ambient_reflect: f32,
};
