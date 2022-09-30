use cgmath::*;

pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn new(config: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn to_uniform(&self) -> CameraUniform {
        CameraUniform {
            view_proj: self.build_view_projection_matrix().into(),
        }
    }

    pub fn pan<T: Clone + Into<cgmath::Vector3<f32>>>(&mut self, mov: T) {
        let vmov: cgmath::Vector3<f32> = mov.into();
        let mut forwards = self.target - self.eye;
        forwards.y = 0.0;
        forwards.normalize();
        let upwards = cgmath::Vector3::unit_y();
        let sideways = forwards.cross(upwards);
        let offset = forwards * -vmov.z + sideways * vmov.x + upwards * vmov.y;
        self.eye += offset.clone().into();
        self.target += offset.clone().into();
    }

    pub fn rotate_h(&mut self, angle: f32) {
        let sa = angle.sin();
        let ca = angle.cos();
        let off_target = self.target - self.eye;
        let new_off_target: cgmath::Vector3<f32> = (
            off_target.x * ca - off_target.z * sa,
            off_target.y,
            off_target.z * ca + off_target.x * sa,
        )
            .into();
        self.target = self.eye + new_off_target;
    }

    pub fn rotate_v(&mut self, dy: f32) {
        self.eye.y -= dy;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);
