//! `meta axiom`：看归位时不重新论证的前提。
//!
//! 公理只描述系统已经成立的前提，不参与判定。判定仍看 `[[cell]]`、
//! `[[transition]]` 与 `[[question]]`。

use std::path::PathBuf;

use clap::Args;
use serde_json::json;

use super::contract::{Axiom, Contract};

#[derive(Args, Debug)]
pub struct AxiomArgs {
    /// 只看一条公理
    pub id: Option<String>,
    /// 契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &AxiomArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    if contract.axiom.is_empty() {
        return Err("契约里没有公理：在 [[axiom]] 里补".to_string());
    }
    let axioms: Vec<&Axiom> = match &args.id {
        Some(id) => vec![axiom_of(&contract, id)?],
        None => contract.axiom.iter().collect(),
    };
    if args.json {
        print_json(&axioms);
    } else {
        print_text(&axioms);
    }
    Ok(0)
}

fn axiom_of<'a>(contract: &'a Contract, name: &str) -> Result<&'a Axiom, String> {
    contract
        .axiom
        .iter()
        .find(|axiom| axiom.name == name)
        .ok_or_else(|| {
            let names: Vec<&str> = contract
                .axiom
                .iter()
                .map(|axiom| axiom.name.as_str())
                .collect();
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
    fn 公理缺省按怀疑算() {
        let axiom = Axiom {
            name: "x".to_string(),
            description: "甲".to_string(),
            why: "乙".to_string(),
            status: String::new(),
        };
        assert_eq!(axiom.status_key(), "suspected");
        assert_eq!(axiom.status_label(), "怀疑");
    }

    #[test]
    fn 未知公理会报可选项() {
        let contract = Contract::bundled();
        let error = axiom_of(&contract, "nope").unwrap_err();
        assert!(error.contains("one-place"));
    }
}
