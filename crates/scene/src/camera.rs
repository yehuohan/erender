//! 一个简单的摄像机模块

use magx::*;

/// 摄像机
pub struct Camera {
    /// 摄像机位置
    pub eye: Vec3,
    /// 摄像机看向的目标位置
    pub center: Vec3,
    /// 摄像机的正上方向
    pub up: Vec3,
    /// 摄像机屏幕大小(w, h)
    pub sz: (u32, u32),
}

impl Camera {
    pub fn new(eye: Vec3, center: Vec3, up: Vec3, sz: (u32, u32)) -> Self {
        Self { eye, center, up, sz }
    }

    #[inline]
    pub fn view(&self) -> Mat4 {
        // 看向的位置总是center，便于模型观察，类似arcball相机
        look_at(&self.eye, &self.center, &self.up)
    }

    #[inline]
    pub fn proj(&self) -> Mat4 {
        let aspect = (self.sz.0 as Tyf) / (self.sz.1 as Tyf);
        persp(Angle::Ang(45.0), aspect, 0.1, 100.0)
    }

    /// 以过center的x轴方向，对eye旋转
    #[inline]
    pub fn rotate_x(&mut self, ang: Angle) {
        let r = self.center + (self.center - self.eye).cross(&self.up);
        let m = rotate(&Mat4::eye(1.0), &r, ang);
        self.eye = m.mul_vec(&self.eye.to_vec4(1.0)).to_vec3();
        self.up = m.mul_vec(&self.up.to_vec4(1.0)).to_vec3();
    }

    /// 以过center的y轴方向，对eye旋转
    #[inline]
    pub fn rotate_y(&mut self, ang: Angle) {
        let r = self.center + self.up;
        let m = rotate(&Mat4::eye(1.0), &r, ang);
        self.eye = m.mul_vec(&self.eye.to_vec4(1.0)).to_vec3();
    }

    /// 以过center的z轴方向，对eye旋转
    #[inline]
    pub fn rotate_z(&mut self, ang: Angle) {
        let r = self.center + (self.center - self.eye);
        let m = rotate(&Mat4::eye(1.0), &r, ang);
        self.eye = m.mul_vec(&self.eye.to_vec4(1.0)).to_vec3();
        self.up = m.mul_vec(&self.up.to_vec4(1.0)).to_vec3();
    }

    /// 沿着(center - eye)的方向移动
    #[inline]
    pub fn move_forward(&mut self, value: f32) {
        let r = (self.center - self.eye).normalize() * value;
        let m = translate(&Mat4::eye(1.0), &r);
        self.eye = m.mul_vec(&self.eye.to_vec4(1.0)).to_vec3();
    }
}
