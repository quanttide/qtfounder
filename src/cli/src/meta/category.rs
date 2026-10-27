//! `meta category`：看十八格——范畴本身长什么样。
//!
//! 只读契约。`meta classify` 是把内容放进某一格，这里只看格有哪些、各收什么。

use std::path::PathBuf;

use clap::Args;
use serde_json::json;

use super::contract::{Cell, Contract};

#[derive(Args, Debug)]
pub struct CategoryArgs {
    /// 只看一个格
    pub name: Option<String>,
    /// 契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &CategoryArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    let cells: Vec<&Cell> = match &args.name {
        Some(name) => vec![category_of(&contract, name)?],
        None => contract.cell.iter().collect(),
    };
    if args.json {
        print_json(&cells);
    } else {
        print_text(&cells);
    }
    Ok(0)
}

pub fn category_of<'a>(contract: &'a Contract, name: &str) -> Result<&'a Cell, String> {
    contract.cell(name).ok_or_else(|| {
        let names: Vec<&str> = contract
            .cell
            .iter()
            .map(|cell| cell.name.as_str())
            .collect();
        format!("没有这一格：{name}；可选：{}", names.join("、"))
    })
}

/// 按族分组，组的先后与格内顺序都照契约。
#[cfg(test)]
pub fn family_groups<'a>(contract: &'a Contract) -> Vec<(&'a str, Vec<&'a Cell>)> {
    let mut groups: Vec<(&str, Vec<&Cell>)> = Vec::new();
    for cell in &contract.cell {
        let family = cell.family.as_str();
        match groups.iter_mut().find(|(name, _)| *name == family) {
            Some((_, cells)) => cells.push(cell),
            None => groups.push((family, vec![cell])),
        }
    }
    groups
}

fn print_text(cells: &[&Cell]) {
    let mut last_family = String::new();
    for (index, cell) in cells.iter().enumerate() {
        if cell.family != last_family {
            if index > 0 {
                println!();
            }
            let label = if cell.family.is_empty() {
                "未分族"
            } else {
                cell.family.as_str()
            };
            println!("{label}");
            last_family = cell.family.clone();
        }
        println!("  {}  {}  {}", cell.name, cell.label, cell.path);
        if !cell.description.is_empty() {
            println!("    {}", cell.description);
        }
    }
}

fn print_json(cells: &[&Cell]) {
    let items: Vec<serde_json::Value> = cells
        .iter()
        .map(|cell| {
            json!({
                "name": cell.name,
                "label": cell.label,
                "path": cell.path,
                "family": cell.family,
                "description": cell.description,
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
    fn 两族各九格() {
        let contract = Contract::bundled();
        let groups = family_groups(&contract);
        assert_eq!(groups.len(), 2);
        for (family, cells) in groups {
            assert_eq!(cells.len(), 9, "{family}");
        }
    }

    #[test]
    fn 未知格会报可选项() {
        let contract = Contract::bundled();
        let error = category_of(&contract, "nope").unwrap_err();
        assert!(error.contains("insight"), "{error}");
    }
}
