//! 语用层：名字诚不诚实、注释有没有解释「为什么」、错误信息能不能让人知道怎么修。
//!
//! 对应 Grice：质准则（不误导）与方式准则（不歧义）。这一层只读 [`Facts`]。

use super::ir::{Facts, FunctionFacts};
use super::model::{Finding, Layer, Principle};
use super::rules::Rules;

const UNINFORMATIVE_NAMES: [&str; 15] = [
    "handle", "process", "do_it", "run", "exec", "execute", "work", "deal", "manage", "util",
    "utils", "helper", "helpers", "func", "stuff",
];
const ACTION_WORDS: [&str; 10] = [
    "请", "检查", "应该", "必须", "不能", "需要", "配置", "传入", "指定", "重试",
];

fn name_finding(function: &FunctionFacts, path: &str) -> Option<Finding> {
    let bare = function.name.trim_matches('_').to_lowercase();
    if !UNINFORMATIVE_NAMES.contains(&bare.as_str()) {
        return None;
    }
    Some(Finding::new(
        Layer::Pragmatic,
        "uninformative_name",
        path,
        function.line,
        format!(
            "「{}」只说了动作没说对象：读者不知道它处理的是什么、结果长什么样",
            function.name
        ),
        Principle::Quality,
        "把宾语写进名字（handle_retry_of_failed_upload），或直接改成说清结果的名字",
    ))
}

fn docstring_echo_finding(function: &FunctionFacts, path: &str) -> Option<Finding> {
    let doc = function.doc.as_str();
    if doc.is_empty() || doc.chars().count() >= 40 {
        return None;
    }
    let mut rest = doc.to_lowercase();
    for token in function.name.to_lowercase().split('_') {
        if !token.is_empty() {
            rest = rest.replace(token, " ");
        }
    }
    let leftover: String = rest
        .chars()
        .filter(|ch| !ch.is_whitespace() && !"，。：；、,.:;".contains(*ch))
        .collect();
    if leftover.chars().count() > 2 {
        return None;
    }
    Some(Finding::new(
        Layer::Pragmatic,
        "docstring_echo",
        path,
        function.line,
        format!(
            "文档「{}」只是把函数名说了一遍：读者拿不到新信息",
            doc.chars().take(30).collect::<String>()
        ),
        Principle::Relation,
        "删掉，或改成说明「为什么这样做、什么情况下会失败」",
    ))
}

fn missing_why_finding(function: &FunctionFacts, path: &str, rules: &Rules) -> Option<Finding> {
    let threshold = rules.pragmatic.missing_why_lines;
    if function.lines() <= threshold
        || function.max_depth <= 2
        || function.has_comment
        || !function.doc.is_empty()
    {
        return None;
    }
    Some(Finding::new(
        Layer::Pragmatic,
        "missing_why",
        path,
        function.line,
        format!(
            "{} 有 {} 行、多处分支，却没有一句说明为什么这样做",
            function.name,
            function.lines()
        ),
        Principle::Quality,
        "在判断处补一行「为什么选这条分支」，不要复述代码",
    ))
}

fn vague_error_findings(facts: &Facts, path: &str, rules: &Rules) -> Vec<Finding> {
    let min_chars = rules.pragmatic.error_msg_min_chars;
    let mut out = Vec::new();
    for function in &facts.functions {
        for (line, text) in &function.error_messages {
            let trimmed = text.trim();
            if trimmed.is_empty() || trimmed.chars().count() >= min_chars {
                continue;
            }
            if ACTION_WORDS.iter().any(|word| trimmed.contains(word)) {
                continue;
            }
            out.push(Finding::new(
                Layer::Pragmatic,
                "vague_error",
                path,
                *line,
                format!("错误信息「{trimmed}」太短，读者看到它只知道自己错了，不知道该怎么做"),
                Principle::Manner,
                "补成「出了什么事 + 你该怎么做」，例如「配置文件缺 model 字段，请在 llm 段下补上」",
            ));
        }
    }
    out
}

pub fn check(facts: &Facts, path: &str, rules: &Rules) -> Vec<Finding> {
    let mut out = Vec::new();
    for function in &facts.functions {
        for finding in [
            name_finding(function, path),
            docstring_echo_finding(function, path),
            missing_why_finding(function, path, rules),
        ]
        .into_iter()
        .flatten()
        {
            out.push(finding);
        }
    }
    out.extend(vague_error_findings(facts, path, rules));
    out
}
