//! `meta model`：看两族九宫格的轴。
//!
//! 模型只描述本体，不参与判定。族的成员来自 `[[cell]]` 的 `family`，
//! 轴来自 `[[model.family.axis]]`；两处对不上时在契约加载阶段就报错。

use std::path::PathBuf;

use clap::Args;
use serde_json::json;

use super::contract::Contract;

#[derive(Args, Debug)]
pub struct ModelArgs {
    /// 契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn cells_of<'a>(contract: &'a Contract, family: &str) -> Vec<&'a str> {
    contract
        .cell
        .iter()
        .filter(|cell| cell.family == family)
        .map(|cell| cell.name.as_str())
        .collect()
}

pub fn run(args: &ModelArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    if contract.model.family.is_empty() {
        return Err("契约里没有本体模型：在 [model] 与 [[model.family]] 里补".to_string());
    }
    if args.json {
        print_json(&contract);
    } else {
        print_text(&contract);
    }
    Ok(0)
}

fn print_text(contract: &Contract) {
    println!("归位本体");
    if !contract.model.description.is_empty() {
        println!("{}", contract.model.description);
    }
    for family in &contract.model.family {
        println!();
        println!("{}  {}", family.name, family.question);
        if !family.description.is_empty() {
            println!("  {}", family.description);
        }
        for axis in &family.axis {
            println!("  {}：{}", axis.name, axis.values.join(" / "));
        }
        let cells = cells_of(contract, &family.name);
        println!("  九格：{}", cells.join("、"));
    }
    println!();
    if contract.domain.is_empty() {
        println!("领域：（未登记）");
    } else {
        println!("领域（{}）", contract.domain.len());
        for domain in &contract.domain {
            println!("  {}  {}", domain.name, domain.label());
        }
    }
}

fn print_json(contract: &Contract) {
    let families: Vec<serde_json::Value> = contract
        .model
        .family
        .iter()
        .map(|family| {
            let axes: Vec<serde_json::Value> = family
                .axis
                .iter()
                .map(|axis| json!({"name": axis.name, "values": axis.values}))
                .collect();
            json!({
                "name": family.name,
                "question": family.question,
                "description": family.description,
                "axes": axes,
                "cells": cells_of(contract, &family.name),
            })
        })
        .collect();
    let domains: Vec<serde_json::Value> = contract
        .domain
        .iter()
        .map(|domain| json!({"name": domain.name, "label": domain.label()}))
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "description": contract.model.description,
            "families": families,
            "domains": domains,
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 两族都有九格() {
        let contract = Contract::bundled();
        for family in &contract.model.family {
            assert_eq!(
                cells_of(&contract, &family.name).len(),
                9,
                "{}",
                family.name
            );
        }
    }

    #[test]
    fn 每族两条轴() {
        let contract = Contract::bundled();
        for family in &contract.model.family {
            assert_eq!(family.axis.len(), 2, "{}", family.name);
            for axis in &family.axis {
                assert_eq!(axis.values.len(), 3, "{} 的 {}", family.name, axis.name);
            }
        }
    }

    #[test]
    fn 领域也列进模型() {
        let contract = Contract::bundled();
        assert!(!contract.domain.is_empty());
    }
}
