//! `meta validate`：查契约自身是否自洽。
//!
//! 只读契约，不读任何仓库文档：契约是唯一事实源，镜像文档只是给人看的。
//! 查重复的名字、悬空的引用与落空的路径，只比对，不改写。

use std::collections::HashSet;
use std::path::PathBuf;

use clap::Args;
use serde_json::json;

use super::contract::Contract;

#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// 契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

const ROLES: [&str; 3] = ["source", "memory", "exit"];
const LAYOUTS: [&str; 2] = ["domain-asset", "asset-domain"];
const RELATION_KINDS: [&str; 2] = ["intake", "archive"];
const TRANSFORMS: [&str; 2] = ["flip", "same"];
/// 十八类长期记忆的统称：关系边的端点写它，指任意一类记忆仓。
const MEMORY_ROLE: &str = "memory";

pub fn run(args: &ValidateArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    let mut problems = Vec::new();
    problems.extend(check_assets(&contract));
    problems.extend(check_relations(&contract));
    problems.extend(check_categories(&contract));
    problems.extend(check_homes(&contract));
    problems.extend(check_transitions(&contract));

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "domain": contract.domain,
                "assets": contract.asset.len(),
                "relations": contract.relation.len(),
                "categories": contract.category.len(),
                "homes": contract.home.len(),
                "axioms": contract.axiom.len(),
                "properties": contract.property.len(),
                "paradigms": contract.paradigm.len(),
                "problems": problems,
            }))
            .unwrap_or_else(|_| "{}".to_string())
        );
    } else if problems.is_empty() {
        println!("对得上：契约内部一致");
    } else {
        println!("对不上 {} 处：", problems.len());
        for problem in &problems {
            println!("  {problem}");
        }
    }
    Ok(if problems.is_empty() { 0 } else { 1 })
}

fn check_assets(contract: &Contract) -> Vec<String> {
    let mut problems = Vec::new();
    let mut names: HashSet<&str> = HashSet::new();
    let mut paths: HashSet<&str> = HashSet::new();
    for asset in &contract.asset {
        if asset.name.trim().is_empty() {
            problems.push("asset：有资产缺 name".to_string());
        } else if !names.insert(asset.name.as_str()) {
            problems.push(format!("asset：name 重复：{}", asset.name));
        }
        if asset.path.trim().is_empty() {
            problems.push(format!("asset：{} 没有 path", asset.name));
        } else if !paths.insert(asset.path.as_str()) {
            problems.push(format!("asset：path 重复：{}", asset.path));
        }
        let role = asset.role.trim();
        if !ROLES.contains(&role) {
            problems.push(format!(
                "asset：{} 的 role 只能写 source、memory 或 exit，写了「{role}」",
                asset.name
            ));
        }
        let layout = asset.layout.trim();
        if !LAYOUTS.contains(&layout) {
            problems.push(format!(
                "asset：{} 的 layout 只能写 domain-asset 或 asset-domain，写了「{layout}」",
                asset.name
            ));
        }
    }
    problems
}

fn check_relations(contract: &Contract) -> Vec<String> {
    let mut problems = Vec::new();
    for relation in &contract.relation {
        let label = format!("{} → {}", relation.from, relation.to);
        let kind = relation.kind.trim();
        if !RELATION_KINDS.contains(&kind) {
            problems.push(format!(
                "relation {label}：kind 只能写 intake 或 archive，写了「{kind}」"
            ));
        }
        for end in [&relation.from, &relation.to] {
            if end == MEMORY_ROLE {
                continue;
            }
            if contract.asset(end).is_none() {
                problems.push(format!("relation {label}：指向不存在的资产：{end}"));
            }
        }
        let transform = relation.transform.trim();
        if !transform.is_empty() && !TRANSFORMS.contains(&transform) {
            problems.push(format!(
                "relation {label}：transform 只能写 flip 或 same，写了「{transform}」"
            ));
        }
    }
    problems
}

fn check_categories(contract: &Contract) -> Vec<String> {
    let mut problems = Vec::new();
    let mut paths: HashSet<&str> = HashSet::new();
    for category in &contract.category {
        if category.name.trim().is_empty() {
            problems.push("category：有格缺 name".to_string());
        }
        if category.label.trim().is_empty() {
            problems.push(format!("category：{} 没有 label", category.name));
        }
        if category.path.trim().is_empty() {
            problems.push(format!("category：{} 没有 path", category.name));
        } else if !paths.insert(category.path.as_str()) {
            problems.push(format!("category：path 重复：{}", category.path));
        }
        if category.cues.is_empty() {
            problems.push(format!(
                "category：{} 没有 cues，classify 永远判不进去",
                category.name
            ));
        }
    }
    problems
}

fn check_homes(contract: &Contract) -> Vec<String> {
    let mut problems = Vec::new();
    for home in &contract.home {
        if home.cues.is_empty() {
            problems.push(format!("home：{} 没有 cues，判定永远不会指到它", home.name));
        }
    }
    problems
}

fn check_transitions(contract: &Contract) -> Vec<String> {
    let mut problems = Vec::new();
    for transition in &contract.transition {
        if transition.from == transition.to {
            problems.push(format!(
                "transition：{} → {} 指向自己",
                transition.from, transition.to
            ));
        }
    }
    problems
}
