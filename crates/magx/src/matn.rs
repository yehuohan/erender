//! 低维矩阵

#![allow(dead_code)]

use super::*;
use crate::{rep2join_expr, rep2join_str};
use float_cmp::ApproxEq;
use rand::distributions::{Distribution, Standard};
use seq_macro::seq;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign};

/// matn运算符(+ - * /)操作
///
/// - t: trait
/// - f: function
macro_rules! impl_matn_ops {
    ($t:ident, $f:ident, $MatN:ident, $VecN:ident, $Row:expr, $Col:expr, $($Ele:ident),+) => {
        impl<T: GmPrimitive> $t for $MatN<T> {
            type Output = Self;

            fn $f(self, rhs: Self) -> Self::Output {
                Self { $($Ele: self.$Ele.$f(rhs.$Ele)),+}
            }
        }
        impl<T: GmPrimitive> $t<T> for $MatN<T> {
            type Output = Self;

            fn $f(self, rhs: T) -> Self::Output {
                Self { $($Ele: self.$Ele.$f(rhs)),+}
            }
        }
    };
}

/// matn运算符(+= -= *= /=)操作
///
/// - t: trait
/// - f: function
macro_rules! impl_vecn_ops_assign {
    ($t:ident, $f:ident, $MatN:ident, $VecN:ident, $Row:expr, $Col:expr, $($Ele:ident),+) => {
        impl<T: GmPrimitive> $t for $MatN<T> {
            fn $f(&mut self, rhs: Self) {
                $(self.$Ele.$f(rhs.$Ele);)+
            }
        }
        impl<T: GmPrimitive> $t<T> for $MatN<T> {
            fn $f(&mut self, rhs: T) {
                $(self.$Ele.$f(rhs);)+
            }
        }
    }
}

/// 与维度相关的matn操作（只适用于方矩阵）
macro_rules! impl_matn_inner {
    ($MatN:ident, $VecN:ident, $Row:expr, $Col:expr, $r0:ident, $r1:ident) => {
        impl<T: GmPrimitive> $MatN<T> {
            #[inline]
            pub fn eye(v: T) -> Self {
                Self {
                    $r0: $VecN::<T>::from(v, T::zero()),
                    $r1: $VecN::<T>::from(T::zero(), v),
                }
            }

            #[inline]
            pub fn diag(v: &$VecN<T>) -> Self {
                Self {
                    $r0: $VecN::<T>::from(v.x, T::zero()),
                    $r1: $VecN::<T>::from(T::zero(), v.y),
                }
            }
        }

        impl $MatN<Tyf> {
            /// 矩阵求逆
            #[inline]
            pub fn inverse(&self) -> $MatN<Tyf> {
                let d = 1.0 / (self[0][0] * self[1][1] - self[1][0] * self[0][1]);
                Self {
                    $r0: $VecN::<Tyf>::from(self[1][1] * d, -self[0][1] * d),
                    $r1: $VecN::<Tyf>::from(-self[1][0] * d, self[0][0] * d),
                }
            }
        }
    };
    ($MatN:ident, $VecN:ident, $Row:expr, $Col:expr, $r0:ident, $r1:ident, $r2:ident) => {
        impl<T: GmPrimitive> $MatN<T> {
            #[inline]
            pub fn eye(v: T) -> Self {
                Self {
                    $r0: $VecN::<T>::from(v, T::zero(), T::zero()),
                    $r1: $VecN::<T>::from(T::zero(), v, T::zero()),
                    $r2: $VecN::<T>::from(T::zero(), T::zero(), v),
                }
            }

            #[inline]
            pub fn diag(v: &$VecN<T>) -> Self {
                Self {
                    $r0: $VecN::<T>::from(v.x, T::zero(), T::zero()),
                    $r1: $VecN::<T>::from(T::zero(), v.y, T::zero()),
                    $r2: $VecN::<T>::from(T::zero(), T::zero(), v.z),
                }
            }

            #[inline]
            pub const fn to_mat4(&self, v: T) -> MatN4<T> {
                MatN4::<T>::from(
                    self.$r0.to_vec4(v),
                    self.$r1.to_vec4(v),
                    self.$r2.to_vec4(v),
                    VecN4::<T>::fill(v),
                )
            }
        }

        impl $MatN<Tyf> {
            /// 矩阵求逆（参考glm的算法）
            #[inline]
            pub fn inverse(&self) -> $MatN<Tyf> {
                let s = self[0][0] * (self[1][1] * self[2][2] - self[1][2] * self[2][1])
                    - self[0][1] * (self[1][0] * self[2][2] - self[1][2] * self[2][0])
                    + self[0][2] * (self[1][0] * self[2][1] - self[1][1] * self[2][0]);
                let d = 1.0 / s;
                Self {
                    $r0: $VecN::<Tyf>::from(
                        (self[1][1] * self[2][2] - self[1][2] * self[2][1]) * d,
                        -(self[0][1] * self[2][2] - self[0][2] * self[2][1]) * d,
                        (self[0][1] * self[1][2] - self[0][2] * self[1][1]) * d,
                    ),
                    $r1: $VecN::<Tyf>::from(
                        -(self[1][0] * self[2][2] - self[1][2] * self[2][0]) * d,
                        (self[0][0] * self[2][2] - self[0][2] * self[2][0]) * d,
                        -(self[0][0] * self[1][2] - self[0][2] * self[1][0]) * d,
                    ),
                    $r2: $VecN::<Tyf>::from(
                        (self[1][0] * self[2][1] - self[1][1] * self[2][0]) * d,
                        -(self[0][0] * self[2][1] - self[0][1] * self[2][0]) * d,
                        (self[0][0] * self[1][1] - self[0][1] * self[1][0]) * d,
                    ),
                }
            }
        }
    };
    ($MatN:ident, $VecN:ident, $Row:expr, $Col:expr, $r0:ident, $r1:ident, $r2:ident, $r3:ident) => {
        impl<T: GmPrimitive> $MatN<T> {
            #[inline]
            pub fn eye(v: T) -> Self {
                Self {
                    $r0: $VecN::<T>::from(v, T::zero(), T::zero(), T::zero()),
                    $r1: $VecN::<T>::from(T::zero(), v, T::zero(), T::zero()),
                    $r2: $VecN::<T>::from(T::zero(), T::zero(), v, T::zero()),
                    $r3: $VecN::<T>::from(T::zero(), T::zero(), T::zero(), v),
                }
            }

            #[inline]
            pub fn diag(v: &$VecN<T>) -> Self {
                Self {
                    $r0: $VecN::<T>::from(v.x, T::zero(), T::zero(), T::zero()),
                    $r1: $VecN::<T>::from(T::zero(), v.y, T::zero(), T::zero()),
                    $r2: $VecN::<T>::from(T::zero(), T::zero(), v.z, T::zero()),
                    $r3: $VecN::<T>::from(T::zero(), T::zero(), T::zero(), v.w),
                }
            }

            #[inline]
            pub const fn to_mat3(&self) -> MatN3<T> {
                MatN3::<T>::from(self.$r0.to_vec3(), self.$r1.to_vec3(), self.$r2.to_vec3())
            }
        }

        impl $MatN<Tyf> {
            /// 矩阵求逆（参考glm的算法）
            #[inline]
            pub fn inverse(&self) -> $MatN<Tyf> {
                let coef00 = self[2][2] * self[3][3] - self[2][3] * self[3][2];
                let coef02 = self[2][1] * self[3][3] - self[2][3] * self[3][1];
                let coef03 = self[2][1] * self[3][2] - self[2][2] * self[3][1];

                let coef04 = self[1][2] * self[3][3] - self[1][3] * self[3][2];
                let coef06 = self[1][1] * self[3][3] - self[1][3] * self[3][1];
                let coef07 = self[1][1] * self[3][2] - self[1][2] * self[3][1];

                let coef08 = self[1][2] * self[2][3] - self[1][3] * self[2][2];
                let coef10 = self[1][1] * self[2][3] - self[1][3] * self[2][1];
                let coef11 = self[1][1] * self[2][2] - self[1][2] * self[2][1];

                let coef12 = self[0][2] * self[3][3] - self[0][3] * self[3][2];
                let coef14 = self[0][1] * self[3][3] - self[0][3] * self[3][1];
                let coef15 = self[0][1] * self[3][2] - self[0][2] * self[3][1];

                let coef16 = self[0][2] * self[2][3] - self[0][3] * self[2][2];
                let coef18 = self[0][1] * self[2][3] - self[0][3] * self[2][1];
                let coef19 = self[0][1] * self[2][2] - self[0][2] * self[2][1];

                let coef20 = self[0][2] * self[1][3] - self[0][3] * self[1][2];
                let coef22 = self[0][1] * self[1][3] - self[0][3] * self[1][1];
                let coef23 = self[0][1] * self[1][2] - self[0][2] * self[1][1];

                let fac0 = $VecN::<Tyf>::from(coef00, coef00, coef02, coef03);
                let fac1 = $VecN::<Tyf>::from(coef04, coef04, coef06, coef07);
                let fac2 = $VecN::<Tyf>::from(coef08, coef08, coef10, coef11);
                let fac3 = $VecN::<Tyf>::from(coef12, coef12, coef14, coef15);
                let fac4 = $VecN::<Tyf>::from(coef16, coef16, coef18, coef19);
                let fac5 = $VecN::<Tyf>::from(coef20, coef20, coef22, coef23);

                let vec0 = $VecN::<Tyf>::from(self[0][1], self[0][0], self[0][0], self[0][0]);
                let vec1 = $VecN::<Tyf>::from(self[1][1], self[1][0], self[1][0], self[1][0]);
                let vec2 = $VecN::<Tyf>::from(self[2][1], self[2][0], self[2][0], self[2][0]);
                let vec3 = $VecN::<Tyf>::from(self[3][1], self[3][0], self[3][0], self[3][0]);

                let inv0 = vec1 * fac0 - vec2 * fac1 + vec3 * fac2;
                let inv1 = vec0 * fac0 - vec2 * fac3 + vec3 * fac4;
                let inv2 = vec0 * fac1 - vec1 * fac3 + vec3 * fac5;
                let inv3 = vec0 * fac2 - vec1 * fac4 + vec2 * fac5;

                let signa = $VecN::<Tyf>::from(1.0, -1.0, 1.0, -1.0);
                let signb = $VecN::<Tyf>::from(-1.0, 1.0, -1.0, 1.0);
                let inv = $MatN::<Tyf>::from(inv0 * signa, inv1 * signb, inv2 * signa, inv3 * signb).transpose();

                let row0 = $VecN::<Tyf>::from(inv[0][0], inv[0][1], inv[0][2], inv[0][3]);
                let dot0 = self.col(0) * row0;
                let dot1 = (dot0.x + dot0.y) + (dot0.z + dot0.w);
                let d = 1.0 / dot1;

                inv * d
            }
        }
    };
}

/// 基于宏实现matn的基本操作
///
/// 矩阵基于行向量实现（和GLM的列向量实现有所不同），第1行到第4行向量的字段名
/// 分别为‘r0 r1 r2 r3’。
///
/// - MatN: 矩阵名（matrix name）
/// - VecN: 矩阵行向量类型（vector name）
/// - Row: 矩阵行数
/// - Col: 矩阵列数
/// - Ele: 矩阵行向量字段名
macro_rules! def_matn {
    ($MatN:ident, $VecN:ident, $Row:expr, $Col:expr, $($Ele:ident=$Idx:expr),+) => {
    // start definition

    /// MatN结构定义
    #[repr(C)] // 按c语言格式分配内存
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct $MatN<T: GmPrimitive> {
        $(pub $Ele: $VecN::<T>),+
    }

    impl<T: GmPrimitive> $MatN<T> {
        #[inline]
        pub fn new() -> Self {
            Self { $($Ele: $VecN::<T>::new()),+ }
        }

        #[inline]
        pub const fn fill(v: T) -> Self {
            Self { $($Ele: $VecN::<T>::fill(v)),+ }
        }

        /// 由行向量创建矩阵
        #[inline]
        pub const fn from($($Ele: $VecN::<T>),+) -> Self {
            Self { $($Ele: $Ele),+ }
        }

        /// 由列向量创建矩阵
        #[inline]
        pub fn from_col($($Ele: $VecN::<T>),+) -> Self {
            (Self { $($Ele: $Ele),+ }).transpose()
        }

        #[inline]
        pub fn from_array<U>(v: &[[U; $Col]; $Row]) -> Self
        where
            U: Copy + std::convert::Into<T>,
            T: From<U>
        {
            Self { $($Ele: $VecN::<T>::from_array(&v[$Idx])),+ }
        }

        $(
        #[inline]
        pub const fn $Ele(mut self, v: $VecN::<T>) -> Self {
            self.$Ele = v;
            self
        }
        )+

        /// 获取矩阵列向量（只适用于方矩阵）
        #[inline]
        pub fn col(&self, index: usize) -> $VecN::<T> {
            $VecN::<T>::from($(self.$Ele[index]),+)
        }

        /// 矩阵与向量相乘（只适用于方矩阵）
        ///
        /// 矩阵的列数 必须等于 向量的维度 。
        #[inline]
        pub fn mul_vec(&self, rhs: &$VecN::<T>) -> $VecN::<T> {
            $VecN::<T>::from($(self.$Ele.dot(rhs)),+)
        }

        /// 矩阵乘法（只适用于方矩阵）
        ///
        /// self的列数 必须等于 rhs的行数（这里rhs的行和列均为Col，保证为方矩阵）。
        #[inline]
        pub fn mul_mat(&self, rhs: &$MatN::<T>) -> $MatN::<T> {
            // 生成rhs的列向量
            seq!(N in 0..$Col {
                let c~N = rhs.col(N);
            });

            Self { $(
                $Ele: seq!(N in 0..$Col{
                        $VecN::<T>::from( #(self.$Ele.dot(&c~N),)* )
                    })
            ),+ }
        }

        /// 矩阵转置
        #[inline]
        pub fn transpose(&self) -> $MatN::<T> {
            Self { $(
                $Ele: self.col($Idx)
            ),+ }
        }
    }

    impl<T: GmPrimitive> $MatN<T>
    where
        Standard: Distribution<T>
    {
        /// 生成随机矩阵
        #[inline]
        pub fn random() -> Self {
            Self { $($Ele: $VecN::<T>::random()),+ }
        }
    }

    impl From<$MatN<Tyf>> for $MatN<Tyi> {
        /// 浮点转定点
        fn from(v: $MatN<Tyf>) -> Self {
            Self { $($Ele: v.$Ele.into()),+ }
        }
    }

    impl From<$MatN<Tyi>> for $MatN<Tyf> {
        /// 定点转浮点
        fn from(v: $MatN<Tyi>) -> Self {
            Self { $($Ele: v.$Ele.into()),+ }
        }
    }


    impl $MatN<Tyf> {
        /// 向量归一化，即`1.0/norm()`（只针对浮点）
        pub fn normalize(&self) -> Self {
            let r = 1.0 / self.norm();
            Self { $($Ele: self.$Ele * r),+ }
        }

        /// 向量的L2范数(L2 norm)
        pub fn norm(&self) -> Tyf {
            (rep2join_expr!(+, $(self.$Ele.dot(&self.$Ele)),+)).sqrt()
        }
    }

    impl_matn_inner!{$MatN, $VecN, $Row, $Col, $($Ele),+}
    impl_matn_ops!{Add, add, $MatN, $VecN, $Row, $Col, $($Ele),+}
    impl_matn_ops!{Sub, sub, $MatN, $VecN, $Row, $Col, $($Ele),+}
    impl_matn_ops!{Mul, mul, $MatN, $VecN, $Row, $Col, $($Ele),+}
    impl_matn_ops!{Div, div, $MatN, $VecN, $Row, $Col, $($Ele),+}
    impl_vecn_ops_assign!{AddAssign, add_assign, $MatN, $VecN, $Row, $Col, $($Ele),+}
    impl_vecn_ops_assign!{SubAssign, sub_assign, $MatN, $VecN, $Row, $Col, $($Ele),+}
    impl_vecn_ops_assign!{MulAssign, mul_assign, $MatN, $VecN, $Row, $Col, $($Ele),+}
    impl_vecn_ops_assign!{DivAssign, div_assign, $MatN, $VecN, $Row, $Col, $($Ele),+}

    impl<T> Neg for $MatN<T>
    where
        T: GmPrimitive + Neg<Output=T>
    {
        type Output = Self;

        fn neg(self) -> Self::Output {
            Self { $($Ele: -self.$Ele),+ }
        }
    }

    impl<T: GmPrimitive> Index<usize> for $MatN<T> {
        type Output = $VecN::<T>;

        fn index(&self, index: usize) -> &Self::Output {
            assert!(index < $Row);
            // 将MatN结构体内存按VecN数组格式读取
            unsafe {
                &std::mem::transmute::<&Self, &[$VecN::<T>; $Row]>(self)[index]
            }
        }
    }

    impl<T: GmPrimitive> IndexMut<usize> for $MatN<T> {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            assert!(index < $Row);
            unsafe {
                &mut std::mem::transmute::<&mut Self, &mut [$VecN::<T>; $Row]>(self)[index]
            }
        }
    }

    impl<T: GmPrimitive> Display for $MatN<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                std::concat!("[", rep2join_str!("{}", ",\n ", $($Ele),+), "]"),
                $(self.$Ele),+)
        }
    }

    // 浮点vecn的等值比较
    impl ApproxEq for $MatN<f32> {
        type Margin = float_cmp::F32Margin;

        fn approx_eq<T: Into<Self::Margin>>(self, other: Self, margin: T) -> bool {
            let margin = margin.into();
            // 所有vector相等，则认为matn相等
            rep2join_expr!(&&, $(self.$Ele.approx_eq(other.$Ele, margin)),+)
        }
    }

    impl ApproxEq for $MatN<f64> {
        type Margin = float_cmp::F64Margin;

        fn approx_eq<T: Into<Self::Margin>>(self, other: Self, margin: T) -> bool {
            let margin = margin.into();
            // 所有vector相等，则认为matn相等
            rep2join_expr!(&&, $(self.$Ele.approx_eq(other.$Ele, margin)),+)
        }
    }

    // end definition
    };
}

def_matn! {MatN2, VecN2, 2, 2, r0=0, r1=1}
def_matn! {MatN3, VecN3, 3, 3, r0=0, r1=1, r2=2}
def_matn! {MatN4, VecN4, 4, 4, r0=0, r1=1, r2=2, r3=3}

pub type Mat2 = MatN2<Tyf>;
pub type Mat3 = MatN3<Tyf>;
pub type Mat4 = MatN4<Tyf>;
pub type Mat2i = MatN2<Tyi>;
pub type Mat3i = MatN3<Tyi>;
pub type Mat4i = MatN4<Tyi>;

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::approx_eq;

    #[test]
    fn matn_initialization() {
        let m2 = Mat2::new().r0(Vec2::fill(1.0)).r1(Vec2::fill(2.0));
        let m3 = Mat3::fill(2.0);
        let m4 = Mat4::from(Vec4::fill(1.0), Vec4::fill(2.0), Vec4::fill(3.0), Vec4::fill(4.0));
        println!("im2:\n{}", Into::<Mat2i>::into(m2));
        println!("im3:\n{}", Into::<Mat3i>::into(m3));
        println!("im4:\n{}", Into::<Mat4i>::into(m4));

        let m2 = Mat2i::from_array(&[[1, 2], [2, 4]]);
        let m3 = Mat3i::random();
        let m4 = Mat4i::eye(5);
        println!("m2:\n{}", Into::<Mat2>::into(m2));
        println!("m3:\n{}", Into::<Mat3>::into(m3));
        println!("m4:\n{}", Into::<Mat4>::into(m4));
        let mm = Mat2i::diag(&Vec2i::from(6, 7));
        println!("m2:\n{}", Into::<Mat2>::into(mm));
        let mm = Mat3i::diag(&Vec3i::from(6, 7, 8));
        println!("m3:\n{}", Into::<Mat3>::into(mm));
        let mm = Mat4i::diag(&Vec4i::from(6, 7, 8, 9));
        println!("m4:\n{}", Into::<Mat4>::into(mm));
    }

    #[test]
    fn matn_calculation() {
        // mul_vec
        let n = Mat4i::from(
            Vec4i::from(1, 2, 3, 4),
            Vec4i::from(2, 3, 4, 5),
            Vec4i::from(3, 4, 5, 6),
            Vec4i::from(4, 5, 6, 7),
        );
        let v = Vec4i::from(1, 2, 3, 4);
        println!("nxv:\n{}", n.mul_vec(&v));
        assert_eq!(n.mul_vec(&v), Vec4i::from(30, 40, 50, 60));

        // diag
        let n = Mat4::from(
            Vec4::from(1.0, 2.0, 3.0, 4.0),
            Vec4::from(2.0, 3.0, 4.0, 5.0),
            Vec4::from(3.0, 4.0, 5.0, 6.0),
            Vec4::from(4.0, 5.0, 6.0, 7.0),
        );
        let m = Mat4::diag(&Vec4::from(1.0, 2.0, 3.0, 4.0));
        println!("nxm:\n{}", n.mul_mat(&m));
        assert!(approx_eq!(
            Mat4,
            n.mul_mat(&m),
            Mat4::from(
                Vec4::from(1.0, 4.0, 9.0, 16.0),
                Vec4::from(2.0, 6.0, 12.0, 20.0),
                Vec4::from(3.0, 8.0, 15.0, 24.0),
                Vec4::from(4.0, 10.0, 18.0, 28.0)
            ),
            epsilon = 0.000001
        ));

        // norm & normalize
        let m = Mat4::diag(&Vec4::from(1.0, 2.0, 3.0, 4.0));
        println!("normalize: \n{}", m.normalize());
        println!("norm: {}", m.norm());
        assert!(approx_eq!(
            Mat4,
            m.normalize(),
            m / ((1.0 * 1.0 + 2.0 * 2.0 + 3.0 * 3.0 + 4.0 * 4.0) as Tyf).sqrt(),
            epsilon = 0.000001
        ));
        assert!(approx_eq!(
            Tyf,
            m.norm(),
            ((1.0 * 1.0 + 2.0 * 2.0 + 3.0 * 3.0 + 4.0 * 4.0) as Tyf).sqrt(),
            epsilon = 0.000001
        ));

        // transpose
        #[rustfmt::skip]
        let m = Mat4i::from_array(&[
            [1, 1, 1, 1],
            [2, 2, 2, 2],
            [3, 3, 3, 3],
            [4, 4, 4, 4],
        ]);
        #[rustfmt::skip]
        assert_eq!(m.transpose(),
            Mat4i::from_array(&[
                [1, 2, 3, 4],
                [1, 2, 3, 4],
                [1, 2, 3, 4],
                [1, 2, 3, 4]])
        );

        // inverse
        #[rustfmt::skip]
        let m = Mat2::from_array(&[
            [1.0, 2.0],
            [3.0, 4.0],
        ]);
        let im = m.inverse();
        println!("inverse2:\n{}", im);
        #[rustfmt::skip]
        assert!(approx_eq!(
            Mat2,
            im,
            Mat2::from_array(&[
                [-2.0, 1.0],
                [1.5, -0.5],
            ]),
            epsilon = 0.001
        ));
        #[rustfmt::skip]
        let m = Mat3::from_array(&[
            [ 1.707, 0.293,  1.000],
            [ 0.439, 2.561, -1.500],
            [-2.500, 2.500,  3.536],
        ]);
        let im = m.inverse();
        println!("inverse3:\n{}", im);
        #[rustfmt::skip]
        assert!(approx_eq!(Mat3, im,
            Mat3::from_array(&[
                [0.427,  0.049, -0.100],
                [0.073,  0.285,  0.100],
                [0.250, -0.167,  0.141],
            ]),
            epsilon = 0.001));
        #[rustfmt::skip]
        let m = Mat4::from_array(&[
            [ 3.732, 0.268,  1.414, 5.000],
            [ 0.201, 2.799, -1.061, 2.000],
            [-1.768, 1.768,  4.330, 3.000],
            [ 0.000, 0.000,  0.000, 1.000],
        ]);
        let im = m.inverse();
        println!("inverse4:\n{}", im);
        #[rustfmt::skip]
        assert!(approx_eq!(Mat4, im,
            Mat4::from_array(&[
            [  0.233,  0.022, -0.071, -0.999],
            [  0.017,  0.311,  0.071, -0.918],
            [  0.088, -0.118,  0.173, -0.726],
            [ -0.000,  0.000, -0.000,  1.000],
            ]),
            epsilon = 0.001));

        assert!(approx_eq!(
            Mat4,
            m.inverse().transpose(),
            m.transpose().inverse(),
            epsilon = 0.001
        ));
    }

    #[test]
    fn matn_operators() {
        let mut m4 = Mat4::new();
        m4[0] = Vec4::fill(1.0);
        m4[1] = Vec4::fill(2.0);
        m4[2] = Vec4::fill(3.0);
        m4[3] = Vec4::fill(4.0);
        m4 += 2.0;
        m4 -= 1.0;
        m4 *= 6.0;
        m4 /= 2.0;
        assert!(approx_eq!(
            Mat4,
            m4,
            Mat4::from(Vec4::fill(6.0), Vec4::fill(9.0), Vec4::fill(12.0), Vec4::fill(15.0)),
            epsilon = 0.000001
        ));

        let mut m4 = Mat4i::new();
        m4[0] = Vec4i::fill(1);
        m4[1] = Vec4i::fill(2);
        m4[2] = Vec4i::fill(3);
        m4[3] = Vec4i::fill(4);
        m4 += 2;
        m4 -= 1;
        m4 *= 6;
        m4 /= 2;
        assert_eq!(
            m4,
            Mat4i::from(Vec4i::fill(6), Vec4i::fill(9), Vec4i::fill(12), Vec4i::fill(15))
        );
    }
} /* tests */
