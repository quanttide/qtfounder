//! `meta axiom`：看归位时不重新论证的前提。
//!
//! 公理只描述系统已经成立的前提，不参与判定。判定仍看 `[[category]]`、
//! `[[transition]]` 与 `[[question]]`。

use std::path::PathBuf;

use clap::Args;
use serde_json::json;

use super::contract::{Axiom, Contract};

#[derive(Args, Debug)]
pub struct AxiomArgs {
    /// 只看一条公理
    pub name: Option<String>,
    /// 契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &AxiomArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    print_domain(&contract, true, args.name.as_deref(), args.json)
}

/// 按领域打公理：`want_meta` 选元模型领域，否则资产领域。
///
/// 两个领域怎么分与新增领域的规矩，见 user-guide/meta.md「两个领域的边界」。
pub fn print_domain(
    contract: &Contract,
    want_meta: bool,
    name: Option<&str>,
    json: bool,
) -> Result<i32, String> {
    let layer: Vec<&Axiom> = contract
        .axiom
        .iter()
        .filter(|axiom| axiom.is_meta() == want_meta)
        .collect();
    if layer.is_empty() {
        let which = if want_meta {
            "元模型领域"
        } else {
            "资产领域"
        };
        return Err(format!(
            "契约里没有{which}的公理：在 [[axiom]] 里补 domain = \"{}\"",
            if want_meta { "meta" } else { "asset" }
        ));
    }
    let axioms: Vec<&Axiom> = match name {
        Some(name) => vec![axiom_of(&layer, name)?],
        None => ordered(&layer),
    };
    if json {
        print_json(&axioms);
    } else {
        print_text(&axioms);
    }
    Ok(0)
}

/// 公理按状态排：确认的在前，同组内保持契约里的顺序。
pub fn ordered<'a>(layer: &[&'a Axiom]) -> Vec<&'a Axiom> {
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
            format!("没有这条公理：{name}；这一层可选：{}", names.join("、"))
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

    fn meta_domain(contract: &Contract) -> Vec<&Axiom> {
        contract
            .axiom
            .iter()
            .filter(|axiom| axiom.is_meta())
            .collect()
    }

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
            domain: String::new(),
        };
        assert_eq!(axiom.status_key(), "suspected");
        assert_eq!(axiom.status_label(), "怀疑");
        assert!(axiom.is_meta(), "domain 空按 meta 算");
    }

    #[test]
    fn 确认的公理排在前面() {
        let contract = Contract::bundled();
        let axioms = ordered(&meta_domain(&contract));
        assert_eq!(axioms.first().unwrap().name, "template-scope");
        assert_eq!(axioms.first().unwrap().status_key(), "confirmed");
        assert!(
            axioms[1..]
                .iter()
                .all(|axiom| axiom.status_key() == "suspected")
        );
    }

    #[test]
    fn 未知公理会报可选项() {
        let contract = Contract::bundled();
        let error = axiom_of(&meta_domain(&contract), "nope").unwrap_err();
        assert!(error.contains("one-place"));
    }

    #[test]
    fn 公理按领域归且两域都非空() {
        let contract = Contract::bundled();
        let meta: Vec<&Axiom> = meta_domain(&contract);
        let asset: Vec<&Axiom> = contract
            .axiom
            .iter()
            .filter(|axiom| axiom.is_asset())
            .collect();
        assert!(!meta.is_empty(), "元模型领域不该空");
        assert!(!asset.is_empty(), "资产领域不该空");
        // 两域不重叠、合起来是全集。
        assert_eq!(meta.len() + asset.len(), contract.axiom.len());
    }

    #[test]
    fn 落位卡点名的公理都在元模型领域() {
        // 硬不变量：被分类消费的公理必须是 meta 的。破了这条，
        // 一条资产领域的公理就会混进落位卡，域界名存实亡。
        let contract = Contract::bundled();
        for name in crate::meta::classify::cited_axiom_names() {
            let axiom = contract
                .axiom
                .iter()
                .find(|axiom| axiom.name == name)
                .unwrap_or_else(|| panic!("classify 点名了契约里没有的公理：{name}"));
            assert!(
                axiom.is_meta(),
                "classify 点名的 {name} 归了资产领域：要么改 domain，要么别在落位卡上点它"
            );
        }
    }
}
