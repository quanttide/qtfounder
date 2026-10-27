//! `meta domain`：看登记的领域——内容搬出本仓的去处。
//!
//! 只读契约。`classify` 判域时用同一份名单；这里只看名单本身。

use std::path::PathBuf;

use clap::Args;
use serde_json::json;

use super::contract::{Contract, Domain};

#[derive(Args, Debug)]
pub struct DomainArgs {
    /// 只看一个领域
    pub name: Option<String>,
    /// 契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &DomainArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    if contract.domain.is_empty() {
        return Err("契约里没有领域：在 [[domain]] 里补".to_string());
    }
    let domains: Vec<&Domain> = match &args.name {
        Some(name) => vec![domain_of(&contract, name)?],
        None => ordered(&contract),
    };
    if args.json {
        print_json(&domains);
    } else {
        print_text(&domains);
    }
    Ok(0)
}

/// 领域按状态排：确认的在前，同组内保持契约顺序。
pub fn ordered(contract: &Contract) -> Vec<&Domain> {
    let mut domains: Vec<&Domain> = contract.domain.iter().collect();
    domains.sort_by_key(|domain| domain.status_key() != "confirmed");
    domains
}

pub fn domain_of<'a>(contract: &'a Contract, name: &str) -> Result<&'a Domain, String> {
    contract
        .domain
        .iter()
        .find(|domain| domain.name == name)
        .ok_or_else(|| {
            let names: Vec<&str> = contract
                .domain
                .iter()
                .map(|domain| domain.name.as_str())
                .collect();
            format!("没有这个领域：{name}；可选：{}", names.join("、"))
        })
}

fn print_text(domains: &[&Domain]) {
    for (index, domain) in domains.iter().enumerate() {
        if index > 0 {
            println!();
        }
        println!(
            "{}  {}  {}",
            domain.name,
            domain.label(),
            domain.status_label()
        );
        if !domain.cues.is_empty() {
            println!("  线索：{}", domain.cues.join("、"));
        }
    }
}

fn print_json(domains: &[&Domain]) {
    let items: Vec<serde_json::Value> = domains
        .iter()
        .map(|domain| {
            json!({
                "name": domain.name,
                "label": domain.label(),
                "cues": domain.cues,
                "status": domain.status_key(),
                "status_label": domain.status_label(),
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
    fn 六个领域且确认的在前() {
        let contract = Contract::bundled();
        let domains = ordered(&contract);
        assert_eq!(domains.len(), 6);
        let mut seen_suspected = false;
        for domain in domains {
            if domain.status_key() == "confirmed" {
                assert!(!seen_suspected, "{} 排在怀疑后面", domain.name);
            } else {
                seen_suspected = true;
            }
        }
    }

    #[test]
    fn 未知领域会报可选项() {
        let contract = Contract::bundled();
        let error = domain_of(&contract, "nope").unwrap_err();
        assert!(error.contains("quanttide-write"), "{error}");
    }
}
