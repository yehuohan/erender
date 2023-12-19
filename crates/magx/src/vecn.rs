//! 低维向量

#![allow(dead_code)]

use std::ops::{
    Add, AddAssign,
    Sub, SubAssign,
    Mul, MulAssign,
    Div, DivAssign,
    Neg,
    Index, IndexMut,
};
use std::fmt::{Display, Formatter};
use float_cmp::ApproxEq;
use rand::distributions::{Standard, Distribution};
use crate::{rep2join_expr, rep2join_str};
use super::*;



/// vecn运算符(+ - * /)操作
///
/// - t: trait
/// - f: function
macro_rules! impl_vecn_ops {
    ( $t: ident, $f: ident, $VecN: ident, $Dim: expr, $($Ele: ident),+ ) => (
        impl<T: GmPrimitive> $t for $VecN<T> {
            type Output = Self;

            fn $f(self, rhs: Self) -> Self::Output {
                Self { $($Ele: self.$Ele.$f(rhs.$Ele)),+ }
            }
        }
        impl<T: GmPrimitive> $t<T> for $VecN<T> {
            type Output = Self;

            fn $f(self, rhs: T) -> Self::Output {
                Self { $($Ele: self.$Ele.$f(rhs)),+ }
            }
        }
    );
}

/// vecn运算符(+= -= *= /=)操作
///
/// - t: trait
/// - f: function
macro_rules! impl_vecn_ops_assign {
    ( $t: ident, $f: ident, $VecN: ident, $Dim: expr, $($Ele: ident),+ ) => (
        impl<T: GmPrimitive> $t for $VecN<T> {
            fn $f(&mut self, rhs: Self) {
                $(self.$Ele.$f(rhs.$Ele);)+
            }
        }
        impl<T: GmPrimitive> $t<T> for $VecN<T> {
            fn $f(&mut self, rhs: T) {
                $(self.$Ele.$f(rhs);)+
            }
        }
    )
}

/// 与维度相关的vecn操作
macro_rules! impl_vecn_inner {
    ( $VecN: ident, $Dim:expr, $x: ident, $y: ident ) => (
        impl<T: GmPrimitive> $VecN<T> {
            #[inline]
            pub const fn to_vec3(&self, v: T) -> VecN3::<T> {
                VecN3::<T>::from(self.$x, self.$y, v)
            }

            #[inline]
            pub const fn to_vec4(&self, v: T) -> VecN4::<T> {
                VecN4::<T>::from(self.$x, self.$y, v, v)
            }
        }
    );
    ( $VecN: ident, $Dim:expr, $x: ident, $y: ident, $z: ident ) => (
        impl<T: GmPrimitive> $VecN<T> {
            #[inline]
            pub const fn to_vec2(&self) -> VecN2::<T> {
                VecN2::<T>::from(self.$x, self.$y)
            }

            #[inline]
            pub const fn to_vec4(&self, v: T) -> VecN4::<T> {
                VecN4::<T>::from(self.$x, self.$y, self.$z, v)
            }

            /// 向量叉乘
            ///
            /// 利用行列式记忆，右手定则判断方向：
            ///
            /// | i  j  k  |
            /// | x1 y1 z1 |
            /// | x2 y2 z2 |
            #[inline]
            pub fn cross(&self, rhs: &Self) -> Self {
                Self {
                    $x: self.$y * rhs.$z - self.$z * rhs.$y,
                    $y: self.$z * rhs.$x - self.$x * rhs.$z,
                    $z: self.$x * rhs.$y - self.$y * rhs.$x,
                }
            }
        }
    );
    ( $VecN: ident, $Dim:expr, $x: ident, $y: ident, $z: ident, $w: ident ) => (
        impl<T: GmPrimitive> $VecN<T> {
            #[inline]
            pub const fn to_vec2(&self) -> VecN2::<T> {
                VecN2::<T>::from(self.$x, self.$y)
            }

            #[inline]
            pub const fn to_vec3(&self) -> VecN3::<T> {
                VecN3::<T>::from(self.$x, self.$y, self.$z)
            }
        }
    );
}

/// 基于宏实现vecn的基本操作
///
/// 向量元素字段为‘x y z w’。
///
/// - VecN: 向量名（vector name）
/// - Dim: 向量维度
/// - Ele: 向量元素字段名
/// - Idx: 向量元素对应下标
macro_rules! def_vecn {
    ( $VecN: ident, $Dim: expr, $($Ele: ident = $Idx: expr),+ ) => (
    // start definition

    /// VecN结构定义
    #[repr(C)] // 按c语言格式分配内存
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct $VecN<T: GmPrimitive> {
        $(pub $Ele: T),+
    }

    impl<T: GmPrimitive> $VecN<T> {
        #[inline]
        pub fn new() -> Self {
            Self { $($Ele: T::zero()),+ }
        }

        #[inline]
        pub const fn fill(v: T) -> Self {
            Self { $($Ele: v),+ }
        }

        #[inline]
        pub const fn from($($Ele: T),+) -> Self {
            Self { $($Ele: $Ele),+ }
        }

        #[inline]
        pub fn from_array<U>(v: &[U; $Dim]) -> Self
        where
            U: Copy + std::convert::Into<T>,
            T: From<U>
        {
            Self { $($Ele: T::from(v[$Idx])),+ }
        }

        $(
        #[inline]
        pub const fn $Ele(mut self, v: T) -> Self {
            self.$Ele = v;
            self
        }
        )+

        /// 向量点乘
        #[inline]
        pub fn dot(&self, rhs: &Self) -> T {
            rep2join_expr!(+, $(self.$Ele * rhs.$Ele),+)
        }

        /// 向量的平方范数(squared 2-norm of a vector)，即向量对自身做点积
        pub fn squared_norm(&self) -> T {
            self.dot(&self)
        }
    }

    // random要求T实现Distribution<T>
    impl<T: GmPrimitive> $VecN<T>
    where
        Standard: Distribution<T>
    {
        /// 生成随机向量
        #[inline]
        pub fn random() -> Self {
            Self { $($Ele: rand::random::<T>()),+ }
        }
    }

    impl From<$VecN<Tyf>> for $VecN<Tyi> {
        /// 浮点转定点
        fn from(v: $VecN<Tyf>) -> Self {
            Self { $($Ele: v.$Ele as Tyi),+ }
        }
    }

    impl From<$VecN<Tyi>> for $VecN<Tyf> {
        /// 定点转浮点
        fn from(v: $VecN<Tyi>) -> Self {
            Self { $($Ele: v.$Ele as Tyf),+ }
        }
    }

    impl $VecN<Tyf> {
        /// 向量归一化，即`1.0/norm()`（只针对浮点）
        pub fn normalize(&self) -> Self {
            let r = 1.0 / self.norm();
            Self { $($Ele: self.$Ele * r),+ }
        }

        /// 向量的L2范数(L2 norm)
        pub fn norm(&self) -> Tyf {
            self.dot(&self).sqrt()
        }
    }

    impl_vecn_inner!{$VecN, $Dim, $($Ele),+}
    impl_vecn_ops!{Add, add, $VecN, $Dim, $($Ele),+}
    impl_vecn_ops!{Sub, sub, $VecN, $Dim, $($Ele),+}
    impl_vecn_ops!{Mul, mul, $VecN, $Dim, $($Ele),+}
    impl_vecn_ops!{Div, div, $VecN, $Dim, $($Ele),+}
    impl_vecn_ops_assign!{AddAssign, add_assign, $VecN, $Dim, $($Ele),+}
    impl_vecn_ops_assign!{SubAssign, sub_assign, $VecN, $Dim, $($Ele),+}
    impl_vecn_ops_assign!{MulAssign, mul_assign, $VecN, $Dim, $($Ele),+}
    impl_vecn_ops_assign!{DivAssign, div_assign, $VecN, $Dim, $($Ele),+}

    impl<T> Neg for $VecN<T>
    where
        T: GmPrimitive + Neg<Output=T>
    {
        type Output = Self;

        fn neg(self) -> Self::Output {
            Self { $($Ele: -self.$Ele),+ }
        }
    }

    impl<T: GmPrimitive> Index<usize> for $VecN<T> {
        type Output = T;

        fn index(&self, index: usize) -> &Self::Output {
            assert!(index < $Dim);
            // 将VecN结构体内存按数组格式读取
            unsafe {
                &std::mem::transmute::<&Self, &[T; $Dim]>(self)[index]
            }
        }
    }

    impl<T: GmPrimitive> IndexMut<usize> for $VecN<T> {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            assert!(index < $Dim);
            unsafe {
                &mut std::mem::transmute::<&mut Self, &mut [T; $Dim]>(self)[index]
            }
        }
    }

    impl<T: GmPrimitive> Display for $VecN<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                std::concat!("[", rep2join_str!("{:>10.6}", ",", $($Ele),+), "]"),
                $(self.$Ele),+)
        }
    }

    // 浮点vecn的等值比较
    impl ApproxEq for $VecN<f32> {
        type Margin = float_cmp::F32Margin;

        fn approx_eq<T: Into<Self::Margin>>(self, other: Self, margin: T) -> bool {
            let margin = margin.into();
            // 所有element相等，则认为vecn相等
            rep2join_expr!(&&, $(self.$Ele.approx_eq(other.$Ele, margin)),+)
        }
    }

    impl ApproxEq for $VecN<f64> {
        type Margin = float_cmp::F64Margin;

        fn approx_eq<T: Into<Self::Margin>>(self, other: Self, margin: T) -> bool {
            let margin = margin.into();
            // 所有element相等，则认为vecn相等
            rep2join_expr!(&&, $(self.$Ele.approx_eq(other.$Ele, margin)),+)
        }
    }

    // end definition
    );
}

def_vecn!{VecN2, 2, x=0, y=1}
def_vecn!{VecN3, 3, x=0, y=1, z=2}
def_vecn!{VecN4, 4, x=0, y=1, z=2, w=3}

pub type Vec2 = VecN2<Tyf>;
pub type Vec3 = VecN3<Tyf>;
pub type Vec4 = VecN4<Tyf>;
pub type Vec2i = VecN2<Tyi>;
pub type Vec3i = VecN3<Tyi>;
pub type Vec4i = VecN4<Tyi>;



#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::approx_eq;

    /// 测试vecn的初始化
    #[test]
    fn vecn_initialization() {
        let v2 = Vec2::new().x(1.0).y(2.0);
        let v3 = Vec3::random();
        let v4 = Vec4::from(1.0, 2.0, 3.0, 4.0);
        println!("iv2: {}", Into::<Vec2i>::into(v2));
        println!("iv3: {}", Into::<Vec3i>::into(v3));
        println!("iv4: {}", Into::<Vec4i>::into(v4));

        let v2 = Vec2i::new().x(1).y(2);
        let v3 = Vec3i::fill(2);
        let v4 = Vec4i::from_array(&[1, 2, 3, 4]);
        println!("v2: {}", Into::<Vec2>::into(v2));
        println!("v3: {}", Into::<Vec3>::into(v3));
        println!("v4: {}", Into::<Vec4>::into(v4));
    }

    /// 测试vecn向量计算
    #[test]
    fn vecn_calculation() {
        let v2 = Vec2::new().x(1.0).y(2.0);
        let v3 = Vec3::new().x(1.0).y(2.0).z(3.0);
        let v4 = Vec4::new().x(1.0).y(2.0).z(3.0).w(4.0);
        assert!(approx_eq!(Tyf, v2.dot(&v2), 5.0, epsilon = 0.000001));
        assert!(approx_eq!(Tyf, v3.dot(&v3), 14.0, epsilon = 0.000001));
        assert!(approx_eq!(Tyf, v4.dot(&v4), 30.0, epsilon = 0.000001));
        assert!(approx_eq!(Vec2, v2.normalize(), Vec2::from(0.4472136, 0.89442719), epsilon = 0.000001));
        assert!(approx_eq!(Vec4, v4.normalize(), Vec4::from(0.18257419, 0.36514837, 0.54772256, 0.73029674), epsilon = 0.000001));
        let u = Vec3::from(1.0, 2.0, 3.0);
        let v = Vec3::from(3.0, 2.0, 1.0);
        assert!(approx_eq!(Vec3, u.cross(&v), Vec3::from(-4.0, 8.0, -4.0), epsilon = 0.000001));

        let v2 = Vec2i::from(1, 2);
        let v3 = Vec3i::from(1, 2, 3);
        let v4 = Vec4i::from(1, 2, 3, 4);
        assert_eq!((5, 14, 30), (v2.dot(&v2), v3.dot(&v3), v4.dot(&v4)));
        let u = Vec3i::from(1, 2, 3);
        let v = Vec3i::from(3, 2, 1);
        assert_eq!(Vec3i::from(-4, 8, -4), u.cross(&v));
    }

    /// 测试vecn运算符计算
    #[test]
    fn vecn_operators() {
        let mut v4 = Vec4::new();
        v4[0] = 1.0;
        v4[1] = 2.0;
        v4[2] = 3.0;
        v4[3] = 4.0;
        v4 += 2.0;
        v4 -= 1.0;
        v4 *= 6.0;
        v4 /= 2.0;
        assert!(approx_eq!(Vec4, v4, Vec4::from(6.0, 9.0, 12.0, 15.0), epsilon = 0.000001));

        let mut v4 = Vec4i::new();
        v4[0] = 1;
        v4[1] = 2;
        v4[2] = 3;
        v4[3] = 4;
        v4 += 2;
        v4 -= 1;
        v4 *= 6;
        v4 /= 2;
        assert_eq!(v4, Vec4i::from(6, 9, 12, 15));
    }
} /* tests */

