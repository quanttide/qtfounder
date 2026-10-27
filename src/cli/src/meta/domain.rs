//! `meta domain`：看本体论根——这个域里有什么、能说什么、认哪些前提。
//!
//! 一个域由三样组成：范畴（有哪些类目）、属性（每个类目上有什么字段）、公理（哪些前提不再论证）。
//! 这是本体论视角；同一批类目的结构视角看 `meta paradigm`，`model` 则是范畴作为点的读法。

use std::path::PathBuf;

use clap::Args;
use serde_json::json;

use super::contract::Contract;

#[derive(Args, Debug)]
pub struct DomainArgs {
    /// 契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &DomainArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    if contract.domain.trim().is_empty() {
        return Err("契约里没有域：在 [domain] 里写上这份契约属于哪个域".to_string());
    }
    let groups = property_groups(&contract);
    let confirmed = contract
        .axiom
        .iter()
        .filter(|axiom| axiom.status_key() == "confirmed")
        .count();

    if args.json {
        let value = json!({
            "domain": contract.domain,
            "categories": contract.category.iter().map(|category| category.name.clone()).collect::<Vec<_>>(),
            "properties": groups
                .iter()
                .map(|(group, count)| json!({"group": group, "count": count}))
                .collect::<Vec<_>>(),
            "axioms": {
                "total": contract.axiom.len(),
                "confirmed": confirmed,
                "suspected": contract.axiom.len() - confirmed,
            },
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(0);
    }

    println!("域：{}", contract.domain);
    println!();
    println!("范畴（{}）", contract.category.len());
    println!(
        "  {}",
        contract
            .category
            .iter()
            .map(|category| category.name.as_str())
            .collect::<Vec<_>>()
            .join("、")
    );
    println!();
    println!("属性（{}）", contract.property.len());
    for (group, count) in &groups {
        println!("  {group} {count}");
    }
    println!();
    println!("公理（{}）", contract.axiom.len());
    println!(
        "  确认 {confirmed}，怀疑 {}",
        contract.axiom.len() - confirmed
    );
    Ok(0)
}

/// 属性按组计数，组的先后照契约。
fn property_groups(contract: &Contract) -> Vec<(String, usize)> {
    let mut groups: Vec<(String, usize)> = Vec::new();
    for property in &contract.property {
        match groups
            .iter_mut()
            .find(|(group, _)| *group == property.group)
        {
            Some((_, count)) => *count += 1,
            None => groups.push((property.group.clone(), 1)),
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 元契约的域是_meta() {
        let contract = Contract::bundled();
        assert_eq!(contract.domain, "meta");
    }

    #[test]
    fn 属性分组按契约顺序() {
        let contract = Contract::bundled();
        let groups = property_groups(&contract);
        assert_eq!(
            groups.first().map(|(group, _)| group.as_str()),
            Some("标识")
        );
        assert_eq!(groups.iter().map(|(_, count)| count).sum::<usize>(), 19);
    }
}
