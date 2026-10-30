//! `qtfounder code reflect`：完成一个任务后照契约的反思清单看一遍当前架构。
//!
//! 清单在 `src/code/default-contract.toml` 的 `[[reflect]]`（每条 description／why／example）。
//! 本命令只打清单，**不下结论、不扫代码**——对着代码哪几条成立是 AI 的活，见 Skill
//! `code-reflect`；发现写回路线图 `data/roadmap/qtfounder/index.md`，不在这儿落第二份。

use std::path::Path;

use clap::Args;
use serde::Serialize;

use crate::code::rules::{ReflectRule, Rules};
use crate::core::exit;

/// 完成任务后的架构反思清单：看什么、为什么算缺陷、长什么样。
#[derive(Args, Debug)]
pub struct ReflectArgs {
    /// 只看某一条（按 name），缺省全打
    pub name: Option<String>,
    /// 连 example 一起打——要不要例子按需，缺省省掉
    #[arg(long)]
    pub example: bool,
}

#[derive(Serialize)]
struct RuleOut<'a> {
    name: &'a str,
    description: &'a str,
    why: &'a str,
    example: &'a str,
}

pub fn run(args: &ReflectArgs, rules_path: Option<&Path>, json: bool) -> Result<i32, String> {
    let rules = Rules::load(rules_path)?;
    if rules.reflect.is_empty() {
        return Err(
            "契约里没有 [[reflect]]：在 default-contract.toml 末尾补一段反思规则再跑".to_string(),
        );
    }

    let all: &[ReflectRule] = &rules.reflect;
    let picked: Vec<&ReflectRule> = match &args.name {
        Some(want) => {
            let hit: Vec<&ReflectRule> = all.iter().filter(|r| r.name == *want).collect();
            if hit.is_empty() {
                let known: Vec<&str> = all.iter().map(|r| r.name.as_str()).collect();
                return Err(format!(
                    "契约里没有这条反思规则「{want}」；有的是：{}",
                    known.join("、")
                ));
            }
            hit
        }
        None => all.iter().collect(),
    };

    print!("{}", render(&picked, all.len(), args.example, json)?);
    Ok(exit::OK)
}

/// 把选中的规则拼成报告；抽出来是为了能对输出断言，见 `tests::反思输出例子按需才给`。
pub(crate) fn render(
    picked: &[&ReflectRule],
    total: usize,
    example: bool,
    json: bool,
) -> Result<String, String> {
    if json {
        let out: Vec<RuleOut> = picked
            .iter()
            .map(|r| RuleOut {
                name: &r.name,
                description: &r.description,
                why: &r.why,
                example: &r.example,
            })
            .collect();
        return serde_json::to_string_pretty(&out)
            .map(|body| format!("{body}\n"))
            .map_err(|e| e.to_string());
    }

    let head = if picked.len() == total {
        format!("架构反思清单（{total} 条）——完成一个任务后照顺序看一遍")
    } else {
        format!("架构反思清单（取 {}/{total} 条）", picked.len())
    };

    let mut out = format!("{head}\n");
    for rule in picked {
        out.push_str(&format!("{}  {}\n", rule.name, rule.description));
        out.push_str(&format!("  为什么算缺陷：{}\n", rule.why));
        if example && !rule.example.is_empty() {
            out.push_str(&format!("  长什么样：{}\n", rule.example));
        }
        out.push('\n');
    }
    out.push_str("发现写回路线图 data/roadmap/qtfounder/index.md，不在这里存第二份。\n");
    Ok(out)
}
