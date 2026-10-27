//! `meta validate`：查文档镜像与契约是否还对得上。
//!
//! 只比对，不改写。公理与领域看二级、三级标题，属性看列表项；
//! 名字、状态、分组、定义任一处不一致都算漂移。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use clap::Args;
use serde_json::json;

use super::contract::{Contract, status_label};

const LABELS: [(&str, &str); 2] = [("（确认）", "confirmed"), ("（怀疑）", "suspected")];

#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// 文档目录，缺省 docs/gallery/meta
    #[arg(long, default_value = "docs/gallery/meta")]
    pub docs: PathBuf,
    /// 契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &ValidateArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    if !args.docs.is_dir() {
        return Err(format!("找不到文档目录：{}", args.docs.display()));
    }
    let mut problems = Vec::new();
    problems.extend(check_axioms(&contract, &args.docs));
    problems.extend(check_properties(&contract, &args.docs));
    problems.extend(check_domains(&contract, &args.docs));

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "axioms": contract.axiom.len(),
                "properties": contract.property.len(),
                "domains": contract.domain.len(),
                "problems": problems,
            }))
            .unwrap_or_else(|_| "{}".to_string())
        );
    } else if problems.is_empty() {
        println!("对得上：公理、属性与领域都一致");
    } else {
        println!("对不上 {} 处：", problems.len());
        for problem in &problems {
            println!("  {problem}");
        }
    }
    Ok(if problems.is_empty() { 0 } else { 1 })
}

fn check_axioms(contract: &Contract, docs: &Path) -> Vec<String> {
    let path = docs.join("axioms.md");
    let Some(text) = read(&path) else {
        return vec![format!("读不到 {}", path.display())];
    };
    let doc = headings(&text, "### ");
    let mut problems = Vec::new();
    for axiom in &contract.axiom {
        match doc.get(&axiom.name) {
            None => problems.push(format!("axioms.md：契约里有 {}，文档没写", axiom.name)),
            Some(key) => {
                if *key != axiom.status_key() {
                    problems.push(status_problem(
                        "axioms.md",
                        &axiom.name,
                        key,
                        axiom.status_label(),
                    ));
                }
            }
        }
    }
    problems.extend(extra("axioms.md", &doc, |name| {
        contract.axiom.iter().any(|axiom| axiom.name == name)
    }));
    problems
}

fn check_properties(contract: &Contract, docs: &Path) -> Vec<String> {
    let path = docs.join("property.md");
    let Some(text) = read(&path) else {
        return vec![format!("读不到 {}", path.display())];
    };
    let mut doc: BTreeMap<String, (String, &str, String)> = BTreeMap::new();
    let mut group = String::new();
    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            if let Some(name) = heading.trim().strip_suffix("属性") {
                group = name.to_string();
            }
            continue;
        }
        let Some(item) = line.strip_prefix("- ") else {
            continue;
        };
        if let Some((name, key, description)) = labeled(item) {
            doc.insert(name, (group.clone(), key, description));
        }
    }
    let mut problems = Vec::new();
    for property in &contract.property {
        match doc.get(&property.name) {
            None => problems.push(format!("property.md：契约里有 {}，文档没写", property.name)),
            Some((doc_group, key, description)) => {
                if *doc_group != property.group {
                    problems.push(format!(
                        "property.md：{} 的分组不一致——文档 {}，契约 {}",
                        property.name, doc_group, property.group
                    ));
                }
                if *key != property.status_key() {
                    problems.push(status_problem(
                        "property.md",
                        &property.name,
                        key,
                        property.status_label(),
                    ));
                }
                if description.trim() != property.description.trim() {
                    problems.push(format!(
                        "property.md：{} 的定义不一致——文档「{}」，契约「{}」",
                        property.name, description, property.description
                    ));
                }
            }
        }
    }
    for name in doc.keys() {
        if !contract
            .property
            .iter()
            .any(|property| &property.name == name)
        {
            problems.push(format!("property.md：文档写了 {name}，契约里没有"));
        }
    }
    problems
}

fn check_domains(contract: &Contract, docs: &Path) -> Vec<String> {
    let path = docs.join("domain.md");
    let Some(text) = read(&path) else {
        return vec![format!("读不到 {}", path.display())];
    };
    let doc = headings(&text, "## ");
    let mut problems = Vec::new();
    for domain in &contract.domain {
        match doc.get(&domain.name) {
            None => problems.push(format!("domain.md：契约里有 {}，文档没写", domain.name)),
            Some(key) => {
                if *key != domain.status_key() {
                    problems.push(status_problem(
                        "domain.md",
                        &domain.name,
                        key,
                        domain.status_label(),
                    ));
                }
            }
        }
    }
    problems.extend(extra("domain.md", &doc, |name| {
        contract.domain.iter().any(|domain| domain.name == name)
    }));
    problems
}

/// 取带标记的标题：`### name（确认）`。
fn headings(text: &str, prefix: &str) -> BTreeMap<String, &'static str> {
    let mut found = BTreeMap::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix(prefix) else {
            continue;
        };
        if let Some((name, key)) = marked(rest.trim()) {
            found.insert(name, key);
        }
    }
    found
}

/// 文档里有、契约里没有的名字。
fn extra(
    file: &str,
    doc: &BTreeMap<String, &'static str>,
    exists: impl Fn(&str) -> bool,
) -> Vec<String> {
    doc.keys()
        .filter(|name| !exists(name))
        .map(|name| format!("{file}：文档写了 {name}，契约里没有"))
        .collect()
}

fn status_problem(file: &str, name: &str, doc_key: &str, contract_label: &str) -> String {
    format!(
        "{file}：{name} 的状态不一致——文档 {}，契约 {contract_label}",
        status_label(doc_key)
    )
}

/// `name（确认）` → (name, confirmed)。
fn marked(text: &str) -> Option<(String, &'static str)> {
    for (label, key) in LABELS {
        if let Some((name, _)) = text.split_once(label) {
            return Some((name.trim().to_string(), key));
        }
    }
    None
}

/// `name（确认）：定义` → (name, confirmed, 定义)。
fn labeled(text: &str) -> Option<(String, &'static str, String)> {
    for (label, key) in LABELS {
        if let Some((name, tail)) = text.split_once(label) {
            let description = tail.trim_start_matches('：').trim().to_string();
            return Some((name.trim().to_string(), key, description));
        }
    }
    None
}

fn read(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::contract::status_key;

    #[test]
    fn 标题能拆出名字与状态() {
        assert_eq!(
            marked("template-scope（确认）"),
            Some(("template-scope".to_string(), "confirmed"))
        );
        assert_eq!(
            marked("one-place（怀疑）"),
            Some(("one-place".to_string(), "suspected"))
        );
    }

    #[test]
    fn 列表项能拆出名字状态与定义() {
        assert_eq!(
            labeled("id（确认）：全局唯一键，用 uuid；改名不动它，引用认它。"),
            Some((
                "id".to_string(),
                "confirmed",
                "全局唯一键，用 uuid；改名不动它，引用认它。".to_string()
            ))
        );
    }

    #[test]
    fn 状态标签映射唯一() {
        assert_eq!(status_label(status_key("confirmed")), "确认");
        assert_eq!(status_label(status_key("")), "怀疑");
    }
}
