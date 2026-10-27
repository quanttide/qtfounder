//! `meta category`：看十八格与两端——范畴与位置各收什么。
//!
//! 只读契约。`meta classify` 是把内容放进某一格，这里只看格有哪些、各收什么。

use std::path::PathBuf;

use clap::Args;
use serde_json::json;

use super::contract::{Asset, Category, Contract};

#[derive(Args, Debug)]
pub struct CategoryArgs {
    /// 只看一个格，或一个端（context、archive）
    pub name: Option<String>,
    /// 契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug)]
enum Found<'a> {
    Category(&'a Category),
    End(&'a Asset),
}

pub fn run(args: &CategoryArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    let (categories, ends): (Vec<&Category>, Vec<&Asset>) = match &args.name {
        Some(name) => match lookup(&contract, name)? {
            Found::Category(category) => (vec![category], Vec::new()),
            Found::End(asset) => (Vec::new(), vec![asset]),
        },
        None => (contract.category.iter().collect(), ends(&contract)),
    };
    if args.json {
        print_json(&categories, &ends);
    } else {
        print_text(&categories, &ends);
    }
    Ok(0)
}

/// 两端：context（入口）与 archive（出口），顺序照契约。
pub fn ends<'a>(contract: &'a Contract) -> Vec<&'a Asset> {
    contract
        .asset
        .iter()
        .filter(|asset| !asset.is_memory())
        .collect()
}

fn lookup<'a>(contract: &'a Contract, name: &str) -> Result<Found<'a>, String> {
    if let Some(category) = contract.category(name) {
        return Ok(Found::Category(category));
    }
    if let Some(asset) = contract.asset(name).filter(|asset| !asset.is_memory()) {
        return Ok(Found::End(asset));
    }
    let names: Vec<&str> = contract
        .category
        .iter()
        .map(|category| category.name.as_str())
        .chain(ends(contract).into_iter().map(|asset| asset.name.as_str()))
        .collect();
    Err(format!("没有这一格：{name}；可选：{}", names.join("、")))
}

/// 按族分组，组的先后与格内顺序都照契约。
#[cfg(test)]
pub fn family_groups<'a>(contract: &'a Contract) -> Vec<(&'a str, Vec<&'a Category>)> {
    let mut groups: Vec<(&str, Vec<&Category>)> = Vec::new();
    for category in &contract.category {
        let family = category.family.as_str();
        match groups.iter_mut().find(|(name, _)| *name == family) {
            Some((_, categories)) => categories.push(category),
            None => groups.push((family, vec![category])),
        }
    }
    groups
}

fn role_label(asset: &Asset) -> &'static str {
    if asset.is_source() {
        "入口"
    } else if asset.is_exit() {
        "出口"
    } else {
        "常驻"
    }
}

fn print_text(categories: &[&Category], ends: &[&Asset]) {
    let mut last_family = String::new();
    for (index, category) in categories.iter().enumerate() {
        if category.family != last_family {
            if index > 0 {
                println!();
            }
            let label = if category.family.is_empty() {
                "未分族"
            } else {
                category.family.as_str()
            };
            println!("{label}");
            last_family = category.family.clone();
        }
        println!("  {}  {}  {}", category.name, category.label, category.path);
        if !category.description.is_empty() {
            println!("    {}", category.description);
        }
    }
    if ends.is_empty() {
        return;
    }
    if !categories.is_empty() {
        println!();
    }
    println!("两端");
    for asset in ends {
        println!(
            "  {}  {}  {}  {}",
            asset.name,
            asset.label(),
            asset.path,
            role_label(asset)
        );
        if !asset.description.is_empty() {
            println!("    {}", asset.description);
        }
    }
}

fn print_json(categories: &[&Category], ends: &[&Asset]) {
    let mut items: Vec<serde_json::Value> = categories
        .iter()
        .map(|category| {
            json!({
                "name": category.name,
                "label": category.label,
                "path": category.path,
                "family": category.family,
                "description": category.description,
            })
        })
        .collect();
    for asset in ends {
        items.push(json!({
            "name": asset.name,
            "label": asset.label(),
            "path": asset.path,
            "family": "两端",
            "description": asset.description,
            "role": asset.role,
        }));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_string())
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 两族各九格() {
        let contract = Contract::bundled();
        let groups = family_groups(&contract);
        assert_eq!(groups.len(), 2);
        for (family, categories) in groups {
            assert_eq!(categories.len(), 9, "{family}");
        }
    }

    #[test]
    fn 两端各收什么() {
        let contract = Contract::bundled();
        let ends = ends(&contract);
        assert_eq!(ends.len(), 2);
        assert!(ends.iter().any(|asset| asset.is_source()));
        assert!(ends.iter().any(|asset| asset.is_exit()));
        for asset in ends {
            assert!(!asset.description.is_empty(), "{} 缺收什么", asset.name);
        }
    }

    #[test]
    fn 按名字能查到两端() {
        let contract = Contract::bundled();
        assert!(matches!(lookup(&contract, "context"), Ok(Found::End(_))));
        assert!(matches!(lookup(&contract, "archive"), Ok(Found::End(_))));
        assert!(matches!(
            lookup(&contract, "insight"),
            Ok(Found::Category(_))
        ));
    }

    #[test]
    fn 未知格会报可选项() {
        let contract = Contract::bundled();
        let error = lookup(&contract, "nope").unwrap_err();
        assert!(error.contains("insight"), "{error}");
        assert!(error.contains("archive"), "{error}");
    }
}
