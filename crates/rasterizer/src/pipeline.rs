//! Graphics Pipeline

use crate::rasterizer::IRasterizer;
use crate::shader::{IGlsl, IShader};
use magx::*;
use std::ops::Range;

/// 图元接口
///
/// 这里IPrimitive是指图元集合，便于着色器IShader应用到所有图元。
/// 这里只支持三角形图元（即三角形片段）。
pub trait IPrimitive: IShader {
    /// 图元index范围
    fn indices(&self) -> Range<usize>;
}

/// 图像渲染管线接口
///
/// 使用`dyn trait`可以让不同的IPrimitive放到一个数组中，方便遍历。
pub trait IPipeline: IRasterizer + IGlsl {
    // 管线入口，组装图元开始渲染
    //fn input_assembly(&mut self);

    /// 顶点着色器
    fn vertex(&mut self, primitive: &Box<dyn IPrimitive>, pidx: usize);

    /// 屏幕映射
    ///
    /// 计算片段三个点顶在屏幕空间中的像素坐标，包括深度信息
    fn mapping(&mut self);

    /// 背面剃除、裁剪等
    ///
    /// 返回true表示当前片段是正面
    fn culling(&mut self) -> bool;

    /// 对屏幕中的像素进行光栅化
    ///
    /// 返回在片段内部的屏幕坐标(u32, u32)和重心坐标(Vec3)
    fn rasterization(&mut self) -> Vec<(u32, u32, Vec3)>;

    /// 片段着色器（计算像素的最终颜色）
    ///
    /// - pixels: 当前正在处理片段的屏幕坐标(u32, u32)和重心坐标(Vec3)
    fn fragment(&mut self, primitive: &Box<dyn IPrimitive>, pidx: usize, pixels: &Vec<(u32, u32, Vec3)>);

    /// 深度测试
    ///
    /// - i: 屏幕坐标（通过一维数组索引）
    /// - z: 深度值
    fn test_depth(&mut self, i: usize, z: f32) -> bool;

    /// 绘制模型
    fn draw(&mut self, primitive: &Box<dyn IPrimitive>) {
        if self.glsl_vars().en_wire_frame {
            self.draw_wire(primitive);
        } else {
            self.draw_fill(primitive);
        }
    }

    /// 绘制实体模型
    fn draw_fill(&mut self, primitive: &Box<dyn IPrimitive>) {
        for pidx in primitive.indices() {
            self.vertex(primitive, pidx);
            self.mapping();
            if (!self.glsl_vars().en_cull_back_face) || (self.glsl_vars().en_cull_back_face && self.culling()) {
                let pixels = self.rasterization();
                self.fragment(primitive, pidx, &pixels);
            }
        }
    }

    /// 绘制网格模型
    fn draw_wire(&mut self, primitive: &Box<dyn IPrimitive>) {
        let fg = Vec4::fill(1.0);
        let bg = Vec4::fill(0.6).w(1.0);
        for pidx in primitive.indices() {
            self.vertex(primitive, pidx);
            self.mapping();
            let culling = self.culling();
            let color = if culling { fg } else { bg };
            if (!self.glsl_vars().en_cull_back_face) || (self.glsl_vars().en_cull_back_face && culling) {
                let (a, b, c) = self.glsl_vars().gl_FragCoord;
                let a = a.to_vec2();
                let b = b.to_vec2();
                let c = c.to_vec2();
                self.line(&[a, b], color);
                self.line(&[a, c], color);
                self.line(&[b, c], color);
            }
        }
    }
}
