//! 各层共用的小工具：汉字计数、摘要截取。

use super::contract::Contract;

pub(super) fn hanzi_count(text: &str) -> usize {
    text.chars()
        .filter(|ch| ('\u{4e00}'..='\u{9fa5}').contains(ch))
        .count()
}

pub(super) fn excerpt_of(body: &str) -> String {
    body.chars().take(70).collect()
}

pub(super) fn in_dir(rel: &str, prefix: &str) -> bool {
    rel == prefix || rel.starts_with(&format!("{}/", prefix.trim_end_matches('/')))
}

/// exclude 条目：带 `/` 的按路径前缀算，不带的按同名文件或目录算（任意层级）。
/// 这样 `CHANGELOG.md`、`node_modules` 这类通用约定写一条就够，不会漏掉嵌套的那几份。
pub(super) fn is_excluded(rel: &str, contract: &Contract) -> bool {
    contract.scope.exclude.iter().any(|skip| {
        if skip.contains('/') {
            in_dir(rel, skip)
        } else {
            rel.split('/').any(|part| part == skip)
        }
    })
}

pub(super) const COMMON_ACRONYMS: [&str; 34] = [
    "AI", "LLM", "JSON", "YAML", "URL", "URI", "HTTP", "HTTPS", "API", "SQL", "HTML", "CSS", "PDF",
    "CSV", "ID", "KB", "MB", "GB", "TB", "UI", "UX", "CPU", "RAM", "GPU", "SDK", "DNS", "TLS",
    "SSL", "UTC", "GMT", "OK", "IP", "OS", "PC",
];
