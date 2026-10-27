//! 资产维护：每类资产共用的维护做法。
//!
//! 规则来自归位契约的 `[maintain]` 段，不在这里硬编码。
//! 这个视图只读；写文件、打标签这些动作归各资产自己的发布流程。

use std::path::Path;

use clap::Args;
use serde_json::json;

use crate::meta::contract::{Category, Contract};

#[derive(Args, Debug)]
pub struct MaintainArgs {
    /// 只看一类资产的维护规则
    pub name: Option<String>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &MaintainArgs, rules: Option<&Path>) -> Result<i32, String> {
    let contract = Contract::load(rules)?;
    let category = match &args.name {
        Some(name) => Some(category_of(&contract, name)?),
        None => None,
    };
    if args.json {
        print_json(&contract, category);
    } else {
        print_text(&contract, category);
    }
    Ok(0)
}

fn category_of<'a>(contract: &'a Contract, name: &str) -> Result<&'a Category, String> {
    contract.category(name).ok_or_else(|| {
        let names: Vec<&str> = contract
            .category
            .iter()
            .map(|category| category.name.as_str())
            .collect();
        format!("没有这类资产：{name}；可选：{}", names.join("、"))
    })
}

fn print_text(contract: &Contract, category: Option<&Category>) {
    match category {
        Some(category) => {
            println!(
                "{}  {}  {}  {}",
                category.name, category.label, category.path, category.family
            );
            println!(
                "  边界与写法看 `qtfounder asset contract {}`",
                category.name
            );
        }
        None => println!("资产维护：{} 类资产共用", contract.category.len()),
    }
    if !contract.maintain.description.is_empty() {
        println!("{}", contract.maintain.description);
    }
    if contract.maintain.rule.is_empty() {
        println!();
        println!("契约里没有维护规则。");
        return;
    }
    println!();
    for (index, rule) in contract.maintain.rule.iter().enumerate() {
        println!("{}  {}", index + 1, rule.name);
        println!("   {}", rule.description);
    }
}

fn print_json(contract: &Contract, category: Option<&Category>) {
    let rules: Vec<serde_json::Value> = contract
        .maintain
        .rule
        .iter()
        .map(|rule| {
            json!({
                "name": rule.name,
                "description": rule.description,
            })
        })
        .collect();
    let asset = category.map(|category| {
        json!({
            "name": category.name,
            "label": category.label,
            "path": category.path,
            "family": category.family,
        })
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "asset": asset,
            "description": contract.maintain.description,
            "rules": rules,
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 契约里有维护规则() {
        let contract = Contract::bundled();
        assert!(!contract.maintain.rule.is_empty());
        for rule in &contract.maintain.rule {
            assert!(!rule.name.is_empty());
            assert!(!rule.description.is_empty(), "{} 缺说明", rule.name);
        }
    }

    #[test]
    fn 未知资产会报可选项() {
        let contract = Contract::bundled();
        let error = category_of(&contract, "nope").unwrap_err();
        assert!(error.contains("report"));
    }
}
