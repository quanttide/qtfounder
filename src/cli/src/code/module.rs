//! 语篇层：一个文件在讲几件事、导入有没有用上、有没有永远走不到的路。
//!
//! 对应 Grice：关系准则（只说相关的）。

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ImplItem, Item};

use super::model::{Finding, Layer, Principle};
use super::rules::Rules;

fn dead_branch_finding(expr: &Expr, path: &str) -> Option<Finding> {
    let line = expr.span().start().line;
    let dead = match expr {
        Expr::If(node) => is_false_literal(&node.cond),
        Expr::While(node) => is_false_literal(&node.cond),
        _ => false,
    };
    if !dead {
        return None;
    }
    Some(Finding::new(
        Layer::Structure,
        "dead_branch",
        path,
        line,
        "条件是常量 false：这段永远不会执行，但读者要花时间确认这一点".to_string(),
        Principle::Relation,
        "删掉；要留作记录就写进提交信息",
    ))
}

fn is_false_literal(expr: &Expr) -> bool {
    matches!(expr, Expr::Lit(literal) if matches!(&literal.lit, syn::Lit::Bool(value) if !value.value))
}

struct DeadBranchFinder<'a> {
    path: &'a str,
    out: Vec<Finding>,
}

impl<'ast> Visit<'ast> for DeadBranchFinder<'_> {
    fn visit_expr(&mut self, node: &'ast Expr) {
        if let Some(finding) = dead_branch_finding(node, self.path) {
            self.out.push(finding);
        }
        visit::visit_expr(self, node);
    }
}

pub fn check(syntax: &syn::File, path: &str, rules: &Rules) -> Vec<Finding> {
    let config = &rules.module;
    let mut out = Vec::new();
    // 「导入了却没用」不在这一层判：rustc 的 unused_imports 已经权威且零误报，
    // 我们靠 AST 判只会漏掉 derive 与 trait 方法这两种用法，重复做更差。

    let top = syntax
        .items
        .iter()
        .filter(|item| matches!(item, Item::Fn(_) | Item::Struct(_) | Item::Enum(_) | Item::Trait(_) | Item::Impl(_)))
        .count();
    if top > config.max_top_defs {
        out.push(Finding::new(
            Layer::Structure,
            "too_many_defs",
            path,
            0,
            format!("一个文件里有 {top} 个顶层定义（阈 {}）：读者记不住这个文件到底负责什么", config.max_top_defs),
            Principle::Quantity,
            "按职责拆文件；拆不动说明它其实是两个模块",
        ));
    }

    for item in &syntax.items {
        if let Item::Impl(block) = item {
            let methods = block.items.iter().filter(|inner| matches!(inner, ImplItem::Fn(_))).count();
            if methods > config.max_methods {
                out.push(Finding::new(
                    Layer::Structure,
                    "too_many_methods",
                    path,
                    block.span().start().line,
                    format!("这个 impl 有 {methods} 个方法：调用方要在几十个入口里找"),
                    Principle::Quantity,
                    "按「谁会调它」拆开，或把成组的参数收进结构体",
                ));
            }
        }
    }

    let mut dead = DeadBranchFinder { path, out: Vec::new() };
    dead.visit_file(syntax);
    out.extend(dead.out);
    out
}
