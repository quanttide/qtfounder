//! 语篇层：一个文件在讲几件事、有没有永远走不到的路。
//!
//! 对应 Grice：关系准则（只说相关的）。这一层只读 [`Facts`]。
//!
//! 「导入了却没用」不在这一层判：rustc 的 unused_imports 已经权威且零误报，
//! 靠语法树自己判会漏掉 derive 与 trait 方法这两种用法（实测过），重复做更差。

use super::model::{Finding, Layer, Principle};
use crate::code::ir::Facts;
use crate::code::rules::Rules;

pub fn check(facts: &Facts, path: &str, rules: &Rules) -> Vec<Finding> {
    let config = &rules.module;
    let mut out = Vec::new();

    if facts.top_defs > config.max_top_defs {
        out.push(Finding::new(
            Layer::Structure,
            "too_many_defs",
            path,
            0,
            format!(
                "一个文件里有 {} 个顶层定义（阈 {}）：读者记不住这个文件到底负责什么",
                facts.top_defs, config.max_top_defs
            ),
            Principle::Quantity,
            "按职责拆文件；拆不动说明它其实是两个模块",
        ));
    }

    for class in &facts.classes {
        if class.methods > config.max_methods {
            out.push(Finding::new(
                Layer::Structure,
                "too_many_methods",
                path,
                class.line,
                format!(
                    "{} 有 {} 个方法：调用方要在几十个入口里找",
                    class.name, class.methods
                ),
                Principle::Quantity,
                "按「谁会调它」拆开，或把成组的参数收进结构体",
            ));
        }
    }

    for line in &facts.false_conditions {
        out.push(Finding::new(
            Layer::Structure,
            "dead_branch",
            path,
            *line,
            "条件是常量假：这段永远不会执行，但读者要花时间确认这一点".to_string(),
            Principle::Relation,
            "删掉；要留作记录就写进提交信息",
        ));
    }
    out
}
