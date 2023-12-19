use std::ops::Range;
use std::any::Any;
use magx::*;
use rasterizer::{
    pipeline::IPrimitive,
    shader::{IShader, UniformMatrix},
};
use super::asset::*;
use super::ModelUniformVarsRef;


/// mesh类型
pub enum EMesh {
    /// 标准的全材质贴图+光照
    Standard,
    /// 轻量级材质，只有Diffuse贴图
    Lite,
    /// mesh调试
    Debug,
}

/// mesh模型数据
pub struct Mesh {
    name: String,
    e: EMesh,
    o: Obj,
    m: Mtl,
    uniforms: ModelUniformVarsRef,
}

impl Mesh {
    pub fn new(e: EMesh, name: &str, uniforms: ModelUniformVarsRef) -> Self {
        let o = Obj::new(name).unwrap();
        let m = Mtl::new(name);
        println!(r#"
{}:
    Obj: {}
    Mtl: {}"#, name, o, m);

        Self {
            name: name.to_string(),
            e, o, m,
            uniforms,
        }
    }
}

impl IPrimitive for Mesh {
    fn indices(&self) -> Range<usize> {
        return Range{ start: 0,  end: self.o.f.len() }
    }
}

impl IShader for Mesh {
    fn vertex(&self, pidx: usize) -> (Vec4, Vec4, Vec4) {
        let idx = &self.o.f[pidx];
        let uni = self.uniforms.borrow();
        (
            uni.mat.mvp.mul_vec(&self.o.v[idx.v.0].to_vec4(1.0)),
            uni.mat.mvp.mul_vec(&self.o.v[idx.v.1].to_vec4(1.0)),
            uni.mat.mvp.mul_vec(&self.o.v[idx.v.2].to_vec4(1.0)),
        )
    }

    fn fragment(&self, pidx: usize, bc: &Vec3) -> Vec4 {
        let idx = &self.o.f[pidx];
        let uni = self.uniforms.borrow();
        // 片段三个顶点的纹理坐标插值
        let u = interpolate(&bc,
            &self.o.vt[idx.vt.0].x,
            &self.o.vt[idx.vt.1].x,
            &self.o.vt[idx.vt.2].x);
        let v = interpolate(&bc,
            &self.o.vt[idx.vt.0].y,
            &self.o.vt[idx.vt.1].y,
            &self.o.vt[idx.vt.2].y);
        // 片段的缺省diffuse颜色
        let dd = Vec4::from(0.0, 0.0, 1.0, 1.0);
        // 片段的缺省specular颜色
        let ss = Vec4::fill(1.0);
        // 片段三个顶点的法向量插值
        let nn = interpolate(&bc,
            &self.o.vn[idx.vn.0],
            &self.o.vn[idx.vn.1],
            &self.o.vn[idx.vn.2]);
        // 通过模型变换，将片段的坐标变换世界坐标系中，用于计算光照
        let frag_pos = uni.mat.model.mul_vec(&interpolate(&bc,
            &self.o.v[idx.v.0],
            &self.o.v[idx.v.1],
            &self.o.v[idx.v.2],
            ).to_vec4(1.0)).to_vec3();

        match self.e {
            EMesh::Standard => {
                let d = self.m.diff.color(u, v).unwrap_or(dd);
                let s = self.m.spec.color(u, v).unwrap_or(ss);
                let n = uni.mat.mit.mul_vec(&self.m.norm.o_vec(u, v).unwrap_or(nn)).normalize();
                uni.comps.borrow().light.calc_blinn_phong(&d, &s, &n,
                    &(uni.comps.borrow().camera.eye - frag_pos)).w(1.0)
            },
            EMesh::Lite => {
                // 只用diffuse贴图，渲染出“光滑”的模型
                self.m.diff.color(u, v).unwrap_or(dd)
            },
            EMesh::Debug => {
                let n = uni.mat.mit.mul_vec(&nn).normalize();

                // Test: 可视化法向量
                return ((n + Vec3::fill(1.0)) / 2.0).to_vec4(1.0);

                // Test: 可视化切线空间的法向量
                //return self.m.norm.t_vec(u, v, &n).to_vec4(1.0);
            }
        }
    }
}


/// 锥台模型
pub struct MFrustum {
    name: String,
    faces: Vec<FaceIdx>,
    pos: Vec<Vec3>,
    color: Vec<Vec3>,
    /// 来自model的mvp
    mvp: Mat4,
}

impl MFrustum {
    pub fn new() -> Self {
        let v = vec![
            // -- position --     -- color --   -- texture --
            [-0.5, -0.5, -0.5,  1.0, 0.0, 0.0,   0.0, 0.0,   // back
              0.5,  0.5, -0.5,  0.0, 1.0, 0.0,   1.0, 1.0,
              0.5, -0.5, -0.5,  0.0, 0.0, 1.0,   1.0, 0.0,],
            [ 0.5,  0.5, -0.5,  1.0, 1.0, 1.0,   1.0, 1.0,
             -0.5, -0.5, -0.5,  0.0, 0.0, 0.0,   0.0, 0.0,
             -0.5,  0.5, -0.5,  0.0, 0.0, 0.0,   0.0, 1.0,],

            [-0.2, -0.2,  0.5,  1.0, 1.0, 1.0,   0.0, 0.0,   // front
              0.2, -0.2,  0.5,  1.0, 1.0, 1.0,   1.0, 0.0,
              0.2,  0.2,  0.5,  1.0, 1.0, 1.0,   1.0, 1.0,],
            [ 0.2,  0.2,  0.5,  1.0, 1.0, 1.0,   1.0, 1.0,
             -0.2,  0.2,  0.5,  1.0, 1.0, 1.0,   0.0, 1.0,
             -0.2, -0.2,  0.5,  1.0, 1.0, 1.0,   0.0, 0.0,],

            [-0.2,  0.2,  0.5,  1.0, 0.0, 0.0,   1.0, 0.0,   // left
             -0.5,  0.5, -0.5,  1.0, 0.0, 0.0,   1.0, 1.0,
             -0.5, -0.5, -0.5,  1.0, 0.0, 0.0,   0.0, 1.0,],
            [-0.5, -0.5, -0.5,  1.0, 0.0, 0.0,   0.0, 1.0,
             -0.2, -0.2,  0.5,  1.0, 0.0, 0.0,   0.0, 0.0,
             -0.2,  0.2,  0.5,  1.0, 0.0, 0.0,   1.0, 0.0,],

            [ 0.2,  0.2,  0.5,  0.0, 0.0, 1.0,   1.0, 0.0,   // right
              0.5, -0.5, -0.5,  0.0, 0.0, 1.0,   0.0, 1.0,
              0.5,  0.5, -0.5,  0.0, 0.0, 1.0,   1.0, 1.0,],
            [ 0.5, -0.5, -0.5,  0.0, 0.0, 1.0,   0.0, 1.0,
              0.2,  0.2,  0.5,  0.0, 0.0, 1.0,   1.0, 0.0,
              0.2, -0.2,  0.5,  0.0, 0.0, 1.0,   0.0, 0.0,],

            [-0.5, -0.5, -0.5,  0.0, 1.0, 0.0,   0.0, 1.0,   // down
              0.5, -0.5, -0.5,  0.0, 1.0, 0.0,   1.0, 1.0,
              0.2, -0.2,  0.5,  0.0, 1.0, 0.0,   1.0, 0.0,],
            [ 0.2, -0.2,  0.5,  0.0, 1.0, 0.0,   1.0, 0.0,
             -0.2, -0.2,  0.5,  0.0, 1.0, 0.0,   0.0, 0.0,
             -0.5, -0.5, -0.5,  0.0, 1.0, 0.0,   0.0, 1.0,],

            [-0.5,  0.5, -0.5,  1.0, 0.0, 1.0,   0.0, 1.0,   // up
              0.2,  0.2,  0.5,  1.0, 0.0, 1.0,   1.0, 0.0,
              0.5,  0.5, -0.5,  1.0, 0.0, 1.0,   1.0, 1.0,],
            [ 0.2,  0.2,  0.5,  1.0, 0.0, 1.0,   1.0, 0.0,
             -0.5,  0.5, -0.5,  1.0, 0.0, 1.0,   0.0, 1.0,
             -0.2,  0.2,  0.5,  1.0, 0.0, 1.0,   0.0, 0.0,],
        ];

        let mut faces = Vec::new();
        let mut pos = Vec::new();
        let mut color = Vec::new();

        for (index, item) in v.iter().enumerate() {
            pos.push(Vec3::from(item[0], item[1], item[2]));
            pos.push(Vec3::from(item[8], item[9], item[10]));
            pos.push(Vec3::from(item[16], item[17], item[18]));
            color.push(Vec3::from(item[3], item[4], item[5]));
            color.push(Vec3::from(item[11], item[12], item[13]));
            color.push(Vec3::from(item[19], item[20], item[21]));
            faces.push(FaceIdx::new(
                FaceAttrIdx(index * 3, index * 3 + 1, index * 3 + 2),
                FaceAttrIdx(index * 3, index * 3 + 1, index * 3 + 2),
                FaceAttrIdx(index * 3, index * 3 + 1, index * 3 + 2),
            ));
        }

        println!("Frustum: {{faces: {}, vertices: {}, colors: {}}}",
            faces.len(), pos.len(), color.len());

        Self {
            name: String::from("Frustum"),
            faces, pos, color,
            mvp: Mat4::eye(1.0),
        }
    }

    pub fn new_cube() -> Self {
        let v = vec![
            // -- triangle1    -- triangle2      -- triangle3
            [-0.5, -0.5, -0.5,  0.5,  0.5, -0.5,  0.5, -0.5, -0.5,], // back
            [ 0.5,  0.5, -0.5, -0.5, -0.5, -0.5, -0.5,  0.5, -0.5,],

            [-0.5, -0.5,  0.5,  0.5, -0.5,  0.5,  0.5,  0.5,  0.5,], // front
            [ 0.5,  0.5,  0.5, -0.5,  0.5,  0.5, -0.5, -0.5,  0.5,],

            [-0.5,  0.5,  0.5, -0.5,  0.5, -0.5, -0.5, -0.5, -0.5,], // left
            [-0.5, -0.5, -0.5, -0.5, -0.5,  0.5, -0.5,  0.5,  0.5,],

            [ 0.5,  0.5,  0.5,  0.5, -0.5, -0.5,  0.5,  0.5, -0.5,], // right
            [ 0.5, -0.5, -0.5,  0.5,  0.5,  0.5,  0.5, -0.5,  0.5,],

            [-0.5, -0.5, -0.5,  0.5, -0.5, -0.5,  0.5, -0.5,  0.5,], // down
            [ 0.5, -0.5,  0.5, -0.5, -0.5,  0.5, -0.5, -0.5, -0.5,],

            [-0.5,  0.5, -0.5,  0.5,  0.5,  0.5,  0.5,  0.5, -0.5,], // up
            [ 0.5,  0.5,  0.5, -0.5,  0.5, -0.5, -0.5,  0.5,  0.5,],
        ];

        let mut faces = Vec::new();
        let mut pos = Vec::new();
        let mut color = Vec::new();

        for (index, item) in v.iter().enumerate() {
            pos.push(Vec3::from(item[0], item[1], item[2]));
            pos.push(Vec3::from(item[3], item[4], item[5]));
            pos.push(Vec3::from(item[6], item[7], item[8]));
            color.push(Vec3::fill(1.0));
            color.push(Vec3::fill(1.0));
            color.push(Vec3::fill(1.0));
            faces.push(FaceIdx::new(
                FaceAttrIdx(index * 3, index * 3 + 1, index * 3 + 2),
                FaceAttrIdx(index * 3, index * 3 + 1, index * 3 + 2),
                FaceAttrIdx(index * 3, index * 3 + 1, index * 3 + 2),
            ));
        }

        println!("Cube: {{faces: {}, vertices: {}, colors: {}}}",
            faces.len(), pos.len(), color.len());

        Self {
            name: String::from("Cube"),
            faces, pos, color,
            mvp: Mat4::eye(1.0),
        }
    }
}

impl IPrimitive for MFrustum {
    fn indices(&self) -> Range<usize> {
        return Range{ start: 0,  end: self.faces.len() }
    }
}

impl IShader for MFrustum {
    fn set_uniforms(&mut self, u: Box<dyn Any>) {
        if let Ok(u) = u.downcast::<UniformMatrix>() {
            self.mvp = (*u).mvp;
        } else {
            println!("Failed to set uniforms to mesh '{}'", self.name);
        }
    }

    fn vertex(&self, pidx: usize) -> (Vec4, Vec4, Vec4) {
        let idx = &self.faces[pidx];
        (
            self.mvp.mul_vec(&self.pos[idx.v.0].to_vec4(1.0)),
            self.mvp.mul_vec(&self.pos[idx.v.1].to_vec4(1.0)),
            self.mvp.mul_vec(&self.pos[idx.v.2].to_vec4(1.0)),
        )
    }

    fn fragment(&self, pidx: usize, bc: &Vec3) -> Vec4 {
        let idx = &self.faces[pidx];
        // 基于顶点颜色插值
        interpolate(&bc,
            &self.color[idx.v.0],
            &self.color[idx.v.1],
            &self.color[idx.v.2]).to_vec4(1.0)
    }
}
