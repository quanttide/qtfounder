//! 词汇层：命名有信息量吗、名实相符吗、同一个概念有几种说法。
//!
//! 对应 Grice：量准则（信息量不足）与质准则（命名不诚实）。

use std::collections::HashSet;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, Item, Pat, Stmt};

use super::model::{Finding, Layer, Principle};
use super::rules::Rules;

/// 收集「定义名字」的位置，并标出它活多久。
///
/// 循环变量、闭包参数、match 分支绑定只活几行，读者不用记住——短名在这里是惯例，不是缺陷。
#[derive(Default)]
struct NameCollector {
    names: Vec<(String, usize, bool)>,
    short_scope: usize,
}

impl NameCollector {
    fn record(&mut self, ident: &syn::Ident) {
        let line = ident.span().start().line;
        self.names.push((ident.to_string(), line, self.short_scope > 0));
    }

    fn record_pat(&mut self, pat: &Pat) {
        if let Pat::Ident(binding) = pat {
            self.record(&binding.ident);
        }
    }
}

impl<'ast> Visit<'ast> for NameCollector {
    fn visit_local(&mut self, node: &'ast syn::Local) {
        self.record_pat(&node.pat);
        visit::visit_local(self, node);
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
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

fn name_finding(name: &str, line: usize, short_lived: bool, path: &str, rules: &Rules) -> Option<Finding> {
    let lexical = &rules.lexical;
    if name == "_" {
        return None;
    }
    if lexical.vague_names.iter().any(|item| item == name) {
        return Some(Finding::new(
            Layer::Lexical,
            "vague_name",
            path,
            line,
            format!("「{name}」没告诉读者它是什么：凡是叫这个名字的都得回头读代码才知道装的是什么"),
            Principle::Quantity,
            "换成说清内容的词（如 pending_orders / retry_budget），别用万能名",
        ));
    }
    if name.chars().count() < lexical.min_len && !short_lived && !rules.is_whitelisted(name) {
        return Some(Finding::new(
            Layer::Lexical,
            "short_name",
            path,
            line,
            format!("「{name}」只有 {} 个字符，而它要在整个函数里被记住：读者得靠上下文猜它是什么", name.chars().count()),
            Principle::Manner,
            "换成完整词。循环变量、闭包参数、match 绑定里的短名不报——它们只活几行",
        ));
    }
    if let Some(full) = lexical.abbreviations.get(name) {
        return Some(Finding::new(
            Layer::Lexical,
            "abbreviation",
            path,
            line,
            format!("缩写「{name}」至少有两种读法（{full}？）：团队里没有唯一约定时读者会读错"),
            Principle::Manner,
            "写全名，或在模块文档里展开一次",
        ));
    }
    None
}

/// 名字说是什么类型，初始化却是另一种叫得出名字的类型。
fn type_mismatch_finding(local: &syn::Local, path: &str) -> Option<Finding> {
    let Pat::Ident(binding) = &local.pat else { return None };
    let name = binding.ident.to_string().to_lowercase();
    let init = local.init.as_ref()?;
    let init_text = quote_shape(&init.expr);
    let expects = ["list", "items", "vec", "dict", "map", "str", "text"]
        .iter()
        .find(|suffix| name.ends_with(*suffix))?;
    let actual = init_text?;
    let compatible = match *expects {
        "dict" | "map" => actual.contains("HashMap") || actual.contains("BTreeMap"),
        "list" | "items" | "vec" => actual.contains("vec") || actual.contains("Vec"),
        _ => actual.contains("String") || actual.contains("str"),
    };
    if compatible {
        return None;
    }
    Some(Finding::new(
        Layer::Lexical,
        "name_type_mismatch",
        path,
        local.span().start().line,
        format!("「{}」的名字说是 {expects}，赋的却是 {actual}", binding.ident),
        Principle::Quality,
        "要么改名字，要么改值——名字不诚实会让人按错的假设改代码",
    ))
}

/// 初始化表达式的大致形状（只看构造器名字，够判名实不符）。
fn quote_shape(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Macro(node) => Some(node.mac.path.segments.last()?.ident.to_string()),
        Expr::Call(node) => match &*node.func {
            Expr::Path(path) => Some(path.path.segments.last()?.ident.to_string()),
            _ => None,
        },
        Expr::Reference(node) => quote_shape(&node.expr),
        _ => None,
    }
}

pub fn check(syntax: &syn::File, path: &str, rules: &Rules) -> Vec<Finding> {
    let mut collector = NameCollector::default();
    collector.visit_file(syntax);

    let mut out = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for (name, line, short_lived) in &collector.names {
        if seen.contains(name) {
            continue;
        }
        if let Some(finding) = name_finding(name, *line, *short_lived, path, rules) {
            seen.insert(name.clone());
            out.push(finding);
        }
    }

    for item in &syntax.items {
        if let Item::Fn(function) = item {
            for stmt in &function.block.stmts {
                if let Stmt::Local(local) = stmt {
                    if let Some(finding) = type_mismatch_finding(local, path) {
                        out.push(finding);
                    }
                }
            }
        }
    }
    out
}

/// 仓库级：同一个动作出现多种说法。
pub fn check_consistency(functions: &[String], path: &str, rules: &Rules) -> Vec<Finding> {
    let mut out = Vec::new();
    for group in &rules.lexical.synonym_groups {
        let used: HashSet<&String> = functions.iter().filter(|name| group.iter().any(|word| word == *name)).collect();
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
            format!("同一种动作在仓库里有 {} 种说法（{}）：读者每次都要重新判断它们是不是一回事", used.len(), shown.join("、")),
            Principle::Relation,
            "选一种作为正名，其余改名或写进规则表当同义词",
        ));
    }
    out
}
