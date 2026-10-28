//! `qtfounder design`：交互设计评判的数字面。
//!
//! 十二条评判原则（见 design-review Skill）里，机器只判得了两条的数字面：
//! 第九条的字号（中文微排版）与第五条的主按钮（主行动唯一）。`design review`
//! 扫 HTML 交付物：`font-size` 低于 12px 报缺陷、12–14px 报嫌疑（分不出正文
//! 与辅助），class 含 `primary` 的按钮一个文件多于一个报嫌疑（多屏合订本可
//! 豁免，归逐屏自查）。其余十条判不了，归 Skill 里的走查与自评分——
//! 数字先看，结论再读。
//!
//! `design principles` 打印十二条全文。正文不在这个模块里：随二进制嵌入的
//! design-review Skill 只有一份，命令只是把它摊开。

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use regex::Regex;
use walkdir::WalkDir;

use crate::agent::registry;

/// 第九条的两条线：辅助文字 ≥ 12px，正文 ≥ 14px；更小一律是缺陷。
const MIN_AUX_PX: f64 = 12.0;
const MIN_BODY_PX: f64 = 14.0;
/// rem 的根字号：浏览器缺省 16px。没解析 `<html>` 的自定义根字号，先按缺省算。
const ROOT_PX: f64 = 16.0;
/// 走查时跳过的目录。
const SKIP_DIRS: [&str; 4] = [".git", "node_modules", "target", "_build"];

/// `design` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum DesignCommand {
    /// 走查：数 HTML 交付物的字号与主按钮，再报命中
    Review {
        /// 要走查的目录或 HTML 文件
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    /// 打印十二条交互设计评判原则全文
    Principles,
}

/// 交互设计入口。
#[derive(Args, Debug)]
pub struct DesignArgs {
    #[command(subcommand)]
    pub command: DesignCommand,
    /// 输出 JSON
    #[arg(long, global = true)]
    pub json: bool,
    /// 目录模式最多列几个文件
    #[arg(long, global = true, default_value_t = 15)]
    pub top: usize,
}

/// 一处命中。报的是嫌疑不是判决：逐处读上下文再改。
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub path: String,
    pub line: usize,
    pub severity: Severity,
    /// `font`（第九条）或 `primary`（第五条）
    pub kind: &'static str,
    pub excerpt: String,
    pub advice: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// 违反原则：更小的中文文字一律视为缺陷
    Error,
    /// 嫌疑：分不出正文与辅助，也分不出单屏与多屏合订本
    Suspect,
}

impl Severity {
    fn label(self) -> &'static str {
        match self {
            Severity::Error => "缺陷",
            Severity::Suspect => "嫌疑",
        }
    }
}

/// 一篇交付物的数字面。全零表示一个 `<style>` 也没有的裸 HTML。
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Status {
    /// `font-size` 声明总数（px + rem + em）
    pub fonts: usize,
    pub px: usize,
    pub rem: usize,
    /// em 相对父级，换算不了，只计数
    pub em: usize,
    /// 可换算声明里的最小字号（px 计）；一处都换算不出来时为空
    pub min_px: Option<f64>,
    /// class 含 primary 的按钮数
    pub primaries: usize,
}

impl Status {
    fn merge(&mut self, other: &Status) {
        self.fonts += other.fonts;
        self.px += other.px;
        self.rem += other.rem;
        self.em += other.em;
        self.min_px = match (self.min_px, other.min_px) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, b) => b,
        };
        self.primaries += other.primaries;
    }
}

/// 一篇交付物：先给数字，再给命中。
pub fn check(rel: &str, text: &str) -> (Status, Vec<Finding>) {
    let font =
        Regex::new(r"(?i)font-size\s*:\s*([0-9.]+)\s*(px|rem|em)\b").expect("字号正则写不对");
    let button = Regex::new(r#"(?is)<button[^>]*class\s*=\s*["'][^"']*\bprimary\b"#)
        .expect("主按钮正则写不对");

    let mut status = Status::default();
    let mut findings = Vec::new();

    for captures in font.captures_iter(text) {
        let whole = captures.get(0).expect("整串在");
        let value: f64 = captures[1].parse().unwrap_or(0.0);
        let unit = captures[2].to_ascii_lowercase();
        let px = match unit.as_str() {
            "px" => {
                status.px += 1;
                Some(value)
            }
            "rem" => {
                status.rem += 1;
                Some(value * ROOT_PX)
            }
            _ => {
                status.em += 1;
                None
            }
        };
        status.fonts += 1;
        let line = line_of(text, whole.start());
        let excerpt = truncate(whole.as_str(), 60);
        let Some(px) = px else {
            continue;
        };
        status.min_px = Some(match status.min_px {
            Some(current) => current.min(px),
            None => px,
        });
        if px < MIN_AUX_PX {
            findings.push(Finding {
                path: rel.to_string(),
                line,
                severity: Severity::Error,
                kind: "font",
                excerpt,
                advice: "第九条：更小的中文文字一律视为缺陷；辅助文字 ≥ 12px，正文 ≥ 14px"
                    .to_string(),
            });
        } else if px < MIN_BODY_PX {
            findings.push(Finding {
                path: rel.to_string(),
                line,
                severity: Severity::Suspect,
                kind: "font",
                excerpt,
                advice: "第九条：12–14px 只够辅助文字；是正文就得抬到 ≥ 14px".to_string(),
            });
        }
    }

    for whole in button.find_iter(text) {
        status.primaries += 1;
        if status.primaries == 1 {
            continue;
        }
        // 一个文件多于一个主按钮只报一次：多屏合订本一屏一个主按钮是合规的，
        // 逐屏自查归人；每个按钮都报一遍只是噪音。
        if status.primaries == 2 {
            findings.push(Finding {
                path: rel.to_string(),
                line: line_of(text, whole.start()),
                severity: Severity::Suspect,
                kind: "primary",
                excerpt: truncate(whole.as_str(), 60),
                advice: format!(
                    "第五条：主行动唯一；本文件 class 含 primary 的按钮共 {} 个——多屏合订本可豁免，逐屏自查",
                    count_matches(&button, text)
                ),
            });
        }
    }

    (status, findings)
}

fn count_matches(pattern: &Regex, text: &str) -> usize {
    pattern.find_iter(text).count()
}

fn line_of(text: &str, offset: usize) -> usize {
    text[..offset].lines().count().max(1)
}

fn truncate(text: &str, max: usize) -> String {
    let mut out = String::new();
    for (index, ch) in text.chars().enumerate() {
        if index >= max {
            out.push('…');
            break;
        }
        out.push(ch);
    }
    out
}

/// 要走查的 HTML 文件：给文件就走查它，给目录就递归找 `.html`。
fn html_files(root: &Path) -> Vec<PathBuf> {
    if root.is_file() {
        return vec![root.to_path_buf()];
    }
    let mut out: Vec<PathBuf> = WalkDir::new(root)
        .into_iter()
        // filter_entry 才能拦住下钻；普通 filter 只过滤条目，目录里的文件照样扫出来
        .filter_entry(|entry| {
            !(entry.file_type().is_dir()
                && SKIP_DIRS.contains(&entry.file_name().to_string_lossy().as_ref()))
        })
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .filter(|path| path.extension().map(|ext| ext == "html").unwrap_or(false))
        .collect();
    out.sort();
    out
}

fn rel_of(path: &Path, base: Option<&Path>) -> String {
    match base {
        Some(base) => path
            .strip_prefix(base)
            .unwrap_or(path)
            .display()
            .to_string(),
        None => path.display().to_string(),
    }
}

pub fn run_design(args: &DesignArgs) -> Result<i32, String> {
    match &args.command {
        DesignCommand::Review { root } => run_review(args, root),
        DesignCommand::Principles => run_principles(),
    }
}

/// `design principles`：正文在嵌入的 design-review Skill 里，这里只摊开。
fn run_principles() -> Result<i32, String> {
    let Some(skill) = registry::find("design-review") else {
        return Err("这版二进制没嵌 design-review Skill，打印不了原则全文".to_string());
    };
    let body = skill
        .auxiliary
        .iter()
        .find(|(path, _)| *path == "references/principles.md")
        .map(|(_, text)| *text)
        .unwrap_or(skill.skill_md);
    print!("{body}");
    Ok(0)
}

/// `design review`：0 没命中 / 1 有命中 / 2 有文件读不了。
fn run_review(args: &DesignArgs, root: &Path) -> Result<i32, String> {
    let base = if root.is_dir() { Some(root) } else { None };
    let files = html_files(root);
    if files.is_empty() {
        println!("设计走查：没有 HTML 文件");
        return Ok(0);
    }

    let mut rows: Vec<(String, Status, Vec<Finding>)> = Vec::new();
    let mut broken = Vec::new();
    for path in &files {
        let rel = rel_of(path, base);
        let Ok(text) = std::fs::read_to_string(path) else {
            broken.push(rel);
            continue;
        };
        let (status, findings) = check(&rel, &text);
        rows.push((rel, status, findings));
    }
    for path in &broken {
        eprintln!("读不了：{path}");
    }

    if args.json {
        render_json(&rows);
        return Ok(exit_code(&rows, &broken));
    }

    match rows.as_slice() {
        [(rel, status, findings)] => {
            println!("设计走查：{rel}");
            println!(
                "  font-size 声明 {} 处（px {}，rem {}，em {}），{}",
                status.fonts,
                status.px,
                status.rem,
                status.em,
                match status.min_px {
                    Some(px) => format!("最小 {px:.1}px（rem 按 16px 根换算）"),
                    None => "换算不出最小字号".to_string(),
                }
            );
            println!("  class 含 primary 的按钮 {} 个", status.primaries);
            print_findings(findings);
        }
        many => {
            print_status_rows(many, args.top);
            let all: Vec<Finding> = many
                .iter()
                .flat_map(|(_, _, findings)| findings.iter().cloned())
                .collect();
            print_findings(&all);
        }
    }
    Ok(exit_code(&rows, &broken))
}

fn exit_code(rows: &[(String, Status, Vec<Finding>)], broken: &[String]) -> i32 {
    if !broken.is_empty() {
        return 2;
    }
    i32::from(rows.iter().any(|(_, _, findings)| !findings.is_empty()))
}

fn print_status_rows(rows: &[(String, Status, Vec<Finding>)], top: usize) {
    let mut sorted: Vec<&(String, Status, Vec<Finding>)> = rows.iter().collect();
    sorted.sort_by(|a, b| b.2.len().cmp(&a.2.len()).then_with(|| a.0.cmp(&b.0)));
    let mut total = Status::default();
    let mut hits = 0usize;
    for (_, status, findings) in rows {
        total.merge(status);
        hits += findings.len();
    }
    println!(
        "设计走查：{} 个文件，font-size 声明 {} 处（最小 {}），primary 按钮 {} 个，命中 {} 处（按命中数从多到少，最多列 {top} 个）",
        rows.len(),
        total.fonts,
        match total.min_px {
            Some(px) => format!("{px:.1}px"),
            None => "—".to_string(),
        },
        total.primaries,
        hits
    );
    for (rel, status, findings) in sorted.iter().take(top) {
        println!(
            "  {rel}  字号 {} 处 最小 {}  primary {} 个  命中 {}",
            status.fonts,
            match status.min_px {
                Some(px) => format!("{px:.1}px"),
                None => "—".to_string(),
            },
            status.primaries,
            findings.len()
        );
    }
    if rows.len() > top {
        println!("  （还有 {} 个文件没列）", rows.len() - top);
    }
}

fn print_findings(findings: &[Finding]) {
    if findings.is_empty() {
        println!("没有命中：字号与主按钮都在线上");
        return;
    }
    println!("命中 {} 处：", findings.len());
    for finding in findings {
        println!(
            "  [{}] {} L{}  {}",
            finding.severity.label(),
            finding.kind,
            finding.line,
            finding.excerpt
        );
        println!("          → {}", finding.advice);
    }
    println!("报的是嫌疑不是判决：逐处读上下文再改。");
}

fn render_json(rows: &[(String, Status, Vec<Finding>)]) {
    let files: Vec<String> = rows
        .iter()
        .map(|(rel, status, _)| {
            format!(
                "    {{\"path\":\"{}\",\"fonts\":{},\"px\":{},\"rem\":{},\"em\":{},\"min_px\":{},\"primaries\":{}}}",
                json_escape(rel),
                status.fonts,
                status.px,
                status.rem,
                status.em,
                match status.min_px {
                    Some(px) => format!("{px:.2}"),
                    None => "null".to_string(),
                },
                status.primaries
            )
        })
        .collect();
    let hits: Vec<String> = rows
        .iter()
        .flat_map(|(_, _, findings)| findings.iter())
        .map(|finding| {
            format!(
                "    {{\"kind\":\"{}\",\"severity\":\"{}\",\"path\":\"{}\",\"line\":{},\"excerpt\":\"{}\",\"advice\":\"{}\"}}",
                finding.kind,
                match finding.severity {
                    Severity::Error => "error",
                    Severity::Suspect => "suspect",
                },
                json_escape(&finding.path),
                finding.line,
                json_escape(&finding.excerpt),
                json_escape(&finding.advice)
            )
        })
        .collect();
    println!(
        "{{\n  \"files\": [\n{}\n  ],\n  \"findings\": [\n{}\n  ]\n}}",
        files.join(",\n"),
        hits.join(",\n")
    );
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

#[cfg(test)]
mod tests;
