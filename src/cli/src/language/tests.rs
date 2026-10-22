//! 七层各一条真样本，加契约查找。验收标准要能证伪。

use super::contract::Contract;
use super::model::Kind;
use super::scan;

fn contract() -> Contract {
    Contract::load(None).expect("仓里应能找到默认契约")
}

fn verdicts(text: &str) -> Vec<&'static str> {
    let contract = contract();
    let dir = std::env::temp_dir().join(format!("qfl-verdict-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("v-{}.md", text.len()));
    std::fs::write(&path, text).expect("写样本");
    let out = scan::check_file(&path, &contract, Some(&dir))
        .into_iter()
        .map(|finding| finding.verdict.key())
        .collect();
    let _ = std::fs::remove_file(&path);
    out
}

fn kinds(text: &str, rel: &str) -> Vec<&'static str> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let contract = contract();
    let dir = std::env::temp_dir().join(format!("qfl-test-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    // 并行测试共用文件名会互相覆盖，用序号错开
    let path = dir.join(format!(
        "sample-{}.md",
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
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
    assert!(
        found.is_file(),
        "契约必须是磁盘上的文件：{}",
        found.display()
    );
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
    assert!(
        findings
            .iter()
            .any(|finding| finding.kind == Kind::Placement)
    );
}

#[test]
fn number_without_denominator_is_reported() {
    assert!(kinds("改了 22 处\n", "").contains(&"numbers"));
}

#[test]
fn denominator_patterns_actually_match() {
    // 2026-10-22 修过：契约里原来写 '占\\s*\\d'（TOML 字面量 + 正则双层转义），
    // 编译出来要求字面反斜杠，七条分母模式一条都不生效。现在固定住「有分母就放过」。
    assert!(!kinds("返工 22 处，占 37%\n", "").contains(&"numbers"));
    assert!(!kinds("2026-10-22 实测：误报 8 处\n", "").contains(&"numbers"));
}

#[test]
fn descriptive_numbers_are_not_evidence() {
    // 估算、区间、阈值、上限、序数都是描述用的数字，不该要求分母
    assert!(!kinds("桌边三四个工作人员，人均 50–60 元。\n", "").contains(&"numbers"));
    assert!(!kinds("小红书最多 10 个标签，抖音最多 5 个。\n", "").contains(&"numbers"));
    assert!(!kinds("第 10 行报了一次，≥2 次才算类型。\n", "").contains(&"numbers"));
    // 但论据数字仍然要出处
    assert!(kinds("返工 22 处。\n", "").contains(&"numbers"));
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
fn conclusion_pushed_back_to_reader_is_reported() {
    assert!(kinds("先跑一遍再决定补不补，改不改由你定。\n", "").contains(&"syntax"));
}

#[test]
fn count_word_is_reported() {
    assert!(
        kinds(
            "工具判不了三件事：体裁、词的多义、某句是引述还是自述。\n",
            ""
        )
        .contains(&"syntax")
    );
    // 给事情命名也报：他说这类写法全是我写的，他讨厌
    assert!(kinds("搬迁收尾三件事、重写前先读现文件。\n", "").contains(&"syntax"));
}

#[test]
fn two_column_table_is_reported() {
    let text = "| 名词 | 解释 |\n|---|---|\n| 甲 | 乙 |\n";
    assert!(kinds(text, "").contains(&"table"));
}

#[test]
fn voice_reports_internal_id_wall() {
    let text =
        "改了 `a/b.md`、`c/d.rs`、`e/f.toml`、`g/h.json`、`i/j.yaml` 这五处，`k/l.sh` 也算。\n";
    let contract = contract();
    let dir = std::env::temp_dir().join(format!("qfl-voice-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("v-{}.md", std::process::id()));
    std::fs::write(&path, text).expect("写样本");
    let found = scan::check_file(&path, &contract, Some(&dir))
        .into_iter()
        .any(|f| f.kind == Kind::Voice);
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
    assert!(
        scan::check_file(&path, &contract, Some(&dir))
            .iter()
            .any(|f| f.kind == Kind::Syntax)
    );
    contract.syntax.max_sentence_chars = 0;
    contract.syntax.max_sentence_error_chars = 0;
    contract.syntax.max_de_chain = 0;
    contract.syntax.max_commas = 0;
    contract.syntax.max_nominal = 0;
    contract.syntax.max_paragraph_chars = 0;
    assert!(
        !scan::check_file(&path, &contract, Some(&dir))
            .iter()
            .any(|f| f.kind == Kind::Syntax)
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn sense_parses_json_verdict() {
    let verdict = super::sense::parse_verdict(
        "{\"kind\":\"process\",\"why\":\"在记操作\",\"fix\":\"改成对读者说\"}",
    );
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
        Finding::new(
            Kind::Voice,
            Severity::Suspect,
            "a.md",
            3,
            "一段内部标识符堆积的话".into(),
            "x".into(),
        ),
        Finding::new(
            Kind::Banned,
            Severity::Error,
            "b.md",
            1,
            "不变式".into(),
            "y".into(),
        ),
        Finding::new(
            Kind::Syntax,
            Severity::Suspect,
            "c.md",
            9,
            "一句特别长的话".into(),
            "z".into(),
        ),
    ];
    let segments = super::sense::suspect_segments(&findings, 5);
    assert_eq!(segments.len(), 2, "只挑 voice 与 syntax");
    assert_eq!(segments[0].text, "一段内部标识符堆积的话");
}

#[test]
fn sense_prompt_is_on_disk() {
    let prompt = super::sense::load_prompt();
    assert!(
        prompt.contains("reader") && prompt.contains("process"),
        "提示词要能读到且含两个判定类"
    );
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
        "default 走 exclude，跳过 data/journal，实际拿到：{:?}",
        default_files
    );
    assert_eq!(
        all_files.len(),
        2,
        "all 连 exclude 也不放过，实际拿到：{:?}",
        all_files
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn files_outside_docs_prefix_are_audited() {
    // categories/quanttide-handbook 那种布局：正文在 domains/ 与 default/，不在 docs/ 下。
    // 默认契约不填 include_default，所以这里该审到两个文件，而不是静默报「没有命中」。
    let contract = contract();
    let dir = std::env::temp_dir().join(format!("qfl-layout-{}", std::process::id()));
    let _ = std::fs::create_dir_all(dir.join("domains/human"));
    let _ = std::fs::create_dir_all(dir.join("default/company"));
    std::fs::write(dir.join("domains/human/a.md"), "不变式\n").expect("写样本");
    std::fs::write(dir.join("default/company/b.md"), "不变式\n").expect("写样本");
    let files = scan::markdown_files(&dir, &contract, "default");
    assert_eq!(
        files.len(),
        2,
        "domains/ 与 default/ 下的文件也该在范围内：{:?}",
        files
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn judgment_layer_separates_rule_noise_from_real_problems() {
    // 提及不是使用：词被引号包着讨论
    assert!(verdicts("- 词表里「接手人」该改成读者。\n").contains(&"mention"));
    // 引用工具自己的输出
    assert!(verdicts("合计 64 处（词汇层 9，句法层 27）\n").contains(&"quoted-output"));
    // 加粗小标签被并进句子
    assert!(
        verdicts(
            "**执行自己的想法**：让他按自己的思路做一次，观察他的做法与他的结果，看他的反应。\n"
        )
        .contains(&"lead-in")
    );
    // 真问题不带误报标
    assert!(!verdicts("改了 22 处。\n").iter().any(|v| *v != "real"));
}
