//! 词汇层：命名有信息量吗、名实相符吗、同一个概念有几种说法。
//!
//! 对应 Grice：量准则（信息量不足）与质准则（命名不诚实）。
//! 这一层只读 [`Facts`]，不关心源码是什么语言写的。

use std::collections::HashSet;

use super::ir::{BindingFacts, Facts};
use super::model::{Finding, Layer, Principle};
use super::rules::Rules;

/// 初始化构造器归到哪一类形状：判名实不符用。
fn shape_class(shape: &str) -> Option<&'static str> {
    let lowered = shape.to_lowercase();
    if ["dict", "hashmap", "btreemap", "map", "defaultdict"].contains(&lowered.as_str()) {
        return Some("dict");
    }
    if ["list", "vec", "array", "deque", "vecdeque"].contains(&lowered.as_str()) {
        return Some("list");
    }
    if ["string", "str", "to_string", "from"].contains(&lowered.as_str()) {
        return Some("str");
    }
    None
}

fn expected_class(name: &str) -> Option<&'static str> {
    let lowered = name.to_lowercase();
    if lowered.ends_with("list") || lowered.ends_with("items") || lowered.ends_with("vec") {
        return Some("list");
    }
    if lowered.ends_with("dict") || lowered.ends_with("map") {
        return Some("dict");
    }
    if lowered.ends_with("str") || lowered.ends_with("text") {
        return Some("str");
    }
    None
}

fn name_finding(binding: &BindingFacts, path: &str, rules: &Rules) -> Option<Finding> {
    let lexical = &rules.lexical;
    let name = binding.name.as_str();
    if name == "_" {
        return None;
    }
    if lexical.vague_names.iter().any(|item| item == name) {
        return Some(Finding::new(
            Layer::Lexical,
            "vague_name",
            path,
            binding.line,
            format!("「{name}」没告诉读者它是什么：凡是叫这个名字的都得回头读代码才知道装的是什么"),
            Principle::Quantity,
            "换成说清内容的词（如 pending_orders / retry_budget），别用万能名",
        ));
    }
    if name.chars().count() < lexical.min_len && !binding.short_lived && !rules.is_whitelisted(name)
    {
        return Some(Finding::new(
            Layer::Lexical,
            "short_name",
            path,
            binding.line,
            format!(
                "「{name}」只有 {} 个字符，而它要在整个函数里被记住：读者得靠上下文猜它是什么",
                name.chars().count()
            ),
            Principle::Manner,
            "换成完整词。循环变量、闭包参数、match 绑定里的短名不报——它们只活几行",
        ));
    }
    if let Some(full) = lexical.abbreviations.get(name) {
        return Some(Finding::new(
            Layer::Lexical,
            "abbreviation",
            path,
            binding.line,
            format!("缩写「{name}」至少有两种读法（{full}？）：团队里没有唯一约定时读者会读错"),
            Principle::Manner,
            "写全名，或在模块文档里展开一次",
        ));
    }
    None
}

fn mismatch_finding(binding: &BindingFacts, path: &str) -> Option<Finding> {
    let shape = binding.init_shape.as_deref()?;
    let actual = shape_class(shape)?;
    let expects = expected_class(&binding.name)?;
    if expects == actual {
        return None;
    }
    Some(Finding::new(
        Layer::Lexical,
        "name_type_mismatch",
        path,
        binding.line,
        format!(
            "「{}」的名字说是 {expects}，赋的却是 {actual}（{shape}）",
            binding.name
        ),
        Principle::Quality,
        "要么改名字，要么改值——名字不诚实会让人按错的假设改代码",
    ))
}

pub fn check(facts: &Facts, path: &str, rules: &Rules) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for binding in &facts.bindings {
        if seen.contains(&binding.name) {
            continue;
        }
        if let Some(finding) = name_finding(binding, path, rules) {
            seen.insert(binding.name.clone());
            out.push(finding);
        }
    }
    for binding in &facts.bindings {
        if let Some(finding) = mismatch_finding(binding, path) {
            out.push(finding);
        }
    }
    out
}

/// 仓库级：同一个动作出现多种说法。
pub fn check_consistency(functions: &[String], path: &str, rules: &Rules) -> Vec<Finding> {
    let mut out = Vec::new();
    for group in &rules.lexical.synonym_groups {
        let used: HashSet<&String> = functions
            .iter()
            .filter(|name| group.iter().any(|word| word == *name))
            .collect();
        if used.len() < 3.min(group.len()) {
            continue;
        }
        let mut shown: Vec<&str> = used.iter().map(|name| name.as_str()).collect();
        shown.sort_unstable();
        out.push(Finding::new(
            Layer::Lexical,
            "synonym_drift",
            path,
            0,
            format!(
                "同一种动作在仓库里有 {} 种说法（{}）：读者每次都要重新判断它们是不是一回事",
                used.len(),
                shown.join("、")
            ),
            Principle::Relation,
            "选一种作为正名，其余改名或写进规则表当同义词",
        ));
    }
    out
}
