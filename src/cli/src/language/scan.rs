//! 七层检查：落点、自造术语、体系词、数字、句法、表格、语用。
//!
//! 验收标准只有能机判的才进来。逻辑跳跃、这句话是谁写的、这篇该放哪层——判不了，不进这里；
//! 误报多了验收标准就被忽略。

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;

use super::checks::{check_banned, check_numbers, check_placement, check_required};
use super::common::{in_dir, is_excluded};
use super::contract::Contract;
use super::model::{Finding, Kind, Verdict};
use super::syntax::check_syntax;
use super::voice::check_voice;

static QUOTED_OUTPUT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"合计\s*\d+\s*处").expect("引用输出正则写坏了：改 QUOTED_OUTPUT，再跑 cargo test")
});

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

/// 落点：文件放错了层。
pub fn check_file(path: &Path, contract: &Contract, root: Option<&Path>) -> Vec<Finding> {
    let rel = rel_of(path, root);
    let text = fs::read_to_string(path).unwrap_or_default();
    let mut findings = check_placement(&rel, contract);
    findings.extend(check_banned(&rel, &text, contract));
    findings.extend(check_required(&rel, &text, contract));
    findings.extend(check_numbers(&rel, &text, contract));
    findings.extend(check_syntax(&rel, &text, contract));
    findings.extend(check_voice(&rel, &text, contract));
    let lines: Vec<&str> = text.lines().collect();
    for finding in &mut findings {
        let line = lines
            .get(finding.line.saturating_sub(1))
            .copied()
            .unwrap_or("");
        judge(finding, line);
    }
    findings
}

/// 复核层：命中的这一条，是规则本身导致的误报，还是要改的真问题。
/// 只判机判得了的三种形态；判不了的不在这里假装判过。
fn judge(finding: &mut Finding, line: &str) {
    let verdict = match finding.kind {
        // 提及不是使用：词被引号包着讨论，比如「判据」→ 验收标准 这种段落
        Kind::Banned if quoted_word(&finding.advice).is_some_and(|word| is_quoted(line, word)) => {
            Verdict::Mention
        }
        // 引用工具自己的输出：合计 64 处（词汇层 9…）
        Kind::Numbers if QUOTED_OUTPUT.is_match(line) => Verdict::QuotedOutput,
        // 切分错：加粗小标签被并进了句子，句首的「**…**：」不是正文
        Kind::Syntax if line.trim_start().starts_with("**") && line.contains("**：") => {
            Verdict::LeadIn
        }
        _ => Verdict::Real,
    };
    finding.mark(verdict);
}

/// advice 形如「「判据」→ 验收标准」，取出被引的那个词。
fn quoted_word(advice: &str) -> Option<&str> {
    let rest = advice.strip_prefix('「')?;
    let end = rest.find('」')?;
    Some(&rest[..end])
}

/// 词在行里只当「被引的东西」出现：包在「」或反引号里，或作为反引号内术语的一部分
/// （`判定层` 这种组件名，词本身没被单独拿来用）。
fn is_quoted(line: &str, word: &str) -> bool {
    if line.contains(&format!("「{word}」")) || line.contains(&format!("`{word}`")) {
        return true;
    }
    let mut rest = line;
    while let Some(open) = rest.find('`') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('`') else { break };
        if after[..close].contains(word) {
            return true;
        }
        rest = &after[close + 1..];
    }
    false
}

/// 扫描范围：`default` 审全量但跳过 `[scope] exclude`（流水、第三方依赖）；`all` 连 exclude 也不放过。
/// `include_default` 为空 = 不按目录挑；仓自己的契约可以填前缀把范围收窄。
pub fn markdown_files(root: &Path, contract: &Contract, scope: &str) -> Vec<PathBuf> {
    if root.is_file() {
        return vec![root.to_path_buf()];
    }
    let includes = &contract.scope.include_default;
    let narrow = !includes.is_empty();

    let mut out: Vec<PathBuf> = WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .filter(|path| path.extension().map(|ext| ext == "md").unwrap_or(false))
        .filter(|path| {
            if scope == "all" {
                return true;
            }
            !is_excluded(&rel_of(path, Some(root)), contract)
        })
        .filter(|path| {
            // 空 include 就是不挑目录：审什么由 root 决定，不再由工具内嵌的目录名单决定
            if scope == "all" || !narrow {
                return true;
            }
            let rel = rel_of(path, Some(root));
            // root 本身可能在深层（如 docs/handbook）：祖先目录里出现过 include 前缀就算在范围内
            let ancestor_hit = root
                .components()
                .filter_map(|part| part.as_os_str().to_str())
                .any(|part| {
                    includes
                        .iter()
                        .any(|prefix| !prefix.is_empty() && part == prefix.trim_end_matches('/'))
                });
            if ancestor_hit {
                return true;
            }
            includes
                .iter()
                .any(|prefix| !prefix.is_empty() && in_dir(&rel, prefix))
        })
        .collect();
    out.sort();
    out
}

/// 全量扫描：返回（命中，读不了的文件，审了几个文件）。
pub fn scan(root: &Path, contract: &Contract, scope: &str) -> (Vec<Finding>, Vec<String>, usize) {
    let mut findings = Vec::new();
    let mut broken = Vec::new();
    let base = if root.is_dir() { Some(root) } else { None };
    let files = markdown_files(root, contract, scope);
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

/// 按类计数，给汇总行用。
pub fn summarize(findings: &[Finding]) -> HashMap<&'static str, usize> {
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    for finding in findings {
        *counts.entry(finding.kind.key()).or_insert(0) += 1;
    }
    counts
}
