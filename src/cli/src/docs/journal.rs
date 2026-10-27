//! 日志模板：一天一个文件，标题是日期。
//!
//! 日志是事件型资产，可以有固定模板；洞察这类陈述型不设模板，章节随论证走。
//! 这一层只查文件名与标题是否对得上，正文按时间写，不查句子。

use super::contract::Journal;
use super::model::{Finding, Kind, Severity};

/// 日志仓里的说明文件不是日志正文，不按模板查。
const NON_ENTRY: [&str; 4] = ["README.md", "CONTRIBUTING.md", "CHANGELOG.md", "index.md"];

/// 路径里出现契约配置的片段，就按日志处理。
pub fn is_journal(path: &str, journal: &Journal) -> bool {
    if !journal.enabled {
        return false;
    }
    let name = path.rsplit('/').next().unwrap_or(path);
    if NON_ENTRY.iter().any(|skip| name.eq_ignore_ascii_case(skip)) {
        return false;
    }
    journal.paths.iter().any(|needle| {
        let needle = needle.trim_matches('/');
        !needle.is_empty() && path.contains(needle)
    })
}

/// 文件名里的日期：只认 `YYYY-MM-DD.md`。
pub fn date_of(path: &str) -> Option<&str> {
    let name = path.rsplit('/').next()?;
    let stem = name.strip_suffix(".md")?;
    let bytes = stem.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let digits = [0, 1, 2, 3, 5, 6, 8, 9];
    if !digits.iter().all(|&at| bytes[at].is_ascii_digit()) {
        return None;
    }
    Some(stem)
}

pub fn check_journal(path: &str, text: &str, journal: &Journal) -> Vec<Finding> {
    let Some(date) = date_of(path) else {
        return vec![Finding::new(
            Kind::Journal,
            Severity::Suspect,
            path,
            1,
            path.rsplit('/').next().unwrap_or(path).to_string(),
            "日志文件名用 `YYYY-MM-DD.md`".to_string(),
        )];
    };
    if !journal.title_must_be_date {
        return Vec::new();
    }

    let Some((line_no, line)) = text
        .lines()
        .enumerate()
        .find(|(_, line)| !line.trim().is_empty())
    else {
        return Vec::new();
    };
    let first = line.trim();
    let expected = format!("# {date}");
    if first.starts_with("# ") && first != expected {
        return vec![Finding::new(
            Kind::Journal,
            Severity::Suspect,
            path,
            line_no + 1,
            first.to_string(),
            format!("日志标题写成 `{expected}`，与文件名一致；不写标题也可以"),
        )];
    }
    Vec::new()
}
