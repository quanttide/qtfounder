//! `qtfounder docs`：文档结构审计。
//!
//! 判的不是「有没有语法错误」，而是**这段内容该不该是这个形状**。
//! 缺陷规则（表格、列表、加粗、分割线）逐处定位；密度规则看整篇，只在缺陷规则没命中时兜底。
//! 契约在 `src/docs/default-contract.toml`。

mod bold;
mod check;
mod common;
pub mod contract;
pub mod density;
mod list;
pub mod model;
mod rule;
pub mod scan;
mod table;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::PathBuf;

use clap::{Args, Subcommand};

use contract::Contract;
use model::{Finding, Kind, Severity};

/// 文档审计：表格、列表、密度、加粗与分割线五层。
#[derive(Args, Debug)]
pub struct DocsArgs {
    #[command(subcommand)]
    pub command: Option<DocsCommand>,
    /// 要扫的目录或文件
    #[arg(default_value = ".")]
    pub root: PathBuf,
    /// 只看一层：table／list／density／bold／rule
    #[arg(long)]
    pub kind: Option<String>,
    /// 契约路径（缺省按 src/docs/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 最多列几个文件
    #[arg(long, default_value_t = 15)]
    pub top: usize,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
    /// 只打密度数字，不下结论：阈值标定前先看分布
    #[arg(long)]
    pub stats: bool,
}

#[derive(Subcommand, Debug)]
pub enum DocsCommand {
    /// 技能索引与实际目录比对
    Check {
        /// 仓根目录（含 AGENTS.md 与 .agents/skills）
        #[arg(default_value = ".")]
        root: PathBuf,
    },
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

fn line_of(finding: &Finding) -> String {
    format!(
        "[{}] {} L{}  {}",
        finding.severity.label(),
        finding.kind.key(),
        finding.line,
        finding.excerpt
    )
}

fn print_report(findings: &[Finding], scanned: usize, top: usize) {
    if findings.is_empty() {
        println!("审了 {scanned} 个文件，没有命中：表格、列表、密度、加粗与分割线都没有越界");
        return;
    }
    let mut by_file: HashMap<&str, Vec<&Finding>> = HashMap::new();
    for finding in findings {
        by_file
            .entry(finding.path.as_str())
            .or_default()
            .push(finding);
    }
    let mut files: Vec<(&str, Vec<&Finding>)> = by_file.into_iter().collect();
    files.sort_by_key(|(_, group)| std::cmp::Reverse(group.len()));
    for (path, group) in files.iter().take(top) {
        println!("\n{path}  {} 处", group.len());
        for finding in group.iter().take(8) {
            println!("  {}", line_of(finding));
            println!("          → {}", finding.advice);
        }
    }
    let mut counts: Vec<(&str, usize)> = scan::summarize(findings).into_iter().collect();
    counts.sort();
    println!(
        "\n合计 {} 处（{}）；审了 {scanned} 个文件，其中 {} 个有命中",
        findings.len(),
        counts
            .iter()
            .map(|(kind, count)| format!("{kind} {count}"))
            .collect::<Vec<String>>()
            .join("，"),
        files.len()
    );
    println!("报的是嫌疑不是判决：逐处读上下文再改。");
}

fn print_stats(stats: &[(String, density::Measures)], top: usize) {
    println!("\n密度观测（阈值未标定，只看分布）");
    let mut rows: Vec<&(String, density::Measures)> = stats.iter().collect();
    rows.sort_by(|a, b| {
        b.1.block_ratio()
            .partial_cmp(&a.1.block_ratio())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (path, measures) in rows.iter().take(top) {
        println!(
            "  {}  块占比 {:.0}%  无散文率 {:.0}%  结构块 {}  字数 {}",
            path,
            measures.block_ratio() * 100.0,
            measures.run_ratio() * 100.0,
            measures.blocks,
            measures.chars
        );
    }
}

fn render_json(findings: &[Finding], scanned: usize) {
    let parts: Vec<String> = findings
        .iter()
        .map(|finding| {
            format!(
                "  {{\"kind\":\"{}\",\"severity\":\"{}\",\"path\":\"{}\",\"line\":{},\"excerpt\":\"{}\",\"advice\":\"{}\"}}",
                finding.kind.key(),
                match finding.severity {
                    Severity::Error => "error",
                    Severity::Suspect => "suspect",
                },
                json_escape(&finding.path),
                finding.line,
                json_escape(&finding.excerpt),
                json_escape(&finding.advice),
            )
        })
        .collect();
    println!(
        "{{\n  \"scanned_files\": {},\n  \"findings\": [\n{}\n  ]\n}}",
        scanned,
        parts.join(",\n")
    );
}

/// 入口：返回退出码（0 无命中 / 1 有命中 / 2 契约或文件读不了）。
pub fn run_docs(args: &DocsArgs) -> Result<i32, String> {
    if let Some(DocsCommand::Check { root }) = &args.command {
        return check::run_check(root);
    }
    let contract = Contract::load(args.rules.as_deref())?;

    let (mut findings, broken, scanned) = scan::scan(&args.root, &contract);
    if let Some(kind_name) = &args.kind {
        let Some(kind) = Kind::parse(kind_name) else {
            return Err(format!(
                "不认识的层：{kind_name}（只有 table、list、density、bold、rule）"
            ));
        };
        findings.retain(|finding| finding.kind == kind);
    }
    for path in &broken {
        eprintln!("读不了：{path}");
    }

    if args.json {
        render_json(&findings, scanned);
        return Ok(if broken.is_empty() {
            i32::from(!findings.is_empty())
        } else {
            2
        });
    }

    print_report(&findings, scanned, args.top);
    if args.stats {
        print_stats(&scan::stats(&args.root, &contract), args.top);
    }
    Ok(if broken.is_empty() {
        i32::from(!findings.is_empty())
    } else {
        2
    })
}
