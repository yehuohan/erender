use crate::camera::Camera;
use crate::light::Light;
use crate::model::{Model, ModelLight};
use magx::*;
use rasterizer::{pipeline::IPipeline, rasterizer::Rasterizer};
use std::cell::RefCell;
use std::rc::Rc;

/// 背景颜色
const COLOR_BG: Vec4 = Vec4::from(0.4, 0.2, 0.3, 1.0);

/// 场景组件
pub struct SceneComponents {
    /// 摄像机
    pub camera: Camera,
    /// 光照（定向光）
    pub light: Light,
}

pub type SceneComponentsRef = Rc<RefCell<SceneComponents>>;

impl SceneComponents {
    pub fn new(camera: Camera, light: Light) -> SceneComponentsRef {
        Rc::new(RefCell::new(SceneComponents { camera, light }))
    }
}

/// 场景模型
pub struct Scene {
    /// 基本场景模型
    pub model: Model,
    /// 光源模型
    pub model_light: ModelLight,
    pub comps: SceneComponentsRef,
    /// 屏幕大小(w, h)
    pub sz: (u32, u32),
}

impl Scene {
    /// 构建场景
    ///
    /// - sz: 场景屏幕大小
    pub fn new(sz: (u32, u32)) -> Self {
        let mut light = Light::new();
        light.pos = Vec3::from(1.0, 1.0, 2.5); // 大致示意光源位置
        light.dir = -light.pos.normalize();

        let comps = SceneComponents::new(
            Camera::new(
                Vec3::from(0.0, 0.0, 3.5), // eye
                Vec3::from(0.0, 0.0, 0.0), // center
                Vec3::from(0.0, 1.0, 0.0), // up
                sz,
            ),
            light,
        );

        Self {
            model: Model::new(Rc::clone(&comps)),
            model_light: ModelLight::new(),
            comps,
            sz,
        }
    }

    /// 更新场景数据
    pub fn update(&mut self, r: &mut Rasterizer) {
        self.model.update();
        self.model_light
            .update(&self.comps.borrow().camera, &self.comps.borrow().light);

        r.clear_color(&COLOR_BG);
        r.clear_depth();
        for p in &self.model.meshes {
            r.draw(&p);
        }
        r.draw(&self.model_light.cube);
    }
}
