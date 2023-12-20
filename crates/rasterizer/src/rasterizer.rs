//! 图形光栅渲染器

use magx::*;
use crate::pipeline::{IPipeline, IPrimitive};
use crate::shader::{GlslVars, IGlsl};


/// 光栅渲染器接口
pub trait IRasterizer {
    /// 输出color buffer像素颜色
    ///
    /// - i,j: 像素坐标
    fn set_color(&mut self, i: u32, j: u32, color: &Vec4);

    /// 绘制直线
    ///
    /// - points: 2个端点
    /// - color: 线段颜色
    ///
    /// 此算法直接使用浮点计算，性能一般，代码注重易于理解。
    ///
    /// 注意：这里直接在屏幕坐标空间上进行颜色线性插值，对于3D渲染，应当在三维坐标空间插值；
    /// 因为在三维空间是线性变化，被投射到屏幕坐标空间的时候就不再是线性的了。
    fn line(&mut self, points: &[Vec2; 2], color: Vec4) {
        let ab: Vec2 = points[1] - points[0]; // 从第0个点向第1个点画线
        let num = ab.x.abs().max(ab.y.abs());
        let dxy = ab / num;
        let mut p = points[0]; // 从第0个点开始画
        for _ in 0 .. num as u32 {
            p += dxy;
            self.set_color(p.x as u32, p.y as u32, &color);
        }
    }

    /// 绘制三角形
    ///
    /// - points: 3个顶点
    /// - colors: 3个顶点颜色（至少提供1个颜色）
    ///
    /// 注意：这里直接在屏幕坐标空间上进行颜色线性插值，对于3D渲染，应当在三维坐标空间插值；
    /// 因为在三维空间是线性变化，被投射到屏幕坐标空间的时候就不再是线性的了。
    fn triangle(&mut self, points: &[Vec2; 3], colors: &[Vec4]) {
        let (lb, rt) = bound_box(points);

        for i in lb.x as u32 .. rt.x as u32 { // 小于0的坐标，为自动转成0
            for j in lb.y as u32 .. rt.y as u32 {
                if let Some(bc) = barycentric(points, Vec2::from(i as Tyf, j as Tyf)) {
                    // 基于顶点颜色进行插值
                    let c = if colors.len() >= 3 {
                        interpolate(&bc, &colors[0], &colors[1], &colors[2])
                    } else {
                        colors[0]
                    };
                    self.set_color(i, j, &c);
                }
            }
        }
    }
}

/// 光栅渲染器
#[allow(dead_code)]
pub struct Rasterizer {
    /// 屏幕大小(w, h)
    pub sz: (u32, u32),
    /// 视口变换矩阵
    mat_viewport: Mat4,
    gv: GlslVars,
}

impl Rasterizer {
    /// 创建Rasterizer，设置相关数据
    pub fn new(sz: (u32, u32)) -> Self {
        Self {
            mat_viewport: viewport(0.0, 0.0, sz.0 as Tyf, sz.1 as Tyf),
            sz,
            gv: GlslVars::new(sz),
        }
    }

    /// 返回颜色buffer
    #[inline]
    pub fn get_color(&self) -> &Vec<[u8; 4]> {
        &self.gv.cbuf
    }

    /// 清除颜色buffer
    ///
    /// - color: 用于填充color buffer的颜色
    #[inline]
    pub fn clear_color(&mut self, color: &Vec4) {
        let c = (*color) * 255.0;
        self.gv.cbuf.fill([c.x as u8, c.y as u8, c.z as u8, c.w as u8]);
    }

    #[inline]
    pub fn get_depth(&self) -> &Vec<f32> {
        &self.gv.zbuf
    }

    #[inline]
    pub fn clear_depth(&mut self) {
        self.gv.zbuf.fill(f32::MAX);
    }
}

impl IRasterizer for Rasterizer {
    #[inline]
    fn set_color(&mut self, i: u32, j: u32, color: &Vec4) {
        if i < self.sz.0 && j < self.sz.1 {
            let c = (*color) * 255.0;
            self.gv.cbuf[(i + j * self.sz.0) as usize] = [c.x as u8, c.y as u8, c.z as u8, c.w as u8];
        }
    }
}

impl IGlsl for Rasterizer {
    #[inline]
    fn glsl_vars(&self) -> &GlslVars {
        &self.gv
    }

    #[inline]
    fn enable_wire_frame(&mut self, val: bool) {
        self.gv.en_wire_frame = val;
    }

    #[inline]
    fn enable_cull_face(&mut self, val: bool) {
        self.gv.en_cull_back_face = val;
    }
}

impl IPipeline for Rasterizer {
    #[inline]
    fn vertex(&mut self, primitive: &Box<dyn IPrimitive>, pidx: usize) {
        self.gv.gl_Postion = primitive.vertex(pidx);
    }

    fn mapping(&mut self) {
        let (mut a, mut b, mut c) = self.gv.gl_Postion;
        let aw = a.w;
        let bw = b.w;
        let cw = c.w;

        // 对于视口边界的片段，应该进行裁剪，
        // 这里不便修改Model中数据，放到rasterization中自动实现

        // 透视除法
        a = a / a.w;
        b = b / b.w;
        c = c / c.w;

        // 视口变换，得到屏幕坐标
        a = self.mat_viewport.mul_vec(&a);
        b = self.mat_viewport.mul_vec(&b);
        c = self.mat_viewport.mul_vec(&c);

        self.gv.gl_FragCoord = (a, b, c);
        // 保存值1/w到gl_FragCoord，用于透视逆运算（重心坐标校正）
        self.gv.gl_FragCoord.0.w = aw;
        self.gv.gl_FragCoord.1.w = bw;
        self.gv.gl_FragCoord.2.w = cw;
    }

    fn culling(&mut self) -> bool {
        let (a, b, c) = self.gv.gl_FragCoord;
        let a = a.to_vec2().to_vec3(0.0);
        let b = b.to_vec2().to_vec3(0.0);
        let c = c.to_vec2().to_vec3(0.0);

        // back-face culling（通过三角形顶点顺序剃除背面的片段）
        // 在屏幕坐标空间中，摄像头看向的方向即是-z方向，
        // 所以三角形的法向量只需要z方向比较即可
        let normal: Vec3 = (b - a).cross(&(c - a)).normalize();
        self.gv.gl_FrontFacing = normal.dot(&Vec3::from(0.0, 0.0, 1.0)) > 0.0;

        return self.gv.gl_FrontFacing;
    }

    fn rasterization(&mut self) -> Vec<(u32, u32, Vec3)> {
        let (a, b, c) = self.gv.gl_FragCoord;
        let abc = [a.to_vec2(), b.to_vec2(), c.to_vec2()];

        // 自动丢弃视口外的像素点，实现裁剪
        let (lb, rt) = bound_box(&abc);
        let xlo = lb.x.min(0.0) as u32;
        let ylo = lb.y.min(0.0) as u32;
        let xhi = rt.x.min(self.sz.0 as Tyf) as u32;
        let yhi = rt.y.min(self.sz.1 as Tyf) as u32;
        // 光栅化，用pixels保存一个片段中所有需要着色的像素点
        let mut pixels = Vec::new();

        for i in xlo .. xhi { // 小于0的坐标，为自动转成0
            for j in ylo .. yhi {
                if let Some(bc) = barycentric(&abc, Vec2::from(i as Tyf, j as Tyf)) {
                    // 重心坐标校正
                    // - https://www.comp.nus.edu.sg/~lowkl/publications/lowk_persp_interp_techrep.pdf
                    // - https://zhuanlan.zhihu.com/p/144331875
                    // 这里除以w和z是等效的，因为三维空间中，w和z成线性关系（因为透视的原理即z起大，片段越小）
                    let bcc = Vec3::from(bc.x / a.w, bc.y / b.w, bc.z / c.w);
                    let bc = bcc / (bcc.x + bcc.y + bcc.z);

                    // Test: 取三个点的均值，渲染出三角面模型效果
                    //let bc = Vec3::fill(1.0) / 3.0;

                    // 深度测试
                    let z = interpolate(&bc, &a.z, &b.z, &c.z) as f32;
                    if self.test_depth((i + j * self.sz.0) as usize, z) {
                        pixels.push((i, j, bc));
                    }
                }
            }
        }
        pixels
    }

    #[inline]
    fn fragment(&mut self, primitive: &Box<dyn IPrimitive>, pidx: usize, pixels: &Vec<(u32, u32, Vec3)>) {
        for &(i, j, bc) in pixels {
            self.set_color(i, j, &primitive.fragment(pidx, &bc));
        }
    }

    #[inline]
    fn test_depth(&mut self, i: usize, z: f32) -> bool {
        // 通过zbuffer剃除被遮挡的片段，z值通过重心坐标插值计算；
        // 丢弃 深度值>=当前深度缓冲值 的片段（丢离视点更远的片段）；
        // viewport()计算的z范围为[-1.0, 1.0]。
        if -1.0 <= z && z <= 1.0 && self.gv.zbuf[i] > z {
            self.gv.zbuf[i] = z;
            return true;
        }
        false
    }
}



#[test]
fn rasterizer_test() {
    use magx::*;
    use viewer::{Viewer, EBuffer};

    struct R {
        sz: (u32, u32),
        cbuf: Vec<[u8; 4]>,
    }

    impl R {
        pub fn new(sz: (u32, u32)) -> Self {
            let max = (sz.0 * sz.1) as usize;
            Self { sz, cbuf: vec![[0; 4]; max] }
        }

        pub fn get_color(&self) -> &Vec<[u8; 4]> {
            &self.cbuf
        }
    }

    impl IRasterizer for R {
        #[inline]
        fn set_color(&mut self, i: u32, j: u32, color: &Vec4) {
            if i < self.sz.0 && j < self.sz.1 {
                let c = (*color) * 255.0;
                self.cbuf[(i + j * self.sz.0) as usize] = [c.x as u8, c.y as u8, c.z as u8, c.w as u8];
            }
        }
    }

    let mut r = R::new((640, 480));
    let a = Vec2::from(10.0, 10.0);
    let b = Vec2::from(50.0, 460.0);
    let c = Vec2::from(630.0, 430.0);
    let d = Vec2::from(600.0, 100.0);
    let colors = [
        Vec4::from(1.0, 0.0, 0.0, 1.0),
        Vec4::from(0.0, 1.0, 0.0, 1.0),
        Vec4::from(0.0, 0.0, 1.0, 1.0),
    ];
    r.line(&[a, b], colors[0]);
    r.line(&[b, c], colors[1]);
    r.triangle(&[a, c, d], &colors);

    let u = Viewer::new((640, 480));
    u.swap(r.sz, EBuffer::Color(r.get_color()));
    u.disp();
}
