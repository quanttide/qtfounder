//! `qtfounder language`：文字审计七层。
//!
//! 判的不是「有没有语法错误」，而是**这段文字是不是在跟读者说话**。
//! 契约在 `src/language/default-contract.toml`；与 `code` 那份同构——一份契约 = 一类介质的验收标准。

pub mod contract;
pub mod model;
pub mod scan;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::PathBuf;

use clap::Args;

use contract::Contract;
use model::{Finding, Kind};

/// 语言检查：落点、自造术语、体系词、数字、句法、表格、语用七层。
#[derive(Args, Debug)]
pub struct LanguageArgs {
    /// 要扫的目录或文件
    #[arg(default_value = ".")]
    pub root: PathBuf,
    /// 扫描范围：default 只审规范文档，all 连流水与报告一起
    #[arg(long, default_value = "default")]
    pub scope: String,
    /// 只看一类：placement／banned／required／numbers／syntax／table／voice
    #[arg(long)]
    pub kind: Option<String>,
    /// 契约路径（缺省按 src/language/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 用户文风基准目录，用于「这个词你自己也在用」的自查（缺省 data/profile）
    #[arg(long, default_value = "data/profile")]
    pub pool: PathBuf,
    /// 最多列几个文件
    #[arg(long, default_value_t = 15)]
    pub top: usize,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

/// 禁用词也出现在用户自己的文风基准里，说明这个词他会用，该从词表出库。
fn own_usage_hint(contract: &Contract, pool: &PathBuf) -> Vec<String> {
    if !pool.is_dir() || contract.banned.is_empty() {
        return Vec::new();
    }
    let mut text = String::new();
    if let Ok(entries) = std::fs::read_dir(pool) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|ext| ext == "md").unwrap_or(false)
                && let Ok(body) = std::fs::read_to_string(&path)
            {
                text.push_str(&body);
                text.push('\n');
            }
        }
    }
    let mut hits: Vec<String> = contract
        .banned
        .keys()
        .filter(|word| text.contains(word.as_str()))
        .cloned()
        .collect();
    hits.sort();
    hits
}

fn json_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

fn render_json(findings: &[Finding], hint: &[String]) {
    let mut parts = Vec::new();
    for finding in findings {
        parts.push(format!(
            "  {{\"kind\":\"{}\",\"severity\":\"{}\",\"path\":\"{}\",\"line\":{},\"excerpt\":\"{}\",\"advice\":\"{}\"}}",
            finding.kind.key(),
            match finding.severity {
                model::Severity::Error => "error",
                model::Severity::Suspect => "suspect",
            },
            json_escape(&finding.path),
            finding.line,
            json_escape(&finding.excerpt),
            json_escape(&finding.advice),
        ));
    }
    let hint_json: Vec<String> = hint.iter().map(|word| format!("\"{}\"", json_escape(word))).collect();
    println!(
        "{{\n  \"findings\": [\n{}\n  ],\n  \"banned_in_own_prose\": [{}]\n}}",
        parts.join(",\n"),
        hint_json.join(", ")
    );
}

/// 入口：返回退出码（0 无命中 / 1 有命中 / 2 契约或文件读不了）。
pub fn run(args: &LanguageArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    if args.scope != "default" && args.scope != "all" {
        return Err(format!("不认识的扫描范围：{}（只有 default 与 all）", args.scope));
    }

    let (mut findings, broken) = scan::scan(&args.root, &contract, &args.scope);
    if let Some(kind_name) = &args.kind {
        let Some(kind) = Kind::parse(kind_name) else {
            return Err(format!("不认识的层：{kind_name}"));
        };
        findings.retain(|finding| finding.kind == kind);
    }
    for path in &broken {
        eprintln!("读不了：{path}");
    }

    let hint = own_usage_hint(&contract, &args.pool);
    if args.json {
        render_json(&findings, &hint);
        return Ok(if broken.is_empty() { i32::from(!findings.is_empty()) } else { 2 });
    }

    if findings.is_empty() {
        println!("没有命中：这批文字没有自造术语、体系词缺失、数字无据或落点违规");
    } else {
        let mut by_file: HashMap<&str, Vec<&Finding>> = HashMap::new();
        for finding in &findings {
            by_file.entry(finding.path.as_str()).or_default().push(finding);
        }
        let mut files: Vec<(&str, Vec<&Finding>)> = by_file.into_iter().collect();
        files.sort_by_key(|(_, group)| std::cmp::Reverse(group.len()));
        for (path, group) in files.iter().take(args.top) {
            println!("\n{path}  {} 处", group.len());
            for finding in group.iter().take(8) {
                println!(
                    "  [{}] {} L{}  {}",
                    finding.severity.label(),
                    finding.kind.key(),
                    finding.line,
                    finding.excerpt
                );
                println!("          → {}", finding.advice);
            }
        }
        let summary = scan::summarize(&findings);
        let mut counts: Vec<(&str, usize)> = summary.iter().map(|(kind, count)| (*kind, *count)).collect();
        counts.sort();
        let total: Vec<String> = counts.iter().map(|(kind, count)| format!("{kind} {count}")).collect();
        println!("\n合计 {} 处（{}）；涉及 {} 个文件", findings.len(), total.join("，"), files.len());
        println!("报的是嫌疑不是判决：数字与术语逐处读上下文再改。");
    }
    if !hint.is_empty() {
        println!(
            "\n词表自查：这些禁用词你自己的文风里也在用（{}），可考虑出库——{}",
            args.pool.display(),
            hint.join("、")
        );
    }

    if !broken.is_empty() {
        return Ok(2);
    }
    Ok(i32::from(!findings.is_empty()))
}
