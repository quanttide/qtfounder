//! 资产关系：家族与目录两条分组线。
//!
//! 只读归位契约，不推断依赖。家族回答「它属于哪一类」，目录回答「它和谁住在一起」。

use std::path::Path;

use clap::Args;
use serde_json::json;

use crate::meta::contract::{Category, Contract};

#[derive(Args, Debug)]
pub struct RelationArgs {
    /// 只看这类资产的关系
    pub name: Option<String>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

/// 目录根：路径的第一段。
pub fn root_of(path: &str) -> &str {
    path.split('/').next().unwrap_or(path)
}

/// 按某个键分组，保留契约里的出现顺序。
fn group<'a, F>(categories: &'a [Category], key: F) -> Vec<(String, Vec<&'a Category>)>
where
    F: Fn(&Category) -> String,
{
    let mut groups: Vec<(String, Vec<&Category>)> = Vec::new();
    for category in categories {
        let name = key(category);
        match groups.iter_mut().find(|(existing, _)| *existing == name) {
            Some((_, items)) => items.push(category),
            None => groups.push((name, vec![category])),
        }
    }
    groups
}

pub fn family_groups(categories: &[Category]) -> Vec<(String, Vec<&Category>)> {
    group(categories, |category| {
        if category.family.is_empty() {
            "未分家族".to_string()
        } else {
            category.family.clone()
        }
    })
}

pub fn root_groups(categories: &[Category]) -> Vec<(String, Vec<&Category>)> {
    group(categories, |category| root_of(&category.path).to_string())
}

pub fn run(args: &RelationArgs, rules: Option<&Path>) -> Result<i32, String> {
    let contract = Contract::load(rules)?;
    if let Some(name) = &args.name {
        let category = contract.category(name).ok_or_else(|| {
            let names: Vec<&str> = contract
                .category
                .iter()
                .map(|category| category.name.as_str())
                .collect();
            format!("没有这类资产：{name}；可选：{}", names.join("、"))
        })?;
        return show_one(category, &contract, args.json);
    }

    let families = family_groups(&contract.category);
    let roots = root_groups(&contract.category);
    if args.json {
        let families: Vec<serde_json::Value> = families
            .iter()
            .map(|(name, categories)| {
                json!({
                    "name": name,
                    "assets": categories.iter().map(|category| &category.name).collect::<Vec<_>>(),
                })
            })
            .collect();
        let roots: Vec<serde_json::Value> = roots
            .iter()
            .map(|(name, categories)| {
                json!({
                    "name": name,
                    "assets": categories.iter().map(|category| &category.name).collect::<Vec<_>>(),
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "families": families,
                "roots": roots,
            }))
            .unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(0);
    }

    for (name, categories) in &families {
        println!("{name}（{}）", categories.len());
        for category in categories {
            println!("  {}  {}", category.name, category.path);
        }
        println!();
    }
    println!("按目录");
    for (name, categories) in &roots {
        let names: Vec<&str> = categories
            .iter()
            .map(|category| category.name.as_str())
            .collect();
        println!("  {name}/  {}", names.join("、"));
    }
    Ok(0)
}

fn show_one(category: &Category, contract: &Contract, json: bool) -> Result<i32, String> {
    let root = root_of(&category.path);
    let siblings: Vec<&str> = contract
        .category
        .iter()
        .filter(|other| root_of(&other.path) == root && other.name != category.name)
        .map(|other| other.name.as_str())
        .collect();
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "asset": category.name,
                "label": category.label,
                "family": category.family,
                "root": root,
                "siblings": siblings,
            }))
            .unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(0);
    }
    println!("{}  {}", category.name, category.label);
    println!("  家族：{}", category.family);
    println!("  目录：{root}/");
    println!("  同目录：{}", siblings.join("、"));
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 家族按契约分组() {
        let contract = Contract::bundled();
        let families = family_groups(&contract.category);
        let names: Vec<&str> = families.iter().map(|(name, _)| name.as_str()).collect();
        assert!(names.contains(&"陈述型"));
        assert!(names.contains(&"程序型"));
    }

    #[test]
    fn 目录根取第一段() {
        assert_eq!(root_of("data/report/"), "data");
        assert_eq!(root_of("docs/bylaw/"), "docs");
        assert_eq!(root_of("packages/"), "packages");
    }

    #[test]
    fn 同目录列出兄弟资产() {
        let contract = Contract::bundled();
        let report = contract.category("report").unwrap();
        let siblings: Vec<&str> = contract
            .category
            .iter()
            .filter(|other| root_of(&other.path) == "data" && other.name != "report")
            .map(|other| other.name.as_str())
            .collect();
        assert!(siblings.contains(&"insight"));
        assert!(!siblings.contains(&"report"));
        let _ = report;
    }
}
