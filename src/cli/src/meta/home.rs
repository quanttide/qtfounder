//! `meta home`：看去处——内容搬出本仓能去哪里。
//!
//! 只读契约。`classify` 判域时用同一份名单；这里只看名单本身。

use std::path::PathBuf;

use clap::Args;
use serde_json::json;

use super::contract::{Contract, Home};

#[derive(Args, Debug)]
pub struct HomeArgs {
    /// 只看一个去处
    pub name: Option<String>,
    /// 契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &HomeArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    if contract.home.is_empty() {
        return Err("契约里没有去处：在 [[home]] 里补".to_string());
    }
    let homes: Vec<&Home> = match &args.name {
        Some(name) => vec![home_of(&contract, name)?],
        None => ordered(&contract),
    };
    if args.json {
        print_json(&homes);
    } else {
        print_text(&homes);
    }
    Ok(0)
}

/// 去处按状态排：确认的在前，同组内保持契约顺序。
pub fn ordered(contract: &Contract) -> Vec<&Home> {
    let mut homes: Vec<&Home> = contract.home.iter().collect();
    homes.sort_by_key(|home| home.status_key() != "confirmed");
    homes
}

pub fn home_of<'a>(contract: &'a Contract, name: &str) -> Result<&'a Home, String> {
    contract
        .home
        .iter()
        .find(|home| home.name == name)
        .ok_or_else(|| {
            let names: Vec<&str> = contract
                .home
                .iter()
                .map(|home| home.name.as_str())
                .collect();
            format!("没有这个去处：{name}；可选：{}", names.join("、"))
        })
}

fn print_text(homes: &[&Home]) {
    for (index, home) in homes.iter().enumerate() {
        if index > 0 {
            println!();
        }
        println!("{}  {}  {}", home.name, home.label(), home.status_label());
        if !home.cues.is_empty() {
            println!("  线索：{}", home.cues.join("、"));
        }
    }
}

fn print_json(homes: &[&Home]) {
    let items: Vec<serde_json::Value> = homes
        .iter()
        .map(|home| {
            json!({
                "name": home.name,
                "label": home.label(),
                "cues": home.cues,
                "status": home.status_key(),
                "status_label": home.status_label(),
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
    fn 六个去处且确认的在前() {
        let contract = Contract::bundled();
        let homes = ordered(&contract);
        assert_eq!(homes.len(), 6);
        let mut seen_suspected = false;
        for home in homes {
            if home.status_key() == "confirmed" {
                assert!(!seen_suspected, "{} 排在怀疑后面", home.name);
            } else {
                seen_suspected = true;
            }
        }
    }

    #[test]
    fn 未知去处会报可选项() {
        let contract = Contract::bundled();
        let error = home_of(&contract, "nope").unwrap_err();
        assert!(error.contains("quanttide-write"), "{error}");
    }
}
