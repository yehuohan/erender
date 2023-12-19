//! 几何计算(Geometry Mathematics)
//!
//! ## 3D坐标变换
//!
//! - 变换算法均直接套用公式，未作优化，方便学习对比；
//!
//! 变换矩阵的乘法：
//!
//! ```text, no_run
//! |a 0 0|
//! |0 b 0| * M
//! |0 0 c|
//! ```
//! - 从左矩阵看：对M每个列向量，第1行只保留a，第2行只保留b ...
//!
//! ```text, no_run
//!     |a 0 0|
//! M * |0 b 0|
//!     |0 0 c|
//! ```
//! - 从右矩阵看：对M每个行向量，第1列只保留a，第2列只保留b ...

use std::ops::{ Add, Mul };
use super::*;


/// 角度类型
pub enum Angle {
    /// 角度
    Ang(Tyf),
    /// 弧度
    Rad(Tyf),
}

impl Angle {
    #[inline]
    pub fn to_ang(&self) -> Tyf {
        match *self {
            Angle::Ang(a) => a,
            Angle::Rad(r) => r * 180.0 / std::f64::consts::PI as Tyf,
        }
    }

    #[inline]
    pub fn to_rad(&self) -> Tyf {
        match *self {
            Angle::Ang(a) => a * std::f64::consts::PI as Tyf / 180.0,
            Angle::Rad(r) => r,
        }
    }
}

/// 缩放变换
///
/// | s1  0  0  0 |   | x |   | s1*x |
/// |  0 s2  0  0 | * | y | = | s2*y |
/// |  0  0 s3  0 |   | z |   | s3*z |
/// |  0  0  0  1 |   | 1 |   |  1   |
pub fn scale(m: &Mat4, s: &Vec3) -> Mat4 {
    let ms = Mat4::diag(&Vec4::from(s.x, s.y, s.z, 1.0));
    m.mul_mat(&ms)
}

/// 位置变换
///
/// | 1  0  0 tx |   | x |   | x+tx |
/// | 0  1  0 ty | * | y | = | y+ty |
/// | 0  0  1 tz |   | z |   | z+tz |
/// | 0  0  0  1 |   | 1 |   |  1   |
pub fn translate(m: &Mat4, t: &Vec3) -> Mat4 {
    let mt = Mat4::from(
        Vec4::from(1.0, 0.0, 0.0, t.x),
        Vec4::from(0.0, 1.0, 0.0, t.y),
        Vec4::from(0.0, 0.0, 1.0, t.z),
        Vec4::from(0.0, 0.0, 0.0, 1.0)
    );
    m.mul_mat(&mt)
}

/// 旋转变换
///
/// -a: 旋转轴，用向量表示
/// -t: 旋转角，单位：角度
pub fn rotate(m: &Mat4, a: &Vec3, t: Angle) -> Mat4 {
    let t = t.to_rad();
    let sint = t.sin();
    let cost = t.cos();
    let a = a.normalize();
    let mr = Mat4::from(
        Vec4::from(cost + a.x * a.x * (1.0 - cost)       , a.x * a.y * (1.0 - cost) - a.z * sint , a.x * a.z * (1.0 - cost) + a.y * sint, 0.0),
        Vec4::from(a.y * a.x * (1.0 - cost) + a.z * sint , cost + a.y * a.y * (1.0 - cost)       , a.y * a.z * (1.0 - cost) - a.x * sint, 0.0),
        Vec4::from(a.z * a.x * (1.0 - cost) - a.y * sint , a.z * a.y * (1.0 - cost) + a.x * sint , cost + a.z * a.z * (1.0 - cost)      , 0.0),
        Vec4::from(0.0, 0.0, 0.0, 1.0)
    );
    m.mul_mat(&mr)
}

/// 视图变换矩阵，用于将坐标变换到摄像机坐标系中
///
/// - eye: 摄像机位置（即眼睛所在位置）
/// - center: 看向位置
/// - up: 正上方向（世界坐标系的上方向）
///
/// ```text, no_run
/// | Rx Ry Rz 0 |   | 1  0  0 -Px |
/// | Ux Uy Uz 0 | * | 0  1  0 -Py |
/// | Dx Dy Dz 0 |   | 0  0  1 -Pz |
/// | 0  0  0  1 |   | 0  0  0  1  |
/// ```
///
/// R、U、D分别为视图坐标系（摄像头坐标系）的正交右、上、前方向向量；
/// P为摄像头位置向量；
pub fn look_at(eye: &Vec3, center: &Vec3, up: &Vec3) -> Mat4 {
    // 左手坐标系
    #[cfg(feature = "coord_left_hand")]
    #[allow(unused_variables)]
    let v = {
        let d = (*center - *eye).normalize();
        let r = up.cross(&d).normalize(); // d和up之间不一定是90度
        let u = d.cross(&r); // r, d, u一定是正交直角坐标系

        let v = Mat4::from(
            r.to_vec4(0.0), u.to_vec4(0.0), d.to_vec4(0.0),
            Vec4::from(0.0, 0.0, 0.0, 1.0)
        );
        v
    };

    // 右手坐标系
    #[cfg(feature = "coord_right_hand")]
    #[allow(unused_variables)]
    let v = {
        let d = (*center - *eye).normalize();
        let r = d.cross(&up).normalize();
        let u = r.cross(&d);

        let v = Mat4::from(
            r.to_vec4(0.0), u.to_vec4(0.0), -d.to_vec4(0.0),
            Vec4::from(0.0, 0.0, 0.0, 1.0)
        );
        v
    };

    let p = Mat4::from(
        Vec4::from(1.0, 0.0, 0.0, -eye.x),
        Vec4::from(0.0, 1.0, 0.0, -eye.y),
        Vec4::from(0.0, 0.0, 1.0, -eye.z),
        Vec4::from(0.0, 0.0, 0.0, 1.0)
    );

    v.mul_mat(&p)
}

/// 正交投影
///
/// 公式符合opengl标准定义。
///
/// - near/far: 视点到近/远平面的距离
pub fn ortho(left: Tyf, right: Tyf, bottom: Tyf, top: Tyf, near: Tyf, far: Tyf) -> Mat4 {
    Mat4::from(
        Vec4::new().x(2.0/(right - left)).w(-(right + left)/(right - left)),
        Vec4::new().y(2.0/(top - bottom)).w(-(top + bottom)/(top - bottom)),
        Vec4::new().z(2.0/(near - far  )).w( (near + far)  /(near - far  )),
        Vec4::new().w(1.0),
    )
}

/// 透视投影
///
/// 公式符合opengl标准定义。
///
/// - fovy: Y轴方向的视角
/// - aspect: 宽/高的比值
/// - near/far: 视点到近/远平面的距离
pub fn persp(fovy: Angle, aspect: Tyf, near: Tyf, far: Tyf) -> Mat4 {
    let half = (fovy.to_rad() / 2.0).tan();
    Mat4::from(
        Vec4::new().x(1.0 / (half * aspect)),
        Vec4::new().y(1.0 / half),
        Vec4::new().z((near + far)/(near - far)).w(2.0*near*far/(near - far)),
        Vec4::new().z(-1.0),
    )
}

/// 视口变换
///
/// 用于将NC坐标映射到屏幕坐标系上，即将[-1,1]x3的立方体，映射到[x,x+w]-[y,y+h]-[0,1]上。
/// 这里将z从[-1, 1]变换至[0, 1]。
pub fn viewport(x: Tyf, y:Tyf, width: Tyf, height: Tyf) -> Mat4 {
    Mat4::from(
        Vec4::new().x(width/2.0).w(x + width/2.0),
        Vec4::new().y(height/2.0).w(y + height/2.0),
        Vec4::new().z(1.0/2.0).w(1.0/2.0),
        Vec4::from(0.0, 0.0, 0.0, 1.0),
    )
}

/// lerp函数
#[inline]
pub fn lerp<T>(a: &T, b: &T, t: Tyf) -> T
where
    T: Copy + Add<Tyf, Output=T> + Mul<Tyf, Output=T> + Add<T, Output=T>
{
    return (*a) * (1.0 - t) + (*b) * t;
}

/// 计算1个点在三角形重心坐标系中的坐标
///
/// P点在三角形内则有：
///
/// ```text, no_run
/// u * AC + v * AB + PA = 0
/// ```
///
/// 设AC、AB、PA的x坐标组成三维向量为vx，设AC、AB、PA的y坐标组成三维向量vy，
/// 则三维向量(u, v, 1)与vx、vy均垂直（即与vx、vy的叉乘向量共线）；
/// 则P在重心坐标系的坐标为`(1-u-v, v, u)`。
pub fn barycentric(abc: &[Vec2; 3], p: Vec2) -> Option<Vec3> {
    let ab = abc[2] - abc[0];
    let ac = abc[1] - abc[0];
    let pa = abc[0] - p;

    let mut r = Vec3::from(ab.x, ac.x, pa.x).cross(&Vec3::from(ab.y, ac.y, pa.y));
    if r.z.is_zero() {
        return None;
    }
    r /= r.z; // 除以z，得到(u, v, 1)
    let r = Vec3::from(1.0 - (r.x + r.y), r.y, r.x);
    if r.x < 0.0 || r.y < 0.0 || r.z < 0.0 {
        None
    } else {
        Some(r)
    }
}

/// 基于重心坐标插值
#[inline]
pub fn interpolate<T>(bc: &Vec3, a: &T, b: &T, c: &T) -> T
where
    T: Copy + Add<Tyf, Output=T> + Mul<Tyf, Output=T> + Add<T, Output=T>
{
    (*a) * bc.x + (*b) * bc.y + (*c) * bc.z
}

/// 获取点集的矩形区域
///
/// 这里的点均用浮点计算，而图形坐标是正整型；
/// 为了保证转成图形坐标的点仍在矩形区域内，需要将矩形向外扩大1个单位；
/// 若不外扩，绘制共边三角形时，可能出现间隙。
pub fn bound_box(points: &[Vec2]) -> (Vec2, Vec2) {
    let mut lb = Vec2::from(Tyf::MAX, Tyf::MAX); // left bottom
    let mut rt = Vec2::fill(0.0); // right top

    for p in points {
        lb.x = lb.x.min(p.x);
        lb.y = lb.y.min(p.y);
        rt.x = rt.x.max(p.x);
        rt.y = rt.y.max(p.y);
    }

    // 因为浮点类型，将矩形向外扩大1个单位，保证包含有所的点
    (lb - 1.0, rt + 1.0)
}

/// 光线反射方向计算
///
/// ```text, no_run
///  O   N   I
///   ^  ^  /
///    \ | /
///     \|v
///  ----*----
///
/// (-I) + O = 2 * (-I).dot(N) * N.normalize()
/// ```
///
/// 只计算方向，N是否normalize()由调用者决定。
///
/// - I: 光线入射方向，由外指向反射点
/// - N: 反射点法线方向，由反射点指向外
#[allow(non_snake_case)]
pub fn reflect(I: &Vec3, N: &Vec3) -> Vec3 {
    return (*I) - (*N) * I.dot(N) * 2.0;
}

/// 光线折射方向计算
///
/// ```text, no_run
///      N   I
///      ^  /
///      | /
///      |v
///  ----*----
///     /|
///    / |
///   v  |
///  O
///
/// eta = ior_i / ior_o
///     = 1.0 / ior
///     = sin_o / sin_i
///     = sqrt( (1 - cos_o * cos_o) / (1 - cos_i * cos_i) )
/// ```
///
/// 入射角为i，折射角为o；
/// 介质的折射率ior = c / v，即光速除以光在介质的传播速度；
/// 相对折射率ior = ior_o / ior_i，这里o指物体内，i指物体外面；
/// 折射定率 ior_i * sin_i = ior_o * sin_o。
///
/// - I: 光线入射方向，由外指向折射点
/// - N: 折射点法线方向，由折射点指向外
/// - ior: 相对的折射率（index of refraction）
#[allow(non_snake_case)]
pub fn refract(I: &Vec3, N: &Vec3, ior: Tyf) -> Vec3 {
    let mut cosi = I.dot(N).clamp(-1.0, 1.0);
    let n;
    let eta;
    if cosi < 0.0 {
        // 光线由外面折射进入物体
        cosi = -cosi;
        n = *N;
        eta = 1.0 / ior;
    } else {
        // 光线由物体折射进入外面
        n = -(*N);
        eta = ior;
    }
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
    if k < 0.0 {
        // 发生全反射
        Vec3::fill(0.0)
    } else {
        (*I) * eta + n * (eta * cosi - k.sqrt())
    }
}

/// 求解一元二次方程
///
/// 返回根(x0, x1)，且保证x0 <= x1。
pub fn solve_quadratic(a: Tyf, b: Tyf, c: Tyf) -> Option<(Tyf, Tyf)> {
    let discr = b * b - 4.0 * a * c;
    if discr < 0.0 {
        return None;
    } else if discr.is_zero() {
        let x = -0.5 * b / a;
        return Some((x, x));
    } else {
        let q = if b > 0.0 {
            -0.5 * (b + discr.sqrt())
        } else {
            -0.5 * (b - discr.sqrt())
        };
        let x0 = q / a;
        let x1 = c / q;
        if x0 < x1 {
            return Some((x0, x1));
        } else {
            return Some((x1, x0));
        }
    }
}

/// 计算射线与球的交点
///
/// ```text, no_run
/// 射线: p = ori + t * dir
/// 球  : (p - center)^2 = radius ^ 2
/// ```
///
/// 求解一元次方程，可解得射线的参数t，t需要>=0。
///
/// - center: 球心坐标
/// - radius: 球半径
/// - ori: 射线原点
/// - dir: 射线方向
pub fn intersect_sphere(center: &Vec3, radius: Tyf, ori: &Vec3, dir: &Vec3) -> Option<Tyf> {
    let l = *ori - *center;
    let a = dir.dot(dir);
    let b = dir.dot(&l) * 2.0;
    let c = l.dot(&l) - radius * radius;

    if let Some((x0, x1)) = solve_quadratic(a, b, c) {
        if x0 >= 0.0 {
            return Some(x0);
        } else if x1 >= 0.0 {
            return Some(x1);
        } else {
            return None;
        }
    } else {
        return None;
    }
}

/// 计算射线与三角面的交点
///
/// ```text, no_run
/// 射线: p = ori + t * dir
/// 面  : (p - p0) * N = 0
/// ```
///
/// 这里使用Moller Trumbore Algorithm求解射线参数t，同时求解交点在三角面内的重心坐标。
///
/// - abc: 三角面顶点
/// - ori: 射线原点
/// - dir: 射线方向
pub fn intersect_triangle(abc: &[Vec3; 3], ori: &Vec3, dir: &Vec3) -> Option<(Tyf, Vec3)> {
    let [a, b, c] = abc;
    let e1 = *b - *a;
    let e2 = *c - *a;
    let s0 = *ori - *a;
    let s1 = dir.cross(&e2);
    let s2 = s0.cross(&e1);
    let s = Vec3::from(s2.dot(&e2), s1.dot(&s0), s2.dot(&dir)) / s1.dot(&e1);
	let t = s.x;
	let u = s.y;
	let v = s.z;

    if t >= 0.0 && u >= 0.0 && v >= 0.0 && (u + v) <= 1.0 {
        return Some((t, Vec3::from(1.0 - u - v, u, v)));
    } else {
        return None;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::approx_eq;

    #[test]
    fn transformation() {
        let m = Mat4::eye(1.0);
        let m = translate(&m, &Vec3::from(1.0, 1.0, 1.0));
        let m = scale(&m, &Vec3::from(5.0, 5.0, 5.0));
        let m = rotate(&m, &Vec3::from(0.0, 1.0, 0.0), Angle::Ang(90.0));
        println!("trans matrix:\n{}", m);
        assert!(approx_eq!(
            Mat4, m,
            Mat4::from(
                Vec4::from( 0.0, 0.0, 5.0, 1.0),
                Vec4::from( 0.0, 5.0, 0.0, 1.0),
                Vec4::from(-5.0, 0.0, 0.0, 1.0),
                Vec4::from( 0.0, 0.0, 0.0, 1.0)),
            epsilon = 0.000001));

        let v = Vec4::from(1.0, 2.0, 3.0, 1.0);
        println!("trans point: \n{}", m.mul_vec(&v));
        assert!(approx_eq!(
            Vec4,
            m.mul_vec(&v),
            Vec4::from(16.0, 10.99999999, -4.0, 1.0),
            epsilon = 0.000001));
    }

    #[test]
    fn calculation() {
        let c = look_at(
            &Vec3::from(1.0, 2.0, 3.0),
            &Vec3::from(-1.0, -1.0, -1.0),
            &Vec3::from(0.0, 1.0, 0.0)
        );
        println!("look at:\n{}", c);
        assert!(approx_eq!(
            Mat4,
            c,
            Mat4::from(
                Vec4::from( 0.894427, 0.000000, -0.447214, 0.447214) ,
                Vec4::from(-0.249136, 0.830455, -0.498273, 0.083045) ,
                Vec4::from( 0.371391, 0.557086, 0.742781 , -3.713907),
                Vec4::from( 0.000000, 0.000000, 0.000000 , 1.000000)),
            epsilon = 0.000001));

        // projection
        let proj = ortho(-160.0, 160.0, -160.0, 160.0, -1.0, 10.0);
        println!("orthogrphic: \n{}", proj);
        assert!(approx_eq!(
            Mat4,
            proj,
            Mat4::from(
                Vec4::from(0.006250, 0.000000,  0.000000,  0.000000),
                Vec4::from(0.000000, 0.006250,  0.000000,  0.000000),
                Vec4::from(0.000000, 0.000000, -0.181818, -0.818182),
                Vec4::from(0.000000, 0.000000,  0.000000,  1.000000)),
            epsilon = 0.000001));
        let proj = persp(Angle::Ang(45.0), 2.0/1.0, 0.1, 10.0);
        println!("perspective: \n{}", proj);
        assert!(approx_eq!(
            Mat4,
            proj,
            Mat4::from(
                Vec4::from(1.207107, 0.000000,  0.000000,  0.000000),
                Vec4::from(0.000000, 2.414214,  0.000000,  0.000000),
                Vec4::from(0.000000, 0.000000, -1.020202, -0.202020),
                Vec4::from(0.000000, 0.000000, -1.000000,  0.000000)),
            epsilon = 0.000001));
    }

    #[test]
    fn interpolation() {
        let a = Vec3::from(1.0, 0.0, 2.0);
        let b = Vec3::from(0.0, 1.0, 3.0);
        let o = lerp(&a, &b, 0.5);
        println!("lerp: \n{}", o);
        assert!(approx_eq!(
            Vec3,
            o,
            Vec3::from(0.5, 0.5, 2.5),
            epsilon = 0.000001));

        let bc = Vec3::fill(1.0) / 3.0;
        let o = interpolate(&bc,
            &Vec3::from(1.0, 0.0, 0.0),
            &Vec3::from(0.0, 1.0, 0.0),
            &Vec3::from(0.0, 0.0, 1.0),
            );
        println!("interpolate: \n{}", o);
        assert!(approx_eq!(
            Vec3,
            o,
            Vec3::fill(1.0) / 3.0,
            epsilon = 0.000001));
    }

    #[test]
    fn reflect_refract() {
        let i = Vec3::fill(-1.0);
        let n = Vec3::from(0.0, 0.0, 1.0);
        let o = reflect(&i, &n);
        println!("reflect: {}", o);
        assert!(approx_eq!(
            Vec3,
            o,
            Vec3::from(-1.0, -1.0, 1.0),
            epsilon = 0.000001));

        let o = refract(&i, &n, 1.0);
        println!("refract: {}", o);
        assert!(approx_eq!(
            Vec3,
            o,
            Vec3::from(-1.0, -1.0, -1.0),
            epsilon = 0.000001));
    }

    #[test]
    fn intersection() {
        let center = Vec3::fill(0.0);
        let radius = 1.0;
        let ori = Vec3::from(-5.0, 0.0, 0.0);
        let dir = Vec3::from(10.0, 0.0, 1.0).normalize();
        if let Some(t) = intersect_sphere(&center, radius, &ori, &dir) {
            println!("line & sphere: {}", t);
            assert!(approx_eq!(
                Tyf,
                t,
                4.108,
                epsilon = 0.001));
        } else {
            println!("line & sphere: None");
        }

        let abc = [
            Vec3::from(1.0, 0.0, 0.0),
            Vec3::from(0.0, 1.0, 0.0),
            Vec3::from(0.0, 0.0, 1.0),
        ];
        let ori = Vec3::from(0.0, 0.0, 0.0);
        let dir = Vec3::from(1.0, 1.0, 1.0).normalize();
        if let Some((t, bc)) = intersect_triangle(&abc, &ori, &dir) {
            println!("line & triangle: {}, {}", t, bc);
            assert!(approx_eq!(
                Tyf,
                t,
                0.577,
                epsilon = 0.001));
            assert!(approx_eq!(
                Vec3,
                bc,
                Vec3::fill(1.0) / 3.0,
                epsilon = 0.000001));
        } else {
            println!("line & triangle: None");
        }
    }
} /* tests */
