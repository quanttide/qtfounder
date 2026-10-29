//! 退出码。CLI 的 API 面之一（命令、输出、退出码、落盘、契约），`cli/v0.1.0` 起冻结。
//!
//! 四码，语义单点定义在这里，各命令只消费，不自带字面量。
//! 2 与 3 的分界一句话：输入写法没错、对象缺席 = 3。

/// 跑通，无命中。
pub const OK: i32 = 0;
/// 跑通，有命中（审计有发现、判类为拒）。
pub const FINDINGS: i32 = 1;
/// 用法或环境错：参数不认识、契约读不了、层名写错。
pub const USAGE: i32 = 2;
/// 目标缺席：输入写法没错，但对象不在（路径不存在、记录缺失）。
pub const NOT_FOUND: i32 = 3;

#[cfg(test)]
mod tests {
    use super::*;

    /// 冻结锚点：四码互不相同且恰为 0..=3，谁改常量的值，`cargo test` 直接红。
    #[test]
    fn 四码钉在零到三() {
        let mut codes = [OK, FINDINGS, USAGE, NOT_FOUND];
        codes.sort_unstable();
        assert_eq!(codes, [0, 1, 2, 3]);
    }
}
