//! 四层各一条真样本，加反样本。检查要能证伪：只报不静的规则等于没写。

use super::checks::{Selection, check_text};
use super::contract::Contract;
use super::model::Severity;

fn contract() -> Contract {
    Contract::load(None).expect(
        "仓里应能找到默认契约：确认 src/write/default-contract.toml 在，或设 QTFOUNDER_WRITE_CONTRACT",
    )
}

fn kind_list(text: &str) -> Vec<&'static str> {
    kind_list_with(text, &Selection::default())
}

fn kind_list_with(text: &str, selection: &Selection) -> Vec<&'static str> {
    check_text("sample.md", text, &contract(), selection)
        .into_iter()
        .map(|finding| finding.kind.key())
        .collect()
}

#[test]
fn slogan_is_reported() {
    let text = "# 标题\n\n## 一节\n\n这不是工具，而是制度。\n";
    assert!(kind_list(text).contains(&"slogan"));
}

#[test]
fn slogan_inside_code_block_is_ignored() {
    let text = "# 标题\n\n## 一节\n\n```markdown\n这不是工具，而是制度。\n```\n";
    assert!(!kind_list(text).contains(&"slogan"));
}

#[test]
fn reference_is_reported() {
    let text = "# 标题\n\n## 一节\n\n详见 docs/guide.md。\n";
    assert!(kind_list(text).contains(&"reference"));
}

#[test]
fn reference_inside_code_block_is_ignored() {
    let text = "# 标题\n\n## 一节\n\n```bash\n参见 docs/guide.md\n```\n";
    assert!(!kind_list(text).contains(&"reference"));
}

#[test]
fn missing_heading_is_reported() {
    let text = "没有标题的一段文字。\n";
    assert!(kind_list(text).contains(&"structure"));
}

#[test]
fn heading_jump_is_reported() {
    let text = "# 标题\n\n### 跳级\n\n正文\n";
    assert!(kind_list(text).contains(&"structure"));
}

#[test]
fn long_document_without_sections_is_reported() {
    let body = "正文".repeat(300);
    let text = format!("# 标题\n\n{body}\n");
    assert!(kind_list(&text).contains(&"structure"));
}

#[test]
fn heavy_empty_words_are_reported() {
    let body = "正文".repeat(160);
    let text = format!("# 标题\n\n## 一节\n\n{body}进一步进一步进一步进一步进一步\n");
    assert!(kind_list(&text).contains(&"empty"));
}

#[test]
fn plain_document_is_quiet() {
    let text = "# 标题\n\n## 一节\n\n先说结论。再说做法。\n";
    assert!(kind_list(text).is_empty());
}

#[test]
fn design_genre_requires_sections() {
    let text = "# 标题\n\n## 问题\n\n## 定义\n\n## 验证\n\n## 结论\n";
    let selection = Selection {
        genre: Some("design".to_string()),
        ..Selection::default()
    };
    assert!(kind_list_with(text, &selection).contains(&"genre"));
}

#[test]
fn roadmap_genre_forbids_insight_words() {
    let text = "# 标题\n\n## 顺序\n\n达成标志。\n\n后置代价。\n\n假说。\n";
    let selection = Selection {
        genre: Some("roadmap".to_string()),
        ..Selection::default()
    };
    assert!(kind_list_with(text, &selection).contains(&"genre"));
}

#[test]
fn human_audience_needs_reader_words() {
    let text = "# 标题\n\n## 一节\n\n系统负责结算。\n";
    let selection = Selection {
        audience: Some("human".to_string()),
        ..Selection::default()
    };
    assert!(kind_list_with(text, &selection).contains(&"audience"));
}

#[test]
fn game_topic_needs_definition_section() {
    let text = "# 标题\n\n## 一节\n\n事件进入报告。\n";
    let selection = Selection {
        topic: Some("game".to_string()),
        ..Selection::default()
    };
    assert!(kind_list_with(text, &selection).contains(&"topic"));
}

#[test]
fn game_topic_is_quiet_with_definition() {
    let text = "# 标题\n\n## 定义\n\n事件是发生的事。\n";
    let selection = Selection {
        topic: Some("game".to_string()),
        ..Selection::default()
    };
    assert!(!kind_list_with(text, &selection).contains(&"topic"));
}

#[test]
fn unknown_profile_is_an_error() {
    let text = "# 标题\n\n## 一节\n\n正文。\n";
    let selection = Selection {
        genre: Some("nope".to_string()),
        ..Selection::default()
    };
    let findings = check_text("sample.md", text, &contract(), &selection);
    assert!(
        findings
            .iter()
            .any(|finding| finding.kind.key() == "genre" && finding.severity == Severity::Error)
    );
}
