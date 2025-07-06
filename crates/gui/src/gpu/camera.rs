use egui_wgpu::wgpu::{self, util::DeviceExt};
use glam::{Mat4, Vec3};

// Define how to connect the vertices to form triangles.
pub struct Camera {
    pub center: glam::Vec2,
    pub distance: f32,
    pub up: Vec3,
    pub fovy: f32,   // Field of view in radians
    pub z_near: f32, // Near clipping plane
    pub z_far: f32,  // Far clipping plane
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            center: glam::Vec2::new(0.0, 0.0),
            distance: 5.0,
            up: Vec3::Y,
            fovy: 45.0_f32.to_radians(),
            z_near: 0.01,
            z_far: 10000.0,
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
}

impl Camera {
    fn eye(&self) -> Vec3 {
        self.center.extend(self.distance)
    }

    fn target(&self) -> Vec3 {
        self.center.extend(0.0)
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
        }
    }

    pub fn into_buffered(self, device: &wgpu::Device) -> CameraBuf {
        let uniform = self.uniform((1280.0, 720.0)); // placeholder value
        CameraBuf {
            camera: self,
            buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }
}

pub struct CameraBuf {
    pub camera: Camera,
    pub buf: wgpu::Buffer,
}

impl CameraBuf {
    pub fn update_uniforms(&self, queue: &wgpu::Queue, width: u32, height: u32) {
        let camera = self.camera.uniform((width as f32, height as f32));
        queue.write_buffer(&self.buf, 0, bytemuck::cast_slice(&[camera]));
    }
}
