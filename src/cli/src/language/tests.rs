//! 七层各一条真样本，加契约查找。判据要能证伪。

use super::contract::Contract;
use super::model::Kind;
use super::scan;

fn contract() -> Contract {
    Contract::load(None).expect("仓里应能找到默认契约")
}

fn kinds(text: &str, rel: &str) -> Vec<&'static str> {
    let contract = contract();
    let dir = std::env::temp_dir().join(format!("qfl-test-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("sample.md");
    std::fs::write(&path, text).expect("写样本");
    let findings = scan::check_file(&path, &contract, Some(&dir));
    let _ = std::fs::remove_file(&path);
    findings
        .into_iter()
        .filter(|finding| rel.is_empty() || finding.path == rel || finding.kind.key() == rel)
        .map(|finding| finding.kind.key())
        .collect()
}

#[test]
fn contract_is_a_real_file_on_disk() {
    let found = super::contract::find_contract().expect("应能找到契约");
    assert!(found.is_file(), "契约必须是磁盘上的文件：{}", found.display());
    assert!(contract().syntax.max_sentence_chars > 0);
}

#[test]
fn banned_word_is_reported() {
    assert!(kinds("这里写了不变式三个字\n", "").contains(&"banned"));
}

#[test]
fn placement_misplacement_is_reported() {
    let contract = contract();
    let findings = scan::check_file(std::path::Path::new("data/essay/x.md"), &contract, None);
    assert!(findings.iter().any(|finding| finding.kind == Kind::Placement));
}

#[test]
fn number_without_denominator_is_reported() {
    assert!(kinds("改了 22 处\n", "").contains(&"numbers"));
}

#[test]
fn denominator_patterns_are_known_broken_in_both_implementations() {
    // 契约里 `denominator_patterns` 写的是 '占\\s*\\d' 这种（TOML 字面量 + regex 双层转义），
    // 编译出来要求字面反斜杠，两边一致地不生效。这里把现状钉住：谁要修契约，先看这条。
    let contract = contract();
    let broken = contract
        .numbers
        .denominator_patterns
        .iter()
        .any(|pattern| pattern.contains("\\\\s"));
    assert!(broken, "契约若已修成单反斜杠，请把这条测试改成「有分母就通过」");
}

#[test]
fn threshold_is_not_a_statistic() {
    assert!(!kinds("命中 ≥2 次才算\n", "").contains(&"numbers"));
}

#[test]
fn long_sentence_is_reported() {
    let long = "字".repeat(120);
    assert!(kinds(&format!("{long}\n"), "").contains(&"syntax"));
}

#[test]
fn two_column_table_is_reported() {
    let text = "| 名词 | 解释 |\n|---|---|\n| 甲 | 乙 |\n";
    assert!(kinds(text, "").contains(&"table"));
}

#[test]
fn voice_reports_internal_id_wall() {
    let text = "改了 `a/b.md`、`c/d.rs`、`e/f.toml`、`g/h.json`、`i/j.yaml` 这五处，`k/l.sh` 也算。\n";
    let contract = contract();
    let dir = std::env::temp_dir().join(format!("qfl-voice-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("v.md");
    std::fs::write(&path, text).expect("写样本");
    let found = scan::check_file(&path, &contract, Some(&dir)).into_iter().any(|f| f.kind == Kind::Voice);
    let _ = std::fs::remove_file(&path);
    assert!(found);
}

#[test]
fn clean_paragraph_reports_nothing() {
    assert!(kinds("你要改文档之前，先跑一遍检查，看有没有自造术语。\n", "").is_empty());
}

#[test]
fn a_changed_contract_changes_the_verdict() {
    let text = "字".repeat(120);
    let mut contract = contract();
    let dir = std::env::temp_dir().join(format!("qfl-change-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("c.md");
    std::fs::write(&path, &text).expect("写样本");
    assert!(scan::check_file(&path, &contract, Some(&dir)).iter().any(|f| f.kind == Kind::Syntax));
    contract.syntax.max_sentence_chars = 0;
    contract.syntax.max_sentence_error_chars = 0;
    contract.syntax.max_de_chain = 0;
    contract.syntax.max_commas = 0;
    contract.syntax.max_nominal = 0;
    contract.syntax.max_paragraph_chars = 0;
    assert!(!scan::check_file(&path, &contract, Some(&dir)).iter().any(|f| f.kind == Kind::Syntax));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn missing_contract_is_an_explicit_error() {
    let error = Contract::load(Some(std::path::Path::new("/nonexistent/c.toml")))
        .expect_err("契约不存在时必须报错");
    assert!(error.contains("契约读不了"), "{error}");
}

#[test]
fn scope_default_skips_journal() {
    let contract = contract();
    let dir = std::env::temp_dir().join(format!("qfl-scope-{}", std::process::id()));
    let _ = std::fs::create_dir_all(dir.join("docs"));
    let _ = std::fs::create_dir_all(dir.join("data/journal"));
    std::fs::write(dir.join("docs/a.md"), "不变式\n").expect("写 samples");
    std::fs::write(dir.join("data/journal/b.md"), "不变式\n").expect("写 samples");
    let default_files = scan::markdown_files(&dir, &contract, "default");
    let all_files = scan::markdown_files(&dir, &contract, "all");
    assert_eq!(
        default_files.len(),
        1,
        "default 只审规范文档，实际拿到：{:?}",
        default_files
    );
    assert_eq!(all_files.len(), 2, "all 含流水，实际拿到：{:?}", all_files);
    let _ = std::fs::remove_dir_all(&dir);
}
