//! `meta compare`：比两个范式——同名、同称呼、各自独有与形状。
//!
//! 只读契约，不改任何一边。同名不算同构，同称呼也不算；它们只是对上的线索，
//! 加不加由人决定。可吸收的清单只列「对方有、本边没有对上」的对象。

use std::path::PathBuf;

use clap::Args;
use serde_json::json;

use super::contract::{Contract, ParadigmObject, ResolvedParadigm};
use super::paradigm::{Shape, resolve_of, shape};

#[derive(Args, Debug)]
pub struct CompareArgs {
    /// 左边的范式
    pub left: String,
    /// 右边的范式
    pub right: String,
    /// 契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &CompareArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    if contract.paradigm.is_empty() {
        return Err("契约里没有范式：在 [[paradigm]] 里补一条".to_string());
    }
    let left = resolve_of(&contract, &args.left)?;
    let right = resolve_of(&contract, &args.right)?;
    let diff = diff(&left, &right);
    let left_shape = shape(&left);
    let right_shape = shape(&right);
    if args.json {
        print_json(&left, &right, &left_shape, &right_shape, &diff);
    } else {
        print_text(&left, &right, &left_shape, &right_shape, &diff);
    }
    Ok(0)
}

/// 两个范式对上的东西：同名、同称呼，以及各自独有。
#[derive(Debug, Clone, Default)]
pub struct Diff {
    pub shared: Vec<String>,
    /// 左边有、右边没有。
    pub left_only: Vec<ParadigmObject>,
    /// 右边有、左边没有；这就是可吸收的候选。
    pub right_only: Vec<ParadigmObject>,
    /// 名字不同但称呼相同。
    pub same_label: Vec<(String, String)>,
}

pub fn diff(left: &ResolvedParadigm, right: &ResolvedParadigm) -> Diff {
    let mut shared = Vec::new();
    let mut left_only = Vec::new();
    let mut right_only = Vec::new();
    let mut same_label = Vec::new();

    for object in &left.objects {
        match right.object(&object.name) {
            Some(_) => shared.push(object.name.clone()),
            None => {
                if let Some(counterpart) = matching_label(right, object) {
                    same_label.push((object.name.clone(), counterpart.name.clone()));
                }
                left_only.push(object.clone());
            }
        }
    }
    for object in &right.objects {
        if left.object(&object.name).is_some() {
            continue;
        }
        // 称呼对上的算已对上，不进可吸收清单。
        if matching_label(left, object).is_some() {
            continue;
        }
        right_only.push(object.clone());
    }

    Diff {
        shared,
        left_only,
        right_only,
        same_label,
    }
}

fn matching_label<'a>(
    paradigm: &'a ResolvedParadigm,
    object: &ParadigmObject,
) -> Option<&'a ParadigmObject> {
    if object.label.trim().is_empty() {
        return None;
    }
    paradigm
        .objects
        .iter()
        .find(|candidate| candidate.name != object.name && candidate.label == object.label)
}

fn print_text(
    left: &ResolvedParadigm,
    right: &ResolvedParadigm,
    left_shape: &Shape,
    right_shape: &Shape,
    diff: &Diff,
) {
    println!("{}  ↔  {}", left.label(), right.label());
    println!();
    println!(
        "  形状    对象 {} ↔ {}；态射 {} ↔ {}；最长链 {} ↔ {}",
        left_shape.objects,
        right_shape.objects,
        left_shape.morphisms,
        right_shape.morphisms,
        left_shape.longest_chain,
        right_shape.longest_chain
    );
    println!(
        "  合成    {}（缺 {}） ↔ {}（缺 {}）",
        left.compose_label(),
        left_shape.missing.len(),
        right.compose_label(),
        right_shape.missing.len()
    );
    println!();

    if diff.shared.is_empty() {
        println!("  同名：无");
    } else {
        println!(
            "  同名 {} 个：{}",
            diff.shared.len(),
            diff.shared.join("、")
        );
    }
    if diff.same_label.is_empty() {
        println!("  同称呼：无");
    } else {
        let pairs: Vec<String> = diff
            .same_label
            .iter()
            .map(|(left, right)| format!("{left} ↔ {right}"))
            .collect();
        println!(
            "  同称呼 {} 对：{}",
            diff.same_label.len(),
            pairs.join("、")
        );
    }

    println!(
        "  只在左边 {} 个：{}",
        diff.left_only.len(),
        names(&diff.left_only)
    );
    println!(
        "  只在右边 {} 个：{}",
        diff.right_only.len(),
        names(&diff.right_only)
    );

    if !diff.right_only.is_empty() {
        println!();
        println!("  可吸收（右边独有，名字与称呼都没对上）：");
        for object in &diff.right_only {
            if object.label.trim().is_empty() {
                println!("    {}", object.name);
            } else {
                println!("    {}  {}", object.name, object.label);
            }
            if !object.description.trim().is_empty() {
                println!("      {}", object.description);
            }
        }
    }
}

fn names(objects: &[ParadigmObject]) -> String {
    if objects.is_empty() {
        "（无）".to_string()
    } else {
        objects
            .iter()
            .map(|object| object.name.as_str())
            .collect::<Vec<_>>()
            .join("、")
    }
}

fn print_json(
    left: &ResolvedParadigm,
    right: &ResolvedParadigm,
    left_shape: &Shape,
    right_shape: &Shape,
    diff: &Diff,
) {
    let value = json!({
        "left": left.name,
        "right": right.name,
        "shape": {
            "left": {
                "objects": left_shape.objects,
                "morphisms": left_shape.morphisms,
                "missing": left_shape.missing.len(),
                "mutual": left_shape.mutual.len(),
                "longest_chain": left_shape.longest_chain,
            },
            "right": {
                "objects": right_shape.objects,
                "morphisms": right_shape.morphisms,
                "missing": right_shape.missing.len(),
                "mutual": right_shape.mutual.len(),
                "longest_chain": right_shape.longest_chain,
            },
        },
        "shared": diff.shared,
        "same_label": diff.same_label,
        "left_only": diff.left_only.iter().map(|object| object.name.clone()).collect::<Vec<_>>(),
        "right_only": diff.right_only.iter().map(|object| object.name.clone()).collect::<Vec<_>>(),
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paradigm(name: &str, objects: &[(&str, &str)]) -> ResolvedParadigm {
        ResolvedParadigm {
            name: name.to_string(),
            objects: objects
                .iter()
                .map(|(object, label)| ParadigmObject {
                    name: object.to_string(),
                    label: label.to_string(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn 同名的算对上() {
        let left = paradigm("a", &[("insight", "洞察")]);
        let right = paradigm("b", &[("insight", "洞见")]);
        let diff = diff(&left, &right);
        assert_eq!(diff.shared, vec!["insight".to_string()]);
        assert!(diff.right_only.is_empty());
    }

    #[test]
    fn 同称呼不同名的算对上但不进可吸收() {
        let left = paradigm("a", &[("insight", "洞察")]);
        let right = paradigm("b", &[("insight_v2", "洞察")]);
        let diff = diff(&left, &right);
        assert_eq!(
            diff.same_label,
            vec![("insight".to_string(), "insight_v2".to_string())]
        );
        assert!(diff.right_only.is_empty());
    }

    #[test]
    fn 名字称呼都对不上的是可吸收候选() {
        let left = paradigm("a", &[("insight", "洞察")]);
        let right = paradigm("b", &[("substance", "实体")]);
        let diff = diff(&left, &right);
        assert_eq!(diff.right_only.len(), 1);
        assert_eq!(diff.right_only[0].name, "substance");
        assert_eq!(diff.left_only.len(), 1);
    }
}
