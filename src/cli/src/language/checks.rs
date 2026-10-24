//! 词与数字两层：落点、自造术语、体系词、论据数字。

use super::common::in_dir;
use super::context;
use super::contract::{BannedEntry, Contract};
use super::model::{Finding, Kind, Severity};
use regex::Regex;
pub(super) fn check_placement(rel: &str, contract: &Contract) -> Vec<Finding> {
    let mut out = Vec::new();
    for (bad, why) in &contract.placement.banned_paths {
        if in_dir(rel, bad) {
            out.push(Finding::new(
                Kind::Placement,
                Severity::Error,
                rel,
                0,
                bad.clone(),
                why.clone(),
            ));
        }
    }
    out
}

/// 词汇层：自造术语。每次命中都带替代写法，否则读者不知道换成什么。
///
/// 词在自己的语境里合格：文件声明了 `context`，标了同一语境的词就不报。
pub(super) fn check_banned(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let declared = context::declared(text);
    let in_own_context = |entry: &BannedEntry| {
        declared
            .as_deref()
            .is_some_and(|name| entry.contexts.iter().any(|item| item == name))
    };
    let mut out = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        for (word, entry) in &contract.banned {
            if in_own_context(entry) {
                continue;
            }
            if line.contains(word.as_str()) {
                out.push(Finding::new(
                    Kind::Banned,
                    Severity::Error,
                    rel,
                    lineno + 1,
                    line.trim().chars().take(90).collect(),
                    format!("「{word}」→ {}", entry.advice),
                ));
            }
        }
    }
    out
}

/// 体系词：这类目录的文档里必须出现用户的说法，一次都不出现就是写歪了。
pub(super) fn check_required(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let mut out = Vec::new();
    for (prefix, words) in &contract.required {
        if !in_dir(rel, prefix) {
            continue;
        }
        if words.iter().any(|word| text.contains(word.as_str())) {
            continue;
        }
        out.push(Finding::new(
            Kind::Required,
            Severity::Error,
            rel,
            0,
            format!("{rel} 属 {prefix}"),
            format!(
                "这类目录的文档里没有出现：{}。要么补一段判例结构（现象／错法／正法），要么说明它是做法型内容、该迁 handbook",
                words.join("／")
            ),
        ));
    }
    out
}

/// 数字：嫌疑不是判决——分母与来源都缺才报。
pub(super) fn check_numbers(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let config = &contract.numbers;
    if config.denominator_patterns.is_empty() && config.source_words.is_empty() {
        return Vec::new();
    }
    // 只收「论据数字」：百分比、总数计数。描述用的数字（三四个人、最多十条、第 10 行、
    // 出现 1 次才算类型）不在此列——读者要的是量感，不问出处，要求分母反而啰嗦。
    let evidence_number = Regex::new(r"\d+(?:\.\d+)?\s*%|\d[\d,]*\s*(?:处|次|条|份|行)")
        .expect("论据数字正则写坏了：改 scan.rs 的 evidence_number，再跑 cargo test");
    let denominators: Vec<Regex> = config
        .denominator_patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect();

    let mut out = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        let hits: Vec<&str> = evidence_number
            .find_iter(line)
            .filter(|found| !is_descriptive_number(line, found.start(), found.end()))
            .map(|found| found.as_str())
            .collect();
        if hits.is_empty() {
            continue;
        }
        let sourced = denominators.iter().any(|pattern| pattern.is_match(line))
            || config
                .source_words
                .iter()
                .any(|word| line.contains(word.as_str()));
        if sourced {
            continue;
        }
        out.push(Finding::new(
            Kind::Numbers,
            Severity::Suspect,
            rel,
            lineno + 1,
            line.trim().chars().take(90).collect(),
            format!(
                "「{}」是论据数字，却没有分母也没有出处：补「共 N」「占 N/M」、文件:行、实测日期，否则读者无从复核",
                hits.join("」「")
            ),
        ));
    }
    out
}

/// 描述用的数字：阈值、上限、序数、估算、区间。这些不是论据，不该要求分母。
fn is_descriptive_number(line: &str, start: usize, end: usize) -> bool {
    let before: String = line[..start].chars().rev().take(8).collect();
    let before: String = before.chars().rev().collect();
    // 「第 10 行」中间常有空格，去掉尾空格再比对
    let before = before.trim_end();
    let after: String = line[end..].chars().take(3).collect();
    // 前面的限定词：多字的按包含算（「最多展示 10 条」），单字的按紧邻算（「约 20 处」）
    const LEAD_WORDS: [&str; 8] = ["最多", "至少", "至多", "不超过", "大概", "不到", "只", "才"];
    const LEAD_CHARS: [&str; 7] = ["约", "近", "第", "≥", "≤", "<", ">"];
    // 后面的模糊词与区间：多、来、左右、上下、以上/以下/以内、区间号，以及阈值句式的「只算／才算」
    const TAILS: [&str; 15] = [
        "多", "来", "左右", "上下", "以上", "以下", "以内", "–", "-", "~", "只算", "才算", "就算",
        "即算", "封顶",
    ];
    LEAD_WORDS.iter().any(|lead| before.contains(lead))
        || LEAD_CHARS.iter().any(|lead| before.ends_with(lead))
        || TAILS.iter().any(|tail| after.starts_with(tail))
}
