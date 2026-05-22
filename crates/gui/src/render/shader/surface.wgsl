// Shared depth and normal checking logic for composite and SSAO

struct Surface {
    depth: f32,
    normal: vec3<f32>,
    has_opaque: bool,
    is_mesh: bool,
    is_voxel: bool,
}

fn get_surface(
    in_uv: vec2<f32>,
    screen_size: vec2<f32>
) -> Surface {
    let pixel_coords = vec2<i32>(in_uv * screen_size);
    let flags = textureLoad(flags_texture, pixel_coords, 0).r;
    let mesh_has_opaque = (flags & (1u << 2u)) != 0u;
    let voxel_has_opaque = (flags & (1u << 3u)) != 0u;

    var surf: Surface;
    surf.depth = 1.0;
    surf.normal = vec3<f32>(0.0);
    surf.has_opaque = mesh_has_opaque || voxel_has_opaque;
    surf.is_mesh = false;
    surf.is_voxel = false;

    if (mesh_has_opaque && voxel_has_opaque) {
        let m_depth = textureLoad(mesh_depth_texture, pixel_coords, 0);
        let v_depth = textureLoad(voxel_depth_texture, pixel_coords, 0);
        if (m_depth < v_depth + 1e-3) {
            surf.depth = m_depth;
            surf.normal = textureSampleLevel(mesh_normal_texture, mesh_normal_sampler, in_uv, 0.0).rgb;
            surf.is_mesh = true;
        } else {
            surf.depth = v_depth;
            surf.normal = textureSampleLevel(voxel_normal_texture, voxel_normal_sampler, in_uv, 0.0).rgb;
            surf.is_voxel = true;
        }
    } else if (mesh_has_opaque) {
        surf.depth = textureLoad(mesh_depth_texture, pixel_coords, 0);
        surf.normal = textureSampleLevel(mesh_normal_texture, mesh_normal_sampler, in_uv, 0.0).rgb;
        surf.is_mesh = true;
    } else if (voxel_has_opaque) {
        surf.depth = textureLoad(voxel_depth_texture, pixel_coords, 0);
        surf.normal = textureSampleLevel(voxel_normal_texture, voxel_normal_sampler, in_uv, 0.0).rgb;
        surf.is_voxel = true;
    }

    return surf;
}

fn get_surface_depth_only(
    in_uv: vec2<f32>,
    screen_size: vec2<f32>
) -> f32 {
    let pixel_coords = vec2<i32>(in_uv * screen_size);
    let flags = textureLoad(flags_texture, pixel_coords, 0).r;
    let mesh_has_opaque = (flags & (1u << 2u)) != 0u;
    let voxel_has_opaque = (flags & (1u << 3u)) != 0u;

    if (mesh_has_opaque && voxel_has_opaque) {
        let m_depth = textureLoad(mesh_depth_texture, pixel_coords, 0);
        let v_depth = textureLoad(voxel_depth_texture, pixel_coords, 0);
        return min(m_depth, v_depth);
    } else if (mesh_has_opaque) {
        return textureLoad(mesh_depth_texture, pixel_coords, 0);
    } else if (voxel_has_opaque) {
        return textureLoad(voxel_depth_texture, pixel_coords, 0);
    }

    return 1.0;
}
