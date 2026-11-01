//! `asset axiom`：看资产领域的前提——管这个领域具体怎么运转的那些。
//!
//! 公理在本目录 `default-contract.toml` 的 `[[axiom]]` 段，都是 `domain = "asset"`。
//! 归位判断的前提随 `meta` 子命令淘汰，不再进这张表。这里只读，不标层。

use std::path::Path;

use clap::Args;
use serde_json::json;

use super::contract::{Axiom, Contract};

#[derive(Args, Debug)]
pub struct AxiomArgs {
    /// 只看一条公理
    pub name: Option<String>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &AxiomArgs, rules: Option<&Path>) -> Result<i32, String> {
    let contract = Contract::load(rules)?;
    let layer: Vec<&Axiom> = contract.axiom.iter().collect();
    if layer.is_empty() {
        return Err("契约里没有资产领域的公理：在 [[axiom]] 里补条目".to_string());
    }
    let axioms: Vec<&Axiom> = match &args.name {
        Some(name) => vec![axiom_of(&layer, name)?],
        None => ordered(&layer),
    };
    if args.json {
        print_json(&axioms);
    } else {
        print_text(&axioms);
    }
    Ok(0)
}

/// 公理按状态排：确认的在前，同组内保持契约里的顺序。
fn ordered<'a>(layer: &[&'a Axiom]) -> Vec<&'a Axiom> {
    let mut axioms: Vec<&Axiom> = layer.to_vec();
    axioms.sort_by_key(|axiom| axiom.status_key() != "confirmed");
    axioms
}

fn axiom_of<'a>(layer: &[&'a Axiom], name: &str) -> Result<&'a Axiom, String> {
    layer
        .iter()
        .find(|axiom| axiom.name == name)
        .copied()
        .ok_or_else(|| {
            let names: Vec<&str> = layer.iter().map(|axiom| axiom.name.as_str()).collect();
            format!("没有这条公理：{name}；可选：{}", names.join("、"))
        })
}

fn print_text(axioms: &[&Axiom]) {
    for (index, axiom) in axioms.iter().enumerate() {
        if index > 0 {
            println!();
        }
        println!(
            "{}  {}  {}",
            axiom.name,
            axiom.status_label(),
            axiom.description,
        );
        if !axiom.why.is_empty() {
            println!("  {}", axiom.why);
        }
    }
}

fn print_json(axioms: &[&Axiom]) {
    let items: Vec<serde_json::Value> = axioms
        .iter()
        .map(|axiom| {
            json!({
                "name": axiom.name,
                "description": axiom.description,
                "why": axiom.why,
                "status": axiom.status_key(),
                "status_label": axiom.status_label(),
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
    fn 公理每条都有出处() {
        let contract = Contract::bundled();
        assert!(!contract.axiom.is_empty());
        for axiom in &contract.axiom {
            assert!(!axiom.name.is_empty());
            assert!(
                !axiom.description.is_empty(),
                "{} 缺 description",
                axiom.name
            );
            assert!(!axiom.why.is_empty(), "{} 缺 why", axiom.name);
        }
    }

    #[test]
    fn 确认的公理排在前面() {
        let contract = Contract::bundled();
        let axioms: Vec<&Axiom> = contract.axiom.iter().collect();
        let ordered = ordered(&axioms);
        let first_suspected = ordered
            .iter()
            .position(|axiom| axiom.status_key() != "confirmed");
        if let Some(cut) = first_suspected {
            assert!(
                ordered[cut..]
                    .iter()
                    .all(|axiom| axiom.status_key() != "confirmed"),
                "确认的排在怀疑后面了"
            );
        }
        assert_eq!(ordered.len(), axioms.len(), "排序不丢条目");
    }

    #[test]
    fn 未知公理会报可选项() {
        let contract = Contract::bundled();
        let layer: Vec<&Axiom> = contract.axiom.iter().collect();
        let error = axiom_of(&layer, "nope").unwrap_err();
        assert!(error.contains("context-first"));
    }

    #[test]
    fn 公理都是资产领域() {
        let contract = Contract::bundled();
        for axiom in &contract.axiom {
            assert_eq!(
                axiom.domain.trim(),
                "asset",
                "{} 不是资产领域公理",
                axiom.name
            );
        }
    }
}
