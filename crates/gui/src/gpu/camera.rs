// use crate::input::{EventResponse, Input};
use eframe::wgpu::{self, util::DeviceExt};
use glam::{Mat4, Vec3, Vec3Swizzles, Vec4Swizzles};

// Define how to connect the vertices to form triangles.
pub struct Camera {
    // we always look at (0, 0)
    fovy: f32,   // Field of view in radians
    z_near: f32, // Near clipping plane
    z_far: f32,  // Far clipping plane
    world_offset: Vec3,
    aspect: f32,
    world_width_blocks: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            world_offset: Vec3::new(0.0, 0.0, 5.0),
            fovy: 45.0_f32.to_radians(),
            z_near: 0.01,
            z_far: 10000.0,
            aspect: 1.0,
            world_width_blocks: Self::DEFAULT_WORLD_WIDTH_BLOCKS,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],     // Combined view and projection matrix
    inv_view_proj: [[f32; 4]; 4], // Combined view and projection matrix
    camera_pos: [f32; 4],         // Camera's world position (vec3 + padding)
    world_offset: [f32; 4],       // World offset
    world_dims: [f32; 4],         // World dimensions (x,y,z + padding)
}

impl Camera {
    pub const MAX_BLOCK_Z: f32 = 3.0;
    pub const MAX_Z: f32 = 1e8;
    const DEFAULT_WORLD_WIDTH_BLOCKS: f32 = 512.0 * 32.0;
    const WORLD_HEIGHT_BLOCKS: f32 = 32.0 * 32.0;
    const WORLD_DEPTH_BLOCKS: f32 = 3.0;

    fn eye(&self) -> Vec3 {
        Vec3::Z
    }

    fn target(&self) -> Vec3 {
        Vec3::ZERO
    }

    fn view_proj(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye(), self.target(), Vec3::Y);
        let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.z_near, self.z_far);
        proj * view
    }

    fn inv_view_proj(&self) -> Mat4 {
        self.view_proj().inverse()
    }

    pub fn set_aspect(&mut self, new_aspect: f32) {
        self.aspect = new_aspect;
    }

    pub fn set_world_width_blocks(&mut self, new_width: f32) {
        if new_width > 0.0 {
            self.world_width_blocks = new_width;
        }
    }

    pub fn world_width_blocks(&self) -> f32 {
        self.world_width_blocks
    }

    pub fn world_offset(&self) -> &Vec3 {
        &self.world_offset
    }

    pub fn world_offset_mut(&mut self) -> &mut Vec3 {
        &mut self.world_offset
    }

    pub fn uniform(&self) -> CameraUniform {
        CameraUniform {
            view_proj: self.view_proj().to_cols_array_2d(),
            inv_view_proj: self.inv_view_proj().to_cols_array_2d(),
            camera_pos: self.eye().extend(1.0).into(),
            world_offset: [
                self.world_offset.x,
                self.world_offset.y,
                self.world_offset.z,
                0.0,
            ],
            world_dims: [
                self.world_width_blocks,
                Self::WORLD_HEIGHT_BLOCKS,
                Self::WORLD_DEPTH_BLOCKS,
                0.0,
            ],
        }
    }

    pub fn to_buf(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let uniform = self.uniform();
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn viewport_to_world_ray(
        &self,
        viewport_pos: (f32, f32),
        viewport_size: (f32, f32),
    ) -> (Vec3, Vec3) {
        let (viewport_x, viewport_y) = viewport_pos;
        let (viewport_width, viewport_height) = viewport_size;

        // Convert screen coordinates to normalized device coordinates (NDC)
        // NDC range from -1 to 1
        let ndc_x = (viewport_x / viewport_width) * 2.0 - 1.0;
        let ndc_y = (1.0 - (viewport_y / viewport_height)) * 2.0 - 1.0; // Y is inverted in screen space

        let inv_view_proj = self.inv_view_proj();

        // Create a ray in clip space (start at z=-1 for near plane, end at z=1 for far plane)
        let ray_clip_start = Vec3::new(ndc_x, ndc_y, -1.0).extend(1.0);
        let ray_clip_end = Vec3::new(ndc_x, ndc_y, 1.0).extend(1.0);

        // Transform ray to world space
        let ray_world_start = inv_view_proj * ray_clip_start;
        let ray_world_end = inv_view_proj * ray_clip_end;

        let ray_world_start = ray_world_start.xyz() / ray_world_start.w;
        let ray_world_end = ray_world_end.xyz() / ray_world_end.w;

        let ray_origin = ray_world_start;
        let ray_direction = (ray_world_end - ray_world_start).normalize();

        (ray_origin, ray_direction)
    }

    fn viewport_to_xy_at_z(
        &self,
        viewport_pos: (f32, f32),
        viewport_size: (f32, f32),
        z_plane: f32,
    ) -> glam::Vec2 {
        let (ray_origin, ray_direction) = self.viewport_to_world_ray(viewport_pos, viewport_size);
        let t = (z_plane - ray_origin.z) / ray_direction.z;
        (ray_origin + t * ray_direction).xy()
    }

    pub fn handle_drag(
        &mut self,
        prev_pos: (f32, f32),
        cur_pos: (f32, f32),
        viewport_size: (f32, f32),
    ) {
        let prev_world_pos_at_z3 = self.viewport_to_xy_at_z(
            prev_pos,
            viewport_size,
            Self::MAX_BLOCK_Z - self.world_offset.z,
        );
        let curr_world_pos_at_z3 = self.viewport_to_xy_at_z(
            cur_pos,
            viewport_size,
            Self::MAX_BLOCK_Z - self.world_offset.z,
        );
        let diff = curr_world_pos_at_z3 - prev_world_pos_at_z3;
        self.world_offset -= diff.extend(0.0);
    }

    pub fn handle_zoom(&mut self, pos: (f32, f32), viewport_size: (f32, f32), scroll_y: f32) {
        let world_pos_before_zoom = self.mouse_at(pos, viewport_size);
        let old_z = self.world_offset.z;

        self.world_offset.z *= 1.0 - scroll_y * 8e-3;
        self.world_offset.z = self.world_offset.z.clamp(Self::MAX_BLOCK_Z, Self::MAX_Z);
        let new_z = self.world_offset.z;

        if new_z != old_z {
            let world_pos_after_zoom = self.mouse_at(pos, viewport_size);

            let drift = world_pos_after_zoom - world_pos_before_zoom;

            self.world_offset.x -= drift.x;
            self.world_offset.y -= drift.y;
        }
    }

    pub fn visible_world_region_2d(&self, screen_size: (f32, f32)) -> [glam::Vec2; 2] {
        let (width, height) = screen_size;

        let top_right = self.viewport_to_xy_at_z(
            (width, 0.0),
            screen_size,
            Self::MAX_BLOCK_Z - self.world_offset.z,
        );
        let bottom_left = self.viewport_to_xy_at_z(
            (0.0, height),
            screen_size,
            Self::MAX_BLOCK_Z - self.world_offset.z,
        );

        [
            bottom_left + self.world_offset.xy(),
            top_right + self.world_offset.xy(),
        ]
    }

    pub fn mouse_at(&self, viewport_pos: (f32, f32), viewport_size: (f32, f32)) -> glam::Vec2 {
        self.viewport_to_xy_at_z(
            viewport_pos,
            viewport_size,
            Self::MAX_BLOCK_Z - self.world_offset.z,
        ) + self.world_offset.xy()
    }
}
