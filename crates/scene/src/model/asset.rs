use std::{error, fs, io, io::BufRead};
use std::fmt::{Display, Formatter};
use image::RgbaImage;
use magx::*;


/// 生成asset路径
macro_rules! gen_asset {
    (obj, $name: tt) => {
        std::format!("assets/objects/{}/{}.obj", $name, $name)
    };
    (diffuse, $name: tt) => {
        std::format!("assets/objects/{}/{}_diffuse.tga", $name, $name)
    };
    (specular, $name: tt) => {
        std::format!("assets/objects/{}/{}_specular.tga", $name, $name)
    };
    (normal, $name: tt) => {
        std::format!("assets/objects/{}/{}_normal.tga", $name, $name)
    };
}

/// 三角面的顶点(a, b, c)的属性索引
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FaceAttrIdx(pub usize, pub usize, pub usize);

/// 三角面的顶点索引数据
#[derive(Debug, Copy, Clone)]
pub struct FaceIdx {
    pub v: FaceAttrIdx, // vertex index
    pub vt: FaceAttrIdx, // texcoord index
    pub vn: FaceAttrIdx, // normal index
}

impl FaceIdx {
    pub fn new(v: FaceAttrIdx, vt: FaceAttrIdx, vn: FaceAttrIdx) -> Self {
        Self { v, vt, vn }
    }
}

/// obj模型
pub struct Obj {
    /// 三角面（即片段）
    pub f: Vec<FaceIdx>,
    /// 顶点坐标
    pub v: Vec<Vec3>,
    /// 纹理坐标
    pub vt: Vec<Vec3>,
    /// 法向量坐标
    pub vn: Vec<Vec3>,
}

impl Obj {
    /// 加载obj模型文件
    ///
    /// name: obj模型名称
    pub fn new(name: &str) -> Result<Self, Box<dyn error::Error>> {
        let obj = fs::File::open(&gen_asset!(obj, name))?;

        let mut faces = Vec::new();
        let mut pos = Vec::new();
        let mut tex = Vec::new();
        let mut nm = Vec::new();

        for item in io::BufReader::new(obj).lines() {
            if let Ok(line) = item {
                if line.starts_with("v ") {
                    // 解析vetices
                    let v: Vec<Tyf> = line.strip_prefix("v ").unwrap()
                        .split_whitespace()
                        .map(|x| x.parse::<Tyf>().unwrap())
                        .collect();
                    pos.push(Vec3::from(v[0], v[1], v[2])); // 变换时需要使用齐次坐标，w=1.0

                } else if line.starts_with("vt ") {
                    // 解析texture coordinates
                    let v: Vec<Tyf> = line.strip_prefix("vt ").unwrap()
                        .split_whitespace()
                        .map(|x| x.parse::<Tyf>().unwrap())
                        .collect();
                    tex.push(Vec3::from(v[0], v[1],
                            if v.len() >= 3 { v[2] } else { 0.0 }));

                } else if line.starts_with("vn ") {
                    // 解析normals
                    let v: Vec<Tyf> = line.strip_prefix("vn ").unwrap()
                        .split_whitespace()
                        .map(|x| x.parse::<Tyf>().unwrap())
                        .collect();
                    nm.push(Vec3::from(v[0], v[1], v[2]));

                } else if line.starts_with("f ") {
                    // 解析faces
                    let v: Vec<(usize, usize, usize)> = line.strip_prefix("f ").unwrap()
                        .split_whitespace()
                        .map(|x| {
                            let mut index = x.split('/');
                            let va = index.next().unwrap().parse::<usize>().unwrap() - 1;
                            let vb = index.next().unwrap().parse::<usize>().unwrap() - 1;
                            let vc = index.next().unwrap().parse::<usize>().unwrap() - 1;
                            (va, vb, vc)
                        }).collect();
                    faces.push(FaceIdx::new(
                        FaceAttrIdx(v[0].0, v[1].0, v[2].0),
                        FaceAttrIdx(v[0].1, v[1].1, v[2].1),
                        FaceAttrIdx(v[0].2, v[1].2, v[2].2),
                    ));
                }
            }
        }

        Ok(Self {
            f: faces,
            v: pos,
            vt: tex,
            vn: nm
        })
    }
}

impl Display for Obj {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "{{faces: {}, vertices: {}, texcoords: {}, normals: {}}}",
            self.f.len(), self.v.len(), self.vt.len(), self.vn.len())
    }
}

/// Texture贴图
pub struct Tex(pub Option<RgbaImage>);

impl Tex {
    /// 加载贴图
    ///
    /// 贴图需要翻转y轴，以左下角为坐标原点，且转成RGBA格式。
    pub fn new(filename: &str) -> Self {
        if let Ok(img) = image::open(&filename) {
            Self(Some( img.flipv().to_rgba8() ))
        } else {
            Self(None)
        }
    }

    pub fn width(&self) -> u32 {
        if let Some(ref img) = self.0 {
            img.width()
        } else {
            0
        }
    }

    pub fn height(&self) -> u32 {
        if let Some(ref img) = self.0 {
            img.height()
        } else {
            0
        }
    }

    /// 从贴图读取浮点颜色
    pub fn color(&self, u: Tyf, v: Tyf) -> Option<Vec4> {
        if let Some(ref img) = self.0 {
            let x = ((u * img.width() as Tyf) as u32).clamp(0, img.width() - 1);
            let y = ((v * img.height() as Tyf) as u32).clamp(0, img.height() - 1);
            let c = img.get_pixel(x, y);
            Some(Vec4::from(
                (c[0] as Tyf) / 255.0,
                (c[1] as Tyf) / 255.0,
                (c[2] as Tyf) / 255.0,
                (c[3] as Tyf) / 255.0))
        } else {
            None
        }
    }

    /// 计算模型空间的法线纹理（object-space normal map）
    pub fn o_vec(&self, u: Tyf, v: Tyf) -> Option<Vec3> {
        if let Some(ref img) = self.0 {
            let x = ((u * img.width() as Tyf) as u32).clamp(0, img.width() - 1);
            let y = ((v * img.height() as Tyf) as u32).clamp(0, img.height() - 1);
            let c = img.get_pixel(x, y);
            // RGB[0, 255] 转 法向量[-1.0, 1.0]
            Some(Vec3::from(
                (c[0] as Tyf / 255.0) * 2.0  - 1.0,
                (c[1] as Tyf / 255.0) * 2.0  - 1.0,
                (c[2] as Tyf / 255.0) * 2.0  - 1.0))
        } else {
            None
        }
    }

    /// 计算切线空间的法线纹理（tangent-space normal map）
    ///
    /// - n: 顶点在世界坐标空间的法线向量
    pub fn t_vec(&self, u:Tyf, v:Tyf, n: &Vec3) -> Option<Vec3> {
        if let Some(ref img) = self.0 {
            // 计算世界坐标系下的切线空间，即T(tangent), B(bi-tangent), N(normal)矩阵：
            //             | Tx, Bx, Nx |
            // [T, B, N] = | Ty, By, Ny |
            //             | Tz, Bz, Nz |
            let nxz = (n.x * n.x + n.z * n.z).sqrt();
            let t = Vec3::from(n.x * n.y / nxz, nxz, n.z * n.y / nxz);
            let b = n.cross(&t);
            let tbn = Mat3::from_col(t, b, *n);

            // 计算切线空间中，模型本地(local)的法向量
            let x = ((u * img.width() as Tyf) as u32).clamp(0, img.width() - 1);
            let y = ((v * img.height() as Tyf) as u32).clamp(0, img.height() - 1);
            let x1 = (x + 1).clamp(0, img.width() - 1);
            let y1 = (y + 1).clamp(0, img.height() - 1);
            let huv0 = (Vec4::from_array(&img.get_pixel(x, y).0).to_vec3() / 255.0).norm();
            let hu1 = (Vec4::from_array(&img.get_pixel(x1, y).0).to_vec3() / 255.0).norm();
            let hv1 = (Vec4::from_array(&img.get_pixel(x, y1).0).to_vec3() / 255.0).norm();
            // 乘一个系数，对颜色值缩放，得到合适的切线空间法向量值
            let du = (hu1 - huv0) * 5.0; // tangent = dp/du
            let dv = (hv1 - huv0) * 5.0; // bi-tangent = dp/dv
            let ln = Vec3::from(-du, -dv, 1.0);

            // 将切线空间的法向量，从模型本地坐标系变换到世界坐标系
            Some(tbn.mul_vec(&ln).normalize())
        } else {
            None
        }
    }
}

impl Display for Tex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(ref img) = self.0 {
            write!(f, "{}x{}", img.width(), img.height())
        } else {
            write!(f, "None")
        }
    }
}

/// 材质贴图
pub struct Mtl {
    /// 材质（在漫反射光照下物体的颜色）贴图（以左下角为坐标原点）
    pub diff: Tex,
    /// 镜面贴图（以左下角为坐标原点）
    pub spec: Tex,
    /// 法向量贴图（可以是object-space或tangent-space）
    pub norm: Tex,
}

impl Mtl {
    /// 加载材质贴图
    ///
    /// name: 对应的obj模型名称
    pub fn new(name: &str) -> Self {
        let diff = Tex::new(&gen_asset!(diffuse, name));
        let spec = Tex::new(&gen_asset!(specular, name));
        let norm = Tex::new(&gen_asset!(normal, name));
        Self {
            diff,
            spec,
            norm,
        }
    }
}

impl Display for Mtl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "{{diffuse: {}, specular: {}, normal: {}}}",
            self.diff, self.spec, self.norm)
    }
}
