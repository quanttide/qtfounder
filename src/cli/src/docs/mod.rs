//! `qtfounder docs`：文档结构审计。
//!
//! 判的不是「有没有语法错误」，而是**这段内容该不该是这个形状**。
//! 缺陷规则（表格、列表、加粗、分割线）逐处定位；密度规则看整篇，只在缺陷规则没命中时兜底。
//! front matter 另查字段名与取值。契约在 `src/docs/default-contract.toml`。
//! `docs format` 是另一路：先给格式状态（数格式元素），再跑同一套审计。

mod bold;
mod check;
mod common;
pub mod contract;
pub mod density;
mod frontmatter;
mod journal;
mod list;
pub mod model;
mod rule;
pub mod scan;
mod status;
mod table;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};

use contract::Contract;
use model::{Finding, Kind, Severity};

/// 文档审计：七层判形状，`format` 数格式元素再审计，`check` 比对技能索引
#[derive(Args, Debug)]
pub struct DocsArgs {
    #[command(subcommand)]
    pub command: Option<DocsCommand>,
    /// 要扫的目录或文件
    #[arg(default_value = ".")]
    pub root: PathBuf,
    /// 只看一层：table／list／density／bold／rule／frontmatter／journal
    #[arg(long, global = true)]
    pub kind: Option<String>,
    /// 契约路径（缺省按 src/docs/default-contract.toml 逐级往上找）
    #[arg(long, global = true)]
    pub rules: Option<PathBuf>,
    /// 最多列几个文件
    #[arg(long, global = true, default_value_t = 15)]
    pub top: usize,
    /// 输出 JSON
    #[arg(long, global = true)]
    pub json: bool,
    /// 只打密度数字，不下结论：阈值标定前先看分布
    #[arg(long, global = true)]
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
    /// 格式状态：数一篇（或一目录）用了多少格式元素，再跑同一套审计
    Format {
        /// 要数的目录或文件
        #[arg(default_value = ".")]
        root: PathBuf,
    },
}

/// `--kind` 只留一层；层名不认识按用不了处理。
fn filter_kind(findings: &mut Vec<Finding>, kind_name: &Option<String>) -> Result<(), String> {
    let Some(name) = kind_name else {
        return Ok(());
    };
    let Some(kind) = Kind::parse(name) else {
        return Err(format!(
            "不认识的层：{name}（只有 table、list、density、bold、rule、frontmatter、journal）"
        ));
    };
    findings.retain(|finding| finding.kind == kind);
    Ok(())
}

/// 0 无命中 / 1 有命中 / 2 用法或环境错；码值取 `core::exit`。
fn exit_code(findings: &[Finding], broken: &[String]) -> i32 {
    if !broken.is_empty() {
        crate::core::exit::USAGE
    } else if findings.is_empty() {
        crate::core::exit::OK
    } else {
        crate::core::exit::FINDINGS
    }
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
        println!(
            "审了 {scanned} 个文件，没有命中：表格、列表、密度、加粗、分割线、front matter 与日志模板都没有越界"
        );
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
    let parts: Vec<String> = findings.iter().map(finding_json).collect();
    println!(
        "{{\n  \"scanned_files\": {},\n  \"findings\": [\n{}\n  ]\n}}",
        scanned,
        parts.join(",\n")
    );
}

fn finding_json(finding: &Finding) -> String {
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
}

/// 入口：返回退出码（0 无命中 / 1 有命中 / 2 用法或环境错 / 3 目标缺席）。
pub fn run_docs(args: &DocsArgs) -> Result<i32, String> {
    match &args.command {
        Some(DocsCommand::Check { root }) => return check::run_check(root),
        Some(DocsCommand::Format { root }) => return run_format(args, root),
        None => {}
    }
    if !args.root.exists() {
        eprintln!("目标不在：{}", args.root.display());
        return Ok(crate::core::exit::NOT_FOUND);
    }
    let contract = Contract::load(args.rules.as_deref())?;

    let (mut findings, broken, scanned) = scan::scan(&args.root, &contract);
    filter_kind(&mut findings, &args.kind)?;
    for path in &broken {
        eprintln!("读不了：{path}");
    }

    if args.json {
        render_json(&findings, scanned);
        return Ok(exit_code(&findings, &broken));
    }

    print_report(&findings, scanned, args.top);
    if args.stats {
        print_stats(&scan::stats(&args.root, &contract), args.top);
    }
    Ok(exit_code(&findings, &broken))
}

/// `docs format`：先给格式状态，再跑同一套审计。
/// 状态看有没有滥用（数字），审计看该改哪一处（命中）；两路用的是同一批文件。
fn run_format(args: &DocsArgs, root: &Path) -> Result<i32, String> {
    if !root.exists() {
        eprintln!("目标不在：{}", root.display());
        return Ok(crate::core::exit::NOT_FOUND);
    }
    let contract = Contract::load(args.rules.as_deref())?;
    let (mut findings, broken, scanned) = scan::scan(root, &contract);
    filter_kind(&mut findings, &args.kind)?;
    for path in &broken {
        eprintln!("读不了：{path}");
    }
    let rows = scan::statuses(root, &contract);

    if args.json {
        render_format_json(&rows, &findings, scanned);
        return Ok(exit_code(&findings, &broken));
    }

    print_status(&rows, args.top);
    print_report(&findings, scanned, args.top);
    if args.stats {
        print_stats(&scan::stats(root, &contract), args.top);
    }
    Ok(exit_code(&findings, &broken))
}

fn level_label(level: usize) -> &'static str {
    match level {
        1 => "一级",
        2 => "二级",
        3 => "三级",
        4 => "四级",
        5 => "五级",
        6 => "六级",
        _ => "无",
    }
}

fn elements_line(status: &status::Status) -> String {
    format!(
        "标题 {} 表格 {} 列表 {} 加粗 {} 分割线 {} 引用 {} 代码块 {}",
        status.headings,
        status.tables,
        status.lists,
        status.bold,
        status.rules,
        status.quotes,
        status.code_blocks
    )
}

fn print_status(rows: &[(String, status::Status)], top: usize) {
    if rows.is_empty() {
        println!("格式状态：没有可数的文件");
        return;
    }
    if let [(path, one)] = rows {
        println!("格式状态：{path}（正文 {} 字）", one.chars);
        println!(
            "  标题 {} 处（一级 {}，最深 {}）",
            one.headings,
            one.h1,
            level_label(one.deepest)
        );
        println!("  表格 {} 张（{} 行）", one.tables, one.table_rows);
        println!("  列表 {} 块（{} 项）", one.lists, one.list_items);
        println!("  加粗 {} 处", one.bold);
        println!("  分割线 {} 条", one.rules);
        println!("  块引用 {} 段", one.quotes);
        println!("  代码块 {} 个", one.code_blocks);
        println!("  格式元素合计 {} 处", one.elements());
        return;
    }

    let mut sorted: Vec<&(String, status::Status)> = rows.iter().collect();
    sorted.sort_by(|a, b| {
        b.1.elements()
            .cmp(&a.1.elements())
            .then_with(|| a.0.cmp(&b.0))
    });
    let mut total = status::Status::default();
    for (_, one) in rows {
        total.chars += one.chars;
        total.headings += one.headings;
        total.h1 += one.h1;
        total.tables += one.tables;
        total.table_rows += one.table_rows;
        total.lists += one.lists;
        total.list_items += one.list_items;
        total.bold += one.bold;
        total.rules += one.rules;
        total.quotes += one.quotes;
        total.code_blocks += one.code_blocks;
    }
    println!(
        "格式状态：{} 个文件，格式元素 {} 处，正文 {} 字（按元素数从多到少，最多列 {top} 个）",
        rows.len(),
        total.elements(),
        total.chars
    );
    for (path, one) in sorted.iter().take(top) {
        println!(
            "  {path}  元素 {}  字 {}  {}",
            one.elements(),
            one.chars,
            elements_line(one)
        );
    }
    if rows.len() > top {
        println!("  （还有 {} 个文件没列）", rows.len() - top);
    }
    println!(
        "  合计  标题 {}（一级 {}） 表格 {} 张（{} 行） 列表 {} 块（{} 项） 加粗 {} 分割线 {} 引用 {} 代码块 {}",
        total.headings,
        total.h1,
        total.tables,
        total.table_rows,
        total.lists,
        total.list_items,
        total.bold,
        total.rules,
        total.quotes,
        total.code_blocks
    );
}

fn render_format_json(rows: &[(String, status::Status)], findings: &[Finding], scanned: usize) {
    let files: Vec<String> = rows
        .iter()
        .map(|(path, one)| {
            format!(
                "    {{\"path\":\"{}\",\"chars\":{},\"headings\":{},\"h1\":{},\"deepest\":{},\"tables\":{},\"table_rows\":{},\"lists\":{},\"list_items\":{},\"bold\":{},\"rules\":{},\"quotes\":{},\"code_blocks\":{},\"elements\":{}}}",
                json_escape(path),
                one.chars,
                one.headings,
                one.h1,
                one.deepest,
                one.tables,
                one.table_rows,
                one.lists,
                one.list_items,
                one.bold,
                one.rules,
                one.quotes,
                one.code_blocks,
                one.elements()
            )
        })
        .collect();
    let hits: Vec<String> = findings.iter().map(finding_json).collect();
    println!(
        "{{\n  \"scanned_files\": {scanned},\n  \"files\": [\n{}\n  ],\n  \"findings\": [\n{}\n  ]\n}}",
        files.join(",\n"),
        hits.join(",\n")
    );
}
