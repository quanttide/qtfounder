//! 语用层：这段是不是在跟读者说话。按内部标识符密度与读者指代两个信号判。

use super::common::COMMON_ACRONYMS;
use super::contract::Contract;
use super::model::{Finding, Kind, Severity};
use regex::Regex;
use std::collections::HashSet;
fn is_listing(body: &str, min_listed: usize) -> bool {
    let spans: Vec<&str> = body.split('`').skip(1).step_by(2).collect();
    if spans.len() < min_listed.max(2) {
        return false;
    }
    let with_slash = spans.iter().filter(|span| span.contains('/')).count();
    with_slash * 2 < spans.len()
}

fn internal_ids(body: &str) -> HashSet<String> {
    static PATTERNS: std::sync::OnceLock<Vec<Regex>> = std::sync::OnceLock::new();
    // 注意：Rust 的 regex 不支持前瞻，所以「提交号必须含字母」「路径要带 ASCII」这两条
    // 从正则里挪到下面的代码判——语义等价，别想着换回 look-ahead。
    let patterns = PATTERNS.get_or_init(|| {
        vec![
            Regex::new(r"\b[0-9a-f]{7,40}\b")
                .expect("十六进制正则写坏了：改 internal_ids，再跑 cargo test"),
            Regex::new(r"[\w\-]+\.(?:md|py|rs|go|dart|toml|yaml|yml|json|sh|lock)\b")
                .expect("文件名正则写坏了：改 internal_ids，再跑 cargo test"),
            Regex::new(r"\b[A-Z]{2,}\d*\b")
                .expect("缩写正则写坏了：改 internal_ids，再跑 cargo test"),
            Regex::new(r"[\w.\-]+/[\w./\-]+")
                .expect("路径正则写坏了：改 internal_ids，再跑 cargo test"),
        ]
    });
    let backtick =
        Regex::new(r"`[^`\n]+`").expect("反引号配对正则写坏了：改 internal_ids，再跑 cargo test");
    let chinese_only = Regex::new(r"^[\u{4e00}-\u{9fa5}，。：；、「」（）《》\s]+$")
        .expect("纯中文判定正则写坏了：改 internal_ids，再跑 cargo test");

    // 反引号里如果全是中文，那是术语强调（`正本`、`验收标准`），不是内部代号
    let mut ids: HashSet<String> = HashSet::new();
    let mut kept: Vec<String> = Vec::new();
    for found in backtick.find_iter(body) {
        let raw = found.as_str().trim_matches('`').trim();
        if chinese_only.is_match(raw) {
            kept.push(found.as_str().to_string());
        } else {
            ids.insert(raw.to_string());
            kept.push(" ".to_string());
        }
    }
    let rest = if kept.is_empty() {
        body.to_string()
    } else {
        let mut result = String::new();
        let mut cursor = 0;
        let matches: Vec<_> = backtick.find_iter(body).collect();
        for (index, found) in matches.iter().enumerate() {
            result.push_str(&body[cursor..found.start()]);
            result.push_str(&kept[index]);
            cursor = found.end();
        }
        result.push_str(&body[cursor..]);
        result
    };

    for (index, pattern) in patterns.iter().enumerate() {
        for found in pattern.find_iter(&rest) {
            let item = found.as_str().to_string();
            if COMMON_ACRONYMS.contains(&item.as_str()) {
                continue;
            }
            // 提交号必须含字母 a-f：纯数字串是秒数或数据库 id，不是 sha
            if index == 0 && !item.chars().any(|ch| ch.is_ascii_alphabetic()) {
                continue;
            }
            // 斜杠两侧至少一侧带 ASCII：「一/二」「A/B」不是路径
            if index == 3 && !item.chars().any(|ch| ch.is_ascii_alphanumeric()) {
                continue;
            }
            ids.insert(item);
        }
    }
    // 去掉互为子串的重复：长的留着，短的扔掉，否则同一个东西被算成两三个 ID
    let all: Vec<String> = ids.into_iter().collect();
    all.iter()
        .filter(|item| {
            !all.iter()
                .any(|other| other != *item && other.contains(item.as_str()))
        })
        .cloned()
        .collect()
}

/// 语用层：这段是不是在跟读者说话——内部标识符密度高且一句读者指代都没有。
pub(super) fn check_voice(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let config = &contract.voice;
    if config.max_internal_ids == 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut paragraph: Vec<String> = Vec::new();
    let mut start = 0usize;
    let mut in_code = false;

    let flush = |paragraph: &mut Vec<String>, start: usize, out: &mut Vec<Finding>| {
        if paragraph.is_empty() {
            return;
        }
        let body = paragraph.join(" ");
        let ids = internal_ids(&body);
        let has_reader = config
            .reader_words
            .iter()
            .any(|word| body.contains(word.as_str()));
        // 成串反引号标识符的列举段不算：参数表、JSON 字段、术语清单，读者就是来查这些的。
        // 但「改了 a/b.md、c/d.rs 这几处」是操作记录不是字段表——路径占比高就不算列举段。
        // 目录段同理：三行以上「- [标题](路径)：一句说明」就是导航，不是让人旁观的操作记录。
        let links = paragraph
            .iter()
            .filter(|line| line.starts_with("- ["))
            .count();
        let listed = links >= config.min_listed_ids || is_listing(&body, config.min_listed_ids);
        if ids.len() > config.max_internal_ids && !has_reader && !listed {
            let mut sample: Vec<String> = ids.iter().take(6).cloned().collect();
            sample.sort();
            out.push(Finding::new(
                Kind::Voice,
                Severity::Suspect,
                rel,
                start,
                body.chars().take(70).collect(),
                format!(
                    "这一段有 {} 个内部标识符（{}…）却没有一处对读者说话：要么逐个说明它是什么，要么这段该进日志",
                    ids.len(),
                    sample.join("、")
                ),
            ));
        }
        paragraph.clear();
    };

    for (lineno, raw) in text.lines().enumerate() {
        let trimmed = raw.trim();
        if trimmed.starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code || trimmed.starts_with('|') || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.is_empty() {
            flush(&mut paragraph, start, &mut out);
            continue;
        }
        if paragraph.is_empty() {
            start = lineno + 1;
        }
        paragraph.push(trimmed.to_string());
    }
    flush(&mut paragraph, start, &mut out);
    out
}
