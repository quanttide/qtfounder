//! 加粗层：满篇重点、整句加粗、一行并列几处，都是「重点太多等于没有重点」。
//!
//! 不判加粗用在哪（那要读语义），只判三件可数的事：每千字几处、一行几处、一处几个字。

use super::common::{char_count, excerpt_of};
use super::contract::Contract;
use super::model::{Finding, Kind, Severity};

pub(super) fn check_bold(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let config = &contract.bold;
    if config.max_per_1000 == 0 && config.max_per_line == 0 && config.max_span_chars == 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut in_code = false;
    let mut total = 0usize;
    let mut chars = 0usize;

    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        if raw.trim().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        chars += char_count(raw);
        let spans = bold_spans(raw);
        total += spans.len();
        if config.max_per_line > 0 && spans.len() > config.max_per_line {
            out.push(Finding::new(
                Kind::Bold,
                Severity::Error,
                rel,
                line,
                excerpt_of(raw.trim()),
                format!(
                    "一行 {} 处加粗（上限 {}）：重点多了就没重点，留一处",
                    spans.len(),
                    config.max_per_line
                ),
            ));
        }
        if config.max_span_chars > 0 {
            for span in spans {
                let count = char_count(&span);
                if count > config.max_span_chars {
                    out.push(Finding::new(
                        Kind::Bold,
                        Severity::Error,
                        rel,
                        line,
                        excerpt_of(&span),
                        format!(
                            "加粗 {count} 字（上限 {}）：整句加粗等于没加粗，改成段落或去掉",
                            config.max_span_chars
                        ),
                    ));
                }
            }
        }
    }

    if config.max_per_1000 > 0 && chars >= config.min_chars && chars > 0 {
        let density = total as f64 * 1000.0 / chars as f64;
        if density > config.max_per_1000 as f64 {
            out.push(Finding::new(
                Kind::Bold,
                Severity::Suspect,
                rel,
                1,
                format!("加粗 {total} 处 / {chars} 字"),
                format!(
                    "每千字加粗 {density:.0} 处（上限 {}）：挑真正要强调的一两处，其余降为正文",
                    config.max_per_1000
                ),
            ));
        }
    }
    out
}

/// 一行里的加粗片段。先摘掉行内代码，反引号里的 `**` 不是加粗。
fn bold_spans(line: &str) -> Vec<String> {
    let mut plain = String::with_capacity(line.len());
    let mut in_code = false;
    for ch in line.chars() {
        if ch == '`' {
            in_code = !in_code;
            plain.push(' ');
            continue;
        }
        plain.push(if in_code { ' ' } else { ch });
    }

    let mut spans = Vec::new();
    let mut rest = plain.as_str();
    while let Some(open) = rest.find("**") {
        let after = &rest[open + 2..];
        let Some(close) = after.find("**") else {
            break;
        };
        let inner = after[..close].trim();
        if !inner.is_empty() {
            spans.push(inner.to_string());
        }
        rest = &after[close + 2..];
    }
    spans
}
