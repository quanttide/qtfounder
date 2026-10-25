//! `qtfounder write`：表达审计。
//!
//! 判的不是词法，也不是块形状，而是**这段文字有没有把话讲清楚**。
//! 四层：空话、口号、指路、结构。词与句归 `language`，表格与列表归 `docs`。
//! 契约在 `src/write/default-contract.toml`。

mod checks;
pub mod contract;
pub mod model;
mod scan;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::PathBuf;

use clap::Args;

use checks::Selection;
use contract::Contract;
use model::{Finding, Kind, Severity};

/// 表达审计：空话、口号、指路、结构四层
#[derive(Args, Debug)]
pub struct WriteArgs {
    /// 要扫的目录或文件
    #[arg(default_value = ".")]
    pub root: PathBuf,
    /// 只看一层：empty／slogan／reference／structure／genre／audience／topic
    #[arg(long)]
    pub kind: Option<String>,
    /// 体裁画像；缺省按 front matter 或路径推断
    #[arg(long)]
    pub genre: Option<String>,
    /// 读者画像；缺省 default
    #[arg(long)]
    pub audience: Option<String>,
    /// 主题画像；缺省按 front matter 或路径推断
    #[arg(long)]
    pub topic: Option<String>,
    /// 契约路径（缺省按 src/write/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 最多列几个文件
    #[arg(long, default_value_t = 15)]
    pub top: usize,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
    /// 列出契约里的体裁、读者与主题，不审文件
    #[arg(long)]
    pub profiles: bool,
}

fn line_label(line: Option<usize>) -> String {
    match line {
        Some(line) => format!("L{line}"),
        None => "文件级".to_string(),
    }
}

fn print_report(findings: &[Finding], scanned: usize, top: usize) {
    if findings.is_empty() {
        println!("审了 {scanned} 个文件，没有命中：体裁、读者、主题与句子形态都没有越界");
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
            println!(
                "  [{}] {} {}  {}",
                finding.severity.label(),
                finding.kind.key(),
                line_label(finding.line),
                finding.excerpt
            );
            println!("          → {}", finding.advice);
        }
    }
    let errors = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count();
    println!(
        "\n合计 {} 处（错误 {errors}，嫌疑 {}）",
        findings.len(),
        findings.len() - errors
    );
    println!("报的是表达嫌疑不是判决：读上下文再改。");
}

fn render_json(findings: &[Finding], scanned: usize) {
    let items: Vec<serde_json::Value> = findings
        .iter()
        .map(|finding| {
            serde_json::json!({
                "kind": finding.kind.key(),
                "severity": finding.severity.key(),
                "path": finding.path,
                "line": finding.line,
                "excerpt": finding.excerpt,
                "advice": finding.advice,
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::json!({"scanned_files": scanned, "findings": items})
    );
}

fn print_profiles(contract: &Contract) {
    println!("体裁：");
    for (name, profile) in &contract.genre {
        println!("  {name}  {}", profile.description);
    }
    println!("\n读者：");
    for (name, profile) in &contract.audience {
        println!("  {name}  {}", profile.description);
    }
    println!("\n主题：");
    for (name, profile) in &contract.topic {
        println!("  {name}  {}", profile.description);
    }
}

/// 入口：返回退出码（0 无命中 / 1 有命中 / 2 契约或文件读不了）。
pub fn run_write(args: &WriteArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    if args.profiles {
        print_profiles(&contract);
        return Ok(0);
    }
    if !args.root.exists() {
        return Err(format!("目录或文件不存在：{}", args.root.display()));
    }
    let (mut findings, broken, scanned) = scan::scan(
        &args.root,
        &contract,
        &Selection {
            genre: args.genre.clone(),
            audience: args.audience.clone(),
            topic: args.topic.clone(),
        },
    );
    if let Some(kind_name) = &args.kind {
        let Some(kind) = Kind::parse(kind_name) else {
            return Err(format!(
                "不认识的层：{kind_name}（只有 empty、slogan、reference、structure、genre、audience、topic）"
            ));
        };
        findings.retain(|finding| finding.kind == kind);
    }
    for path in &broken {
        eprintln!("读不了：{path}");
    }

    if args.json {
        render_json(&findings, scanned);
    } else {
        print_report(&findings, scanned, args.top);
    }
    Ok(if broken.is_empty() {
        i32::from(!findings.is_empty())
    } else {
        2
    })
}
