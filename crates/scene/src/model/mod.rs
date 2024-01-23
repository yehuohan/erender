#![allow(dead_code)]

pub mod asset;
pub mod mesh;

use self::mesh::{EMesh, MFrustum, Mesh};
use crate::camera::Camera;
use crate::light::Light;
use crate::scene::SceneComponentsRef;
use magx::*;
use rasterizer::{pipeline::IPrimitive, shader::UniformMatrix};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// 场景模型需要的uniform变量
pub struct ModelUniformVars {
    /// 矩阵变换变量
    pub mat: UniformMatrix,
    /// 来自scene的场景组件
    pub comps: SceneComponentsRef,
}

pub type ModelUniformVarsRef = Rc<RefCell<ModelUniformVars>>;

impl ModelUniformVars {
    pub fn new(comps: SceneComponentsRef) -> ModelUniformVarsRef {
        Rc::new(RefCell::new(ModelUniformVars {
            mat: UniformMatrix::new(),
            comps,
        }))
    }
}

/// 基本场景模型
pub struct Model {
    /// model中的所有mesh
    ///
    /// 使用`dyn trait`可以让不同的IPrimitive放到一个数组中，方便遍历。
    pub meshes: HashMap<&'static str, Box<dyn IPrimitive>>,
    /// model需要使用uniform变量
    pub uniforms: ModelUniformVarsRef,
}

macro_rules! load_mesh {
    (standard, $name:tt, $uniforms:ident) => {
        Mesh::new(EMesh::Standard, $name, Rc::clone(&$uniforms))
    };
    (lite, $name:tt, $uniforms:ident) => {
        Mesh::new(EMesh::Lite, $name, Rc::clone(&$uniforms))
    };
    (debug, $name:tt, $uniforms:ident) => {
        Mesh::new(EMesh::Debug, $name, Rc::clone(&$uniforms))
    };
    (frustum) => {
        MFrustum::new()
    };
    (cube) => {
        MFrustum::new_cube()
    };
}

impl Model {
    pub fn new(comps: SceneComponentsRef) -> Self {
        let mut meshes: HashMap<&str, Box<dyn IPrimitive>> = HashMap::new();
        let uniforms = ModelUniformVars::new(comps);

        meshes.insert("african_head", Box::new(load_mesh!(standard, "african_head", uniforms)));
        meshes.insert(
            "african_head_eye",
            Box::new(load_mesh!(standard, "african_head_eye_inner", uniforms)),
        );
        meshes.insert("diablo3", Box::new(load_mesh!(standard, "diablo3_pose", uniforms)));
        meshes.insert("floor", Box::new(load_mesh!(lite, "floor", uniforms)));
        meshes.insert("sphere", Box::new(load_mesh!(debug, "sphere", uniforms)));

        meshes.insert("spot", Box::new(load_mesh!(standard, "spot", uniforms)));
        meshes.insert("spot_lite", Box::new(load_mesh!(lite, "spot", uniforms)));
        meshes.insert("spot_debug", Box::new(load_mesh!(debug, "spot", uniforms)));

        meshes.insert("cube", Box::new(load_mesh!(cube)));
        meshes.insert("frustum", Box::new(load_mesh!(frustum)));

        Self { meshes, uniforms }
    }

    /// 更新model的变换矩阵和光照数据
    pub fn update(&mut self) {
        let mat_model = Mat4::eye(1.0);
        //let mat_model = translate(&mat_model, &Vec3::from(-1.0, -2.0, -5.0));
        //let mat_model = scale(&mat_model, &Vec3::from(0.5, 0.5, 0.5));
        let mat_model = rotate(&mat_model, &Vec3::from(0.0, 1.0, 0.0), Angle::Ang(-5.0));
        //let mat_model = rotate(&mat_model, &Vec3::from(0.0, 1.0, 0.0), Angle::Ang(140.0));

        let mut u = self.uniforms.borrow_mut();
        let view = u.comps.borrow().camera.view();
        let proj = u.comps.borrow().camera.proj();
        u.mat.model = mat_model;
        u.mat.view = view;
        u.mat.proj = proj;
        u.mat.calc_mit();
        u.mat.calc_mvp();

        for (_, mesh) in &mut self.meshes {
            mesh.set_uniforms(Box::new(u.mat))
        }
    }
}

/// 光源模型
pub struct ModelLight {
    /// 用cube示意光源位置
    pub cube: Box<dyn IPrimitive>,
}

impl ModelLight {
    pub fn new() -> Self {
        Self {
            cube: Box::new(load_mesh!(cube)),
        }
    }

    pub fn update(&mut self, camera: &Camera, light: &Light) {
        let mat_model = Mat4::eye(1.0);
        let mat_model = translate(&mat_model, &light.pos);
        let mat_model = scale(&mat_model, &Vec3::from(0.1, 0.1, 0.1));

        let mut u = UniformMatrix::new();
        u.model = mat_model;
        u.view = camera.view();
        u.proj = camera.proj();
        u.calc_mvp();

        self.cube.set_uniforms(Box::new(u));
    }
}

/// 视锥体剃除
///
/// 剃除不在视锥体内的mesh
struct ViewCulling {
    up: Vec4,
    down: Vec4,
    left: Vec4,
    right: Vec4,
    near: Vec4,
    far: Vec4,
}

impl ViewCulling {
    fn new(&mvp: &Mat4) -> Self {
        Self {
            up: mvp.col(3) - mvp.col(1),
            down: mvp.col(3) + mvp.col(1),
            left: mvp.col(3) + mvp.col(0),
            right: mvp.col(3) - mvp.col(0),
            near: mvp.col(3) + mvp.col(2),
            far: mvp.col(3) - mvp.col(2),
        }
    }

    /// 判断一个点是否为视锥体内
    fn contains(&self, point: &Vec3) -> bool {
        let p = point.to_vec4(1.0);

        self.up.dot(&p) >= 0.0
            && self.down.dot(&p) >= 0.0
            && self.left.dot(&p) >= 0.0
            && self.right.dot(&p) >= 0.0
            && self.near.dot(&p) >= 0.0
            && self.far.dot(&p) >= 0.0
    }
}
