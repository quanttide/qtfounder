//! 资产契约：每类资产该写成什么样。
//!
//! 数据源是归位契约的十八格。这里不重复定义资产，只把契约里的边界、分档与写法摊开。

use std::path::Path;

use clap::Args;
use serde_json::json;

use crate::meta::contract::{Cell, Contract};

#[derive(Args, Debug)]
pub struct ContractArgs {
    /// 只看一类资产
    pub key: Option<String>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &ContractArgs, rules: Option<&Path>) -> Result<i32, String> {
    let contract = Contract::load(rules)?;
    let cells: Vec<&Cell> = match &args.key {
        Some(key) => vec![cell_of(&contract, key)?],
        None => contract.cell.iter().collect(),
    };
    if args.json {
        print_json(&cells);
    } else {
        print_text(&cells);
    }
    Ok(0)
}

fn cell_of<'a>(contract: &'a Contract, key: &str) -> Result<&'a Cell, String> {
    contract.cell(key).ok_or_else(|| {
        let keys: Vec<&str> = contract.cell.iter().map(|cell| cell.key.as_str()).collect();
        format!("没有这类资产：{key}；可选：{}", keys.join("、"))
    })
}

fn grade_line(cell: &Cell) -> String {
    if cell.grade.is_empty() {
        return String::new();
    }
    cell.grade
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

fn print_text(cells: &[&Cell]) {
    for (index, cell) in cells.iter().enumerate() {
        if index > 0 {
            println!();
        }
        println!(
            "{}  {}  {}  {}",
            cell.key, cell.name, cell.path, cell.family
        );
        if !cell.boundary.is_empty() {
            println!("  边界：{}", cell.boundary);
        }
        let grades = grade_line(cell);
        if !grades.is_empty() {
            println!("  分档：{grades}");
        }
        if !cell.skeleton.trim().is_empty() {
            println!("  写法：");
            for line in cell.skeleton.lines() {
                if line.is_empty() {
                    println!();
                } else {
                    println!("    {line}");
                }
            }
        }
    }
}

fn print_json(cells: &[&Cell]) {
    let items: Vec<serde_json::Value> = cells
        .iter()
        .map(|cell| {
            let grades: Vec<serde_json::Value> = cell
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
                "key": cell.key,
                "name": cell.name,
                "path": cell.path,
                "family": cell.family,
                "boundary": cell.boundary,
                "skeleton": cell.skeleton,
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
        for cell in &contract.cell {
            assert!(!cell.key.is_empty());
            assert!(!cell.path.is_empty());
            assert!(!cell.boundary.is_empty(), "{} 缺边界", cell.key);
            assert!(!cell.skeleton.trim().is_empty(), "{} 缺写法", cell.key);
        }
    }

    #[test]
    fn 未知资产会报可选项() {
        let contract = Contract::bundled();
        let error = cell_of(&contract, "nope").unwrap_err();
        assert!(error.contains("report"));
    }

    #[test]
    fn 分档写明默认项() {
        let contract = Contract::bundled();
        let insight = contract.cell("insight").unwrap();
        let line = grade_line(insight);
        assert!(line.contains("假说"));
        assert!(line.contains("默认"));
    }
}
