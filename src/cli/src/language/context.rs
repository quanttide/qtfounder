//! 语境声明：文件头写一行 `context: finance`，属于这个语境的词就不再算错配。
//!
//! 章程第五条的例外要能落到文件上。落不下来，例外就只在提交者的判断里，
//! 每次改这份文件都要人工重判一次，或者绕开校验。

/// 读出 `context` 的声明值。没有 front matter、没写这一行，都是 None。
///
/// 只认第一个 `---` 块里的顶格键：缩进行归上一个字段，`- ` 是列表项，`#` 是注释，都不是声明。
pub fn declared(text: &str) -> Option<String> {
    let mut lines = text.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    for raw in lines {
        let trimmed = raw.trim();
        if trimmed == "---" {
            break;
        }
        if raw.starts_with(char::is_whitespace)
            || trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with('-')
        {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        if key.trim() != "context" {
            continue;
        }
        let value = value.trim().trim_matches('"').trim_matches('\'');
        return (!value.is_empty()).then(|| value.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::declared;

    #[test]
    fn reads_declared_context() {
        let text = "---\nstatus: draft\ncontext: finance\n---\n\n# 标题\n";
        assert_eq!(declared(text).as_deref(), Some("finance"));
    }

    #[test]
    fn no_front_matter_means_no_context() {
        assert!(declared("# 标题\n\n正文。\n").is_none());
    }

    #[test]
    fn indented_context_is_not_a_declaration() {
        let text = "---\nauthors:\n  - name: 甲\n    context: finance\n---\n\n# 标题\n";
        assert!(declared(text).is_none());
    }

    #[test]
    fn empty_value_is_not_a_declaration() {
        assert!(declared("---\ncontext:\n---\n\n# 标题\n").is_none());
    }
}
