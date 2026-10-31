//! `qtfounder write review`：读者视角的评审骨架与纪律。
//!
//! 表达审计（`qtfounder write`）判的是「这段文字有没有把话讲清楚」，是机器对句子形态
//! 的嫌疑报告。这边是**人对整篇的读者评审**：从读者角度，看这篇有没有把读者带到该去的
//! 地方。正本是 Skill `write-review`，这份命令只做两件事：
//!
//! 1. 出评审骨架，四节固定，让人与 AI 填同一份形状；
//! 2. 把表达审计的命中当**卡点**贴进去——它们自带 `file:line`，正是报告要的位置。
//!
//! **立场：只从读者角度评，禁止从编辑或作者角度。**不问「作者想表达什么」，不替作者
//! 重写，不评「写得好不好」——只问读者读到哪里、卡在哪、带走了什么。视角一换成作者，
//! 报告就成了改稿意见，读者的真实体验反而被盖住。

use std::io::Read as _;
use std::path::{Path, PathBuf};

use clap::Args;

use super::checks::Selection;
use super::contract::{Contract, ReviewRule};
use super::model::Finding;
use super::scan;
use crate::core::exit;
use crate::review::{Experience, Store, now_iso8601};

/// 读者评审：出骨架，把表达审计的命中当卡点，从读者角度评。
#[derive(Args, Debug)]
pub struct WriteReviewArgs {
    /// 要评的目录或文件
    #[arg(default_value = ".")]
    pub root: PathBuf,
    /// 读者画像；缺省 default——读者是谁决定要交代多少背景
    #[arg(long)]
    pub audience: Option<String>,
    /// 契约路径（缺省按 src/write/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 连纪律的 example 一起打
    #[arg(long)]
    pub example: bool,
    /// 记一条读者体会：正文从标准输入读，配合 --user 指定是谁
    #[arg(long)]
    pub record: bool,
    /// 体会属于谁；记一条时必给，多个读者各记各的
    #[arg(long)]
    pub user: Option<String>,
    /// 列出这个目标已记的读者体会，不出骨架
    #[arg(long)]
    pub experiences: bool,
}

/// 读者角度的四个问题——评审就按这四问，与意图审查的五层对称，但视角锁在读者侧。
pub(super) const READER_LAYERS: [&str; 4] = [
    "读者是谁：带着什么问题来读，愿意花多久",
    "读者要什么：他想从这篇拿走的一个判断或一个动作",
    "读者卡在哪：哪一句、哪一节让他停下、走神、回读",
    "读者带走什么：合上这篇，他记得的、能做的还剩多少",
];

pub fn run(args: &WriteReviewArgs, json: bool) -> Result<i32, String> {
    if args.record {
        return record_experience(args);
    }
    if args.experiences {
        return list_experiences(args, json);
    }

    let contract = Contract::load(args.rules.as_deref())?;
    if contract.review.is_empty() {
        return Err(
            "契约里没有 [[review]]：在 default-contract.toml 末尾补一段读者评审纪律再跑"
                .to_string(),
        );
    }
    if !args.root.exists() {
        eprintln!("目标不在：{}", args.root.display());
        return Ok(exit::NOT_FOUND);
    }

    // 卡点来自表达审计：空话、口号、指路、结构这些让读者停下的地方，机器先指位置。
    let (findings, _broken, _scanned) = scan::scan(
        &args.root,
        &contract,
        &Selection {
            genre: None,
            audience: args.audience.clone(),
            topic: None,
        },
    );

    let experiences = load_experiences(&args.root);
    if json {
        render_json(&contract.review, &findings, &args.root, &experiences);
        return Ok(if findings.is_empty() {
            exit::OK
        } else {
            exit::FINDINGS
        });
    }

    render_text(
        &contract.review,
        &findings,
        &args.root,
        args.example,
        &experiences,
    );
    Ok(if findings.is_empty() {
        exit::OK
    } else {
        exit::FINDINGS
    })
}

/// 记一条读者体会：正文从标准输入读，谁的体会看 --user。多个读者各记各的。
fn record_experience(args: &WriteReviewArgs) -> Result<i32, String> {
    let Some(user) = args.user.as_deref().filter(|u| !u.trim().is_empty()) else {
        return Err("记一条体会要指明是谁的：--user <名字>".to_string());
    };
    let mut body = String::new();
    std::io::stdin()
        .read_to_string(&mut body)
        .map_err(|err| format!("读取标准输入失败：{err}"))?;
    let body = body.trim();
    if body.is_empty() {
        return Err("体会正文不能为空：从标准输入读一段你作为读者的判断".to_string());
    }

    let target = args.root.display().to_string();
    let mut store = Store::load()?;
    store.push(Experience {
        domain: "write".to_string(),
        user: user.to_string(),
        target: target.clone(),
        body: body.to_string(),
        at: now_iso8601(),
    });
    store.save()?;
    println!("已记 {user} 作为读者对 {target} 的一条体会");
    Ok(exit::OK)
}

/// 列出这个目标已记的读者体会（多个读者），不出骨架。
fn list_experiences(args: &WriteReviewArgs, json: bool) -> Result<i32, String> {
    let experiences = load_experiences(&args.root);
    if json {
        let items: Vec<serde_json::Value> = experiences
            .iter()
            .map(|exp| {
                serde_json::json!({
                    "user": exp.user,
                    "body": exp.body,
                    "at": exp.at,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::json!({"target": args.root.display().to_string(), "experiences": items})
        );
        return Ok(exit::OK);
    }
    if experiences.is_empty() {
        println!("{} 还没有记过读者体会", args.root.display());
        return Ok(exit::OK);
    }
    println!(
        "{} 的读者体会（{} 条）：\n",
        args.root.display(),
        experiences.len()
    );
    for exp in experiences {
        println!("[{}] {}  {}", exp.user, exp.at, exp.body);
    }
    Ok(exit::OK)
}

/// 读这个目标已记的体会，按记录顺序。
fn load_experiences(root: &Path) -> Vec<Experience> {
    match Store::load() {
        Ok(store) => store
            .for_target("write", &root.display().to_string())
            .into_iter()
            .cloned()
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn render_text(
    rules: &[ReviewRule],
    findings: &[Finding],
    root: &Path,
    example: bool,
    experiences: &[Experience],
) {
    println!("# 读者评审：{}\n", root.display());

    println!("## 结论");
    println!("<一到三句：读者读完最可能的感受、最卡的一处、最该先补的一件事>\n");

    println!("## 读者视角");
    for line in READER_LAYERS {
        println!("- {line}");
    }
    println!();

    println!("## 卡点清单（按读者停下的先后排）");
    if findings.is_empty() {
        println!("表达审计零命中，没有现成卡点；卡点清单只能靠真读一遍，写不出位置的不写。");
    } else {
        println!(
            "表达审计找到 {} 处可能让读者停下的地方，每处都带 file:line，可直接当卡点：\n",
            findings.len()
        );
        for finding in findings {
            let line = finding
                .line
                .map(|line| format!("L{line}"))
                .unwrap_or_else(|| "文件级".to_string());
            println!(
                "  [{}] {}:{} — {}",
                finding.severity.label(),
                finding.path,
                line,
                finding.excerpt
            );
        }
        println!("\n卡点不排序、不标严重度：读者在哪停下要对着读者判，那是判断。");
    }
    println!("视角别滑回作者侧：不写「作者应改成……」，只写「读者读到这会……」。\n");

    println!("## 建议");
    println!("<从读者角度：哪一处补一句背景、哪一节该挪后、哪一段可以整段删>\n");

    print_experiences(experiences);

    println!("## 交付纪律（{} 条）", rules.len());
    for rule in rules {
        println!("{}  {}", rule.name, rule.description);
        println!("  为什么：{}", rule.why);
        if example && !rule.example.is_empty() {
            println!("  长什么样：{}", rule.example);
        }
        println!();
    }
}

/// 把已记的读者体会摊开：多个读者各一条，是人的判断不是机器的。
fn print_experiences(experiences: &[Experience]) {
    if experiences.is_empty() {
        println!("## 读者体会\n");
        println!("<还没有记过；评完用 --record --user <名字> 记一条你作为读者的真实判断>\n");
        return;
    }
    println!("## 读者体会（{} 条）\n", experiences.len());
    for exp in experiences {
        println!("[{}] {}  {}\n", exp.user, exp.at, exp.body);
    }
}

fn render_json(
    rules: &[ReviewRule],
    findings: &[Finding],
    root: &Path,
    experiences: &[Experience],
) {
    let rules_json: Vec<serde_json::Value> = rules
        .iter()
        .map(|rule| {
            serde_json::json!({
                "name": rule.name,
                "description": rule.description,
                "why": rule.why,
                "example": rule.example,
            })
        })
        .collect();
    let findings_json: Vec<serde_json::Value> = findings
        .iter()
        .map(|finding| {
            serde_json::json!({
                "severity": finding.severity.key(),
                "kind": finding.kind.key(),
                "path": finding.path,
                "line": finding.line,
                "excerpt": finding.excerpt,
                "advice": finding.advice,
            })
        })
        .collect();
    let experiences_json: Vec<serde_json::Value> = experiences
        .iter()
        .map(|exp| {
            serde_json::json!({
                "user": exp.user,
                "at": exp.at,
                "body": exp.body,
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::json!({
            "root": root.display().to_string(),
            "reader_layers": READER_LAYERS,
            "rules": rules_json,
            "findings": findings_json,
            "experiences": experiences_json,
        })
    );
}
