//! `meta paradigm`：看范式——把一套分类写成对象、态射与合成律。
//!
//! 一个范式就是一个范畴的描述：对象、态射、合成律。归位本身是其中一个，
//! 对象取格、态射取降格；别的范式只用于对比与吸收，不参与判域判格。
//!
//! `--check` 查两件事：合成缺没缺，两个对象是否互相可达。前者是范畴与图的分别，
//! 后者是骨架的嫌疑——两个代表互相可达，说明同构类可能只该留一个。

use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;

use clap::Args;
use serde_json::json;

use super::contract::{Contract, ResolvedParadigm};

#[derive(Args, Debug)]
pub struct ParadigmArgs {
    /// 只看一个范式
    pub name: Option<String>,
    /// 查合成：缺哪些合成、哪两个对象互相可达
    #[arg(long)]
    pub check: bool,
    /// 契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &ParadigmArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    if contract.paradigm.is_empty() {
        return Err("契约里没有范式：在 [[paradigm]] 里补一条".to_string());
    }
    let paradigms: Vec<ResolvedParadigm> = match &args.name {
        Some(name) => vec![resolve_of(&contract, name)?],
        None => ordered(&contract),
    };
    let shapes: Vec<Shape> = paradigms.iter().map(shape).collect();

    if args.check {
        let failed = report_check(&paradigms, &shapes, args.json);
        return Ok(if failed { 1 } else { 0 });
    }
    if args.json {
        print_json(&paradigms, &shapes);
    } else {
        print_text(&paradigms);
    }
    Ok(0)
}

/// 范式按状态排：确认的在前，同组内保持契约顺序。
pub fn ordered(contract: &Contract) -> Vec<ResolvedParadigm> {
    let mut paradigms: Vec<ResolvedParadigm> = contract
        .paradigm
        .iter()
        .map(|paradigm| contract.resolve(paradigm))
        .collect();
    paradigms.sort_by_key(|paradigm| paradigm.status_key() != "confirmed");
    paradigms
}

pub fn resolve_of(contract: &Contract, name: &str) -> Result<ResolvedParadigm, String> {
    contract
        .paradigm_of(name)
        .map(|paradigm| contract.resolve(paradigm))
        .ok_or_else(|| {
            let names: Vec<&str> = contract
                .paradigm
                .iter()
                .map(|paradigm| paradigm.name.as_str())
                .collect();
            format!("没有这个范式：{name}；可选：{}", names.join("、"))
        })
}

/// 一个范式的形状：点数、边数、缺哪几处合成、哪两个对象互相可达。
#[derive(Debug, Clone, Default)]
pub struct Shape {
    pub objects: usize,
    pub morphisms: usize,
    /// (a, b, c)：有 a→b 与 b→c，缺 a→c。
    pub missing: Vec<(String, String, String)>,
    /// 互相可达的两个对象。
    pub mutual: Vec<(String, String)>,
    /// 最长链的边数。
    pub longest_chain: usize,
    pub sources: usize,
    pub sinks: usize,
}

pub fn shape(paradigm: &ResolvedParadigm) -> Shape {
    let names: Vec<&str> = paradigm
        .objects
        .iter()
        .map(|object| object.name.as_str())
        .collect();
    let edges: BTreeSet<(&str, &str)> = paradigm
        .morphisms
        .iter()
        .map(|morphism| (morphism.from.as_str(), morphism.to.as_str()))
        .collect();

    let mut missing = BTreeSet::new();
    for (from, middle) in &edges {
        for (start, to) in &edges {
            if middle == start && !edges.contains(&(*from, *to)) {
                missing.insert((from.to_string(), to.to_string(), middle.to_string()));
            }
        }
    }

    let mut mutual = Vec::new();
    for (from, to) in &edges {
        if from < to && edges.contains(&(*to, *from)) {
            mutual.push((from.to_string(), to.to_string()));
        }
    }

    let longest_chain = longest_chain(&names, &edges);
    let sources = names
        .iter()
        .filter(|name| !edges.iter().any(|(_, to)| to == *name))
        .count();
    let sinks = names
        .iter()
        .filter(|name| !edges.iter().any(|(from, _)| from == *name))
        .count();

    Shape {
        objects: names.len(),
        morphisms: edges.len(),
        missing: missing.into_iter().collect(),
        mutual,
        longest_chain,
        sources,
        sinks,
    }
}

/// 最长链：按边数算，遇到回路就截断，不追第二圈。
fn longest_chain(names: &[&str], edges: &BTreeSet<(&str, &str)>) -> usize {
    fn walk(
        node: &str,
        edges: &BTreeSet<(&str, &str)>,
        seen: &mut HashSet<String>,
        memo: &mut std::collections::HashMap<String, usize>,
    ) -> usize {
        if let Some(done) = memo.get(node) {
            return *done;
        }
        if !seen.insert(node.to_string()) {
            return 0;
        }
        let mut best = 0;
        for (from, to) in edges {
            if *from == node {
                best = best.max(1 + walk(to, edges, seen, memo));
            }
        }
        seen.remove(node);
        memo.insert(node.to_string(), best);
        best
    }

    let mut memo = std::collections::HashMap::new();
    names
        .iter()
        .map(|name| walk(name, edges, &mut HashSet::new(), &mut memo))
        .max()
        .unwrap_or(0)
}

fn report_check(paradigms: &[ResolvedParadigm], shapes: &[Shape], json_output: bool) -> bool {
    let mut failed = false;
    for (paradigm, shape) in paradigms.iter().zip(shapes) {
        // 只有声明了传递合成的范式才要求闭合；部分合成缺了是常态，只报个数。
        let strict = paradigm.compose_key() == "transitive";
        if strict && !shape.missing.is_empty() {
            failed = true;
        }
        if !shape.mutual.is_empty() {
            failed = true;
        }
    }

    if json_output {
        let items: Vec<serde_json::Value> = paradigms
            .iter()
            .zip(shapes)
            .map(|(paradigm, shape)| {
                json!({
                    "name": paradigm.name,
                    "label": paradigm.label(),
                    "compose": paradigm.compose_key(),
                    "objects": shape.objects,
                    "morphisms": shape.morphisms,
                    "missing": shape.missing,
                    "mutual": shape.mutual,
                    "longest_chain": shape.longest_chain,
                    "sources": shape.sources,
                    "sinks": shape.sinks,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_string())
        );
        return failed;
    }

    for (index, (paradigm, shape)) in paradigms.iter().zip(shapes).enumerate() {
        if index > 0 {
            println!();
        }
        let strict = paradigm.compose_key() == "transitive";
        println!(
            "{}  {}  合成：{}",
            paradigm.name,
            paradigm.label(),
            paradigm.compose_label()
        );
        println!(
            "  对象 {}  态射 {}  最长链 {}",
            shape.objects, shape.morphisms, shape.longest_chain
        );
        if shape.missing.is_empty() {
            println!("  合成不缺");
        } else {
            let note = if strict {
                "缺了就是断"
            } else {
                "部分合成，缺了不算错"
            };
            println!("  缺合成 {} 处（{note}）：", shape.missing.len());
            for (from, to, middle) in &shape.missing {
                println!("    {from} → {to}（经过 {middle}）");
            }
        }
        if shape.mutual.is_empty() {
            println!("  没有互相可达的两个对象");
        } else {
            println!(
                "  互相可达 {} 对（同构嫌疑，看是不是该只留一个代表）：",
                shape.mutual.len()
            );
            for (left, right) in &shape.mutual {
                println!("    {left} ↔ {right}");
            }
        }
    }
    failed
}

fn print_text(paradigms: &[ResolvedParadigm]) {
    for (index, paradigm) in paradigms.iter().enumerate() {
        if index > 0 {
            println!();
        }
        println!(
            "{}  {}  {}  合成：{}  共 {} 个对象 / {} 条态射",
            paradigm.name,
            paradigm.label(),
            paradigm.status_label(),
            paradigm.compose_label(),
            paradigm.objects.len(),
            paradigm.morphisms.len()
        );
        if !paradigm.description.is_empty() {
            println!("  {}", paradigm.description);
        }
        println!("  对象：");
        for object in &paradigm.objects {
            let outgoing = paradigm
                .morphisms
                .iter()
                .filter(|morphism| morphism.from == object.name)
                .count();
            let incoming = paradigm
                .morphisms
                .iter()
                .filter(|morphism| morphism.to == object.name)
                .count();
            println!(
                "    {}  {}  出 {outgoing} 入 {incoming}",
                object.name,
                if object.label.is_empty() {
                    &object.name
                } else {
                    &object.label
                }
            );
        }
        if !paradigm.morphisms.is_empty() {
            println!("  态射：");
            for morphism in &paradigm.morphisms {
                if morphism.reason.is_empty() {
                    println!("    {} → {}", morphism.from, morphism.to);
                } else {
                    println!(
                        "    {} → {}  {}",
                        morphism.from, morphism.to, morphism.reason
                    );
                }
            }
        }
    }
}

fn print_json(paradigms: &[ResolvedParadigm], shapes: &[Shape]) {
    let items: Vec<serde_json::Value> = paradigms
        .iter()
        .zip(shapes)
        .map(|(paradigm, shape)| {
            json!({
                "name": paradigm.name,
                "label": paradigm.label(),
                "description": paradigm.description,
                "status": paradigm.status_key(),
                "status_label": paradigm.status_label(),
                "source": paradigm.source,
                "compose": paradigm.compose_key(),
                "objects": paradigm.objects.iter().map(|object| json!({
                    "name": object.name,
                    "label": object.label,
                    "description": object.description,
                })).collect::<Vec<_>>(),
                "morphisms": paradigm.morphisms.iter().map(|morphism| json!({
                    "from": morphism.from,
                    "to": morphism.to,
                    "reason": morphism.reason,
                })).collect::<Vec<_>>(),
                "shape": {
                    "objects": shape.objects,
                    "morphisms": shape.morphisms,
                    "missing": shape.missing,
                    "mutual": shape.mutual,
                    "longest_chain": shape.longest_chain,
                    "sources": shape.sources,
                    "sinks": shape.sinks,
                },
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
    use crate::meta::contract::{ParadigmMorphism, ParadigmObject};

    fn paradigm(objects: &[&str], morphisms: &[(&str, &str)], compose: &str) -> ResolvedParadigm {
        ResolvedParadigm {
            name: "test".to_string(),
            compose: compose.to_string(),
            objects: objects
                .iter()
                .map(|name| ParadigmObject {
                    name: name.to_string(),
                    ..Default::default()
                })
                .collect(),
            morphisms: morphisms
                .iter()
                .map(|(from, to)| ParadigmMorphism {
                    from: from.to_string(),
                    to: to.to_string(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn 归位本身是一个范式() {
        let contract = Contract::bundled();
        let founder = resolve_of(&contract, "quanttide-founder").unwrap();
        assert_eq!(founder.objects.len(), 18);
        assert_eq!(founder.morphisms.len(), 3);
        assert_eq!(founder.source, "categories");
    }

    #[test]
    fn 亚里士多德有十个对象和九条态射() {
        let contract = Contract::bundled();
        let aristotle = resolve_of(&contract, "aristotle").unwrap();
        assert_eq!(aristotle.objects.len(), 10);
        assert_eq!(aristotle.morphisms.len(), 9);
        assert!(aristotle.object("substance").is_some());
    }

    #[test]
    fn 合成断了要报出来() {
        let paradigm = paradigm(&["a", "b", "c"], &[("a", "b"), ("b", "c")], "transitive");
        let shape = shape(&paradigm);
        assert_eq!(shape.missing.len(), 1);
        assert_eq!(shape.missing[0].0, "a");
        assert_eq!(shape.missing[0].1, "c");
        assert_eq!(shape.longest_chain, 2);
    }

    #[test]
    fn 合成齐了就不报() {
        let paradigm = paradigm(
            &["a", "b", "c"],
            &[("a", "b"), ("b", "c"), ("a", "c")],
            "transitive",
        );
        assert!(shape(&paradigm).missing.is_empty());
    }

    #[test]
    fn 互相可达算同构嫌疑() {
        let paradigm = paradigm(&["a", "b"], &[("a", "b"), ("b", "a")], "partial");
        let shape = shape(&paradigm);
        assert_eq!(shape.mutual, vec![("a".to_string(), "b".to_string())]);
    }

    #[test]
    fn 未知范式会报可选项() {
        let contract = Contract::bundled();
        let error = resolve_of(&contract, "nope").unwrap_err();
        assert!(error.contains("quanttide-founder"), "{error}");
    }
}
