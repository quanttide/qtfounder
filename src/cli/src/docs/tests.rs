//! 各层一条真样本，加反样本。验收标准要能证伪：只有真样本的检查会「报得越多越对」。

use super::contract::Contract;
use super::density;
use super::list::check_list;
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
    findings
        .into_iter()
        .map(|finding| finding.kind.key())
        .collect()
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
    let long = "这一条本身就是一段话".repeat(12);
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
