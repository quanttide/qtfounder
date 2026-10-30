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

use std::path::PathBuf;

use clap::Args;

use crate::code::rules::{ReviewRule, Rules};
use crate::core::exit;

/// 意图审查：出报告骨架，把语用命中的当锚点贴进去。
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
}

/// 五层意图。反推就按这五问，与重构那边（Skill `code-refactor`）同一套。
pub(super) const INTENT_LAYERS: [&str; 5] = [
    "核心目标：这一坨本质上要干什么，一句话说清",
    "边界：什么它不干",
    "约束：什么条件必须满足",
    "已知假设：基于什么前提在做事",
    "不确定性：哪些地方还没想清楚",
];

pub fn run(args: &ReviewArgs, rules_path: Option<&std::path::Path>, json: bool) -> Result<i32, String> {
    let rules = Rules::load(rules_path)?;
    if rules.review.is_empty() {
        return Err(
            "契约里没有 [[review]]：在 default-contract.toml 末尾补一段审查规则再跑"
                .to_string(),
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
        render_json(&rules.review, &findings, &args.root);
        return Ok(if findings.is_empty() {
            exit::OK
        } else {
            exit::FINDINGS
        });
    }

    render_text(&rules.review, &findings, &args.root, args.example);
    Ok(if findings.is_empty() {
        exit::OK
    } else {
        exit::FINDINGS
    })
}

fn render_text(rules: &[ReviewRule], findings: &[super::model::Finding], root: &PathBuf, example: bool) {
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
        println!("语用检查找到 {} 处，每处都带 file:line，可直接当发现的锚点：\n", findings.len());
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

fn render_json(rules: &[ReviewRule], findings: &[super::model::Finding], root: &PathBuf) {
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
                "    {{\"layer\":\"{}\",\"path\":\"{}\",\"line\":{},\"message\":\"{}\"}}",
                finding.layer.key(),
                super::json_escape(&finding.path),
                finding.line,
                super::json_escape(&finding.message),
            )
        })
        .collect();

    println!("{{");
    println!("  \"root\": \"{}\",", super::json_escape(&root.display().to_string()));
    println!("  \"intent_layers\": [");
    for (index, line) in INTENT_LAYERS.iter().enumerate() {
        let comma = if index + 1 == INTENT_LAYERS.len() { "" } else { "," };
        println!("    \"{line}\"{comma}");
    }
    println!("  ],");
    println!("  \"rules\": [\n{}\n  ],", rules_json.join(",\n"));
    println!("  \"findings\": [\n{}\n  ]", findings_json.join(",\n"));
    println!("}}");
}
