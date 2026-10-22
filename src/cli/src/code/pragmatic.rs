//! 语用层：名字诚不诚实、注释有没有解释「为什么」、错误信息能不能让人知道怎么修。
//!
//! 对应 Grice：质准则（不误导）与方式准则（不歧义）。

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ImplItem, Item, Lit};

use super::model::{Finding, Layer, Principle};
use super::rules::Rules;

const UNINFORMATIVE_NAMES: [&str; 15] = [
    "handle", "process", "do_it", "run", "exec", "execute", "work", "deal", "manage", "util", "utils", "helper",
    "helpers", "func", "stuff",
];
const ACTION_WORDS: [&str; 10] = ["请", "检查", "应该", "必须", "不能", "需要", "配置", "传入", "指定", "重试"];

/// 从 `#[doc = "..."]` 属性里取文档文本。
fn doc_text(attrs: &[syn::Attribute]) -> String {
    let mut parts = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        if let syn::Meta::NameValue(pair) = &attr.meta
            && let Expr::Lit(literal) = &pair.value
            && let Lit::Str(text) = &literal.lit
        {
            parts.push(text.value());
        }
    }
    parts.join(" ").trim().to_string()
}

fn name_finding(name: &str, line: usize, path: &str) -> Option<Finding> {
    let bare = name.trim_matches('_').to_lowercase();
    if !UNINFORMATIVE_NAMES.contains(&bare.as_str()) {
        return None;
    }
    Some(Finding::new(
        Layer::Pragmatic,
        "uninformative_name",
        path,
        line,
        format!("「{name}」只说了动作没说对象：读者不知道它处理的是什么、结果长什么样"),
        Principle::Quality,
        "把宾语写进名字（handle_retry_of_failed_upload），或直接改成说清结果的名字",
    ))
}

fn docstring_echo_finding(name: &str, doc: &str, line: usize, path: &str) -> Option<Finding> {
    if doc.is_empty() || doc.chars().count() >= 40 {
        return None;
    }
    let mut rest = doc.to_lowercase();
    for token in name.to_lowercase().split('_') {
        if !token.is_empty() {
            rest = rest.replace(token, " ");
        }
    }
    let leftover: String = rest.chars().filter(|ch| !ch.is_whitespace() && !"，。：；、,.::;".contains(*ch)).collect();
    if leftover.chars().count() > 2 {
        return None;
    }
    Some(Finding::new(
        Layer::Pragmatic,
        "docstring_echo",
        path,
        line,
        format!("文档「{}」只是把函数名说了一遍：读者拿不到新信息", doc.chars().take(30).collect::<String>()),
        Principle::Relation,
        "删掉，或改成说明「为什么这样做、什么情况下会失败」",
    ))
}

/// `panic!` / `expect` / `Err` 里的字符串是不是说了该怎么办。
struct MessageFinder<'a> {
    path: &'a str,
    min_chars: usize,
    out: Vec<Finding>,
}

impl MessageFinder<'_> {
    fn inspect(&mut self, text: &str, line: usize) {
        let trimmed = text.trim();
        if trimmed.chars().count() >= self.min_chars || trimmed.is_empty() {
            return;
        }
        if ACTION_WORDS.iter().any(|word| trimmed.contains(word)) {
            return;
        }
        self.out.push(Finding::new(
            Layer::Pragmatic,
            "vague_error",
            self.path,
            line,
            format!("错误信息「{trimmed}」太短，读者看到它只知道自己错了，不知道该怎么做"),
            Principle::Manner,
            "补成「出了什么事 + 你该怎么做」，例如「配置文件缺 model 字段，请在 [llm] 下补上」",
        ));
    }
}

impl<'ast> Visit<'ast> for MessageFinder<'_> {
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let name = node.path.segments.last().map(|segment| segment.ident.to_string()).unwrap_or_default();
        if matches!(name.as_str(), "panic" | "expect" | "unreachable" | "bail" | "ensure") {
            let mut texts = Vec::new();
            collect_strings(&node.tokens, &mut texts);
            for text in texts {
                self.inspect(&text, node.span().start().line);
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
            self.inspect(&text.value(), node.span().start().line);
        }
        visit::visit_expr_method_call(self, node);
    }
}

/// 从宏的 token 流里挑出字符串字面量。
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

fn missing_why_finding(
    name: &str,
    doc: &str,
    lines: usize,
    depth: usize,
    has_comment: bool,
    line: usize,
    path: &str,
    rules: &Rules,
) -> Option<Finding> {
    let threshold = rules.pragmatic.missing_why_lines;
    if lines <= threshold || depth <= 2 || has_comment || !doc.is_empty() {
        return None;
    }
    Some(Finding::new(
        Layer::Pragmatic,
        "missing_why",
        path,
        line,
        format!("{name} 有 {lines} 行、多处分支，却没有一句说明为什么这样做"),
        Principle::Quality,
        "在判断处补一行「为什么选这条分支」，不要复述代码",
    ))
}

/// 函数体行范围里有没有 `//` 注释。
fn has_comment(source: &str, start: usize, end: usize) -> bool {
    source
        .lines()
        .skip(start.saturating_sub(1))
        .take(end.saturating_sub(start) + 1)
        .any(|line| line.trim_start().starts_with("//") || line.contains(" //"))
}

pub fn check(syntax: &syn::File, path: &str, source: &str, rules: &Rules) -> Vec<Finding> {
    let mut out = Vec::new();

    let mut inspect_function = |name: &str,
                                attrs: &[syn::Attribute],
                                sig: &syn::Signature,
                                block: &syn::Block| {
        let start = sig.span().start().line;
        let end = block.span().end().line;
        let doc = doc_text(attrs);
        if let Some(finding) = name_finding(name, start, path) {
            out.push(finding);
        }
        if let Some(finding) = docstring_echo_finding(name, &doc, start, path) {
            out.push(finding);
        }
        let mut metrics = super::syntax::Metrics::default();
        metrics.visit_block(block);
        if let Some(finding) = missing_why_finding(
            name,
            &doc,
            end.saturating_sub(start) + 1,
            metrics.depth,
            has_comment(source, start, end),
            start,
            path,
            rules,
        ) {
            out.push(finding);
        }
    };

    for item in &syntax.items {
        match item {
            Item::Fn(function) => {
                inspect_function(&function.sig.ident.to_string(), &function.attrs, &function.sig, &function.block)
            }
            Item::Impl(block) => {
                for inner in &block.items {
                    if let ImplItem::Fn(method) = inner {
                        inspect_function(&method.sig.ident.to_string(), &method.attrs, &method.sig, &method.block);
                    }
                }
            }
            _ => {}
        }
    }

    let mut messages = MessageFinder { path, min_chars: rules.pragmatic.error_msg_min_chars, out: Vec::new() };
    messages.visit_file(syntax);
    out.extend(messages.out);
    out
}
