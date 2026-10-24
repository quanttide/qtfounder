//! Front matter 层：文件头那几行 YAML。
//!
//! 三件事：字段名在不在允许表里，`status` 的取值在不在表里，`authors` 的每一条有没有标
//! human 还是 agent。
//!
//! 署名沿用 MyST 的 `authors`，只是每条多一个 `type`。写模型的，`name` 就是模型名，
//! 像人署名一样；这样以后翻到任何一篇，都知道是哪个模型写的。
//!
//! 只解析块状写法：
//!
//! ```yaml
//! authors:
//!   - name: 张三
//!     type: human
//!   - name: DeepSeek V4.1 Flash
//!     type: agent
//! ```
//!
//! 流式写法（`authors: [...]`）报嫌疑，因为它没法逐条落行号，改起来也看不清。

use super::common::excerpt_of;
use super::contract::Frontmatter;
use super::model::{Finding, Kind, Severity};

/// 一个顶层字段，连带它下面缩进的行。
struct Field {
    line: usize,
    key: String,
    value: String,
    block: Vec<(usize, String)>,
}

fn parse(text: &str) -> Option<Vec<Field>> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.first().map(|line| line.trim()) != Some("---") {
        return None;
    }
    let mut fields: Vec<Field> = Vec::new();
    for (index, raw) in lines.iter().enumerate().skip(1) {
        let trimmed = raw.trim();
        if trimmed == "---" {
            return Some(fields);
        }
        // 缩进行归上一个字段；列表项不是字段；注释跳过。
        if raw.starts_with(char::is_whitespace) {
            if let Some(field) = fields.last_mut() {
                field.block.push((index + 1, raw.to_string()));
            }
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        fields.push(Field {
            line: index + 1,
            key: key.trim().to_string(),
            value: value
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string(),
            block: Vec::new(),
        });
    }
    None
}

/// 一条署名：位置、字段值。
struct Author {
    line: usize,
    fields: Vec<(String, String)>,
}

impl Author {
    fn get(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }
}

fn split_entry(line: usize, body: &str) -> Author {
    let mut author = Author {
        line,
        fields: Vec::new(),
    };
    if let Some((key, value)) = body.split_once(':') {
        author.fields.push((key.trim().to_string(), clean(value)));
    }
    author
}

fn clean(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

/// 解析 `authors` 的块：每条以 `- ` 开头，后面的缩进行归这条。
fn authors_of(field: &Field) -> Vec<Author> {
    let mut authors: Vec<Author> = Vec::new();
    for (line, raw) in &field.block {
        let trimmed = raw.trim();
        if let Some(body) = trimmed.strip_prefix("- ") {
            authors.push(split_entry(*line, body));
            continue;
        }
        if trimmed == "-" {
            authors.push(Author {
                line: *line,
                fields: Vec::new(),
            });
            continue;
        }
        let Some(author) = authors.last_mut() else {
            continue;
        };
        if let Some((key, value)) = trimmed.split_once(':') {
            author.fields.push((key.trim().to_string(), clean(value)));
        }
    }
    authors
}

pub(super) fn check_frontmatter(path: &str, text: &str, contract: &Frontmatter) -> Vec<Finding> {
    let Some(fields) = parse(text) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for field in fields {
        if !contract.keys.is_empty() && !contract.keys.iter().any(|key| key == &field.key) {
            out.push(Finding::new(
                Kind::Frontmatter,
                Severity::Suspect,
                path,
                field.line,
                excerpt_of(&format!("{}: {}", field.key, field.value)),
                format!(
                    "字段 `{}` 不在允许表里：只用 {}；新字段先写进契约",
                    field.key,
                    contract.keys.join("、")
                ),
            ));
            continue;
        }

        if field.key == "status" {
            if contract.status.is_empty() || contract.status.iter().any(|item| item == &field.value)
            {
                continue;
            }
            out.push(Finding::new(
                Kind::Frontmatter,
                Severity::Error,
                path,
                field.line,
                excerpt_of(&format!("status: {}", field.value)),
                format!(
                    "`status` 的取值不在表里：只有 {}；不要另造近义词",
                    contract.status.join("、")
                ),
            ));
            continue;
        }

        if field.key == "authors" {
            out.extend(check_authors(path, &field, contract));
        }

        if field.key == "context" {
            if contract.contexts.is_empty()
                || contract.contexts.iter().any(|item| item == &field.value)
            {
                continue;
            }
            out.push(Finding::new(
                Kind::Frontmatter,
                Severity::Error,
                path,
                field.line,
                excerpt_of(&format!("context: {}", field.value)),
                format!(
                    "`context` 的取值不在表里：只有 {}；写错了，语言检查不会放行该语境的词",
                    contract.contexts.join("、")
                ),
            ));
        }
    }
    out
}

fn check_authors(path: &str, field: &Field, contract: &Frontmatter) -> Vec<Finding> {
    let mut out = Vec::new();
    let authors = authors_of(field);
    if authors.is_empty() {
        if field.value.is_empty() {
            return out;
        }
        out.push(Finding::new(
            Kind::Frontmatter,
            Severity::Suspect,
            path,
            field.line,
            excerpt_of(&format!("authors: {}", field.value)),
            "`authors` 用块列表写，每条才能标 `type` 与模型名".to_string(),
        ));
        return out;
    }
    for author in authors {
        let Some(kind) = author.get("type") else {
            out.push(Finding::new(
                Kind::Frontmatter,
                Severity::Error,
                path,
                author.line,
                excerpt_of(&format!(
                    "- {}",
                    author
                        .fields
                        .iter()
                        .map(|(k, v)| format!("{k}: {v}"))
                        .collect::<Vec<_>>()
                        .join("，")
                )),
                "每条署名要标 `type: human` 或 `type: agent`；不标就分不清谁写的".to_string(),
            ));
            continue;
        };
        if !contract.author_types.is_empty()
            && !contract.author_types.iter().any(|item| item == kind)
        {
            out.push(Finding::new(
                Kind::Frontmatter,
                Severity::Error,
                path,
                author.line,
                excerpt_of(&format!("type: {kind}")),
                format!(
                    "`type` 的取值不在表里：只有 {}",
                    contract.author_types.join("、")
                ),
            ));
            continue;
        }
        if kind == "agent" && author.get("name").is_none() {
            out.push(Finding::new(
                Kind::Frontmatter,
                Severity::Suspect,
                path,
                author.line,
                excerpt_of("- type: agent"),
                "agent 署名缺模型名：把模型名写进 `name`，将来才知道是谁写的".to_string(),
            ));
        }
        if kind == "org" {
            let Some(name) = author.get("name") else {
                out.push(Finding::new(
                    Kind::Frontmatter,
                    Severity::Error,
                    path,
                    author.line,
                    excerpt_of("- type: org"),
                    "法人署名要写全名：见契约 `orgs`，四个写法照抄".to_string(),
                ));
                continue;
            };
            if !contract.orgs.is_empty() && !contract.orgs.iter().any(|org| org == name) {
                out.push(Finding::new(
                    Kind::Frontmatter,
                    Severity::Error,
                    path,
                    author.line,
                    excerpt_of(&format!("name: {name}")),
                    format!(
                        "法人名不在表里：只有 {}；不写简称、不改字",
                        contract.orgs.join("、")
                    ),
                ));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::check_frontmatter;
    use crate::docs::contract::Frontmatter;

    fn contract() -> Frontmatter {
        Frontmatter {
            keys: [
                "name",
                "description",
                "status",
                "title",
                "authors",
                "exports",
                "context",
            ]
            .iter()
            .map(|item| item.to_string())
            .collect(),
            status: ["draft", "pending", "final"]
                .iter()
                .map(|item| item.to_string())
                .collect(),
            author_types: ["human", "agent", "org"]
                .iter()
                .map(|item| item.to_string())
                .collect(),
            orgs: ["量潮创始人", "量潮科技", "量潮实训基地", "量潮创新联盟"]
                .iter()
                .map(|item| item.to_string())
                .collect(),
            contexts: ["finance", "technology", "law"]
                .iter()
                .map(|item| item.to_string())
                .collect(),
        }
    }

    #[test]
    fn good_front_matter_is_quiet() {
        let text = "---\nstatus: pending\nauthors:\n  - name: 张三\n    type: human\n  - name: DeepSeek V4.1 Flash\n    type: agent\n---\n\n# 标题\n";
        assert!(check_frontmatter("a.md", text, &contract()).is_empty());
    }

    #[test]
    fn unknown_status_is_reported() {
        let text = "---\nstatus: 待验证\n---\n\n# 标题\n";
        let findings = check_frontmatter("a.md", text, &contract());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].line, 2);
    }

    #[test]
    fn unknown_field_is_a_suspect() {
        let text = "---\nstatus: draft\nversion: 2\n---\n\n# 标题\n";
        let findings = check_frontmatter("a.md", text, &contract());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind.key(), "frontmatter");
    }

    #[test]
    fn document_without_front_matter_is_ignored() {
        let text = "# 标题\n\n正文。\n";
        assert!(check_frontmatter("a.md", text, &contract()).is_empty());
    }

    #[test]
    fn author_without_type_is_reported() {
        let text = "---\nauthors:\n  - name: 张三\n---\n\n# 标题\n";
        let findings = check_frontmatter("a.md", text, &contract());
        assert_eq!(findings.len(), 1);
        assert!(findings[0].advice.contains("type"));
    }

    #[test]
    fn bad_author_type_is_reported() {
        let text = "---\nauthors:\n  - name: 张三\n    type: 人\n---\n\n# 标题\n";
        let findings = check_frontmatter("a.md", text, &contract());
        assert_eq!(findings.len(), 1);
        assert!(findings[0].advice.contains("human、agent"));
    }

    #[test]
    fn agent_without_name_is_a_suspect() {
        let text = "---\nauthors:\n  - type: agent\n---\n\n# 标题\n";
        let findings = check_frontmatter("a.md", text, &contract());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity.label(), "嫌疑");
    }

    #[test]
    fn org_name_must_come_from_the_list() {
        let good = "---\nauthors:\n  - name: 量潮科技\n    type: org\n---\n\n# 标题\n";
        assert!(check_frontmatter("a.md", good, &contract()).is_empty());
        let bad = "---\nauthors:\n  - name: 量潮\n    type: org\n---\n\n# 标题\n";
        let findings = check_frontmatter("a.md", bad, &contract());
        assert_eq!(findings.len(), 1);
        assert!(findings[0].advice.contains("量潮科技"));
    }

    #[test]
    fn nested_values_are_not_fields() {
        let text = "---\ntitle: 测试\nauthors:\n  - name: 甲\n    type: human\nexports:\n  - format: pdf\n---\n\n# 标题\n";
        assert!(check_frontmatter("a.md", text, &contract()).is_empty());
    }

    #[test]
    fn declared_context_is_allowed() {
        let text = "---\ncontext: finance\n---\n\n# 标题\n";
        assert!(check_frontmatter("a.md", text, &contract()).is_empty());
    }

    #[test]
    fn unknown_context_is_reported() {
        let text = "---\ncontext: financ\n---\n\n# 标题\n";
        let findings = check_frontmatter("a.md", text, &contract());
        assert_eq!(findings.len(), 1);
        assert!(findings[0].advice.contains("finance"));
    }
}
