use crate::input::{EventResponse, Input};
use egui_wgpu::wgpu::{self, util::DeviceExt};
use glam::{Mat4, Vec3, Vec3Swizzles, Vec4Swizzles};

// Define how to connect the vertices to form triangles.
pub struct Camera {
    // we always look at (0, 0)
    pub distance: f32,
    pub up: Vec3,
    pub fovy: f32,   // Field of view in radians
    pub z_near: f32, // Near clipping plane
    pub z_far: f32,  // Far clipping plane
    pub world_offset: glam::Vec2,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            world_offset: glam::Vec2::new(0.0, 0.0),
            distance: 5.0,
            up: Vec3::Y,
            fovy: 45.0_f32.to_radians(),
            z_near: 0.01,
            z_far: 10000.0,
        }
    }
}

impl Camera {
    pub fn with_center(self, center: glam::Vec2) -> Self {
        Self {
            world_offset: center,
            ..self
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],     // Combined view and projection matrix
    inv_view_proj: [[f32; 4]; 4], // Combined view and projection matrix
    camera_pos: [f32; 4],         // Camera's world position (vec3 + padding)
    screen_size: [f32; 4],        // Screen width and height (vec2 + padding)
    world_offset: [f32; 4],       // World offset
}

impl Camera {
    fn eye(&self) -> Vec3 {
        Vec3::new(0.0, 0.0, self.distance)
    }

    fn target(&self) -> Vec3 {
        Vec3::ZERO
    }

    pub fn uniform(&self, window_size: (f32, f32)) -> CameraUniform {
        let (width, height) = window_size;
        let aspect = width / height;
        let view = Mat4::look_at_rh(self.eye(), self.target(), self.up);
        let proj = Mat4::perspective_rh(self.fovy, aspect, self.z_near, self.z_far);
        let pv = proj * view;
        CameraUniform {
            view_proj: pv.to_cols_array_2d(),
            inv_view_proj: pv.inverse().to_cols_array_2d(),
            camera_pos: self.eye().extend(1.0).into(),
            screen_size: [width, height, 0.0, 0.0],
            world_offset: [self.world_offset.x, self.world_offset.y, 0.0, 0.0],
        }
    }

    pub fn into_buffered(self, device: &wgpu::Device) -> CameraBuf {
        let uniform = self.uniform((1280.0, 720.0)); // placeholder value
        CameraBuf {
            camera: self,
            uniform,
            buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }
}

impl CameraUniform {
    fn window_size(&self) -> (f32, f32) {
        (self.screen_size[0], self.screen_size[1])
    }
}

pub struct CameraBuf {
    pub camera: Camera,
    pub uniform: CameraUniform,
    pub buf: wgpu::Buffer,
}

impl CameraBuf {
    const MAX_Z: f32 = 3.0;

    pub fn update_uniforms(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
        self.uniform = self.camera.uniform((width as f32, height as f32));
        queue.write_buffer(&self.buf, 0, bytemuck::cast_slice(&[self.uniform]));
    }

    fn screen_to_world_ray(&self, screen_pos: (f32, f32), window_size: (f32, f32)) -> (Vec3, Vec3) {
        let (screen_x, screen_y) = screen_pos;
        let (window_width, window_height) = window_size;

        // Convert screen coordinates to normalized device coordinates (NDC)
        // NDC range from -1 to 1
        let ndc_x = (screen_x / window_width) * 2.0 - 1.0;
        let ndc_y = (1.0 - (screen_y / window_height)) * 2.0 - 1.0; // Y is inverted in screen space

        let inv_view_proj = Mat4::from_cols_array_2d(&self.uniform.inv_view_proj);

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

    fn screen_to_xy_at_z(
        &self,
        screen_pos: (f32, f32),
        window_size: (f32, f32),
        z_plane: f32,
    ) -> glam::Vec2 {
        let (ray_origin, ray_direction) = self.screen_to_world_ray(screen_pos, window_size);
        let t = (z_plane - ray_origin.z) / ray_direction.z;
        (ray_origin + t * ray_direction).xy()
    }

    pub fn mouse_at(&self, input: &Input) -> glam::Vec2 {
        let window_size = self.uniform.window_size();
        self.screen_to_xy_at_z(input.current_mouse_pos, window_size, Self::MAX_Z)
            + self.camera.world_offset
    }

    pub fn handle_input(&mut self, input: &Input) -> EventResponse {
        let mut any_update = false;
        if input.is_mouse_left_down {
            let window_size = self.uniform.window_size();
            let prev_world_pos_at_z3 =
                self.screen_to_xy_at_z(input.prev_mouse_pos, window_size, Self::MAX_Z);
            let curr_world_pos_at_z3 =
                self.screen_to_xy_at_z(input.current_mouse_pos, window_size, Self::MAX_Z);
            let diff = curr_world_pos_at_z3 - prev_world_pos_at_z3;
            self.camera.world_offset -= diff;
            any_update |= diff != glam::Vec2::ZERO;
        }
        if input.mouse_wheel_delta != 0.0 {
            self.camera.distance *= 1.0 - input.mouse_wheel_delta * 1e-1;
            any_update = true;
        }
        EventResponse {
            repaint: any_update,
            ..Default::default()
        }
    }
}
