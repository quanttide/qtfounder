//! 分割线层：分章靠标题，不靠横线。
//!
//! 两件事：全文处数过量；紧挨标题、只是装饰的那几条单独报。
//! 首尾 `---` 是 frontmatter，文末下面的 `---` 是 setext 标题，都不是分割线。

use super::common::excerpt_of;
use super::contract::Contract;
use super::model::{Finding, Kind, Severity};

pub(super) fn check_rule(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let config = &contract.rule;
    if config.max == 0 && !config.flag_between_sections {
        return Vec::new();
    }
    let lines: Vec<&str> = text.lines().collect();
    let positions = rule_lines(text);

    let mut out = Vec::new();
    if config.max > 0 && positions.len() > config.max {
        let line = positions[config.max];
        out.push(Finding::new(
            Kind::Rule,
            Severity::Error,
            rel,
            line,
            excerpt_of(lines[line - 1].trim()),
            format!(
                "分割线 {} 处（上限 {}）：用标题分章，横线不承担结构",
                positions.len(),
                config.max
            ),
        ));
    }
    if config.flag_between_sections {
        for line in &positions {
            if touches_heading(&lines, line - 1) {
                out.push(Finding::new(
                    Kind::Rule,
                    Severity::Suspect,
                    rel,
                    *line,
                    excerpt_of(lines[line - 1].trim()),
                    "分割线紧挨标题，只是装饰：删掉，靠标题层级分章".to_string(),
                ));
            }
        }
    }
    out
}

/// 全文分割线所在行号（1 起）。格式状态与审计共用这一处识别，免得两边数出来的条数不一样。
pub(super) fn rule_lines(text: &str) -> Vec<usize> {
    let lines: Vec<&str> = text.lines().collect();
    let mut positions = Vec::new();
    let mut in_code = false;
    let mut in_frontmatter = false;
    let mut seen_content = false;

    for (index, raw) in lines.iter().enumerate() {
        let trimmed = raw.trim();
        if !seen_content && trimmed == "---" {
            in_frontmatter = true;
            seen_content = true;
            continue;
        }
        if in_frontmatter {
            if trimmed == "---" {
                in_frontmatter = false;
            }
            continue;
        }
        if trimmed.starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        if !trimmed.is_empty() {
            seen_content = true;
        }
        if !is_rule(trimmed) {
            continue;
        }
        if trimmed.starts_with('-') && is_setext_underline(&lines, index) {
            continue;
        }
        positions.push(index + 1);
    }
    positions
}

fn is_rule(trimmed: &str) -> bool {
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() < 3 {
        return false;
    }
    let marker = chars[0];
    matches!(marker, '-' | '*' | '_') && chars.iter().all(|ch| *ch == marker)
}

/// 上一行紧贴着普通文本、这一行是 `---`：Markdown 里那是二级标题的下划线，不是分割线。
/// 中间隔了空行就不算——分割线前面通常就是空行。
fn is_setext_underline(lines: &[&str], index: usize) -> bool {
    if index == 0 {
        return false;
    }
    let above = lines[index - 1].trim();
    if above.is_empty() {
        return false;
    }
    !(above.starts_with('#')
        || above.starts_with("- ")
        || above.starts_with("* ")
        || above.starts_with('|')
        || is_rule(above))
}

/// 紧挨着标题（上一条或下一条非空行是 `#` 开头）。
fn touches_heading(lines: &[&str], index: usize) -> bool {
    let above = nearest_non_empty(lines, index, true);
    let below = nearest_non_empty(lines, index, false);
    above.is_some_and(|line| line.starts_with('#'))
        || below.is_some_and(|line| line.starts_with('#'))
}

fn nearest_non_empty<'a>(lines: &[&'a str], index: usize, upward: bool) -> Option<&'a str> {
    if upward {
        for line in lines[..index].iter().rev() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    } else {
        for line in lines.get(index + 1..).unwrap_or(&[]) {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}
