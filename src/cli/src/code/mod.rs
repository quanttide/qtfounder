//! `qtfounder code`：给写代码的 AI 配的 **reviewer**。
//!
//! 三道，一道比一道靠后：
//!
//! - `code <root>` 语用检查——判下一个维护者能不能看懂，给 file:line 的事实；
//! - `code reflect` 任务后反思——这轮干完，架构里多了什么以前没有的毛病；
//! - `code review` 意图审查——从代码反推写它时的意图，看还对不对得上。
//!
//! 前两道出的是事实与清单，判断归 AI；第三道连骨架都只是形状，反推与排序都归 AI。
//! 这条分工是刻意的：机械层能给的只有锚点，严重度与意图只能对着上下文判。
//! 验收标准与阈值全在 `src/code/default-contract.toml`，四层见 `docs/user-guide/code.md`。

pub mod extract;
pub mod ir;
pub mod lexical;
pub mod model;
pub mod module;
pub mod pragmatic;
pub mod reflect;
pub mod review;
pub mod rules;
pub mod syntax;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use walkdir::WalkDir;

use model::{Finding, Layer};
use rules::Rules;

/// 代码语用检查：词汇、句法、语篇、语用四层。
#[derive(Args, Debug)]
pub struct CodeArgs {
    #[command(subcommand)]
    pub command: Option<CodeCommand>,
    /// 要检查的目录或文件
    #[arg(default_value = ".")]
    pub root: PathBuf,
    /// 只看某一层：lexical／syntax／structure／pragmatic，可重复
    #[arg(long = "layer")]
    pub layer: Vec<String>,
    /// 每类最多报几条（0 表示全报）
    #[arg(long, default_value_t = 0)]
    pub top: usize,
    /// 输出 JSON
    #[arg(long, global = true)]
    pub json: bool,
    /// 规则表路径（缺省用内嵌的默认表）
    #[arg(long, global = true)]
    pub rules: Option<PathBuf>,
}

/// `code` 的子命令；不给子命令就跑缺省的语用检查。
#[derive(Subcommand, Debug)]
pub enum CodeCommand {
    /// 完成任务后的架构反思清单：看什么、为什么算缺陷、长什么样
    Reflect(reflect::ReflectArgs),
    /// 意图审查：从代码反推写它时的意图，出报告骨架与交付纪律
    Review(review::ReviewArgs),
}

// 归档代码与示例代码不审：由契约的 [scope].exclude 定，与语言、文档两份契约对齐。

pub(super) fn wanted_layers(requested: &[String]) -> Vec<Layer> {
    if requested.is_empty() {
        return Layer::ALL.to_vec();
    }
    requested
        .iter()
        .filter_map(|name| Layer::parse(name))
        .collect()
}

fn source_files(root: &PathBuf, excludes: &[String]) -> Vec<PathBuf> {
    if root.is_file() {
        return vec![root.clone()];
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

/// 跑完整个目录：返回（命中，解析失败的文件）。
pub(super) fn collect(
    root: &PathBuf,
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

fn render(findings: &[Finding], top: usize) {
    for layer in Layer::ALL {
        let group: Vec<&Finding> = findings
            .iter()
            .filter(|finding| finding.layer == layer)
            .collect();
        if group.is_empty() {
            continue;
        }
        println!("\n【{}】{} 处", layer.label(), group.len());
        let mut shown: HashMap<&str, usize> = HashMap::new();
        for finding in group {
            let count = shown.entry(finding.kind).or_insert(0);
            if top > 0 && *count >= top {
                continue;
            }
            *count += 1;
            println!("  [{}] {}:{}", finding.kind, finding.path, finding.line);
            println!("      问题：{}", finding.message);
            println!("      依据：违反{}", finding.principle.label());
            println!("      建议：{}", finding.advice);
        }
    }
}

pub(super) fn json_escape(text: &str) -> String {
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

fn render_json(findings: &[Finding]) {
    let mut parts = Vec::new();
    for finding in findings {
        parts.push(format!(
            "  {{\"layer\":\"{}\",\"layer_name\":\"{}\",\"kind\":\"{}\",\"path\":\"{}\",\"line\":{},\"message\":\"{}\",\"principle\":\"{}\",\"advice\":\"{}\"}}",
            finding.layer.key(),
            finding.layer.label(),
            finding.kind,
            json_escape(&finding.path),
            finding.line,
            json_escape(&finding.message),
            finding.principle.label(),
            json_escape(finding.advice),
        ));
    }
    println!("[\n{}\n]", parts.join(",\n"));
}

/// 入口：返回退出码（0 无命中 / 1 有命中 / 2 读不了）。
pub fn run_code(args: &CodeArgs) -> Result<i32, String> {
    if let Some(CodeCommand::Reflect(reflect_args)) = &args.command {
        return reflect::run(reflect_args, args.rules.as_deref(), args.json);
    }
    if let Some(CodeCommand::Review(review_args)) = &args.command {
        return review::run(review_args, args.rules.as_deref(), args.json);
    }
    let rules = Rules::load(args.rules.as_deref())?;
    let layers = wanted_layers(&args.layer);
    if layers.is_empty() {
        return Err(format!("不认识的层：{}", args.layer.join("、")));
    }

    let (findings, broken) = collect(&args.root, &rules, &layers);
    for (path, why) in &broken {
        eprintln!("解析失败：{path} —— {why}");
    }

    if args.json {
        render_json(&findings);
    } else {
        render(&findings, args.top);
        let mut counts: Vec<String> = Vec::new();
        for layer in Layer::ALL {
            let count = findings
                .iter()
                .filter(|finding| finding.layer == layer)
                .count();
            if count > 0 {
                counts.push(format!("{} {count}", layer.label()));
            }
        }
        let summary = if counts.is_empty() {
            "无".to_string()
        } else {
            counts.join("，")
        };
        println!("\n合计 {} 处（{summary}）", findings.len());
        if !findings.is_empty() {
            println!("报的是读者视角的风险，不是错误：逐条读上下文再改。");
        }
    }

    if !broken.is_empty() {
        return Ok(2);
    }
    Ok(if findings.is_empty() { 0 } else { 1 })
}
