//! `agent relation`：Skill 与 CLI 命令的边。
//!
//! 边从 Skill 正文里的 `qtfounder <组> <动词>` 读出来，覆盖情况对照 Agent 契约的
//! `cli_scopes`。只读，不改 Skill，也不跑命令。

use std::path::PathBuf;

use clap::Args;
use regex::Regex;
use serde_json::json;

use super::contract::Contract;
use super::registry;

#[derive(Args, Debug)]
pub struct RelationArgs {
    /// 只看这一条 Skill
    pub skill: Option<String>,
    /// 契约路径（缺省按 src/agent/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

/// Skill 正文里出现的 `qtfounder` 命令，按出现顺序去重。
pub fn commands_in(text: &str) -> Vec<String> {
    let pattern =
        Regex::new(r"qtfounder\s+([a-z][a-z-]*)(?:\s+([a-z][a-z-]*))?").expect("命令正则写不对");
    let mut commands: Vec<String> = Vec::new();
    for captures in pattern.captures_iter(text) {
        let group = captures
            .get(1)
            .map(|item| item.as_str())
            .unwrap_or_default();
        let command = match captures.get(2).map(|item| item.as_str()) {
            Some(sub) => format!("qtfounder {group} {sub}"),
            None => format!("qtfounder {group}"),
        };
        if !commands.contains(&command) {
            commands.push(command);
        }
    }
    commands
}

/// 命令对应的命令组：`qtfounder meta classify` → `meta`。
fn group_of(command: &str) -> Option<&str> {
    command.split_whitespace().nth(1)
}

/// 契约里登记过、但没有 Skill 引用的命令组。
pub fn uncovered(contract: &Contract, commands: &[String]) -> Vec<String> {
    let covered: Vec<&str> = commands
        .iter()
        .filter_map(|command| group_of(command))
        .collect();
    contract
        .name
        .cli_scopes
        .iter()
        .filter(|scope| !covered.iter().any(|item| item == scope))
        .cloned()
        .collect()
}

pub fn run(args: &RelationArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    let all: Vec<(&str, Vec<String>)> = registry::all()
        .iter()
        .map(|entry| (entry.id, commands_in(entry.skill_md)))
        .collect();

    let skills: Vec<(&str, Vec<String>)> = match &args.skill {
        Some(id) => {
            let found = all
                .iter()
                .find(|(skill, _)| *skill == id)
                .map(|(skill, commands)| (*skill, commands.clone()))
                .ok_or_else(|| {
                    let ids: Vec<&str> = all.iter().map(|(id, _)| *id).collect();
                    format!("没有这条 Skill：{id}；可选：{}", ids.join("、"))
                })?;
            vec![found]
        }
        None => all.clone(),
    };

    let all_commands: Vec<String> = all
        .iter()
        .flat_map(|(_, commands)| commands.iter().cloned())
        .collect();
    let uncovered = uncovered(&contract, &all_commands);

    if args.json {
        let items: Vec<serde_json::Value> = skills
            .iter()
            .map(|(id, commands)| json!({"id": id, "commands": commands}))
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "skills": items,
                "uncovered": uncovered,
            }))
            .unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(0);
    }

    for (id, commands) in &skills {
        println!("{id}");
        if commands.is_empty() {
            println!("  （无命令）");
        }
        for command in commands {
            println!("  {command}");
        }
    }
    if args.skill.is_none() {
        println!();
        if uncovered.is_empty() {
            println!("命令组都有 Skill 引用");
        } else {
            println!("未覆盖的命令组：{}", uncovered.join("、"));
        }
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 从正文提取命令并去重() {
        let text = "先跑 `qtfounder meta classify`，再跑 `qtfounder meta classify --json`，最后 `qtfounder write`。";
        let commands = commands_in(text);
        assert_eq!(
            commands,
            vec![
                "qtfounder meta classify".to_string(),
                "qtfounder write".to_string()
            ]
        );
    }

    #[test]
    fn 选项不会被当成子命令() {
        let commands = commands_in("`qtfounder --version` 与 `qtfounder write`");
        assert_eq!(commands, vec!["qtfounder write".to_string()]);
    }

    #[test]
    fn 未覆盖按契约的命令组算() {
        let contract = Contract::bundled();
        let commands = vec!["qtfounder meta classify".to_string()];
        let missing = uncovered(&contract, &commands);
        assert!(!missing.contains(&"meta".to_string()));
        assert!(missing.contains(&"write".to_string()));
    }
}
