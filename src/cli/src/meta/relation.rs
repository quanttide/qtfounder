//! `meta relation`：归位边——迁移与分辨句。
//!
//! 迁移是单向的降格或升格；分辨句是两格打平时的问话。两者都只读契约。

use std::path::PathBuf;

use clap::Args;
use serde_json::json;

use super::contract::{Cell, Contract, Transition};

#[derive(Args, Debug)]
pub struct RelationArgs {
    /// 只看与这一格有关的迁移与分辨句
    pub cell: Option<String>,
    /// 契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

/// 与某一格有关的边。
pub struct Edges<'a> {
    pub outgoing: Vec<&'a Transition>,
    pub incoming: Vec<&'a Transition>,
    pub questions: Vec<(&'a str, &'a str)>,
}

pub fn edges<'a>(contract: &'a Contract, key: &str) -> Edges<'a> {
    let outgoing = contract
        .transition
        .iter()
        .filter(|transition| transition.from == key)
        .collect();
    let incoming = contract
        .transition
        .iter()
        .filter(|transition| transition.to == key)
        .collect();
    let questions = contract
        .question
        .iter()
        .filter_map(|question| {
            if question.pair.len() != 2 || !question.pair.iter().any(|pair| pair == key) {
                return None;
            }
            let other = if question.pair[0] == key {
                question.pair[1].as_str()
            } else {
                question.pair[0].as_str()
            };
            Some((other, question.ask.as_str()))
        })
        .collect();
    Edges {
        outgoing,
        incoming,
        questions,
    }
}

pub fn run(args: &RelationArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    match &args.cell {
        Some(key) => {
            let cell = contract.cell(key).ok_or_else(|| {
                let names: Vec<&str> = contract
                    .cell
                    .iter()
                    .map(|cell| cell.name.as_str())
                    .collect();
                format!("没有这一格：{key}；可选：{}", names.join("、"))
            })?;
            if args.json {
                print_one_json(cell, &contract);
            } else {
                print_one(cell, &contract);
            }
        }
        None => {
            if args.json {
                print_all_json(&contract);
            } else {
                print_all(&contract);
            }
        }
    }
    Ok(0)
}

fn print_all(contract: &Contract) {
    println!("迁移（{}）", contract.transition.len());
    for transition in &contract.transition {
        println!(
            "  {} → {}  {}",
            transition.from, transition.to, transition.reason
        );
    }
    println!();
    println!("分辨（{}）", contract.question.len());
    for question in &contract.question {
        if question.pair.len() == 2 {
            println!("  {} ↔ {}", question.pair[0], question.pair[1]);
            println!("    {}", question.ask);
        }
    }
}

fn print_one(cell: &Cell, contract: &Contract) {
    let edges = edges(contract, &cell.name);
    println!("{}  {}", cell.name, cell.label);
    println!("  迁出：");
    if edges.outgoing.is_empty() {
        println!("    （无）");
    }
    for transition in &edges.outgoing {
        println!("    → {}  {}", transition.to, transition.reason);
    }
    println!("  迁入：");
    if edges.incoming.is_empty() {
        println!("    （无）");
    }
    for transition in &edges.incoming {
        println!("    ← {}  {}", transition.from, transition.reason);
    }
    println!("  分辨：");
    if edges.questions.is_empty() {
        println!("    （无）");
    }
    for (other, ask) in &edges.questions {
        println!("    ↔ {other}");
        println!("      {ask}");
    }
}

fn print_all_json(contract: &Contract) {
    let transitions: Vec<serde_json::Value> = contract
        .transition
        .iter()
        .map(|transition| {
            json!({
                "from": transition.from,
                "to": transition.to,
                "reason": transition.reason,
            })
        })
        .collect();
    let questions: Vec<serde_json::Value> = contract
        .question
        .iter()
        .filter(|question| question.pair.len() == 2)
        .map(|question| {
            json!({
                "pair": question.pair,
                "ask": question.ask,
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "transitions": transitions,
            "questions": questions,
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
}

fn print_one_json(cell: &Cell, contract: &Contract) {
    let edges = edges(contract, &cell.name);
    let outgoing: Vec<serde_json::Value> = edges
        .outgoing
        .iter()
        .map(|transition| json!({"to": transition.to, "reason": transition.reason}))
        .collect();
    let incoming: Vec<serde_json::Value> = edges
        .incoming
        .iter()
        .map(|transition| json!({"from": transition.from, "reason": transition.reason}))
        .collect();
    let questions: Vec<serde_json::Value> = edges
        .questions
        .iter()
        .map(|(other, ask)| json!({"with": other, "ask": ask}))
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "cell": cell.name,
            "outgoing": outgoing,
            "incoming": incoming,
            "questions": questions,
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 迁移分进出两个方向() {
        let contract = Contract::bundled();
        let insight = edges(&contract, "insight");
        assert!(insight.outgoing.iter().any(|edge| edge.to == "essay"));
        assert!(insight.incoming.is_empty());

        let essay = edges(&contract, "essay");
        assert!(essay.incoming.iter().any(|edge| edge.from == "insight"));
        assert!(
            essay
                .incoming
                .iter()
                .any(|edge| edge.from == "specification")
        );
    }

    #[test]
    fn 分辨句按对端列出() {
        let contract = Contract::bundled();
        let edges = edges(&contract, "insight");
        let others: Vec<&str> = edges.questions.iter().map(|(other, _)| *other).collect();
        assert!(others.contains(&"roadmap"));
        assert!(others.contains(&"profile"));
    }

    #[test]
    fn 没有边的格是空() {
        let contract = Contract::bundled();
        let edges = edges(&contract, "brochure");
        assert!(edges.outgoing.is_empty());
        assert!(edges.incoming.is_empty());
        assert!(edges.questions.is_empty());
    }
}
