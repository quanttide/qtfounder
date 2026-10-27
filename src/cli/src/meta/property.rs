//! `meta property`：看标准属性——字段名、所属组与定义。
//!
//! 定义写在契约的 `[[property]]` 里，这里只读。`meta validate` 拿它跟 docs 镜像逐条对。

use std::path::PathBuf;

use clap::Args;
use serde_json::json;

use super::contract::{Contract, Property};

#[derive(Args, Debug)]
pub struct PropertyArgs {
    /// 只看一个属性
    pub name: Option<String>,
    /// 契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &PropertyArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    if contract.property.is_empty() {
        return Err("契约里没有属性：在 [[property]] 里补".to_string());
    }
    let properties: Vec<&Property> = match &args.name {
        Some(name) => vec![property_of(&contract, name)?],
        None => grouped(&contract).into_iter().flatten().collect(),
    };
    if args.json {
        print_json(&properties);
    } else {
        print_text(&properties);
    }
    Ok(0)
}

/// 按组排：组的先后照契约，组内确认的在前。
pub fn grouped<'a>(contract: &'a Contract) -> Vec<Vec<&'a Property>> {
    let mut groups: Vec<Vec<&Property>> = Vec::new();
    for property in &contract.property {
        match groups
            .iter_mut()
            .find(|group| group.first().map(|first| &first.group) == Some(&property.group))
        {
            Some(group) => group.push(property),
            None => groups.push(vec![property]),
        }
    }
    for group in &mut groups {
        group.sort_by_key(|property| property.status_key() != "confirmed");
    }
    groups
}

pub fn property_of<'a>(contract: &'a Contract, name: &str) -> Result<&'a Property, String> {
    contract
        .property
        .iter()
        .find(|property| property.name == name)
        .ok_or_else(|| {
            let names: Vec<&str> = contract
                .property
                .iter()
                .map(|property| property.name.as_str())
                .collect();
            format!("没有这个属性：{name}；可选：{}", names.join("、"))
        })
}

fn group_label(property: &Property) -> String {
    if property.group.is_empty() {
        "未分组".to_string()
    } else {
        format!("{}属性", property.group)
    }
}

fn print_text(properties: &[&Property]) {
    let mut last_group = String::new();
    for (index, property) in properties.iter().enumerate() {
        let group = group_label(property);
        if group != last_group {
            if index > 0 {
                println!();
            }
            println!("{group}");
            last_group = group;
        }
        println!(
            "  {}  {}  {}",
            property.name,
            property.status_label(),
            property.description
        );
    }
}

fn print_json(properties: &[&Property]) {
    let items: Vec<serde_json::Value> = properties
        .iter()
        .map(|property| {
            json!({
                "name": property.name,
                "group": property.group,
                "description": property.description,
                "status": property.status_key(),
                "status_label": property.status_label(),
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
    fn 每组非空且确认的在前() {
        let contract = Contract::bundled();
        let groups = grouped(&contract);
        assert!(!groups.is_empty());
        for group in groups {
            let mut seen_suspected = false;
            for property in group {
                if property.status_key() == "confirmed" {
                    assert!(!seen_suspected, "{} 排在怀疑后面", property.name);
                } else {
                    seen_suspected = true;
                }
            }
        }
    }

    #[test]
    fn 未知属性会报可选项() {
        let contract = Contract::bundled();
        let error = property_of(&contract, "nope").unwrap_err();
        assert!(error.contains("id"), "{error}");
    }
}
