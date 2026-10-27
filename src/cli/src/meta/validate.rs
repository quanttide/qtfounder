//! `meta validate`：查文档镜像与契约是否还对得上。
//!
//! 只比对，不改写。公理、领域与范式看标题，属性、范畴看列表项；
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
    problems.extend(check_homes(&contract, &args.docs));
    problems.extend(check_categories(&contract, &args.docs));
    problems.extend(check_models(&contract, &args.docs));
    problems.extend(check_paradigms(&contract, &args.docs));

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "axioms": contract.axiom.len(),
                "properties": contract.property.len(),
                "domain": contract.domain,
                "homes": contract.home.len(),
                "categories": contract.category.len(),
                "paradigms": contract.paradigm.len(),
                "problems": problems,
            }))
            .unwrap_or_else(|_| "{}".to_string())
        );
    } else if problems.is_empty() {
        println!("对得上：公理、属性、去处、范畴与范式都一致");
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

fn check_homes(contract: &Contract, docs: &Path) -> Vec<String> {
    let path = docs.join("home.md");
    let Some(text) = read(&path) else {
        return vec![format!("读不到 {}", path.display())];
    };
    let doc = headings(&text, "## ");
    let mut problems = Vec::new();
    for home in &contract.home {
        match doc.get(&home.name) {
            None => problems.push(format!("home.md：契约里有 {}，文档没写", home.name)),
            Some(key) => {
                if *key != home.status_key() {
                    problems.push(status_problem(
                        "home.md",
                        &home.name,
                        key,
                        home.status_label(),
                    ));
                }
            }
        }
    }
    problems.extend(extra("home.md", &doc, |name| {
        contract.home.iter().any(|home| home.name == name)
    }));
    problems
}

#[derive(Default)]
struct CategoryEntry {
    family: String,
    label: String,
    path: String,
    description: String,
}

fn check_categories(contract: &Contract, docs: &Path) -> Vec<String> {
    let path = docs.join("category.md");
    let Some(text) = read(&path) else {
        return vec![format!("读不到 {}", path.display())];
    };
    let doc = parse_categories(&text);
    let mut problems = Vec::new();
    for category in &contract.category {
        match doc.get(&category.name) {
            None => problems.push(format!("category.md：契约里有 {}，文档没写", category.name)),
            Some(entry) => {
                if entry.family != category.family {
                    problems.push(format!(
                        "category.md：{} 的族不一致——文档 {}，契约 {}",
                        category.name, entry.family, category.family
                    ));
                }
                if entry.label != category.label {
                    problems.push(format!(
                        "category.md：{} 的称呼不一致——文档 {}，契约 {}",
                        category.name, entry.label, category.label
                    ));
                }
                if entry.path != category.path {
                    problems.push(format!(
                        "category.md：{} 的目录不一致——文档 {}，契约 {}",
                        category.name, entry.path, category.path
                    ));
                }
                if entry.description.trim() != category.description.trim() {
                    problems.push(format!(
                        "category.md：{} 的界线不一致——文档「{}」，契约「{}」",
                        category.name, entry.description, category.description
                    ));
                }
            }
        }
    }
    for asset in contract.asset.iter().filter(|asset| !asset.is_memory()) {
        match doc.get(&asset.name) {
            None => problems.push(format!("category.md：契约里有 {}，文档没写", asset.name)),
            Some(entry) => {
                if entry.label != asset.label() {
                    problems.push(format!(
                        "category.md：{} 的称呼不一致——文档 {}，契约 {}",
                        asset.name,
                        entry.label,
                        asset.label()
                    ));
                }
                if entry.path != asset.path {
                    problems.push(format!(
                        "category.md：{} 的目录不一致——文档 {}，契约 {}",
                        asset.name, entry.path, asset.path
                    ));
                }
                if entry.description.trim() != asset.description.trim() {
                    problems.push(format!(
                        "category.md：{} 的界线不一致——文档「{}」，契约「{}」",
                        asset.name, entry.description, asset.description
                    ));
                }
            }
        }
    }
    for name in doc.keys() {
        let known_end = contract
            .asset(name)
            .map(|asset| !asset.is_memory())
            .unwrap_or(false);
        if contract.category(name).is_none() && !known_end {
            problems.push(format!("category.md：文档写了 {name}，契约里没有"));
        }
    }
    problems
}

fn parse_categories(text: &str) -> BTreeMap<String, CategoryEntry> {
    let mut doc = BTreeMap::new();
    let mut family = String::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            family = heading.trim().to_string();
            current = None;
            continue;
        }
        if let Some(heading) = line.strip_prefix("### ") {
            let name = heading.trim().to_string();
            doc.insert(
                name.clone(),
                CategoryEntry {
                    family: family.clone(),
                    ..Default::default()
                },
            );
            current = Some(name);
            continue;
        }
        let Some(item) = line.strip_prefix("- ") else {
            continue;
        };
        let Some(name) = current.as_ref() else {
            continue;
        };
        let Some((key, value)) = item.split_once('：') else {
            continue;
        };
        let Some(entry) = doc.get_mut(name) else {
            continue;
        };
        match key.trim() {
            "称呼" => entry.label = value.trim().to_string(),
            "目录" => entry.path = value.trim().trim_matches('`').to_string(),
            "收什么" => entry.description = value.trim().to_string(),
            _ => {}
        }
    }
    doc
}

#[derive(Default)]
struct ParadigmEntry {
    status: &'static str,
    label: String,
    source: String,
    compose: String,
    description: String,
    object_count: Option<usize>,
    objects: Option<Vec<String>>,
    morphism_count: Option<usize>,
    morphisms: Option<Vec<String>>,
}

fn check_paradigms(contract: &Contract, docs: &Path) -> Vec<String> {
    let path = docs.join("paradigm.md");
    let Some(text) = read(&path) else {
        return vec![format!("读不到 {}", path.display())];
    };
    let doc = parse_paradigms(&text);
    let mut problems = Vec::new();
    for paradigm in &contract.paradigm {
        let resolved = contract.resolve(paradigm);
        let Some(entry) = doc.get(&resolved.name) else {
            problems.push(format!("paradigm.md：契约里有 {}，文档没写", resolved.name));
            continue;
        };
        let name = resolved.name.as_str();
        if entry.status != resolved.status_key() {
            problems.push(status_problem(
                "paradigm.md",
                name,
                entry.status,
                resolved.status_label(),
            ));
        }
        if entry.label != resolved.label() {
            problems.push(format!(
                "paradigm.md：{name} 的称呼不一致——文档 {}，契约 {}",
                entry.label,
                resolved.label()
            ));
        }
        if entry.source != resolved.source {
            problems.push(format!(
                "paradigm.md：{name} 的来源不一致——文档 {}，契约 {}",
                entry.source, resolved.source
            ));
        }
        if entry.compose != resolved.compose_label() {
            problems.push(format!(
                "paradigm.md：{name} 的合成不一致——文档 {}，契约 {}",
                entry.compose,
                resolved.compose_label()
            ));
        }
        if entry.description.trim() != resolved.description.trim() {
            problems.push(format!(
                "paradigm.md：{name} 的描述不一致——文档「{}」，契约「{}」",
                entry.description, resolved.description
            ));
        }
        let object_names: Vec<String> = resolved
            .objects
            .iter()
            .map(|object| object.name.clone())
            .collect();
        if let Some(count) = entry.object_count {
            if count != object_names.len() {
                problems.push(format!(
                    "paradigm.md：{name} 的对象数不一致——文档 {count}，契约 {}",
                    object_names.len()
                ));
            }
        }
        if let Some(names) = &entry.objects {
            if names != &object_names {
                problems.push(format!("paradigm.md：{name} 的对象名单不一致"));
            }
        }
        let edges: Vec<String> = resolved
            .morphisms
            .iter()
            .map(|morphism| format!("{} → {}", morphism.from, morphism.to))
            .collect();
        if let Some(count) = entry.morphism_count {
            if count != edges.len() {
                problems.push(format!(
                    "paradigm.md：{name} 的态射数不一致——文档 {count}，契约 {}",
                    edges.len()
                ));
            }
        }
        if let Some(listed) = &entry.morphisms {
            if listed != &edges {
                problems.push(format!("paradigm.md：{name} 的态射名单不一致"));
            }
        }
    }
    for name in doc.keys() {
        if contract.paradigm_of(name).is_none() {
            problems.push(format!("paradigm.md：文档写了 {name}，契约里没有"));
        }
    }
    problems
}

fn parse_paradigms(text: &str) -> BTreeMap<String, ParadigmEntry> {
    let mut doc = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            match marked(heading.trim()) {
                Some((name, key)) => {
                    doc.insert(
                        name.clone(),
                        ParadigmEntry {
                            status: key,
                            ..Default::default()
                        },
                    );
                    current = Some(name);
                }
                None => current = None,
            }
            continue;
        }
        if line.starts_with("### ") {
            current = None;
            continue;
        }
        let Some(item) = line.strip_prefix("- ") else {
            continue;
        };
        let Some(name) = current.as_ref() else {
            continue;
        };
        let Some((raw_key, value)) = item.split_once('：') else {
            continue;
        };
        let value = value.trim();
        let (key, count) = split_count(raw_key.trim());
        let Some(entry) = doc.get_mut(name) else {
            continue;
        };
        match key {
            "称呼" => entry.label = value.to_string(),
            "来源" => entry.source = value.to_string(),
            "合成" => entry.compose = value.to_string(),
            "描述" => entry.description = value.to_string(),
            "对象" => {
                entry.object_count = count;
                entry.objects = if value.starts_with("见 ") {
                    None
                } else {
                    Some(list(value, '、'))
                };
            }
            "态射" => {
                entry.morphism_count = count;
                entry.morphisms = if value.contains('→') {
                    Some(edges(value))
                } else {
                    None
                };
            }
            _ => {}
        }
    }
    doc
}

/// `对象（18）` → («对象», 18)。
fn split_count(key: &str) -> (&str, Option<usize>) {
    match key.split_once('（') {
        Some((name, rest)) => (name.trim(), rest.trim_end_matches('）').trim().parse().ok()),
        None => (key, None),
    }
}

fn list(value: &str, separator: char) -> Vec<String> {
    value
        .split(separator)
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

/// `a → b；c → d` → [«a → b», «c → d»]。
fn edges(value: &str) -> Vec<String> {
    list(value, '；')
        .into_iter()
        .map(|edge| match edge.split_once('→') {
            Some((from, to)) => format!("{} → {}", from.trim(), to.trim()),
            None => edge,
        })
        .collect()
}

/// 模型是范畴作为点的读法：轴与族取自契约，类目名单取自 `[[category]]`。
/// `category.md` 的族标题下写 `- 轴：名（值、值）`，与契约逐条比。
fn check_models(contract: &Contract, docs: &Path) -> Vec<String> {
    let path = docs.join("category.md");
    let Some(text) = read(&path) else {
        return vec![format!("读不到 {}", path.display())];
    };
    let families: Vec<&str> = contract
        .model
        .family
        .iter()
        .map(|family| family.name.as_str())
        .collect();
    let doc = parse_axes(&text, &families);
    let mut problems = Vec::new();
    for family in &contract.model.family {
        match doc.get(&family.name) {
            None => problems.push(format!(
                "category.md：模型里有 {}，文档没写它的轴",
                family.name
            )),
            Some(axes) => {
                let expected: Vec<(String, Vec<String>)> = family
                    .axis
                    .iter()
                    .map(|axis| (axis.name.clone(), axis.values.clone()))
                    .collect();
                if axes != &expected {
                    problems.push(format!(
                        "category.md：{} 的轴不一致——文档 {:?}，契约 {:?}",
                        family.name, axes, expected
                    ));
                }
            }
        }
    }
    problems
}

fn parse_axes(text: &str, families: &[&str]) -> BTreeMap<String, Vec<(String, Vec<String>)>> {
    let mut doc: BTreeMap<String, Vec<(String, Vec<String>)>> = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            let name = heading.trim().to_string();
            current = if families.contains(&name.as_str()) {
                doc.insert(name.clone(), Vec::new());
                Some(name)
            } else {
                None
            };
            continue;
        }
        let Some(item) = line.strip_prefix("- 轴：") else {
            continue;
        };
        let Some(family) = current.as_ref() else {
            continue;
        };
        let Some((name, rest)) = item.split_once('（') else {
            continue;
        };
        let values = list(rest.trim_end_matches('）'), '、');
        if let Some(axes) = doc.get_mut(family) {
            axes.push((name.trim().to_string(), values));
        }
    }
    doc
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
    fn 范畴字段能拆出来() {
        let doc = parse_categories(
            "## 陈述型\n\n### insight\n\n- 称呼：Insight 洞察\n- 目录：`data/insight/`\n- 收什么：一句话能被推翻。\n",
        );
        let entry = doc.get("insight").unwrap();
        assert_eq!(entry.family, "陈述型");
        assert_eq!(entry.label, "Insight 洞察");
        assert_eq!(entry.path, "data/insight/");
        assert_eq!(entry.description, "一句话能被推翻。");
    }

    #[test]
    fn 范式计数与名单能拆出来() {
        let doc =
            parse_paradigms("## aristotle（确认）\n\n- 对象（2）：a、b\n- 态射（1）：a → b\n");
        let entry = doc.get("aristotle").unwrap();
        assert_eq!(entry.status, "confirmed");
        assert_eq!(entry.object_count, Some(2));
        assert_eq!(
            entry.objects.as_ref().unwrap(),
            &vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(
            entry.morphisms.as_ref().unwrap(),
            &vec!["a → b".to_string()]
        );
    }

    #[test]
    fn 状态标签映射唯一() {
        assert_eq!(status_label(status_key("confirmed")), "确认");
        assert_eq!(status_label(status_key("")), "怀疑");
    }
}
