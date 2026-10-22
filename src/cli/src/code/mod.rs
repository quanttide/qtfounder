//! `qtfounder code`：代码语用检查。
//!
//! 判的不是语法错、不是风格、不是圈复杂度，而是**下一个维护者能不能看懂**。
//! 四层判据见 `docs/user-guide/code.md`；阈值全在 `data/rules/code.toml`。

pub mod lexical;
pub mod model;
pub mod module;
pub mod pragmatic;
pub mod rules;
pub mod syntax;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use clap::Args;
use walkdir::WalkDir;

use model::{Finding, Layer};
use rules::Rules;

/// 代码语用检查：词汇、句法、语篇、语用四层。
#[derive(Args, Debug)]
pub struct CodeArgs {
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
    #[arg(long)]
    pub json: bool,
    /// 规则表路径（缺省用内嵌的默认表）
    #[arg(long)]
    pub rules: Option<PathBuf>,
}

const SKIP_DIRS: [&str; 5] = ["target", ".git", "node_modules", "build", "dist"];

fn wanted_layers(requested: &[String]) -> Vec<Layer> {
    if requested.is_empty() {
        return Layer::ALL.to_vec();
    }
    requested.iter().filter_map(|name| Layer::parse(name)).collect()
}

fn rust_files(root: &PathBuf) -> Vec<PathBuf> {
    if root.is_file() {
        return vec![root.clone()];
    }
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .filter(|path| path.extension().map(|ext| ext == "rs").unwrap_or(false))
        .filter(|path| {
            !path.components().any(|part| {
                part.as_os_str().to_str().map(|text| SKIP_DIRS.contains(&text)).unwrap_or(false)
            })
        })
        .collect()
}

fn display_path(root: &PathBuf, path: &PathBuf) -> String {
    if root.is_dir() {
        path.strip_prefix(root).unwrap_or(path).display().to_string()
    } else {
        path.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_default()
    }
}

/// 跑完整个目录：返回（命中，解析失败的文件）。
fn collect(root: &PathBuf, rules: &Rules, layers: &[Layer]) -> (Vec<Finding>, Vec<(String, String)>) {
    let mut findings = Vec::new();
    let mut broken = Vec::new();
    let mut function_names: Vec<String> = Vec::new();

    for path in rust_files(root) {
        let label = display_path(root, &path);
        let source = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) => {
                broken.push((label, err.to_string()));
                continue;
            }
        };
        let syntax = match syn::parse_file(&source) {
            Ok(parsed) => parsed,
            Err(err) => {
                broken.push((label, err.to_string()));
                continue;
            }
        };

        if layers.contains(&Layer::Lexical) {
            findings.extend(lexical::check(&syntax, &label, rules));
        }
        if layers.contains(&Layer::Syntax) {
            findings.extend(syntax::check(&syntax, &label, rules));
        }
        if layers.contains(&Layer::Structure) {
            findings.extend(module::check(&syntax, &label, rules));
        }
        if layers.contains(&Layer::Pragmatic) {
            findings.extend(pragmatic::check(&syntax, &label, &source, rules));
        }

        for item in &syntax.items {
            match item {
                syn::Item::Fn(function) => function_names.push(function.sig.ident.to_string()),
                syn::Item::Impl(block) => {
                    for inner in &block.items {
                        if let syn::ImplItem::Fn(method) = inner {
                            function_names.push(method.sig.ident.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if layers.contains(&Layer::Lexical) {
        findings.extend(lexical::check_consistency(&function_names, "<repo>", rules));
    }
    (findings, broken)
}

fn render(findings: &[Finding], top: usize) {
    for layer in Layer::ALL {
        let group: Vec<&Finding> = findings.iter().filter(|finding| finding.layer == layer).collect();
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
pub fn run(args: &CodeArgs) -> Result<i32, String> {
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
            let count = findings.iter().filter(|finding| finding.layer == layer).count();
            if count > 0 {
                counts.push(format!("{} {count}", layer.label()));
            }
        }
        let summary = if counts.is_empty() { "无".to_string() } else { counts.join("，") };
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
