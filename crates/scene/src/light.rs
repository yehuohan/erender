#![allow(non_upper_case_globals)]

use magx::*;


// 不同的光照分量，光源的影响也不一样（这里使用固定的光照分量系数）
const ambient: Vec4  = Vec4::from(0.05, 0.05, 0.05, 1.0);
const diffuse: Vec4  = Vec4::from(0.9, 0.9, 0.9, 1.0);
const specular: Vec4 = Vec4::from(1.0, 1.0, 1.0, 1.0);
/// 影响镜面高光的散身/半径
const shininess: i32 = 32;

/// 定向光或点光源
#[derive(Debug, Copy, Clone)]
pub struct Light {
    /// 世界坐标系中的光源方向
    pub dir: Vec3,
    /// 世界坐标系中的光原位置（只在点光源时使用，同时作为光源的示意坐标）
    pub pos: Vec3,
}

impl Light {
    pub fn new() -> Self {
        Self {
            dir: Vec3::fill(0.0),
            pos: Vec3::fill(0.0),
        }
    }

    /// 计算Blinn-Phong光照模型（基于定向光）
    ///
    /// - diff: 在漫反射光照下物体的颜色
    /// - spec: 镜面光照下物体的颜色
    /// - norm: 片段法线向量（需要归一化）
    /// - vdir: 观察方向（从片段看向摄像机eye的方向）
    pub fn calc_blinn_phong(&self, diff: &Vec4, spec: &Vec4, norm: &Vec3, vdir: &Vec3) -> Vec4 {
        // 从片段看向光源的方向
        let ldir = -self.dir;

        // 环境光照分量（光源照不到的表面，环境光颜色几乎等于漫反射颜色）
        let a = ambient * (*diff);

        // 漫反射光照分量
        let d = diffuse * (*diff) * norm.dot(&ldir).max(0.0);

        // 镜面反射分量
        let hdir = (*vdir + ldir).normalize(); // 半程向量
        let s = specular * (*spec) * norm.dot(&hdir).max(0.0).powi(shininess);

        a + d + s
    }
}
