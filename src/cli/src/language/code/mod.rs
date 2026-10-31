//! `language` 的编程语言侧：判代码的**语言学特点**（命名、句法、语篇、语用四层）。
//!
//! 与自然语言侧（`language` 根下的六层）对称：那边判 prose，这边判 code。判断层只认
//! [`Facts`]，加语言不改这里。解析在 `crate::code::extract`（一份提取器管多语言），
//! 阈值在 `crate::code::rules`（同一份软件工程契约）——本模块只做编排与判断。
//!
//! 四层见 `docs/user-guide/code.md`；它们回答的是「下一个维护者能不能看懂」，
//! 不是「功能对不对」——功能归 `code review`。

pub mod lexical;
pub mod model;
pub mod module;
pub mod pragmatic;
pub mod syntax;

#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::code::extract;
use crate::code::rules::Rules;
use model::{Finding, Layer};

/// 走完整个目录：返回（命中，解析失败的文件）。
///
/// 与自然语言侧的 `scan::scan` 对称——那边扫 markdown，这边扫代码。
pub fn collect(
    root: &Path,
    rules: &Rules,
    layers: &[Layer],
) -> (Vec<Finding>, Vec<(String, String)>) {
    let mut findings = Vec::new();
    let mut broken = Vec::new();
    let mut function_names: Vec<String> = Vec::new();

    for path in source_files(root, &rules.scope.exclude) {
        let label = display_path(root, &path);
        let source = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                broken.push((label, error.to_string()));
                continue;
            }
        };
        // 解析交给语言提取器；判断层只认 Facts，加语言不改这里
        let facts = match extract::extract(&source, &path) {
            Some(Ok(facts)) => facts,
            Some(Err(error)) => {
                broken.push((label, error));
                continue;
            }
            None => continue,
        };

        if layers.contains(&Layer::Lexical) {
            findings.extend(lexical::check(&facts, &label, rules));
        }
        if layers.contains(&Layer::Syntax) {
            findings.extend(syntax::check(&facts, &label, rules));
        }
        if layers.contains(&Layer::Structure) {
            findings.extend(module::check(&facts, &label, rules));
        }
        if layers.contains(&Layer::Pragmatic) {
            findings.extend(pragmatic::check(&facts, &label, rules));
        }
        function_names.extend(facts.function_names());
    }

    if layers.contains(&Layer::Lexical) {
        findings.extend(lexical::check_consistency(&function_names, "<repo>", rules));
    }
    (findings, broken)
}

/// 要检查的代码文件：根是文件就直接用，是目录就递归挑认得的扩展名。
fn source_files(root: &Path, excludes: &[String]) -> Vec<PathBuf> {
    if root.is_file() {
        return vec![root.to_path_buf()];
    }
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .filter(|path| extract::supports(path))
        .filter(|path| {
            !path.components().any(|part| {
                part.as_os_str()
                    .to_str()
                    .map(|text| excludes.iter().any(|excluded| excluded == text))
                    .unwrap_or(false)
            })
        })
        .collect()
}

/// 报告里的路径：根是目录就给相对路径，是文件就给文件名。
fn display_path(root: &Path, path: &Path) -> String {
    if root.is_dir() {
        path.strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string()
    } else {
        path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default()
    }
}

/// 只看某几层的命中：lexical／syntax／structure／pragmatic，可重复。
pub fn wanted_layers(requested: &[String]) -> Vec<Layer> {
    if requested.is_empty() {
        return Layer::ALL.to_vec();
    }
    requested
        .iter()
        .filter_map(|name| Layer::parse(name))
        .collect()
}

/// 目标下有没有认得的代码文件——`language` 据此决定要不要并跑编程语言那四层。
pub fn has_code_files(root: &Path) -> bool {
    if root.is_file() {
        return extract::supports(root);
    }
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .any(|entry| extract::supports(entry.path()))
}
