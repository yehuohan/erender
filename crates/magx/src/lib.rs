//! 图形计算模块(Mathematics for Graphics)
//!
//! 包基本向量、矩阵计算和相关图形算法。

use num::Zero;
use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// 基本计算单元
pub trait GmPrimitive:
    Send
    + Sized
    + Copy
    + Clone
    + PartialOrd
    + PartialEq
    + Display
    + Add<Self, Output = Self>
    + AddAssign
    + Sub<Self, Output = Self>
    + SubAssign
    + Mul<Self, Output = Self>
    + MulAssign
    + Div<Self, Output = Self>
    + DivAssign
    + Zero
{
}

impl GmPrimitive for f64 {}
impl GmPrimitive for f32 {}
impl GmPrimitive for i64 {}
impl GmPrimitive for i32 {}
impl GmPrimitive for u64 {}
impl GmPrimitive for u32 {}
impl GmPrimitive for u8 {}

/// 浮点单元
pub type Tyf = f32;
/// 整型单元
pub type Tyi = i32;
/// Byte单元
pub type Tyb = u8;

mod geom;
mod matn;
mod vecn;
#[macro_use]
mod macro_utils;
pub use crate::geom::*;
pub use crate::matn::*;
pub use crate::vecn::*;
