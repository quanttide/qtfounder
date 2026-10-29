//! 资产关系：二十种资产与迁入边。
//!
//! 资产分三种角色：context 是入口，十八类是长期记忆，archive 是出口。
//! 关系回答内容从哪搬到哪，`flip` 交换前两段，`same` 保持。

use std::path::Path;

use clap::Args;
use serde_json::json;

use crate::meta::contract::{Contract, Relation};

#[derive(Args, Debug)]
pub struct RelationArgs {
    /// 只看这种资产的关系
    pub name: Option<String>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

/// 目录根：路径的第一段。
pub fn root_of(path: &str) -> &str {
    path.split('/').next().unwrap_or(path)
}

fn role_label(role: &str) -> &'static str {
    match role {
        "source" => "入口",
        "memory" => "长期记忆",
        "exit" => "归档",
        _ => "未分角色",
    }
}

pub fn run(args: &RelationArgs, rules: Option<&Path>) -> Result<i32, String> {
    let contract = Contract::load(rules)?;
    if let Some(name) = &args.name {
        return show_one(name, &contract, args.json);
    }
    if args.json {
        print_json(&contract);
    } else {
        print_text(&contract);
    }
    Ok(0)
}

fn print_text(contract: &Contract) {
    for role in ["source", "memory", "exit"] {
        let assets: Vec<&crate::meta::contract::Asset> = contract
            .asset
            .iter()
            .filter(|asset| asset.role == role)
            .collect();
        if assets.is_empty() {
            continue;
        }
        println!("{}（{}）", role_label(role), assets.len());
        for asset in assets {
            println!("  {}  {}  {}", asset.name, asset.path, asset.category);
        }
        println!();
    }
    println!("关系");
    for relation in &contract.relation {
        println!(
            "  {}  {} -> {}  {}",
            relation.kind, relation.from, relation.to, relation.transform
        );
    }
}

fn print_json(contract: &Contract) {
    let assets: Vec<serde_json::Value> = contract
        .asset
        .iter()
        .map(|asset| {
            json!({
                "name": asset.name,
                "label": asset.label(),
                "role": asset.role,
                "path": asset.path,
                "category": asset.category,
            })
        })
        .collect();
    let relations: Vec<serde_json::Value> = contract
        .relation
        .iter()
        .map(|relation| {
            json!({
                "kind": relation.kind,
                "from": relation.from,
                "to": relation.to,
                "transform": relation.transform,
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "assets": assets,
            "relations": relations,
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
}

fn show_one(name: &str, contract: &Contract, json: bool) -> Result<i32, String> {
    let asset = contract.asset.iter().find(|asset| asset.name == name);
    let Some(asset) = asset else {
        let names: Vec<&str> = contract
            .asset
            .iter()
            .map(|asset| asset.name.as_str())
            .collect();
        return Err(format!("没有这种资产：{name}；可选：{}", names.join("、")));
    };
    let edges: Vec<&Relation> = contract
        .relation
        .iter()
        .filter(|relation| relation.from == name || relation.to == name)
        .collect();
    let category = contract
        .category(&asset.category)
        .filter(|_| !asset.category.is_empty());
    let siblings = category.map(|category| {
        contract
            .category
            .iter()
            .filter(|other| {
                root_of(&other.path) == root_of(&category.path) && other.name != category.name
            })
            .map(|other| other.name.as_str())
            .collect::<Vec<&str>>()
    });

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "asset": asset.name,
                "label": asset.label(),
                "role": asset.role,
                "path": asset.path,
                "category": asset.category,
                "family": category.map(|category| category.family.clone()),
                "siblings": siblings,
                "relations": edges.iter().map(|relation| json!({
                    "kind": relation.kind,
                    "from": relation.from,
                    "to": relation.to,
                    "transform": relation.transform,
                })).collect::<Vec<_>>(),
            }))
            .unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(0);
    }

    println!(
        "{}  {}  {}",
        asset.name,
        asset.label(),
        role_label(&asset.role)
    );
    println!("  路径：{}", asset.path);
    if !asset.category.is_empty() {
        println!("  类别：{}", asset.category);
    }
    if let Some(category) = category {
        println!("  家族：{}", category.family);
        if let Some(siblings) = &siblings {
            println!("  同目录：{}", siblings.join("、"));
        }
    }
    println!("  关系：");
    for relation in edges {
        println!(
            "    {}  {} -> {}  {}",
            relation.kind, relation.from, relation.to, relation.transform
        );
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn 关系覆盖入口长期记忆与归档() {
        let contract = Contract::bundled();
        assert_eq!(contract.asset.len(), 22);
        assert!(
            contract
                .relation
                .iter()
                .any(|relation| relation.kind == "intake")
        );
        assert!(
            contract
                .relation
                .iter()
                .any(|relation| relation.kind == "archive")
        );
    }
}
