//! `agent skill` 的判定测试：一组真样本、一组反样本。
//!
//! 用内存里的正文，不碰磁盘；目录名靠路径推算。

use std::path::Path;

use super::contract::Contract;
use super::lint::{Finding, Severity, SkillFile, duplicates, inspect, parse};

fn run(dir: &str, text: &str) -> Vec<Finding> {
    let contract = Contract::bundled();
    let path = format!("/tmp/skills/{dir}/SKILL.md");
    let skill = parse(Path::new(&path), text);
    inspect(&skill, &contract)
}

fn kinds(findings: &[Finding]) -> Vec<&str> {
    findings.iter().map(|finding| finding.kind).collect()
}

const GOOD: &str = "---\nname: git-commit\ndescription: 规范提交。当用户要提交时使用。\n---\n\n# git-commit\n\n按约定提交。\n";

#[test]
fn 合口径的没有命中() {
    let findings = run("git-commit", GOOD);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn 名词化后缀报错() {
    let text = GOOD.replace("git-commit", "product-management");
    let findings = run("product-management", &text);
    assert_eq!(kinds(&findings), vec!["verb"]);
    assert!(findings[0].message.contains("ment"));
}

#[test]
fn 名字与目录对不上() {
    let findings = run(
        "git-commit",
        &GOOD.replace("name: git-commit", "name: git-revert"),
    );
    assert!(kinds(&findings).contains(&"name"));
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("对不上"))
    );
}

#[test]
fn 不是小写连字符() {
    let text =
        "---\nname: Language_Govern\ndescription: 治理语言。当要清词时使用。\n---\n\n# 治理\n";
    let findings = run("Language_Govern", text);
    assert!(findings.iter().any(|finding| finding.kind == "name"));
}

#[test]
fn 一段名字当嫌疑() {
    let text = "---\nname: docs\ndescription: 查文档。当要查时使用。\n---\n\n# 文档\n\n跑 qtfounder docs。\n";
    let findings = run("docs", text);
    assert_eq!(kinds(&findings), vec!["name"]);
    assert_eq!(findings[0].severity, Severity::Suspect);
}

#[test]
fn 没登记过的范围当嫌疑() {
    let text = "---\nname: foo-bar\ndescription: 做点什么。当需要时使用。\n---\n\n# 标题\n";
    let findings = run("foo-bar", text);
    assert_eq!(kinds(&findings), vec!["scope"]);
}

#[test]
fn 缺说明报错() {
    let text = "---\nname: git-commit\n---\n\n# git-commit\n";
    let findings = run("git-commit", text);
    assert_eq!(kinds(&findings), vec!["description"]);
}

#[test]
fn 说明没写触发时机当嫌疑() {
    let text = "---\nname: git-commit\ndescription: 规范提交。\n---\n\n# git-commit\n";
    let findings = run("git-commit", text);
    assert_eq!(kinds(&findings), vec!["description"]);
    assert_eq!(findings[0].severity, Severity::Suspect);
}

#[test]
fn 命令组名却没写命令() {
    let text =
        "---\nname: think-reflect\ndescription: 自我反思。当情绪不对时使用。\n---\n\n# 反思\n";
    let findings = run("think-reflect", text);
    assert_eq!(kinds(&findings), vec!["command"]);
}

#[test]
fn 正文缺一级标题() {
    let text = "---\nname: think-reflect\ndescription: 自我反思。当情绪不对时使用。\n---\n\n跑 qtfounder think reflect 记一条。\n";
    let findings = run("think-reflect", text);
    assert_eq!(kinds(&findings), vec!["heading"]);
}

#[test]
fn 名字太长报错() {
    let long = format!("git-{}", "a".repeat(40));
    let text = format!("---\nname: {long}\ndescription: 说明。当用时使用。\n---\n\n# 标题\n");
    let findings = run(&long, &text);
    assert!(findings.iter().any(|finding| finding.kind == "name"));
}

#[test]
fn 缺名字只报一条并停下() {
    let text = "---\ndescription: 没有名字。当用时使用。\n---\n\n# 标题\n";
    let findings = run("git-commit", text);
    assert_eq!(kinds(&findings), vec!["name"]);
}

#[test]
fn 重名报错() {
    let first = parse(Path::new("/tmp/a/git-commit/SKILL.md"), GOOD);
    let second = parse(Path::new("/tmp/b/git-commit/SKILL.md"), GOOD);
    let findings = duplicates(&[first, second]);
    assert_eq!(kinds(&findings), vec!["unique"]);
    assert!(findings[0].path.contains("/tmp/b/"));
}

#[test]
fn 多行说明也会读全() {
    let text = "---\nname: git-commit\ndescription: >\n  规范提交。\n  当用户要提交时使用。\n---\n\n# git-commit\n";
    let findings = run("git-commit", text);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn 跳过契约里排除的目录() {
    use super::lint::scan;
    let contract = Contract::bundled();
    let dir = std::env::temp_dir().join(format!("qtfounder-agent-{}", std::process::id()));
    let skipped = dir.join("target/skills/git-commit");
    let kept = dir.join("skills/git-commit");
    for path in [&skipped, &kept] {
        std::fs::create_dir_all(path).expect("建目录");
        std::fs::write(path.join("SKILL.md"), GOOD).expect("写文件");
    }
    let skills: Vec<SkillFile> = scan(&dir, &contract).expect("扫描");
    assert_eq!(skills.len(), 1);
    assert!(skills[0].path.ends_with("skills/git-commit/SKILL.md"));
    std::fs::remove_dir_all(&dir).ok();
}
