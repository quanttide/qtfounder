//! `data rule`：这一个子命令的命令面、实现与加载器。
//!
//! 与另两个子命令同形——各自带模型与加载器，不共用 match 臂。
//! 为什么要拆开：三个子命令是三件不同的事，挤在一个文件里时，
//! 改结果统计要滚过规则与任务包的实现。

use clap::Subcommand;

use serde::{Deserialize, Serialize};


use crate::product::store;

use super::{model::{Rule, RuleLabel, RuleSource, validate_rule}, read_json_file, write_json_file, rules_dir};

#[derive(Subcommand, Debug)]
pub enum RuleCommand {
    /// 列出全部规则
    List {
        #[arg(long)]
        json: bool,
    },
    /// 看一条规则的全文
    Show {
        /// 规则 id，`rul_` 开头
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// 写入一条规则（stdin 给 JSON）；带已有 id 则覆盖，不带则新建
    Save {
        /// stdin 形状：{"name":"…","source":{"kind":"journal","split":"blank-line"},"labels":[{"name":"fact","label":"事实"}],"batch":20}
        #[arg(long)]
        stdin: bool,
        #[arg(long)]
        json: bool,
    },
}

pub(crate) fn run_rule(command: RuleCommand) -> Result<(), crate::product::AppError> {
    match command {
        RuleCommand::List { json } => {
            let rules = load_all_rules()?;
            if json {
                #[derive(Serialize)]
                struct Row<'a> {
                    id: &'a str,
                    name: &'a str,
                    labels: usize,
                    batch: usize,
                }
                let rows: Vec<Row> = rules
                    .iter()
                    .map(|rule| Row {
                        id: &rule.id,
                        name: &rule.name,
                        labels: rule.labels.len(),
                        batch: rule.batch,
                    })
                    .collect();
                let payload = serde_json::json!({ "rules": rows });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_default()
                );
                return Ok(());
            }
            if rules.is_empty() {
                println!("暂无规则，先在 studio 设计一条，或 qtfounder data rule save --stdin");
                return Ok(());
            }
            for rule in &rules {
                println!(
                    "{}  {}  {} 个标签  批次 {}",
                    rule.id,
                    rule.name,
                    rule.labels.len(),
                    rule.batch
                );
            }
            Ok(())
        }
        RuleCommand::Show { id, json } => {
            let rule = load_rule(&id)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&rule).unwrap_or_default()
                );
                return Ok(());
            }
            println!("{}  {}", rule.id, rule.name);
            println!(
                "来源 {} / 切段 {}，批次 {}",
                rule.source.kind, rule.source.split, rule.batch
            );
            for label in &rule.labels {
                println!("  {}（{}）", label.label, label.name);
                if let Some(define) = &label.define {
                    println!("    定义：{define}");
                }
                if let Some(yes) = &label.yes {
                    println!("    正例：{yes}");
                }
                if let Some(no) = &label.no {
                    println!("    反例：{no}");
                }
            }
            Ok(())
        }
        RuleCommand::Save { stdin, json } => {
            if !stdin {
                return Err(crate::product::AppError::usage(
                    "用 --stdin 给规则 JSON".to_string(),
                ));
            }
            let mut buffer = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut buffer)
                .map_err(|err| crate::product::AppError::usage(format!("读 stdin 失败：{err}")))?;

            #[derive(Deserialize)]
            struct RuleInput {
                #[serde(default)]
                id: Option<String>,
                name: String,
                source: RuleSource,
                labels: Vec<RuleLabel>,
                #[serde(default = "default_batch")]
                batch: usize,
            }
            fn default_batch() -> usize {
                20
            }

            let input: RuleInput = serde_json::from_str(&buffer).map_err(|err| {
                crate::product::AppError::usage(format!("stdin 不是合法 JSON：{err}"))
            })?;
            let mut rules = load_all_rules()?;
            let id = match input.id {
                Some(id) => {
                    if !rules.iter().any(|rule| rule.id == id) {
                        return Err(crate::product::AppError::NotFound {
                            message: format!("{id} 不存在"),
                            hint: "新建规则不要带 id，改由工具生成".to_string(),
                        });
                    }
                    id
                }
                None => store::new_id("rul", |candidate| {
                    rules.iter().any(|rule| rule.id == candidate)
                }),
            };
            let rule = Rule {
                id: id.clone(),
                name: input.name,
                source: input.source,
                labels: input.labels,
                batch: input.batch,
                at: store::now_iso8601(),
            };
            validate_rule(&rule)?;
            let path = rules_dir().join(format!("{id}.json"));
            write_json_file(&path, &rule)?;
            rules.retain(|existing| existing.id != id);
            rules.push(rule.clone());
            if json {
                let payload = serde_json::json!({
                    "id": rule.id,
                    "path": path.display().to_string(),
                    "labels": rule.labels.len(),
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_default()
                );
                return Ok(());
            }
            println!(
                "已保存 {}（{} 个标签）：{}",
                rule.id,
                rule.labels.len(),
                rule.name
            );
            println!("{}", path.display());
            Ok(())
        }
    }
}

fn load_rule(id: &str) -> Result<Rule, crate::product::AppError> {
    let path = rules_dir().join(format!("{id}.json"));
    if !path.exists() {
        return Err(crate::product::AppError::NotFound {
            message: format!("{id} 不存在"),
            hint: "用 qtfounder data rule list 查看".to_string(),
        });
    }
    read_json_file(&path)
}

fn load_all_rules() -> Result<Vec<Rule>, crate::product::AppError> {
    let dir = rules_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut rules = vec![];
    for entry in std::fs::read_dir(&dir).map_err(|err| {
        crate::product::AppError::usage(format!("无法读取 {}：{err}", dir.display()))
    })? {
        let entry = entry.map_err(|err| {
            crate::product::AppError::usage(format!("无法读取 {}：{err}", dir.display()))
        })?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let rule: Rule = read_json_file(&path)?;
        rules.push(rule);
    }
    rules.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(rules)
}

