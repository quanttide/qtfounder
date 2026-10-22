//! Python 提取器：用 tree-sitter 把源码摊成事实。
//!
//! 只提取判定用得到的那几样：函数名与参数、嵌套与认知负荷、调用链、
//! 绑定与寿命、常量假条件、字符串字面量、文档字符串。

use tree_sitter::{Node, Parser};

use crate::code::ir::{BindingFacts, ClassFacts, Facts, FunctionFacts};

const NESTING: [&str; 8] = [
    "if_statement",
    "for_statement",
    "while_statement",
    "with_statement",
    "try_statement",
    "match_statement",
    "async_for_statement",
    "async_with_statement",
];
const DEFINITION: [&str; 2] = ["function_definition", "class_definition"];

fn line_of(node: Node<'_>) -> usize {
    node.start_position().row + 1
}

fn text_of(node: Node<'_>, source: &str) -> String {
    source.get(node.byte_range()).unwrap_or("").to_string()
}

/// 函数参数个数：`parameters` 下的命名子节点（含 default / typed / splat）。
fn param_count(parameters: Node<'_>) -> usize {
    let mut cursor = parameters.walk();
    parameters
        .named_children(&mut cursor)
        .filter(|child| {
            matches!(
                child.kind(),
                "identifier" | "default_parameter" | "typed_parameter" | "list_splat_pattern" | "dictionary_splat_pattern"
            )
        })
        .count()
}

/// 嵌套与认知负荷：一趟递归，进入嵌套语句加层、累加。
struct Depth {
    depth: usize,
    depth_line: usize,
    cognitive: usize,
}

impl Depth {
    fn walk(&mut self, node: Node<'_>, level: usize) {
        let mut cursor = node.walk();
        let children: Vec<Node<'_>> = node.named_children(&mut cursor).collect();
        for child in children {
            let nested = NESTING.contains(&child.kind());
            let reached = if nested { level + 1 } else { level };
            if nested {
                if reached > self.depth {
                    self.depth = reached;
                    self.depth_line = line_of(child);
                }
                self.cognitive += reached;
            }
            self.walk(child, reached);
        }
    }
}

/// 调用链长度：调用、属性访问、下标各算一层。
///
/// 口径必须与 Rust 提取器的 `chain_len`（MethodCall／Field／Await）对齐，
/// 否则同一条验收标准会在两种语言上给出不同结论。
fn chain_len(node: Node<'_>) -> usize {
    let mut length = 0;
    let mut current = Some(node);
    while let Some(node) = current {
        match node.kind() {
            "call" => {
                length += 1;
                current = node.child_by_field_name("function");
            }
            "attribute" => {
                length += 1;
                current = node.child_by_field_name("object");
            }
            "subscript" => {
                length += 1;
                current = node.child_by_field_name("value");
            }
            "await" => current = node.named_child(0),
            _ => break,
        }
    }
    length
}

fn collect_chains(node: Node<'_>, out: &mut Vec<(usize, usize)>) {
    if node.kind() == "call" {
        out.push((line_of(node), chain_len(node)));
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_chains(child, out);
    }
}

/// 只收 `raise` 语句里的字符串：那才是显示给人看的报错。
fn collect_error_messages(node: Node<'_>, source: &str, out: &mut Vec<(usize, String)>) {
    if node.kind() == "raise_statement" {
        collect_all_strings(node, source, out);
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_error_messages(child, source, out);
    }
}

fn collect_all_strings(node: Node<'_>, source: &str, out: &mut Vec<(usize, String)>) {
    if matches!(node.kind(), "string" | "concatenated_string") {
        out.push((line_of(node), text_of(node, source)));
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_all_strings(child, source, out);
    }
}

/// 文档字符串：函数体第一条语句是字符串。
fn docstring(body: Node<'_>, source: &str) -> String {
    let Some(first) = body.named_child(0) else { return String::new() };
    if first.kind() != "expression_statement" {
        return String::new();
    }
    let Some(inner) = first.named_child(0) else { return String::new() };
    if inner.kind() != "string" {
        return String::new();
    }
    text_of(inner, source).trim_matches(|ch| ch == '"' || ch == '\'').trim().to_string()
}

fn function_facts(node: Node<'_>, source: &str) -> FunctionFacts {
    let name = node
        .child_by_field_name("name")
        .map(|child| text_of(child, source))
        .unwrap_or_else(|| "<anonymous>".to_string());
    let params = node.child_by_field_name("parameters").map(param_count).unwrap_or(0);
    let body = node.child_by_field_name("body");
    let start = line_of(node);
    let end = node.end_position().row + 1;

    let mut depth = Depth { depth: 0, depth_line: 0, cognitive: 0 };
    if let Some(body) = body {
        depth.walk(body, 0);
    }

    let mut chains = Vec::new();
    let mut messages = Vec::new();
    if let Some(body) = body {
        collect_chains(body, &mut chains);
        collect_error_messages(body, source, &mut messages);
    }

    FunctionFacts {
        name,
        line: start,
        end_line: end,
        params,
        doc: body.map(|body| docstring(body, source)).unwrap_or_default(),
        max_depth: depth.depth,
        max_depth_line: if depth.depth_line == 0 { start } else { depth.depth_line },
        cognitive: depth.cognitive,
        has_comment: false, // 下面按行范围补
        chains,
        error_messages: messages,
    }
}

/// 简单绑定：赋值左侧、for 目标、with as、except as、lambda 参数。
fn collect_bindings(node: Node<'_>, source: &str, short: bool, out: &mut Vec<BindingFacts>) {
    let kind = node.kind();
    if kind == "assignment" {
        if let Some(left) = node.child_by_field_name("left") {
            push_binding(left, source, short, node, out);
        }
    }
    if matches!(kind, "for_statement" | "for_in_clause") {
        if let Some(left) = node.child_by_field_name("left") {
            push_binding(left, source, true, node, out);
        }
    }
    if kind == "function_definition"
        && let Some(parameters) = node.child_by_field_name("parameters")
    {
        let mut cursor = parameters.walk();
        for child in parameters.named_children(&mut cursor) {
            let target = match child.kind() {
                "identifier" => Some(child),
                "default_parameter" | "typed_parameter" => {
                    child.child_by_field_name("name").or_else(|| child.named_child(0))
                }
                "list_splat_pattern" | "dictionary_splat_pattern" => child.named_child(0),
                _ => None,
            };
            if let Some(target) = target {
                push_binding(target, source, false, child, out);
            }
        }
    }
    if kind == "with_item" || kind == "except_clause" || kind == "lambda" {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            push_binding(child, source, true, node, out);
        }
    }

    // 只有循环/推导式的目标本身算短命；循环体里的普通赋值在同一块内还要用，算长命
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_bindings(child, source, short, out);
    }
}

fn push_binding(target: Node<'_>, source: &str, short: bool, owner: Node<'_>, out: &mut Vec<BindingFacts>) {
    if target.kind() != "identifier" {
        return;
    }
    let shape = owner
        .child_by_field_name("right")
        .map(|right| match right.kind() {
            "list" => Some("list".to_string()),
            "dictionary" => Some("dict".to_string()),
            "set" => Some("set".to_string()),
            "call" => right
                .child_by_field_name("function")
                .map(|func| text_of(func, source)),
            _ => None,
        })
        .flatten();
    out.push(BindingFacts {
        name: text_of(target, source),
        line: line_of(target),
        short_lived: short,
        init_shape: shape,
    });
}

fn collect_false_conditions(node: Node<'_>, source: &str, out: &mut Vec<usize>) {
    if matches!(node.kind(), "if_statement" | "while_statement")
        && let Some(condition) = node.child_by_field_name("condition")
        && matches!(condition.kind(), "false" | "none")
    {
        out.push(line_of(node));
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_false_conditions(child, source, out);
    }
}

fn collect_classes(node: Node<'_>, source: &str, out: &mut Vec<ClassFacts>) {
    if node.kind() == "class_definition" {
        let name = node
            .child_by_field_name("name")
            .map(|child| text_of(child, source))
            .unwrap_or_else(|| "<anonymous>".to_string());
        let methods = node
            .child_by_field_name("body")
            .map(|body| {
                let mut cursor = body.walk();
                body.named_children(&mut cursor).filter(|child| child.kind() == "function_definition").count()
            })
            .unwrap_or(0);
        out.push(ClassFacts { name, line: line_of(node), methods });
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_classes(child, source, out);
    }
}

pub fn extract(source: &str) -> Result<Facts, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .map_err(|err| format!("Python 语法装不上：{err}"))?;
    let tree = parser.parse(source, None).ok_or("Python 解析失败")?;
    let root = tree.root_node();
    if root.has_error() {
        return Err("Python 源码有语法错误".to_string());
    }

    let mut facts = Facts::default();
    let mut cursor = root.walk();
    for item in root.named_children(&mut cursor) {
        if item.kind() == "function_definition" {
            facts.functions.push(function_facts(item, source));
        }
    }

    collect_classes(root, source, &mut facts.classes);

    // 类里的方法也算函数，一并摊平
    let mut cursor = root.walk();
    for item in root.named_children(&mut cursor) {
        if item.kind() != "class_definition" {
            continue;
        }
        if let Some(body) = item.child_by_field_name("body") {
            let mut inner = body.walk();
            for child in body.named_children(&mut inner) {
                if child.kind() == "function_definition" {
                    facts.functions.push(function_facts(child, source));
                }
            }
        }
    }

    facts.top_defs = {
        let mut cursor = root.walk();
        root.named_children(&mut cursor).filter(|child| DEFINITION.contains(&child.kind())).count()
    };

    let mut short_scope = Vec::new();
    collect_bindings(root, source, false, &mut short_scope);
    facts.bindings = short_scope;

    collect_false_conditions(root, source, &mut facts.false_conditions);

    // 注释按行范围补：函数体内出现 '#' 就算有
    let lines: Vec<&str> = source.lines().collect();
    for function in &mut facts.functions {
        let start = function.line.saturating_sub(1);
        let end = function.end_line.min(lines.len());
        function.has_comment = lines
            .get(start..end)
            .map(|slice| slice.iter().any(|line| line.trim_start().starts_with('#')))
            .unwrap_or(false);
    }

    Ok(facts)
}
