//! 各层共用的小工具：字数、摘要、目录判断。

use super::contract::Contract;

/// 去掉空白后的字符数，中英一致——汉字数与字符数在这里不分开算。
pub(super) fn char_count(text: &str) -> usize {
    text.chars().filter(|ch| !ch.is_whitespace()).count()
}

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
pub(super) fn is_excluded(rel: &str, contract: &Contract) -> bool {
    contract.scope.exclude.iter().any(|skip| {
        if skip.contains('/') {
            in_dir(rel, skip)
        } else {
            rel.split('/').any(|part| part == skip)
        }
    })
}
