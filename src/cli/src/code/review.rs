//! `qtfounder code review`：意图审查的骨架与纪律。
//!
//! 审查是意图驱动编程的**反向流程**：从代码反推写它时的意图，再看代码还对不对得上。
//! 正本是 Skill `code-review`（五层意图、意图考古、报告格式），这份命令只做两件事：
//!
//! 1. 出报告骨架，四节固定，让人与 AI 填同一份形状；
//! 2. 把语用检查的命中当**锚点**贴进「偏移清单」——它们自带 `file:line`，
//!    正是报告要求的证据格式。
//!
//! **锚点不排序、不标严重度**：严重度得对着意图判，那是判断，归写报告的人。
//! 命令只递证据。同理，意图假设五节命令一个字不填——反推是判断，不机械。

use std::io::Read as _;
use std::path::{Path, PathBuf};

use clap::Args;

use crate::code::rules::{ReviewRule, Rules};
use crate::core::exit;
use crate::review::{Experience, Store, now_iso8601};

/// 意图审查：出报告骨架，把语用命中的当锚点贴进去；审完把人的体会记下来。
#[derive(Args, Debug)]
pub struct ReviewArgs {
    /// 要审的目录或文件
    #[arg(default_value = ".")]
    pub root: PathBuf,
    /// 只取某几层的命中当锚点：lexical／syntax／structure／pragmatic，可重复
    #[arg(long = "layer")]
    pub layer: Vec<String>,
    /// 连纪律的 example 一起打——要不要例子按需，缺省省掉
    #[arg(long)]
    pub example: bool,
    /// 记一条使用体会：正文从标准输入读，配合 --user 指定是谁的体会
    #[arg(long)]
    pub record: bool,
    /// 体会属于谁；记一条时必给，多个用户各记各的
    #[arg(long)]
    pub user: Option<String>,
    /// 列出这个目标已记的使用体会，不出报告
    #[arg(long)]
    pub experiences: bool,
}

/// 五层意图。反推就按这五问，与重构那边（Skill `code-refactor`）同一套。
pub(super) const INTENT_LAYERS: [&str; 5] = [
    "核心目标：这一坨本质上要干什么，一句话说清",
    "边界：什么它不干",
    "约束：什么条件必须满足",
    "已知假设：基于什么前提在做事",
    "不确定性：哪些地方还没想清楚",
];

pub fn run(
    args: &ReviewArgs,
    rules_path: Option<&std::path::Path>,
    json: bool,
) -> Result<i32, String> {
    if args.record {
        return record_experience(args);
    }
    if args.experiences {
        return list_experiences(args, json);
    }

    let rules = Rules::load(rules_path)?;
    if rules.review.is_empty() {
        return Err(
            "契约里没有 [[review]]：在 default-contract.toml 末尾补一段审查规则再跑".to_string(),
        );
    }

    let layers = super::wanted_layers(&args.layer);
    if layers.is_empty() {
        return Err(format!("不认识的层：{}", args.layer.join("、")));
    }
    if !args.root.exists() {
        eprintln!("目标不在：{}", args.root.display());
        return Ok(exit::NOT_FOUND);
    }

    let (findings, broken) = super::collect(&args.root, &rules, &layers);
    for (path, why) in &broken {
        eprintln!("解析失败：{path} —— {why}");
    }

    if json {
        let experiences = load_experiences(&args.root);
        render_json(&rules.review, &findings, &args.root, &experiences);
        return Ok(if findings.is_empty() {
            exit::OK
        } else {
            exit::FINDINGS
        });
    }

    let experiences = load_experiences(&args.root);
    render_text(
        &rules.review,
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

/// 记一条使用体会：正文从标准输入读，谁的体会看 --user。多个用户各记各的。
fn record_experience(args: &ReviewArgs) -> Result<i32, String> {
    let Some(user) = args.user.as_deref().filter(|u| !u.trim().is_empty()) else {
        return Err("记一条体会要指明是谁的：--user <名字>".to_string());
    };
    let mut body = String::new();
    std::io::stdin()
        .read_to_string(&mut body)
        .map_err(|err| format!("读取标准输入失败：{err}"))?;
    let body = body.trim();
    if body.is_empty() {
        return Err("体会正文不能为空：从标准输入读一段你的判断".to_string());
    }

    let target = args.root.display().to_string();
    let mut store = Store::load()?;
    store.push(Experience {
        domain: "code".to_string(),
        user: user.to_string(),
        target: target.clone(),
        body: body.to_string(),
        at: now_iso8601(),
    });
    store.save()?;
    println!("已记 {user} 对 {target} 的一条体会");
    Ok(exit::OK)
}

/// 列出这个目标已记的使用体会（多个用户），不出报告。
fn list_experiences(args: &ReviewArgs, json: bool) -> Result<i32, String> {
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
        println!("{} 还没有记过使用体会", args.root.display());
        return Ok(exit::OK);
    }
    println!(
        "{} 的使用体会（{} 条）：\n",
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
            .for_target("code", &root.display().to_string())
            .into_iter()
            .cloned()
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn render_text(
    rules: &[ReviewRule],
    findings: &[super::model::Finding],
    root: &Path,
    example: bool,
    experiences: &[Experience],
) {
    println!("# 意图审查：{}\n", root.display());

    println!("## 结论");
    println!("<一到三句：最可能的意图、整体偏移程度、最该先处理的一件事>\n");

    println!("## 意图假设");
    for line in INTENT_LAYERS {
        println!("- {line}");
    }
    println!();

    println!("## 偏移清单（按严重度排序）");
    if findings.is_empty() {
        println!("语用检查零命中，没有现成锚点；偏移清单只能靠反推，写不出锚点的发现不写。");
    } else {
        println!(
            "语用检查找到 {} 处，每处都带 file:line，可直接当发现的锚点：\n",
            findings.len()
        );
        for finding in findings {
            println!(
                "  [{}] {}:{} — {}",
                finding.layer.label(),
                finding.path,
                finding.line,
                finding.message
            );
        }
        println!("\n锚点不排序、不标严重度：严重度要对着意图判，那是判断。");
    }
    println!("意图层的自己补：与意图矛盾的、越界新增的、无依据的。\n");

    println!("## 建议");
    println!("<可删项、需与作者确认项、下一步>\n");

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

/// 把已记的使用体会摊开：多个用户各一条，是人的判断不是机器的。
fn print_experiences(experiences: &[Experience]) {
    if experiences.is_empty() {
        println!("## 使用体会\n");
        println!("<还没有记过；审完用 --record --user <名字> 记一条你的真实判断>\n");
        return;
    }
    println!("## 使用体会（{} 条）\n", experiences.len());
    for exp in experiences {
        println!("[{}] {}  {}\n", exp.user, exp.at, exp.body);
    }
}

pub(super) fn render_json(
    rules: &[ReviewRule],
    findings: &[super::model::Finding],
    root: &Path,
    experiences: &[Experience],
) {
    print!("{}", render_json_text(rules, findings, root, experiences));
}

/// 拼 JSON 文本；抽出来是为了能对字段断言，见 `tests::审查JSON的锚点带全八键`。
pub(super) fn render_json_text(
    rules: &[ReviewRule],
    findings: &[super::model::Finding],
    root: &Path,
    experiences: &[Experience],
) -> String {
    let rules_json: Vec<String> = rules
        .iter()
        .map(|rule| {
            format!(
                "    {{\"name\":\"{}\",\"description\":\"{}\",\"why\":\"{}\",\"example\":\"{}\"}}",
                super::json_escape(&rule.name),
                super::json_escape(&rule.description),
                super::json_escape(&rule.why),
                super::json_escape(&rule.example),
            )
        })
        .collect();
    let findings_json: Vec<String> = findings
        .iter()
        .map(|finding| {
            format!(
                "    {{\"layer\":\"{}\",\"layer_name\":\"{}\",\"kind\":\"{}\",\"path\":\"{}\",\"line\":{},\"message\":\"{}\",\"principle\":\"{}\",\"advice\":\"{}\"}}",
                finding.layer.key(),
                finding.layer.label(),
                finding.kind,
                super::json_escape(&finding.path),
                finding.line,
                super::json_escape(&finding.message),
                finding.principle.label(),
                super::json_escape(finding.advice),
            )
        })
        .collect();

    let mut out = String::from("{\n");
    out.push_str(&format!(
        "  \"root\": \"{}\",\n",
        super::json_escape(&root.display().to_string())
    ));
    out.push_str("  \"intent_layers\": [\n");
    for (index, line) in INTENT_LAYERS.iter().enumerate() {
        let comma = if index + 1 == INTENT_LAYERS.len() {
            ""
        } else {
            ","
        };
        out.push_str(&format!("    \"{line}\"{comma}\n"));
    }
    out.push_str("  ],\n");
    out.push_str(&format!(
        "  \"rules\": [\n{}\n  ],\n",
        rules_json.join(",\n")
    ));
    out.push_str(&format!(
        "  \"findings\": [\n{}\n  ],\n",
        findings_json.join(",\n")
    ));
    let experiences_json: Vec<String> = experiences
        .iter()
        .map(|exp| {
            format!(
                "    {{\"user\":\"{}\",\"at\":\"{}\",\"body\":\"{}\"}}",
                super::json_escape(&exp.user),
                super::json_escape(&exp.at),
                super::json_escape(&exp.body),
            )
        })
        .collect();
    out.push_str(&format!(
        "  \"experiences\": [\n{}\n  ]\n",
        experiences_json.join(",\n")
    ));
    out.push_str("}\n");
    out
}
