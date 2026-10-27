//! 资产契约：每类资产该写成什么样。
//!
//! 数据源是归位契约的十八格。这里不重复定义资产，只把契约里的边界、分档与写法摊开。

use std::path::Path;

use clap::Args;
use serde_json::json;

use crate::meta::contract::{Category, Contract};

#[derive(Args, Debug)]
pub struct ContractArgs {
    /// 只看一类资产
    pub name: Option<String>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &ContractArgs, rules: Option<&Path>) -> Result<i32, String> {
    let contract = Contract::load(rules)?;
    let categories: Vec<&Category> = match &args.name {
        Some(name) => vec![category_of(&contract, name)?],
        None => contract.category.iter().collect(),
    };
    if args.json {
        print_json(&categories);
    } else {
        print_text(&categories);
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

fn grade_line(category: &Category) -> String {
    if category.grade.is_empty() {
        return String::new();
    }
    category
        .grade
        .iter()
        .map(|grade| {
            let cues = if grade.cues.is_empty() {
                "默认".to_string()
            } else {
                grade.cues.join("、")
            };
            format!("{}（{cues}）", grade.label)
        })
        .collect::<Vec<String>>()
        .join("；")
}

fn print_text(categories: &[&Category]) {
    for (index, category) in categories.iter().enumerate() {
        if index > 0 {
            println!();
        }
        println!(
            "{}  {}  {}  {}",
            category.name, category.label, category.path, category.family
        );
        if !category.description.is_empty() {
            println!("  边界：{}", category.description);
        }
        let grades = grade_line(category);
        if !grades.is_empty() {
            println!("  分档：{grades}");
        }
        if !category.skeleton.trim().is_empty() {
            println!("  写法：");
            for line in category.skeleton.lines() {
                if line.is_empty() {
                    println!();
                } else {
                    println!("    {line}");
                }
            }
        }
    }
}

fn print_json(categories: &[&Category]) {
    let items: Vec<serde_json::Value> = categories
        .iter()
        .map(|category| {
            let grades: Vec<serde_json::Value> = category
                .grade
                .iter()
                .map(|grade| {
                    json!({
                        "label": grade.label,
                        "cues": grade.cues,
                        "fallback": grade.fallback,
                    })
                })
                .collect();
            json!({
                "name": category.name,
                "label": category.label,
                "path": category.path,
                "family": category.family,
                "description": category.description,
                "skeleton": category.skeleton,
                "grades": grades,
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_string())
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 每类资产都有契约() {
        let contract = Contract::bundled();
        for category in &contract.category {
            assert!(!category.name.is_empty());
            assert!(!category.path.is_empty());
            assert!(!category.description.is_empty(), "{} 缺边界", category.name);
            assert!(
                !category.skeleton.trim().is_empty(),
                "{} 缺写法",
                category.name
            );
        }
    }

    #[test]
    fn 未知资产会报可选项() {
        let contract = Contract::bundled();
        let error = category_of(&contract, "nope").unwrap_err();
        assert!(error.contains("report"));
    }

    #[test]
    fn 分档写明默认项() {
        let contract = Contract::bundled();
        let insight = contract.category("insight").unwrap();
        let line = grade_line(insight);
        assert!(line.contains("假说"));
        assert!(line.contains("默认"));
    }
}
