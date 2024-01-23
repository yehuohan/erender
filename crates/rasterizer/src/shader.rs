//! Shader Language

use magx::*;
use std::any::Any;

pub trait IGlsl {
    fn glsl_vars(&self) -> &GlslVars;
    fn wire_frame(&mut self) -> &mut bool;
    fn cull_face(&mut self) -> &mut bool;
    fn enable_wire_frame(&mut self, val: bool);
    fn enable_cull_face(&mut self, val: bool);
}

/// 着色器内建变量
///
/// 类似于GLSL中的内建变量，但这里以片段（三角面）为基础。
#[allow(non_snake_case)]
pub struct GlslVars {
    // 当前绘制的片段索引
    //pub gl_InstanceID: usize,
    /// 顶点着色器输出片段顶点（在投影坐标中）的位置向量
    pub gl_Postion: (Vec4, Vec4, Vec4),
    /// 当前片段面的朝向(正面或背面朝向摄像头)
    pub gl_FrontFacing: bool,
    /// 片段着色器中片段的屏幕坐标
    pub gl_FragCoord: (Vec4, Vec4, Vec4),

    /// 颜色buffer
    pub cbuf: Vec<[u8; 4]>,
    /// 深度buffer
    pub zbuf: Vec<f32>,
    pub en_wire_frame: bool,
    pub en_cull_back_face: bool,
}

impl GlslVars {
    pub fn new(sz: (u32, u32)) -> Self {
        let max = (sz.0 * sz.1) as usize;
        Self {
            gl_Postion: (Vec4::new(), Vec4::new(), Vec4::new()),
            gl_FrontFacing: true,
            gl_FragCoord: (Vec4::new(), Vec4::new(), Vec4::new()),
            cbuf: vec![[0; 4]; max],
            zbuf: vec![1.0; max],
            en_wire_frame: true,
            en_cull_back_face: true,
        }
    }
}

/// 着色器接口
pub trait IShader {
    /// 设置uniform变量
    fn set_uniforms(&mut self, _u: Box<dyn Any>) {}

    /// 顶点着色器
    ///
    /// - pidx: 图元index
    fn vertex(&self, pidx: usize) -> (Vec4, Vec4, Vec4);

    /// 片段着色器
    ///
    /// - pidx: 图元index
    fn fragment(&self, pidx: usize, bc: &Vec3) -> Vec4;
}

/// 着色器基本变换矩阵变量
#[derive(Debug, Copy, Clone)]
pub struct UniformMatrix {
    /// 模型变换矩阵
    pub model: Mat4,
    /// 视图变换矩阵
    pub view: Mat4,
    /// 投影变换矩阵
    pub proj: Mat4,
    /// model-view的逆(inverse)的转置(transpose)矩阵，用于校正法向量
    pub mit: Mat3,
    /// 顶点MVP变换矩阵
    pub mvp: Mat4,
}

impl UniformMatrix {
    pub fn new() -> Self {
        Self {
            model: Mat4::eye(1.0),
            view: Mat4::eye(1.0),
            proj: Mat4::eye(1.0),
            mit: Mat3::eye(1.0),
            mvp: Mat4::eye(1.0),
        }
    }

    /// 计算mit矩阵
    ///
    /// 模型变换时，其法线也会变化：
    /// - 等比变换不会破坏法线
    /// - 不等比变换会导致法向量不再垂直表面，需要使用法线矩阵校正
    pub fn calc_mit(&mut self) {
        // 使用model的inverse-transpose，表示世界坐标系中的法向量；
        // 一般光源也是使用世界坐标系，便于光照计算。
        self.mit = self.model.inverse().transpose().to_mat3();
        // 使用model-view的inverse-transpose，表示相机坐标中的法向量；
        // 此时相机eye的方向为(0, 0, 1)，为了计算光照，需要将光源同样变换相机坐标系中；
        //self.mit = self.view.mul_mat(&self.model).inverse().transpose().to_mat3();
    }

    pub fn calc_mvp(&mut self) {
        self.mvp = self.proj.mul_mat(&self.view).mul_mat(&self.model);
    }
}
