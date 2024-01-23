//! 宏相关操作
//!
//! ## repetions
//!
//! ```
//! macro_rules! rep {
//!     ( $src: expr ) => { $src };
//!     // repetions列表不可以','结尾
//!     ( $src: expr, $($rr: expr),+ ) => {
//!         // $($rr),+  => repetions列表仍以','分隔
//!         // $($rr)+  => repetions列表改用' '分隔
//!         // $($rr,)+  => repetions列表仍以','分隔，但列表末尾多1个','
//!         // 递归宏的repetions最好用'+'，防止造成无限递归
//!         $src ; rep!($($rr),+)
//!     };
//!     // repetions列表可以',结尾
//!     ( $src: expr, $($rr: expr),+ $(,)* ) => {
//!         $src ; rep!($($rr),+)
//!     };
//! }
//! ```

/// 通过repetions生成sep连接的表达式
///
/// - src: 需要连接的expression
/// - sep: 连接src的分隔符
#[macro_export]
macro_rules! rep2join_expr {
    ($sep:tt, $src:expr) => { $src };
    ($sep:tt, $src:expr, $($rr:expr),+) => {
        $src $sep rep2join_expr!($sep, $($rr),+)
    };
}

/// 根据repetions的个数，重复&'static str，使用sep连接
///
/// - str: 需要连接的&'static str
/// - sep: 连接str的分隔符
#[macro_export]
macro_rules! rep2join_str {
    ($str:tt, $sep:tt) => { "" };
    ($str:tt, $sep:tt, $r:expr) => { $str };
    ($str:tt, $sep:tt, $r:expr, $($rr:expr),+) => {
        std::concat!($str, $sep, rep2join_str!($str, $sep, $($rr),+))
    };
}

/// 统计repetions的个数
#[macro_export]
macro_rules! rep2cnt {
    () => { 0usize };
    ($r:tt) => { 1usize };
    ($r:tt, $($rr:tt),+) => {
        1usize + rep2cnt!($($rr),+)
    };
}

/// 根据repetions的个数，循环执行宏
///
/// - idx: loop的起始index
/// - src: 需要执行的宏，宏的参数为{index, expression}
#[allow(unused_macros)]
macro_rules! rep2loop {
    ($idx:expr, $src:ident, $r:tt) => { $src! {$idx, $r} };
    ($idx:expr, $src:ident, $r:tt, $($rr:tt),+) => {
        $src!{$idx, $r}
        rep2loop!($idx + 1usize, $src, $($rr),+);
    };
}

/// loop source of macro
///
/// ```ignore
/// rep2loop!(0, ls_def_idx, $($r),+);
/// ```
#[allow(unused_macros)]
macro_rules! ls_def_idx {
    ($idx:expr, $r:ident) => {
        #[allow(non_upper_case_globals)]
        const $r: usize = $idx;
    };
}

#[cfg(test)]
mod tests {
    macro_rules! test_rep2loop {
        ($($r:ident),+) => {
            let mut v: [usize; rep2cnt!($($r),+)] = [$($r),+];
            rep2loop!(0, ls_def_idx, $($r),+);
            $(v[$r] += 10;)+

            assert_eq!(0, r0);
            assert_eq!(1, r1);
            assert_eq!(2, r2);
            assert_eq!(3, r3);
            println!("vector: {:?}", v);
        };
    }

    /// 测试repetions相关宏
    #[test]
    fn macro_repetions() {
        assert_eq!(0, rep2cnt!());
        assert_eq!(1, rep2cnt!(5));
        assert_eq!("ccc", "c".repeat(rep2cnt!(1, 1, 5)));

        assert_eq!(15, rep2join_expr!(+, 1, 2, 3, 4, 5));

        assert_eq!("s;s;s", rep2join_str!("s", ";", 1, 2, 5));

        test_rep2loop! {r0, r1, r2, r3}
    }
} /* tests */
