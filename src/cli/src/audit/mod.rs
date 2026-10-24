//! `qtfounder audit`：文字体检。
//!
//! 把 `language` 与 `docs` 两套审计的结果合起来，按「一个词、一条规则」归堆，
//! 每期只报一处最值得改的：改一次能带走很多处。完整清单不铺开，改完这一处再出下一期。
//!
//! 排序上术语优先。格式小事命中再多，也只是小事；读者要先把词翻译一遍才读得懂的，
//! 才是先该动的那一处。

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use clap::Args;

use crate::docs::contract::Contract as DocsContract;
use crate::docs::model::Severity as DocsSeverity;
use crate::docs::scan::scan as scan_docs;
use crate::language::contract::Contract as LanguageContract;
use crate::language::model::Severity as LanguageSeverity;
use crate::language::scan::scan as scan_language;

/// 文字体检：合并两套审计，每期只说一处。
#[derive(Args, Debug)]
pub struct AuditArgs {
    /// 要扫的目录，给多个就合并成一份报告
    #[arg(default_value = ".")]
    pub roots: Vec<PathBuf>,
    /// 语言契约路径（缺省按 src/language/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub language_rules: Option<PathBuf>,
    /// 文档契约路径（缺省按 src/docs/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub docs_rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

/// 一处问题按「层 + 改法」归堆后的样子：改一次能带走几处、散在几篇里。
#[derive(Default)]
struct Group {
    count: usize,
    files: HashSet<String>,
}

impl Group {
    fn add(&mut self, path: &str) {
        self.count += 1;
        self.files.insert(path.to_string());
    }
}

/// 一期的结构：看了多少、几处真问题、几处拿不准，以及最值得改的那一处。
struct Digest {
    scanned: usize,
    errors: usize,
    suspects: usize,
    categories: usize,
    top: Option<Top>,
}

struct Top {
    kind: String,
    advice: String,
    count: usize,
    files: usize,
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

/// 改法写成「「词」→ 换成什么：为什么」。把词与后半句拆开，报告才好讲人话。
fn split_advice(advice: &str) -> Option<(String, String)> {
    let rest = advice.strip_prefix('「')?;
    let end = rest.find('」')?;
    let word = rest[..end].to_string();
    let after = rest[end + '」'.len_utf8()..].trim_start_matches('→').trim();
    Some((word, after.replace('：', "——")))
}

fn collect(roots: &[PathBuf], language: &LanguageContract, docs: &DocsContract) -> Digest {
    let mut scanned = 0usize;
    let mut errors = 0usize;
    let mut suspects = 0usize;
    let mut groups: HashMap<(String, String), Group> = HashMap::new();

    for root in roots {
        if !root.exists() {
            continue;
        }

        let (findings, _, count) = scan_language(root, language, "default");
        scanned += count;
        for finding in findings {
            match finding.severity {
                LanguageSeverity::Suspect => suspects += 1,
                LanguageSeverity::Error => {
                    // 目录级的体裁要求（line 0）与三类误报不进体检：那些归人判。
                    if finding.verdict.is_false_positive() || finding.line == 0 {
                        continue;
                    }
                    errors += 1;
                    groups
                        .entry((finding.kind.key().to_string(), finding.advice.clone()))
                        .or_default()
                        .add(&finding.path);
                }
            }
        }

        let (findings, _, _) = scan_docs(root, docs);
        for finding in findings {
            match finding.severity {
                DocsSeverity::Suspect => suspects += 1,
                DocsSeverity::Error => {
                    errors += 1;
                    groups
                        .entry((finding.kind.key().to_string(), finding.advice.clone()))
                        .or_default()
                        .add(&finding.path);
                }
            }
        }
    }

    let categories = groups.len();
    let has_banned = groups.keys().any(|(kind, _)| kind == "banned");
    let mut ranked: Vec<(usize, usize, &String, &String)> = groups
        .iter()
        .filter(|((kind, _), _)| !has_banned || kind == "banned")
        .map(|((kind, advice), group)| (group.count, group.files.len(), kind, advice))
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then(a.3.cmp(b.3)));

    let top = ranked.first().map(|(count, files, kind, advice)| Top {
        kind: (*kind).clone(),
        advice: (*advice).clone(),
        count: *count,
        files: *files,
    });

    Digest {
        scanned,
        errors,
        suspects,
        categories,
        top,
    }
}

fn print_report(digest: &Digest) {
    println!("量潮创始人 · 文字体检\n");
    println!("这次看了 {} 篇文档。\n", digest.scanned);

    let Some(top) = &digest.top else {
        println!("没查出问题。");
        return;
    };

    match split_advice(&top.advice) {
        Some((word, rest)) => {
            println!("这一期先改一处：「{word}」\n");
            println!("它出现 {} 次，落在 {} 篇里。\n", top.count, top.files);
            println!("怎么改：把「{word}」换成 {rest}。\n");
        }
        None => {
            println!("这一期先改一处：{}\n", top.advice);
            println!("它出现 {} 次，落在 {} 篇里。\n", top.count, top.files);
        }
    }

    if digest.categories > 1 {
        println!(
            "另有 {} 类这次不列，改完这一处再看。",
            digest.categories - 1
        );
    }
    if digest.suspects > 0 {
        println!(
            "机器拿不准的有 {} 处，这类归人判，不进这份报告。",
            digest.suspects
        );
    }
}

fn render_json(digest: &Digest) {
    let top = match &digest.top {
        Some(top) => format!(
            "{{\"kind\":\"{}\",\"advice\":\"{}\",\"count\":{},\"files\":{}}}",
            json_escape(&top.kind),
            json_escape(&top.advice),
            top.count,
            top.files
        ),
        None => "null".to_string(),
    };
    println!(
        "{{\n  \"scanned_files\": {},\n  \"errors\": {},\n  \"suspects\": {},\n  \"categories\": {},\n  \"top\": {}\n}}",
        digest.scanned, digest.errors, digest.suspects, digest.categories, top
    );
}

/// 入口：返回退出码（0 没查出问题 / 1 有真问题 / 2 契约读不了）。
pub fn run_audit(args: &AuditArgs) -> Result<i32, String> {
    let language = LanguageContract::load(args.language_rules.as_deref())?;
    let docs = DocsContract::load(args.docs_rules.as_deref())?;

    let digest = collect(&args.roots, &language, &docs);

    if args.json {
        render_json(&digest);
    } else {
        print_report(&digest);
    }

    Ok(i32::from(digest.errors > 0))
}

#[cfg(test)]
mod tests {
    use super::split_advice;

    #[test]
    fn splits_word_and_replacement() {
        let (word, rest) = split_advice("「产物」→ 成品、文件、报告：说清是哪一种东西").unwrap();
        assert_eq!(word, "产物");
        assert_eq!(rest, "成品、文件、报告——说清是哪一种东西");
    }

    #[test]
    fn keeps_formatting_advice_whole() {
        assert!(split_advice("一行 2 处加粗（上限 1）").is_none());
    }
}
