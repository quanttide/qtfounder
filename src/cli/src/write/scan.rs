//! 扫描：目录或单文件，跳过 exclude，逐篇跑四层检查。

use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use super::checks::{self, Selection};
use super::contract::Contract;
use super::model::Finding;

fn rel_of(path: &Path, root: Option<&Path>) -> String {
    match root {
        Some(root) => path
            .strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string(),
        None => path.display().to_string(),
    }
}

fn in_dir(rel: &str, prefix: &str) -> bool {
    rel == prefix || rel.starts_with(&format!("{}/", prefix.trim_end_matches('/')))
}

fn is_excluded(rel: &str, contract: &Contract) -> bool {
    contract.scope.exclude.iter().any(|skip| {
        if skip.contains('/') {
            in_dir(rel, skip)
        } else {
            rel.split('/').any(|part| part == skip)
        }
    })
}

fn markdown_files(root: &Path, contract: &Contract) -> Vec<PathBuf> {
    if root.is_file() {
        return vec![root.to_path_buf()];
    }
    let mut out: Vec<PathBuf> = WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .filter(|path| path.extension().map(|ext| ext == "md").unwrap_or(false))
        .filter(|path| !is_excluded(&rel_of(path, Some(root)), contract))
        .collect();
    out.sort();
    out
}

/// 全量扫描：返回（命中，读不了的文件，审了几个文件）。
pub fn scan(
    root: &Path,
    contract: &Contract,
    selection: &Selection,
) -> (Vec<Finding>, Vec<String>, usize) {
    let base = if root.is_dir() { Some(root) } else { None };
    let files = markdown_files(root, contract);
    let scanned = files.len();
    let mut findings = Vec::new();
    let mut broken = Vec::new();
    for path in files {
        let Ok(text) = fs::read_to_string(&path) else {
            broken.push(rel_of(&path, base));
            continue;
        };
        let rel = rel_of(&path, base);
        findings.extend(checks::check_text(&rel, &text, contract, selection));
    }
    (findings, broken, scanned)
}
