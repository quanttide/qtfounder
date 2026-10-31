//! 句法层：读者要同时记住多少东西。
//!
//! 对应 Grice：方式准则（表达不清晰）。不是圈复杂度——数的是「人读起来有多绕」。
//! 这一层只读 [`Facts`]。

use super::model::{Finding, Layer, Principle};
use crate::code::ir::{Facts, FunctionFacts};
use crate::code::rules::Rules;

fn function_findings(function: &FunctionFacts, path: &str, rules: &Rules) -> Vec<Finding> {
    let config = &rules.syntax;
    let name = function.name.as_str();
    let lines = function.lines();
    let params = function.params;
    let mut out = Vec::new();

    if params > config.max_params {
        out.push(Finding::new(
            Layer::Syntax,
            "too_many_params",
            path,
            function.line,
            format!("{name} 有 {params} 个参数：读者得同时记住 {params} 个位置的含义"),
            Principle::Quantity,
            "把成组的参数收进一个结构体或具名参数",
        ));
    }
    if lines > config.max_function_lines {
        out.push(Finding::new(
            Layer::Syntax,
            "long_function",
            path,
            function.line,
            format!("{name} 跨 {lines} 行：一次读不完，回头找某一步要滚动"),
            Principle::Quantity,
            "按「做完一件事就返回」切开，或把内层循环体提成具名函数",
        ));
    }
    if function.cognitive > config.max_cognitive {
        out.push(Finding::new(
            Layer::Syntax,
            "cognitive_load",
            path,
            function.line,
            format!(
                "{name} 的认知负荷约 {}（阈 {}）：嵌套叠加让读者要维护条件栈",
                function.cognitive, config.max_cognitive
            ),
            Principle::Manner,
            "提前返回压平嵌套，或把条件提成命名布尔值",
        ));
    }
    if function.max_depth > config.max_depth {
        out.push(Finding::new(
            Layer::Syntax,
            "deep_nesting",
            path,
            function.max_depth_line,
            format!(
                "{name} 里嵌套 {} 层（阈 {}）：读者要同时追踪 {} 个条件",
                function.max_depth, config.max_depth, function.max_depth
            ),
            Principle::Manner,
            "提前返回，或把最深那段提成函数并给它一个说清意图的名字",
        ));
    }
    out
}

pub fn check(facts: &Facts, path: &str, rules: &Rules) -> Vec<Finding> {
    let mut out = Vec::new();
    for function in &facts.functions {
        out.extend(function_findings(function, path, rules));
        // 同一条链上的每个节点都会被访问到：按行取最大链长，一行只报一次
        let mut longest: std::collections::BTreeMap<usize, usize> =
            std::collections::BTreeMap::new();
        for (line, length) in &function.chains {
            let entry = longest.entry(*line).or_insert(0);
            *entry = (*entry).max(*length);
        }
        for (line, length) in &longest {
            if *length <= rules.syntax.max_chain {
                continue;
            }
            out.push(Finding::new(
                Layer::Syntax,
                "long_chain",
                path,
                *line,
                format!("一行里链了 {length} 层调用：出错时读者不知道断在哪一环"),
                Principle::Manner,
                "拆成几行并给中间结果起名",
            ));
        }
    }
    out
}
