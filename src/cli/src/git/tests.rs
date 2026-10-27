//! `git` 的判定测试：子模块状态解析、闸门跳过范围与新旧比对。

use std::path::Path;

use crate::language::model::{Finding, Kind, Severity, Verdict};

use super::common::parse_submodule_line;
use super::run::{new_findings, skipped};

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

#[test]
fn 流水与变更日志不查() {
    assert!(skipped("CHANGELOG.md"));
    assert!(skipped("docs/gallery/CHANGELOG.md"));
    assert!(skipped("data/context/journal/2026-10-27.md"));
    assert!(skipped("docs/report/week.md"));
    assert!(skipped("examples/lab/note.md"));
    assert!(!skipped("docs/gallery/org/rank.md"));
}

fn finding(advice: &str) -> Finding {
    Finding::new(
        Kind::Banned,
        Severity::Error,
        "note.md",
        1,
        String::new(),
        advice.to_string(),
    )
}

#[test]
fn 只报这一笔新引入的() {
    let current = vec![finding("「坏词」→ 好词")];
    assert_eq!(
        new_findings(&current, &[]),
        vec![("banned".to_string(), "「坏词」→ 好词".to_string())]
    );
    assert!(new_findings(&current, &current).is_empty());
}

#[test]
fn 同类问题多出一条才算新() {
    let mut current = vec![finding("「坏词」→ 好词")];
    current.push(finding("「坏词」→ 好词"));
    let baseline = vec![finding("「坏词」→ 好词")];
    assert_eq!(new_findings(&current, &baseline).len(), 1);
}

#[test]
fn 嫌疑与误报不拦() {
    let mut suspect = finding("「坏词」→ 好词");
    suspect.severity = Severity::Suspect;
    assert!(new_findings(&[suspect], &[]).is_empty());

    let mut mentioned = finding("「坏词」→ 好词");
    mentioned.mark(Verdict::Mention);
    assert!(new_findings(&[mentioned], &[]).is_empty());
}
