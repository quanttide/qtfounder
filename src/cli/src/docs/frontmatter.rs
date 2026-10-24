//! Front matter 层：文件头那几行 YAML。
//!
//! 只做两件事：字段名在不在允许表里，取值在不在表里。没有 front matter 的文档不管。
//!
//! 字段允许表解决「各写各的」，取值表解决「同一个意思三种写法」。后者尤其要紧：
//! `draft`、`待验证`、`pending` 混用时，谁也说不清哪一篇可以引用。

use super::common::excerpt_of;
use super::contract::Frontmatter;
use super::model::{Finding, Kind, Severity};

/// 解析文件头的 front matter，返回（字段，结束行号）。没有就返回 None。
fn parse(text: &str) -> Option<(Vec<(usize, String, String)>, usize)> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.first().map(|line| line.trim()) != Some("---") {
        return None;
    }
    let mut fields = Vec::new();
    for (index, raw) in lines.iter().enumerate().skip(1) {
        let trimmed = raw.trim();
        if trimmed == "---" {
            return Some((fields, index + 1));
        }
        // 只认顶格的 `键: 值`：缩进行是上一个键的值，列表项不是字段。
        if raw.starts_with(char::is_whitespace)
            || trimmed.starts_with('-')
            || trimmed.starts_with('#')
        {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let value = value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        fields.push((index + 1, key.trim().to_string(), value));
    }
    None
}

pub(super) fn check_frontmatter(path: &str, text: &str, contract: &Frontmatter) -> Vec<Finding> {
    let Some((fields, _)) = parse(text) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (line, key, value) in fields {
        if !contract.keys.is_empty() && !contract.keys.iter().any(|allowed| allowed == &key) {
            out.push(Finding::new(
                Kind::Frontmatter,
                Severity::Suspect,
                path,
                line,
                excerpt_of(&format!("{key}: {value}")),
                format!(
                    "字段 `{key}` 不在允许表里：只用 {}；新字段先写进契约",
                    contract.keys.join("、")
                ),
            ));
            continue;
        }
        let allowed = match key.as_str() {
            "status" => &contract.status,
            "author" => &contract.author,
            _ => continue,
        };
        if allowed.is_empty() || allowed.iter().any(|item| item == &value) {
            continue;
        }
        out.push(Finding::new(
            Kind::Frontmatter,
            Severity::Error,
            path,
            line,
            excerpt_of(&format!("{key}: {value}")),
            format!(
                "`{key}` 的取值不在表里：只有 {}；不要另造近义词",
                allowed.join("、")
            ),
        ));
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
                "author",
                "status",
                "title",
                "authors",
                "exports",
            ]
            .iter()
            .map(|item| item.to_string())
            .collect(),
            status: ["draft", "pending", "final"]
                .iter()
                .map(|item| item.to_string())
                .collect(),
            author: ["agent", "human"]
                .iter()
                .map(|item| item.to_string())
                .collect(),
        }
    }

    #[test]
    fn good_front_matter_is_quiet() {
        let text = "---\nauthor: agent\nstatus: pending\n---\n\n# 标题\n";
        assert!(check_frontmatter("a.md", text, &contract()).is_empty());
    }

    #[test]
    fn unknown_status_is_reported() {
        let text = "---\nauthor: agent\nstatus: 待验证\n---\n\n# 标题\n";
        let findings = check_frontmatter("a.md", text, &contract());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].line, 3);
        assert!(findings[0].advice.contains("draft、pending、final"));
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
    fn nested_values_are_not_fields() {
        let text = "---\ntitle: 测试\nauthors:\n  - 甲\nexports:\n  - format: pdf\n---\n\n# 标题\n";
        assert!(check_frontmatter("a.md", text, &contract()).is_empty());
    }
}
