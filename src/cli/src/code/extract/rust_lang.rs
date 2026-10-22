//! Rust 提取器：用 `syn` 把源码摊成事实。

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprForLoop, ExprIf, ExprLoop, ExprMatch, ExprWhile, ImplItem, Item, Lit, Pat};

use crate::code::ir::{BindingFacts, ClassFacts, Facts, FunctionFacts};

/// 一趟走完量嵌套与认知负荷。
#[derive(Default)]
struct Metrics {
    level: usize,
    depth: usize,
    depth_line: usize,
    cognitive: usize,
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
}

/// 调用链长度：`a.b().c().d()` 是 3。
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

#[derive(Default)]
struct Chains {
    found: Vec<(usize, usize)>,
}

impl<'ast> Visit<'ast> for Chains {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let length = chain_len(&Expr::MethodCall(node.clone()));
        self.found.push((node.span().start().line, length));
        visit::visit_expr_method_call(self, node);
    }
}

/// 只收「会显示给人看的报错」：panic! / unreachable! / bail! 里的字符串，
/// 以及 `.expect("...")` 的第一个参数。普通字符串不算。
#[derive(Default)]
struct ErrorMessages {
    found: Vec<(usize, String)>,
}

fn collect_strings(tokens: &proc_macro2::TokenStream, out: &mut Vec<String>) {
    for token in tokens.clone() {
        match token {
            proc_macro2::TokenTree::Literal(literal) => {
                if let Ok(parsed) = syn::parse_str::<Lit>(&literal.to_string())
                    && let Lit::Str(text) = parsed
                {
                    out.push(text.value());
                }
            }
            proc_macro2::TokenTree::Group(group) => collect_strings(&group.stream(), out),
            _ => {}
        }
    }
}

impl<'ast> Visit<'ast> for ErrorMessages {
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let name = node
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default();
        if matches!(
            name.as_str(),
            "panic" | "unreachable" | "bail" | "ensure" | "expect"
        ) {
            let mut texts = Vec::new();
            collect_strings(&node.tokens, &mut texts);
            for text in texts {
                self.found.push((node.span().start().line, text));
            }
        }
        visit::visit_macro(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "expect"
            && let Some(first) = node.args.first()
            && let Expr::Lit(literal) = first
            && let Lit::Str(text) = &literal.lit
        {
            self.found.push((node.span().start().line, text.value()));
        }
        visit::visit_expr_method_call(self, node);
    }
}

fn doc_text(attrs: &[syn::Attribute]) -> String {
    let mut parts = Vec::new();
    for attribute in attrs {
        if !attribute.path().is_ident("doc") {
            continue;
        }
        if let syn::Meta::NameValue(pair) = &attribute.meta
            && let Expr::Lit(literal) = &pair.value
            && let Lit::Str(text) = &literal.lit
        {
            parts.push(text.value());
        }
    }
    parts.join(" ").trim().to_string()
}

fn has_comment(source: &str, start: usize, end: usize) -> bool {
    source
        .lines()
        .skip(start.saturating_sub(1))
        .take(end.saturating_sub(start) + 1)
        .any(|line| line.trim_start().starts_with("//") || line.contains(" //"))
}

fn function_facts(
    name: &str,
    attrs: &[syn::Attribute],
    sig: &syn::Signature,
    block: &syn::Block,
    source: &str,
) -> FunctionFacts {
    let start = sig.span().start().line;
    let end = block.span().end().line;
    let mut metrics = Metrics::default();
    metrics.visit_block(block);
    let mut chains = Chains::default();
    chains.visit_block(block);
    let mut messages = ErrorMessages::default();
    messages.visit_block(block);
    FunctionFacts {
        name: name.to_string(),
        line: start,
        end_line: end,
        params: sig.inputs.len(),
        doc: doc_text(attrs),
        max_depth: metrics.depth,
        max_depth_line: if metrics.depth_line == 0 {
            start
        } else {
            metrics.depth_line
        },
        cognitive: metrics.cognitive,
        has_comment: has_comment(source, start, end),
        chains: chains.found,
        error_messages: messages.found,
    }
}

/// 绑定的收集：循环变量、闭包参数、match 绑定算短命。
#[derive(Default)]
struct Bindings {
    found: Vec<BindingFacts>,
    short_scope: usize,
}

impl Bindings {
    fn record(&mut self, ident: &syn::Ident) {
        self.found.push(BindingFacts {
            name: ident.to_string(),
            line: ident.span().start().line,
            short_lived: self.short_scope > 0,
            init_shape: None,
        });
    }

    fn record_pat(&mut self, pat: &Pat) {
        if let Pat::Ident(binding) = pat {
            self.record(&binding.ident);
        }
    }
}

impl<'ast> Visit<'ast> for Bindings {
    fn visit_local(&mut self, node: &'ast syn::Local) {
        self.record_pat(&node.pat);
        if let (Pat::Ident(binding), Some(init)) = (&node.pat, &node.init) {
            let shape = init_shape(&init.expr);
            self.found.push(BindingFacts {
                name: binding.ident.to_string(),
                line: binding.ident.span().start().line,
                short_lived: self.short_scope > 0,
                init_shape: shape,
            });
        }
        visit::visit_local(self, node);
    }

    fn visit_expr_for_loop(&mut self, node: &'ast ExprForLoop) {
        self.short_scope += 1;
        self.record_pat(&node.pat);
        visit::visit_expr_for_loop(self, node);
        self.short_scope -= 1;
    }

    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        self.short_scope += 1;
        for input in &node.inputs {
            self.record_pat(input);
        }
        visit::visit_expr_closure(self, node);
        self.short_scope -= 1;
    }

    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        self.short_scope += 1;
        self.record_pat(&node.pat);
        visit::visit_arm(self, node);
        self.short_scope -= 1;
    }

    fn visit_signature(&mut self, node: &'ast syn::Signature) {
        for input in &node.inputs {
            if let syn::FnArg::Typed(typed) = input {
                self.record_pat(&typed.pat);
            }
        }
        visit::visit_signature(self, node);
    }
}

fn init_shape(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Macro(node) => Some(node.mac.path.segments.last()?.ident.to_string()),
        Expr::Call(node) => match &*node.func {
            Expr::Path(path) => Some(path.path.segments.last()?.ident.to_string()),
            _ => None,
        },
        Expr::Reference(node) => init_shape(&node.expr),
        _ => None,
    }
}

#[derive(Default)]
struct FalseConditions {
    lines: Vec<usize>,
}

impl<'ast> Visit<'ast> for FalseConditions {
    fn visit_expr(&mut self, node: &'ast Expr) {
        let is_false = match node {
            Expr::If(inner) => is_false_literal(&inner.cond),
            Expr::While(inner) => is_false_literal(&inner.cond),
            _ => false,
        };
        if is_false {
            self.lines.push(node.span().start().line);
        }
        visit::visit_expr(self, node);
    }
}

fn is_false_literal(expr: &Expr) -> bool {
    matches!(expr, Expr::Lit(literal) if matches!(&literal.lit, syn::Lit::Bool(value) if !value.value))
}

/// 走 syn 而不是 tree-sitter，是因为 Rust 这边要拿类型与签名，语法树给不出；
/// 分支多是逐项翻译 AST 的必然结果，不是判断多。
pub fn extract(source: &str) -> Result<Facts, String> {
    let tree = syn::parse_file(source).map_err(|error| error.to_string())?;
    let mut facts = Facts::default();

    for item in &tree.items {
        match item {
            Item::Fn(function) => facts.functions.push(function_facts(
                &function.sig.ident.to_string(),
                &function.attrs,
                &function.sig,
                &function.block,
                source,
            )),
            Item::Impl(block) => {
                let mut methods = 0;
                for inner in &block.items {
                    if let ImplItem::Fn(method) = inner {
                        methods += 1;
                        facts.functions.push(function_facts(
                            &method.sig.ident.to_string(),
                            &method.attrs,
                            &method.sig,
                            &method.block,
                            source,
                        ));
                    }
                }
                facts.classes.push(ClassFacts {
                    name: "impl".to_string(),
                    line: block.span().start().line,
                    methods,
                });
            }
            Item::Struct(structure) => facts.classes.push(ClassFacts {
                name: structure.ident.to_string(),
                line: structure.span().start().line,
                methods: 0,
            }),
            _ => {}
        }
    }

    facts.top_defs = tree
        .items
        .iter()
        .filter(|item| {
            matches!(
                item,
                Item::Fn(_)
                    | Item::Struct(_)
                    | Item::Enum(_)
                    | Item::Trait(_)
                    | Item::Impl(_)
                    | Item::Mod(_)
            )
        })
        .count();

    let mut bindings = Bindings::default();
    bindings.visit_file(&tree);
    facts.bindings = bindings.found;

    let mut false_conditions = FalseConditions::default();
    false_conditions.visit_file(&tree);
    facts.false_conditions = false_conditions.lines;

    Ok(facts)
}
