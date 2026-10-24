//! `qtfounder language`：文字审计六层。
//!
//! 判的不是「有没有语法错误」，而是**这段文字是不是在跟读者说话**。
//! 契约在 `src/language/default-contract.toml`；与 `code` 那份同构——一份契约 = 一类介质的验收标准。

pub mod checks;
mod common;
pub mod contract;
pub mod model;
pub mod scan;
mod sense;
mod syntax;
mod voice;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use clap::Args;

use contract::Contract;
use model::{Finding, Kind, Verdict};

/// 语言检查：落点、自造术语、体系词、数字、句法、语用六层。
#[derive(Args, Debug)]
pub struct LanguageArgs {
    /// 要扫的目录或文件
    #[arg(default_value = ".")]
    pub root: PathBuf,
    /// 扫描范围：default 审全量（跳过 [scope] exclude 里的流水与第三方）；all 连 exclude 也不放过
    #[arg(long, default_value = "default")]
    pub scope: String,
    /// 只看一类：placement／banned／required／numbers／syntax／voice
    #[arg(long)]
    pub kind: Option<String>,
    /// 契约路径（缺省按 src/language/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 用户手写文字的目录，逗号分隔可给多个，用于「这个词你自己也在用」的自查
    /// 缺省按他 2026-10-22 的说法：侧写及以上是手写（初稿、改稿、定稿、成稿），提纲与场景里混着我写的批注
    #[arg(
        long,
        default_value = "assets/fiction/职场言情/2_侧写,assets/fiction/职场言情/3_初稿,assets/fiction/职场言情/4_改稿,assets/fiction/职场言情/5_定稿,assets/fiction/校园言情/3_初稿,assets/fiction/校园言情/4_改稿,assets/fiction/重生言情/3_初稿,assets/fiction/重生言情/4_成稿,assets/fiction/观察站/1_情绪日记,assets/fiction/观察站/2_社会观察,assets/fiction/观察站/3_创业日记,assets/blog/2_draft,assets/blog/3_final"
    )]
    pub pool: PathBuf,
    /// 最多列几个文件
    #[arg(long, default_value_t = 15)]
    pub top: usize,
    /// 顺带跑语义层：把规则挑出的段送模型判读者视角
    #[arg(long)]
    pub sense: bool,
    /// 语义层最多送判几段（控成本）
    #[arg(long, default_value_t = 10)]
    pub limit: usize,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

/// 禁用词也出现在用户手写的文字里，说明这个词他会用，该从词表出库。
/// 基准只认手写的 assets/fiction 与 assets/blog；data/profile 混着我代笔的段落，不作数。
fn own_usage_hint(contract: &Contract, pool: &Path) -> Vec<String> {
    let (hits, chars, files) = own_usage_report(contract, pool);
    let mut out = Vec::new();
    if files == 0 {
        out.push(format!(
            "词表自查跳过：--pool 没读到 .md（当前 {}）",
            pool.display()
        ));
        return out;
    }
    // 报样本量：语料太小的时候「零命中」什么都不说明，不写出来就会被当成通过
    out.push(format!("词表自查：手写语料 {files} 个文件、{chars} 个汉字"));
    if hits.is_empty() {
        out.push("  禁用词一个都没出现".to_string());
    } else {
        out.push(format!("  这些词你也在用，可考虑出库——{}", hits.join("、")));
    }
    out
}

/// 返回（命中的词及其出处，语料汉字数，读到的文件数）。
fn own_usage_report(contract: &Contract, pool: &Path) -> (Vec<String>, usize, usize) {
    if contract.banned.is_empty() {
        return (Vec::new(), 0, 0);
    }
    // 递归读：手写正文按集分目录（assets/fiction/职场言情/…），只扫一层会一个字都读不到。
    // 带文件与行号：同一个词的命中可能落在我的批注句里，只报个数分不出是谁写的。
    let mut bodies: Vec<(String, String)> = Vec::new();
    for dir in pool.display().to_string().split(',') {
        for entry in walkdir::WalkDir::new(dir.trim())
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if path.extension().map(|ext| ext == "md").unwrap_or(false)
                && let Ok(body) = std::fs::read_to_string(path)
            {
                bodies.push((path.display().to_string(), body));
            }
        }
    }
    let chars: usize = bodies
        .iter()
        .map(|(_, body)| {
            body.chars()
                .filter(|c| ('\u{4e00}'..='\u{9fa5}').contains(c))
                .count()
        })
        .sum();
    let mut hits: Vec<String> = contract
        .banned
        .keys()
        .filter_map(|word| first_hit(word, &bodies).map(|at| format!("{word}（{at}）")))
        .collect();
    hits.sort();
    (hits, chars, bodies.len())
}

/// 一个词在手写语料里第一次出现在哪一行。
fn first_hit(word: &str, bodies: &[(String, String)]) -> Option<String> {
    for (path, body) in bodies {
        if let Some(index) = body.find(word) {
            let line = body[..index].matches('\n').count() + 1;
            return Some(format!("{path}:{line}"));
        }
    }
    None
}

/// 语义层：规则挑段，模型判「这段在不在对读者说话」。返回判为「在记操作」的段数。
fn run_sense(findings: &[Finding], limit: usize) -> Result<usize, String> {
    let results = sense::judge_segments(findings, limit)?;
    if results.is_empty() {
        println!("\n规则没有挑出可疑段：语义层用不上");
        return Ok(0);
    }
    let process = results
        .iter()
        .filter(|(_, verdict)| verdict.kind == "process")
        .count();
    println!("\n送判 {} 段；判为「在记操作」{process} 段", results.len());
    for (segment, verdict) in &results {
        let mark = match verdict.kind.as_str() {
            "process" => "操作记录",
            "reader" => "对读者说话",
            other => other,
        };
        println!("\n  [{mark}] {}:{}", segment.path, segment.line);
        println!(
            "      {}",
            segment.text.chars().take(60).collect::<String>()
        );
        println!(
            "      理由：{}",
            verdict.why.chars().take(100).collect::<String>()
        );
        if !verdict.fix.is_empty() {
            println!(
                "      怎么改：{}",
                verdict.fix.chars().take(100).collect::<String>()
            );
        }
    }
    Ok(process)
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

/// 人读的输出：先报审了几个文件，再按文件分组列命中，最后是合计与复核汇总。
fn print_report(findings: &[Finding], scanned: usize, top: usize) {
    if findings.is_empty() {
        println!(
            "审了 {scanned} 个文件，没有命中：这批文字没有自造术语、体系词缺失、数字无据或放错层"
        );
        return;
    }
    let mut files = group_by_file(findings);
    files.sort_by_key(|(_, group)| std::cmp::Reverse(group.len()));
    for (path, group) in files.iter().take(top) {
        println!("\n{path}  {} 处", group.len());
        for finding in group.iter().take(8) {
            println!("  {}", line_of(finding));
            println!("          → {}", finding.advice);
        }
    }
    println!(
        "\n合计 {} 处（{}）；审了 {} 个文件，其中 {} 个有命中",
        findings.len(),
        kind_counts(findings),
        scanned,
        files.len()
    );
    println!("{}", judge_summary(findings));
    println!("报的是嫌疑不是判决：数字与术语逐处读上下文再改。");
}

fn group_by_file(findings: &[Finding]) -> Vec<(&str, Vec<&Finding>)> {
    let mut by_file: HashMap<&str, Vec<&Finding>> = HashMap::new();
    for finding in findings {
        by_file
            .entry(finding.path.as_str())
            .or_default()
            .push(finding);
    }
    by_file.into_iter().collect()
}

/// 一类命中的一行：`[嫌疑／误报·提及] banned L44  摘要`。
fn line_of(finding: &Finding) -> String {
    let verdict = if finding.verdict.is_false_positive() {
        format!("／{}", finding.verdict.label())
    } else {
        String::new()
    };
    format!(
        "[{}{}] {} L{}  {}",
        finding.severity.label(),
        verdict,
        finding.kind.key(),
        finding.line,
        finding.excerpt
    )
}

/// 「banned 10，numbers 3，voice 1」这种按层计数。
fn kind_counts(findings: &[Finding]) -> String {
    let mut counts: Vec<(&str, usize)> = scan::summarize(findings).into_iter().collect();
    counts.sort();
    counts
        .iter()
        .map(|(kind, count)| format!("{kind} {count}"))
        .collect::<Vec<String>>()
        .join("，")
}

/// 复核的汇总：真问题几处、规则误报几处、各是什么形态。
fn judge_summary(findings: &[Finding]) -> String {
    let real = findings
        .iter()
        .filter(|f| !f.verdict.is_false_positive())
        .count();
    let mut lines = vec![format!(
        "复核：真问题 {real} 处，规则误报 {} 处",
        findings.len() - real
    )];
    for (label, count) in [
        ("提及不是使用", Verdict::Mention),
        ("标题并入句子", Verdict::LeadIn),
        ("引用工具输出", Verdict::QuotedOutput),
    ] {
        let n = findings.iter().filter(|f| f.verdict == count).count();
        if n > 0 {
            lines.push(format!("  {label} {n} 处"));
        }
    }
    if findings.iter().any(|f| f.verdict.is_false_positive()) {
        lines.push("  误报仍在清单里，标了「误报·」，别当通过".to_string());
    }
    lines.join("\n")
}

fn render_json(findings: &[Finding], hint: &[String], scanned: usize) {
    let mut parts = Vec::new();
    for finding in findings {
        parts.push(format!(
            "  {{\"kind\":\"{}\",\"severity\":\"{}\",\"verdict\":\"{}\",\"path\":\"{}\",\"line\":{},\"excerpt\":\"{}\",\"advice\":\"{}\"}}",
            finding.kind.key(),
            match finding.severity {
                model::Severity::Error => "error",
                model::Severity::Suspect => "suspect",
            },
            finding.verdict.key(),
            json_escape(&finding.path),
            finding.line,
            json_escape(&finding.excerpt),
            json_escape(&finding.advice),
        ));
    }
    let hint_json: Vec<String> = hint
        .iter()
        .map(|word| format!("\"{}\"", json_escape(word)))
        .collect();
    println!(
        "{{\n  \"scanned_files\": {},\n  \"findings\": [\n{}\n  ],\n  \"banned_in_own_prose\": [{}]\n}}",
        scanned,
        parts.join(",\n"),
        hint_json.join(", ")
    );
}

/// 入口：返回退出码（0 无命中 / 1 有命中 / 2 契约或文件读不了）。
pub fn run_language(args: &LanguageArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    if args.scope != "default" && args.scope != "all" {
        return Err(format!(
            "不认识的扫描范围：{}（只有 default 与 all）",
            args.scope
        ));
    }

    let (mut findings, broken, scanned) = scan::scan(&args.root, &contract, &args.scope);
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
        render_json(&findings, &hint, scanned);
        if args.sense {
            eprintln!("JSON 模式下语义层不并输出：单独跑一次（不带 --json）看判读");
        }
        return Ok(if broken.is_empty() {
            i32::from(!findings.is_empty())
        } else {
            2
        });
    }

    print_report(&findings, scanned, args.top);
    if !hint.is_empty() {
        println!();
        for line in &hint {
            println!("{line}");
        }
    }

    let sense_process = if args.sense {
        run_sense(&findings, args.limit)?
    } else {
        0
    };

    if !broken.is_empty() {
        return Ok(2);
    }
    Ok(i32::from(!findings.is_empty() || sense_process > 0))
}
