//! `git hook` 的解析测试：只验 `git submodule status` 的读法，不碰仓库。

use std::path::Path;

use super::hook::parse_submodule_line;

const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn 正常子模块带描述() {
    let line = format!(" {SHA} docs/gallery (heads/main)");
    let repo = parse_submodule_line(Path::new("/repo"), &line).expect("应解析");
    assert_eq!(repo.label, "docs/gallery");
    assert_eq!(repo.dir, Path::new("/repo/docs/gallery"));
    assert!(repo.initialized);
}

#[test]
fn 未初始化的子模块() {
    let line = format!("-{SHA} data/context");
    let repo = parse_submodule_line(Path::new("/repo"), &line).expect("应解析");
    assert_eq!(repo.label, "data/context");
    assert!(!repo.initialized);
}

#[test]
fn 路径可以带空格() {
    let line = format!("+{SHA} assets/new thing");
    let repo = parse_submodule_line(Path::new("/repo"), &line).expect("应解析");
    assert_eq!(repo.label, "assets/new thing");
}

#[test]
fn 空行与非状态行忽略() {
    assert!(parse_submodule_line(Path::new("/repo"), "").is_none());
    assert!(parse_submodule_line(Path::new("/repo"), "fatal: 不是仓").is_none());
}
