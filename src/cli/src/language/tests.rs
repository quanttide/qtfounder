//! 七层各一条真样本，加契约查找。判据要能证伪。

use super::contract::Contract;
use super::model::Kind;
use super::scan;

fn contract() -> Contract {
    Contract::load(None).expect("仓里应能找到默认契约")
}

fn kinds(text: &str, rel: &str) -> Vec<&'static str> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let contract = contract();
    let dir = std::env::temp_dir().join(format!("qfl-test-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    // 并行测试共用文件名会互相覆盖，用序号错开
    let path = dir.join(format!("sample-{}.md", COUNTER.fetch_add(1, Ordering::Relaxed)));
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
    let path = dir.join(format!("v-{}.md", std::process::id()));
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
    let path = dir.join(format!("c-{}.md", std::process::id()));
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
fn sense_parses_json_verdict() {
    let verdict = super::sense::parse_verdict("{\"kind\":\"process\",\"why\":\"在记操作\",\"fix\":\"改成对读者说\"}");
    assert_eq!(verdict.kind, "process");
    assert_eq!(verdict.fix, "改成对读者说");
}

#[test]
fn sense_degrades_when_model_returns_prose() {
    let verdict = super::sense::parse_verdict("这段像是在记操作，但我没按格式回");
    assert_eq!(verdict.kind, "unknown");
}

#[test]
fn sense_takes_only_voice_and_syntax_segments() {
    use super::model::{Finding, Kind, Severity};
    let findings = vec![
        Finding::new(Kind::Voice, Severity::Suspect, "a.md", 3, "一段内部标识符堆积的话".into(), "x".into()),
        Finding::new(Kind::Banned, Severity::Error, "b.md", 1, "不变式".into(), "y".into()),
        Finding::new(Kind::Syntax, Severity::Suspect, "c.md", 9, "一句特别长的话".into(), "z".into()),
    ];
    let segments = super::sense::suspect_segments(&findings, 5);
    assert_eq!(segments.len(), 2, "只挑 voice 与 syntax");
    assert_eq!(segments[0].text, "一段内部标识符堆积的话");
}

#[test]
fn sense_prompt_is_on_disk() {
    let prompt = super::sense::load_prompt();
    assert!(prompt.contains("reader") && prompt.contains("process"), "提示词要能读到且含两个判定类");
}

#[test]
fn deep_root_still_counts_as_in_scope() {
    // 在 docs/handbook 里跑，rel 不含 docs 前缀；靠祖先把范围补回来，否则静默报「没有命中」
    let contract = contract();
    let dir = std::env::temp_dir().join(format!("qfl-deep-{}", std::process::id()));
    let deep = dir.join("docs/handbook");
    let _ = std::fs::create_dir_all(&deep);
    std::fs::write(deep.join("a.md"), "不变式\n").expect("写样本");
    let files = scan::markdown_files(&deep, &contract, "default");
    assert_eq!(files.len(), 1, "深层 root 也该在范围内");
    let _ = std::fs::remove_dir_all(&dir);
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
    // exclude 对所有 scope 生效——all 只是放开 include 前缀，不是把流水也审一遍
    assert_eq!(
        all_files.len(),
        1,
        "all 仍跳过 data/journal，实际拿到：{:?}",
        all_files
    );
    let _ = std::fs::remove_dir_all(&dir);
}
