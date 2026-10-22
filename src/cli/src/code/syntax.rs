//! 句法层：读者要同时记住多少东西。
//!
//! 对应 Grice：方式准则（表达不清晰）。不是圈复杂度——数的是「人读起来有多绕」，
//! 嵌套比分支数量更伤人。

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprForLoop, ExprIf, ExprLoop, ExprMatch, ExprWhile, ImplItem, Item};

use super::model::{Finding, Layer, Principle};
use super::rules::Rules;

/// 一趟走完量两个指标：最深嵌套层数、认知负荷。
#[derive(Default)]
pub struct Metrics {
    level: usize,
    pub depth: usize,
    pub depth_line: usize,
    pub cognitive: usize,
}

impl Metrics {
    fn enter(&mut self, line: usize) {
        self.level += 1;
        if self.level > self.depth {
            self.depth = self.level;
            self.depth_line = line;
        }
        // 嵌套越深代价越大：读者要同时记住的条件栈就是这么长起来的
        self.cognitive += self.level;
    }

    fn leave(&mut self) {
        self.level -= 1;
    }
}

macro_rules! visit_nested {
    ($method:ident, $node:ty) => {
        fn $method(&mut self, node: &'ast $node) {
            self.enter(node.span().start().line);
            visit::$method(self, node);
            self.leave();
        }
    };
}

impl<'ast> Visit<'ast> for Metrics {
    visit_nested!(visit_expr_if, ExprIf);
    visit_nested!(visit_expr_for_loop, ExprForLoop);
    visit_nested!(visit_expr_while, ExprWhile);
    visit_nested!(visit_expr_match, ExprMatch);
    visit_nested!(visit_expr_loop, ExprLoop);

    fn visit_expr_break(&mut self, node: &'ast syn::ExprBreak) {
        self.cognitive += 1;
        visit::visit_expr_break(self, node);
    }

    fn visit_expr_continue(&mut self, node: &'ast syn::ExprContinue) {
        self.cognitive += 1;
        visit::visit_expr_continue(self, node);
    }

}

/// 方法调用链有几层：`a.b().c().d()` 是 3。
fn chain_len(expr: &Expr) -> usize {
    let mut length = 0;
    let mut current = expr;
    loop {
        match current {
            Expr::MethodCall(call) => {
                length += 1;
                current = &call.receiver;
            }
            Expr::Field(field) => {
                length += 1;
                current = &field.base;
            }
            Expr::Await(awaited) => current = &awaited.base,
            Expr::Try(tried) => current = &tried.expr,
            _ => return length,
        }
    }
}

fn function_lines(func_span_start: usize, func_span_end: usize) -> usize {
    func_span_end.saturating_sub(func_span_start) + 1
}

fn function_findings(
    name: &str,
    sig: &syn::Signature,
    body: &syn::Block,
    path: &str,
    rules: &Rules,
) -> Vec<Finding> {
    let config = &rules.syntax;
    let mut out = Vec::new();
    let start = sig.span().start().line;
    let end = body.span().end().line;
    let lines = function_lines(start, end);
    let params = sig.inputs.len();

    if params > config.max_params {
        out.push(Finding::new(
            Layer::Syntax,
            "too_many_params",
            path,
            start,
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
            start,
            format!("{name} 跨 {lines} 行：一次读不完，回头找某一步要滚动"),
            Principle::Quantity,
            "按「做完一件事就返回」切开，或把内层循环体提成具名函数",
        ));
    }

    let mut metrics = Metrics::default();
    metrics.visit_block(body);
    if metrics.cognitive > config.max_cognitive {
        out.push(Finding::new(
            Layer::Syntax,
            "cognitive_load",
            path,
            start,
            format!("{name} 的认知负荷约 {}（阈 {}）：嵌套叠加让读者要维护条件栈", metrics.cognitive, config.max_cognitive),
            Principle::Manner,
            "提前返回压平嵌套，或把条件提成命名布尔值",
        ));
    }
    if metrics.depth > config.max_depth {
        out.push(Finding::new(
            Layer::Syntax,
            "deep_nesting",
            path,
            metrics.depth_line,
            format!("{name} 里嵌套 {} 层（阈 {}）：读者要同时追踪 {} 个条件", metrics.depth, config.max_depth, metrics.depth),
            Principle::Manner,
            "提前返回，或把最深那段提成函数并给它一个说清意图的名字",
        ));
    }
    out
}

struct ChainFinder<'a> {
    path: &'a str,
    max_chain: usize,
    out: Vec<Finding>,
}

impl<'ast> Visit<'ast> for ChainFinder<'_> {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let length = chain_len(&Expr::MethodCall(node.clone()));
        if length > self.max_chain {
            self.out.push(Finding::new(
                Layer::Syntax,
                "long_chain",
                self.path,
                node.span().start().line,
                format!("一行里链了 {length} 层调用：出错时读者不知道断在哪一环"),
                Principle::Manner,
                "拆成几行并给中间结果起名",
            ));
        }
        visit::visit_expr_method_call(self, node);
    }
}

pub fn check(syntax: &syn::File, path: &str, rules: &Rules) -> Vec<Finding> {
    let mut out = Vec::new();
    for item in &syntax.items {
        match item {
            Item::Fn(function) => out.extend(function_findings(
                &function.sig.ident.to_string(),
                &function.sig,
                &function.block,
                path,
                rules,
            )),
            Item::Impl(block) => {
                for inner in &block.items {
                    if let ImplItem::Fn(method) = inner {
                        out.extend(function_findings(
                            &method.sig.ident.to_string(),
                            &method.sig,
                            &method.block,
                            path,
                            rules,
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    let mut chains = ChainFinder { path, max_chain: rules.syntax.max_chain, out: Vec::new() };
    chains.visit_file(syntax);
    out.extend(chains.out);
    out
}
