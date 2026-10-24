//! 三层检查：表格、列表、密度。
//!
//! 表格与列表是缺陷规则，逐块定位；密度是兜底，只在缺陷规则没命中时才看整篇。
//! 同一篇已经指出具体位置，再补一条整体判断只会淹没有用信息。

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use super::bold::check_bold;
use super::common::is_excluded;
use super::contract::Contract;
use super::density;
use super::frontmatter::check_frontmatter;
use super::list::check_list;
use super::model::Finding;
use super::rule::check_rule;
use super::table::check_table;

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

/// 一篇文档：先跑表格与列表，没命中再跑密度兜底；加粗与分割线独立判。
pub fn check_file(path: &Path, contract: &Contract, root: Option<&Path>) -> Vec<Finding> {
    let rel = rel_of(path, root);
    let text = fs::read_to_string(path).unwrap_or_default();
    let mut findings = check_frontmatter(&rel, &text, &contract.frontmatter);
    findings.extend(check_table(&rel, &text, contract));
    findings.extend(check_list(&rel, &text, contract));
    if findings.is_empty() {
        let measures = density::measure(&text, contract.density.min_paragraph_chars);
        if let Some(finding) = density::judge(&rel, &measures, &contract.density) {
            findings.push(finding);
        }
    }
    findings.extend(check_bold(&rel, &text, contract));
    findings.extend(check_rule(&rel, &text, contract));
    findings
}

pub fn markdown_files(root: &Path, contract: &Contract) -> Vec<PathBuf> {
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
pub fn scan(root: &Path, contract: &Contract) -> (Vec<Finding>, Vec<String>, usize) {
    let mut findings = Vec::new();
    let mut broken = Vec::new();
    let base = if root.is_dir() { Some(root) } else { None };
    let files = markdown_files(root, contract);
    let scanned = files.len();
    for path in files {
        if fs::read_to_string(&path).is_err() {
            broken.push(rel_of(&path, base));
            continue;
        }
        findings.extend(check_file(&path, contract, base));
    }
    (findings, broken, scanned)
}

/// 密度观测：每篇一行数字，不产生命中。阈值没标定之前，靠它看分布。
pub fn stats(root: &Path, contract: &Contract) -> Vec<(String, density::Measures)> {
    let base = if root.is_dir() { Some(root) } else { None };
    let mut out = Vec::new();
    for path in markdown_files(root, contract) {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let measures = density::measure(&text, contract.density.min_paragraph_chars);
        out.push((rel_of(&path, base), measures));
    }
    out
}

/// 按类计数，给汇总行用。
pub fn summarize(findings: &[Finding]) -> HashMap<&'static str, usize> {
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    for finding in findings {
        *counts.entry(finding.kind.key()).or_insert(0) += 1;
    }
    counts
}
