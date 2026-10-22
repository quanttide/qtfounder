//! 各语言的提取器：把源码摊成 [`crate::code::ir::Facts`]。

pub mod python;
pub mod rust_lang;
pub mod specs;
pub mod tree_sitter_extractor;

use std::path::Path;

use crate::code::ir::Facts;

/// 按扩展名选提取器。返回 None 表示这个语言还没接。
pub fn extract(source: &str, path: &Path) -> Option<Result<Facts, String>> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("rs") => Some(rust_lang::extract(source)),
        Some("py") => Some(python::extract(source)),
        Some("go") => Some(tree_sitter_extractor::extract(source, &specs::go())),
        Some("ts" | "mts" | "cts") => Some(tree_sitter_extractor::extract(source, &specs::typescript())),
        Some("dart") => Some(tree_sitter_extractor::extract(source, &specs::dart())),
        _ => None,
    }
}

/// 这个扩展名认不认。
pub fn supports(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("rs" | "py" | "go" | "ts" | "mts" | "cts" | "dart")
    )
}
