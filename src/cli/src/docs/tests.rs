//! 各层一条真样本，加反样本。验收标准要能证伪：只有真样本的检查会「报得越多越对」。

use super::bold::check_bold;
use super::contract::Contract;
use super::density;
use super::journal::{check_journal, is_journal};
use super::list::check_list;
use super::model::Kind;
use super::rule::check_rule;
use super::status;
use super::table::check_table;

fn contract() -> Contract {
    Contract::load(None).expect(
        "仓里应能找到默认契约：确认 src/docs/default-contract.toml 在，或设 QTFOUNDER_DOCS_CONTRACT",
    )
}

fn kinds(text: &str) -> Vec<&'static str> {
    let contract = contract();
    let mut findings = check_table("sample.md", text, &contract);
    findings.extend(check_list("sample.md", text, &contract));
    findings.extend(check_bold("sample.md", text, &contract));
    findings.extend(check_rule("sample.md", text, &contract));
    findings
        .into_iter()
        .map(|finding| finding.kind.key())
        .collect()
}

#[test]
fn frontmatter_profiles_match_write_contract() {
    let docs = contract();
    let write = crate::write::contract::Contract::load(None)
        .expect("仓里应能找到 write 契约：确认 src/write/default-contract.toml 在");

    let mut docs_genre = docs.frontmatter.genre.clone();
    docs_genre.sort();
    let mut write_genre: Vec<String> = write.genre.keys().cloned().collect();
    write_genre.sort();
    assert_eq!(
        docs_genre, write_genre,
        "docs 的 genre 取值表与 write 的体裁画像不一致"
    );

    let mut docs_audience = docs.frontmatter.audience.clone();
    docs_audience.sort();
    let mut write_audience: Vec<String> = write.audience.keys().cloned().collect();
    write_audience.sort();
    assert_eq!(
        docs_audience, write_audience,
        "docs 的 audience 取值表与 write 的读者画像不一致"
    );

    let mut docs_topic = docs.frontmatter.topic.clone();
    docs_topic.sort();
    let mut write_topic: Vec<String> = write.topic.keys().cloned().collect();
    write_topic.sort();
    assert_eq!(
        docs_topic, write_topic,
        "docs 的 topic 取值表与 write 的主题画像不一致"
    );
}

#[test]
fn two_column_table_is_reported() {
    let text = "| 名称 | 说明 |\n| :-- | :-- |\n| 甲 | 乙 |\n";
    assert!(kinds(text).contains(&"table"));
}

#[test]
fn table_inside_code_block_is_ignored() {
    let text = "```markdown\n| 名称 | 说明 |\n| :-- | :-- |\n| 甲 | 乙 |\n```\n";
    assert!(kinds(text).is_empty());
}

#[test]
fn label_list_block_is_reported() {
    let text = "- **先定约束**：接活前把边界问清楚\n- **旧件新组**：把不相干的东西放一起\n- **先求数量**：一次做十个方案\n";
    assert!(kinds(text).contains(&"list"));
}

#[test]
fn plain_checklist_is_quiet() {
    let text = "- 安装 Rust\n- 跑 cargo test\n- 更新 CHANGELOG\n";
    assert!(kinds(text).is_empty());
}

#[test]
fn long_list_item_is_reported() {
    let long = "这一条本身就是一段话".repeat(13);
    let text = format!("- {long}\n- 短的一条\n- 另一条\n");
    assert!(kinds(&text).contains(&"list"));
}

#[test]
fn list_inside_code_block_is_ignored() {
    let text = "```text\n- **要点**：解释\n- **要点**：解释\n- **要点**：解释\n```\n";
    assert!(kinds(text).is_empty());
}

#[test]
fn density_counts_blocks_and_paragraphs() {
    let text = "开头一段话，讲清楚背景与结论。\n\n- 条目一\n- 条目二\n- 条目三\n\n| 甲 | 乙 |\n| :-- | :-- |\n| 一 | 二 |\n";
    let measures = density::measure(text, 30);
    assert_eq!(measures.blocks, 2);
    assert!(measures.block_ratio() > 0.0);
    // 表格紧跟在列表后，中间没有段落：两块算一个连续串
    assert_eq!(measures.run_blocks, 2);
}

#[test]
fn density_is_off_with_empty_thresholds() {
    let text = "- 条目一\n- 条目二\n- 条目三\n";
    let contract = contract();
    let measures = density::measure(text, contract.density.min_paragraph_chars);
    assert!(density::judge("sample.md", &measures, &contract.density).is_none());
}

#[test]
fn density_reports_when_threshold_is_set() {
    let text = "- 条目一\n- 条目二\n- 条目三\n- 条目四\n";
    let mut contract = contract();
    contract.density.max_block_ratio = 0.5;
    contract.density.min_chars = 0;
    let measures = density::measure(text, contract.density.min_paragraph_chars);
    let finding = density::judge("sample.md", &measures, &contract.density);
    assert_eq!(
        finding.map(|finding| finding.kind),
        Some(super::model::Kind::Density)
    );
}

#[test]
fn short_document_is_not_judged() {
    let text = "- 条目一\n- 条目二\n- 条目三\n";
    let mut contract = contract();
    contract.density.max_block_ratio = 0.5;
    contract.density.min_chars = 500;
    let measures = density::measure(text, contract.density.min_paragraph_chars);
    assert!(density::judge("sample.md", &measures, &contract.density).is_none());
}

#[test]
fn bold_on_one_line_is_reported() {
    let text = "这是 **甲** 与 **乙** 与 **丙**。\n";
    assert!(kinds(text).contains(&"bold"));
}

#[test]
fn long_bold_sentence_is_reported() {
    let bold = "这一整句都加粗了而且它长到超过三十个字应该降为正文处理不要整段强调";
    let text = format!("**{bold}**\n");
    assert!(kinds(&text).contains(&"bold"));
}

#[test]
fn one_short_bold_is_quiet() {
    let text = "**要点**：说明。\n";
    assert!(!kinds(text).contains(&"bold"));
}

#[test]
fn bold_table_header_is_not_counted() {
    let text = "| **验收标准** | **待标定** |\n| :-- | :-- |\n| 甲 | 乙 |\n";
    assert!(!kinds(text).contains(&"bold"));
}

#[test]
fn bold_density_is_reported() {
    let text = "**甲** 乙丙丁戊己庚辛壬癸。\n".repeat(20);
    assert!(kinds(&text).contains(&"bold"));
}

#[test]
fn rules_over_limit_are_reported() {
    let text = "段落。\n\n---\n\n段落。\n\n---\n\n段落。\n\n---\n\n段落。\n\n---\n\n段落。\n";
    assert!(kinds(text).contains(&"rule"));
}

#[test]
fn rule_next_to_heading_is_reported() {
    let text = "# 标题\n\n---\n\n正文。\n";
    assert!(kinds(text).contains(&"rule"));
}

#[test]
fn frontmatter_and_code_rules_are_ignored() {
    let text = "---\ntitle: 样本\n---\n\n正文。\n\n```text\n---\n---\n---\n---\n```\n";
    assert!(!kinds(text).contains(&"rule"));
}

#[test]
fn setext_underline_is_not_a_rule() {
    let text = "甲\n---\n乙\n---\n丙\n---\n丁\n---\n";
    assert!(!kinds(text).contains(&"rule"));
}

#[test]
fn journal_entry_with_matching_title_is_quiet() {
    let contract = contract();
    let text = "# 2026-10-27\n\n今天把日志模板接进 docs。\n";
    assert!(
        check_journal(
            "data/journal/2026-10/2026-10-27.md",
            text,
            &contract.journal
        )
        .is_empty()
    );
}

#[test]
fn journal_entry_with_wrong_title_is_reported() {
    let contract = contract();
    let text = "# 2026-10-26\n\n正文。\n";
    let findings = check_journal(
        "data/journal/2026-10/2026-10-27.md",
        text,
        &contract.journal,
    );
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].kind, Kind::Journal);
    assert!(findings[0].advice.contains("2026-10-27"));
}

#[test]
fn journal_entry_without_title_is_quiet() {
    let contract = contract();
    let text = "今天把日志模板接进 docs。\n";
    assert!(
        check_journal(
            "data/journal/2026-10/2026-10-27.md",
            text,
            &contract.journal
        )
        .is_empty()
    );
}

#[test]
fn journal_entry_with_non_date_filename_is_reported() {
    let contract = contract();
    let text = "今天把日志模板接进 docs。\n";
    let findings = check_journal("data/journal/2026-10/notes.md", text, &contract.journal);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].advice.contains("YYYY-MM-DD"));
}

#[test]
fn journal_readme_is_not_checked() {
    let contract = contract();
    assert!(!is_journal("data/journal/README.md", &contract.journal));
}

#[test]
fn excluded_journal_entry_is_still_checked() {
    let dir = std::env::temp_dir().join(format!("qtfounder-docs-journal-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("data/journal/2026-10")).unwrap();
    std::fs::write(
        dir.join("data/journal/2026-10/2026-10-27.md"),
        "# 2026-10-26\n\n正文。\n",
    )
    .unwrap();

    let (findings, _, _) = super::scan::scan(&dir, &contract());
    assert!(findings.iter().any(|finding| finding.kind == Kind::Journal));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn status_counts_every_format_element() {
    let text = "---\ntitle: 样本\n---\n\n# 标题\n\n正文一段话。\n\n- 甲\n- 乙\n\n| 名称 | 说明 |\n| :-- | :-- |\n| 甲 | 乙 |\n\n---\n\n> 引用一段。\n\n```text\ncode\n```\n";
    let one = status::count(text);
    assert_eq!(one.headings, 1);
    assert_eq!(one.h1, 1);
    assert_eq!(one.deepest, 1);
    assert_eq!(one.tables, 1);
    assert_eq!(one.table_rows, 3);
    assert_eq!(one.lists, 1);
    assert_eq!(one.list_items, 2);
    assert_eq!(one.rules, 1);
    assert_eq!(one.quotes, 1);
    assert_eq!(one.code_blocks, 1);
    assert_eq!(one.bold, 0);
    assert!(one.chars > 0);
    assert_eq!(
        one.elements(),
        one.headings + one.tables + one.lists + one.bold + one.rules + one.quotes + one.code_blocks
    );
}

#[test]
fn status_ignores_format_elements_inside_code_blocks() {
    let text =
        "```markdown\n# 标题\n- 条目\n| 甲 | 乙 |\n| :-- | :-- |\n---\n> 引用\n**加粗**\n```\n";
    let one = status::count(text);
    assert_eq!(one.code_blocks, 1);
    assert_eq!(one.headings, 0);
    assert_eq!(one.lists, 0);
    assert_eq!(one.tables, 0);
    assert_eq!(one.rules, 0);
    assert_eq!(one.quotes, 0);
    assert_eq!(one.bold, 0);
}

#[test]
fn loose_list_is_one_block() {
    let one = status::count("- 甲\n\n- 乙\n\n- 丙\n");
    assert_eq!(one.lists, 1);
    assert_eq!(one.list_items, 3);
}

#[test]
fn heading_levels_are_visible() {
    let one = status::count("# 一\n\n## 二\n\n### 三\n\n#### 四\n");
    assert_eq!(one.headings, 4);
    assert_eq!(one.h1, 1);
    assert_eq!(one.deepest, 4);
}

#[test]
fn table_header_bold_and_setext_rules_are_not_counted() {
    let table = status::count("| **甲** | **乙** |\n| :-- | :-- |\n| 一 | 二 |\n");
    assert_eq!(table.tables, 1);
    assert_eq!(table.bold, 0);

    let setext = status::count("甲\n---\n乙\n---\n");
    assert_eq!(setext.rules, 0);
}

#[test]
fn stray_table_line_is_not_a_table() {
    let one = status::count("| 只有一行 |\n");
    assert_eq!(one.tables, 0);
    assert_eq!(one.table_rows, 0);
}

#[test]
fn quote_segments_split_by_text_are_counted_separately() {
    let one = status::count("> 甲\n> 乙\n\n正文。\n\n> 丙\n");
    assert_eq!(one.quotes, 2);
}

#[test]
fn status_rule_count_agrees_with_audit() {
    let text = "段落。\n\n---\n\n段落。\n\n---\n\n段落。\n";
    let one = status::count(text);
    // 两处都在上限内：状态按处数数，审计只在超限时报
    assert_eq!(one.rules, 2);
    assert!(check_rule("sample.md", text, &contract()).is_empty());
}

#[test]
fn 目标缺席退三() {
    let args = super::DocsArgs {
        command: None,
        root: std::path::PathBuf::from("no-such-target-xyz"),
        kind: None,
        rules: None,
        top: 15,
        json: false,
        stats: false,
    };
    assert_eq!(
        super::run_docs(&args).unwrap(),
        crate::core::exit::NOT_FOUND
    );
}
