// Shared lighting helpers.
// Requires: render_settings (RenderSettings) and camera (CameraUniform) in scope.

fn calculate_lighting(
    raw_light_dir: vec3<f32>,
    p_pos: vec3<f32>,
    p_normal: vec3<f32>,
    p_albedo: vec3<f32>,
    p_specular_val: f32,
    p_occlusion: f32,
    depth: f32,
) -> vec3<f32> {
    let light_dir = normalize(raw_light_dir);
    let view_dir = normalize(camera.camera_pos.xyz - p_pos);
    let half_dir = normalize(light_dir + view_dir);

    // Diffuse
    let diffuse_strength = max(dot(p_normal, light_dir), 0.0);
    let diffuse = p_albedo * (render_settings.ambient_light + diffuse_strength);

    // Specular (Blinn-Phong)
    var specular = vec3<f32>(0.0);
    if (render_settings.enable_reflect != 0u) {
        let spec_strength = pow(max(dot(p_normal, half_dir), 0.0), render_settings.shininess);
        specular = vec3<f32>(spec_strength * render_settings.specular_intensity * p_specular_val);
    }

    // Depth falloff
    let depth_factor = (1.0 - depth) * (1.0 - render_settings.min_depth_factor) + render_settings.min_depth_factor;

    return (diffuse * p_occlusion + specular) * depth_factor;
}

fn perturb_normal(normal: vec3<f32>, destruct_color: vec3<f32>) -> vec3<f32> {
    var tangent: vec3<f32>;
    var bitangent: vec3<f32>;

    if (normal.x != 0.0) {
        tangent = vec3<f32>(0.0, 0.0, 1.0);
        bitangent = vec3<f32>(0.0, -1.0, 0.0);
    } else if (normal.y != 0.0) {
        tangent = vec3<f32>(1.0, 0.0, 0.0);
        bitangent = vec3<f32>(0.0, 0.0, 1.0);
    } else {
        tangent = vec3<f32>(1.0, 0.0, 0.0);
        bitangent = vec3<f32>(0.0, -1.0, 0.0);
    }

    let local_n = vec3<f32>(
        (destruct_color.r - 0.5) * 0.5,
        (destruct_color.g - 0.5) * 0.5,
        1.0
    );

    return normalize(tangent * local_n.x + bitangent * local_n.y + normal * local_n.z);
}
