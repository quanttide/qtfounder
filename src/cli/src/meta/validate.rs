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

/// estate 物理形态词（第三档边界网）。
///
/// 本体论层公理不该点名这些——它们只在 estate 的当前形态下才有意义，
/// 换一套子模块与目录，词就落空了。词表刻意窄到零误报：内容词（归档、日志、
/// 入口）不进表，它们在两层里都是合法词。真误报了，修法通常是改写那条公理——
/// 抽象公理本就不该点名具体物件。
///
/// 判断题本身留在人这边：见 docs/user-guide/meta.md「分层的边界是判断题」。
const ESTATE_PHYSICAL_TERMS: [&str; 6] = [
    "子模块",
    "仓库根",
    "目录名",
    "路径",
    ".gitmodules",
    "data/",
];

pub fn run(args: &ValidateArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    let mut problems = Vec::new();
    problems.extend(check_assets(&contract));
    problems.extend(check_relations(&contract));
    problems.extend(check_categories(&contract));
    problems.extend(check_homes(&contract));
    problems.extend(check_transitions(&contract));
    problems.extend(check_axioms(&contract));

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

/// 公理分层的三档校验。第一档（domain 只认 meta / asset）在读契约时就拦，
/// 这里查后两档：
///
/// 第二档，硬不变量：落位卡点名的公理必须是本体论层。破了这条，
/// 一条资产层公理就混进了分类输出，两层边界名存实亡。
///
/// 第三档，窄词表网：本体论层公理不点名 estate 物理形态词。抓的是「自称抽象、
/// 其实依赖物理形态」这种错标——未来的人（或 AI）最容易犯的那种。
fn check_axioms(contract: &Contract) -> Vec<String> {
    let mut problems = Vec::new();
    for name in crate::meta::classify::cited_axiom_names() {
        match contract.axiom.iter().find(|axiom| axiom.name == name) {
            None => problems.push(format!(
                "axiom：classify 点名的 {name} 不在契约里，落位卡会漏引它"
            )),
            Some(axiom) if axiom.is_asset() => problems.push(format!(
                "axiom：{name} 被落位卡点名，却标了 domain = \"asset\"；被分类消费的公理必须是本体论层"
            )),
            Some(_) => {}
        }
    }
    for axiom in contract.axiom.iter().filter(|axiom| axiom.is_meta()) {
        let text = format!("{} {}", axiom.description, axiom.why);
        for term in ESTATE_PHYSICAL_TERMS {
            if text.contains(term) {
                problems.push(format!(
                    "axiom：{} 是本体论层，description 或 why 里却点了物理形态词「{term}」；把 estate 换一套子模块与目录，这条还成立吗？成立就改写，不成立就标 domain = \"asset\"",
                    axiom.name
                ));
            }
        }
    }
    problems
}
