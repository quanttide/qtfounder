//! 通用 tree-sitter 提取器：一份代码跑所有语言，差异全在 `specs::LangSpec` 里。
//!
//! 加一个语言时不用碰这个文件——这是「加语言只写配置」的兑现方式。

use tree_sitter::{Node, Parser};

use super::specs::LangSpec;
use crate::code::ir::{BindingFacts, ClassFacts, Facts, FunctionFacts};

fn line_of(node: Node<'_>) -> usize {
    node.start_position().row + 1
}

fn text_of(node: Node<'_>, source: &str) -> String {
    source.get(node.byte_range()).unwrap_or("").to_string()
}

/// 沿字段/子节点路径下钻，取最后那个节点的文本。
fn drill<'a>(node: Node<'a>, path: &[&str]) -> Option<Node<'a>> {
    let mut current = node;
    for step in path {
        current = current.child_by_field_name(step).or_else(|| {
            let mut cursor = current.walk();
            let found = current.named_children(&mut cursor).find(|child| child.kind() == *step);
            found
        })?;
    }
    Some(current)
}

fn identifier_text(node: Node<'_>, source: &str) -> Option<String> {
    if node.kind().contains("identifier") {
        return Some(text_of(node, source));
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|child| child.kind().contains("identifier"))
        .map(|child| text_of(child, source))
}

fn function_name(node: Node<'_>, source: &str, spec: &LangSpec) -> String {
    let container = if spec.name_path.is_empty() { Some(node) } else { drill(node, spec.name_path) };
    let Some(container) = container else { return "<anonymous>".to_string() };
    container
        .child_by_field_name("name")
        .or_else(|| {
            let mut cursor = container.walk();
            container.named_children(&mut cursor).find(|child| child.kind().contains("identifier"))
        })
        .and_then(|child| identifier_text(child, source))
        .unwrap_or_else(|| "<anonymous>".to_string())
}

fn parameter_count(node: Node<'_>, spec: &LangSpec) -> usize {
    let container = if spec.name_path.is_empty() { node } else { drill(node, spec.name_path).unwrap_or(node) };
    let Some(parameters) = container.child_by_field_name(spec.params_field) else { return 0 };
    let mut cursor = parameters.walk();
    let items: Vec<Node<'_>> = parameters
        .named_children(&mut cursor)
        .filter(|child| spec.param_item_nodes.contains(&child.kind()))
        .collect();
    // 一个参数条目里可能绑多个名字（Go 的 `a, b int`）
    items
        .iter()
        .map(|item| {
            let mut inner = item.walk();
            let names = item
                .named_children(&mut inner)
                .filter(|child| child.kind() == "identifier" || child.kind() == "field_identifier")
                .count();
            names.max(1)
        })
        .sum()
}

struct Depth {
    depth: usize,
    depth_line: usize,
    cognitive: usize,
}

impl Depth {
    fn walk(&mut self, node: Node<'_>, level: usize, spec: &LangSpec) {
        let mut cursor = node.walk();
        let children: Vec<Node<'_>> = node.named_children(&mut cursor).collect();
        for child in children {
            let nested = spec.nesting_nodes.contains(&child.kind());
            let reached = if nested { level + 1 } else { level };
            if nested {
                if reached > self.depth {
                    self.depth = reached;
                    self.depth_line = line_of(child);
                }
                self.cognitive += reached;
            }
            self.walk(child, reached, spec);
        }
    }
}

fn collect_strings(node: Node<'_>, source: &str, out: &mut Vec<(usize, String)>) {
    let kind = node.kind();
    if kind.contains("string") {
        out.push((line_of(node), text_of(node, source)));
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_strings(child, source, out);
    }
}

fn is_false_condition(node: Node<'_>, spec: &LangSpec) -> bool {
    let condition = node.child_by_field_name("condition").or_else(|| {
        let mut cursor = node.walk();
        node.named_children(&mut cursor).next()
    });
    match condition {
        Some(condition) => spec.false_nodes.contains(&condition.kind()),
        None => false,
    }
}

fn function_facts(node: Node<'_>, source: &str, spec: &LangSpec) -> (FunctionFacts, Vec<(usize, String)>) {
    let name = function_name(node, source, spec);
    let params = parameter_count(node, spec);
    let start = line_of(node);
    let end = node.end_position().row + 1;
    let body = node.child_by_field_name(spec.body_field);

    let mut depth = Depth { depth: 0, depth_line: 0, cognitive: 0 };
    if let Some(body) = body {
        depth.walk(body, 0, spec);
    }

    let mut messages = Vec::new();
    if let Some(body) = body {
        let mut cursor = body.walk();
        let candidates: Vec<Node<'_>> = body
            .named_children(&mut cursor)
            .filter(|child| spec.error_nodes.contains(&child.kind()))
            .collect();
        for candidate in candidates {
            collect_strings(candidate, source, &mut messages);
        }
    }

    let facts = FunctionFacts {
        name,
        line: start,
        end_line: end,
        params,
        doc: String::new(), // 三个语言的文档注释提取待补，见用户指南
        max_depth: depth.depth,
        max_depth_line: if depth.depth_line == 0 { start } else { depth.depth_line },
        cognitive: depth.cognitive,
        has_comment: false,
        chains: Vec::new(), // 调用链验收标准暂只对 Rust／Python 生效
        error_messages: Vec::new(),
    };
    (facts, messages)
}

/// 参数名也是绑定，而且是长命的那种。
fn collect_parameter_bindings(node: Node<'_>, source: &str, spec: &LangSpec, out: &mut Vec<BindingFacts>) {
    let container = if spec.name_path.is_empty() { node } else { drill(node, spec.name_path).unwrap_or(node) };
    let Some(parameters) = container.child_by_field_name(spec.params_field) else { return };
    let mut cursor = parameters.walk();
    for item in parameters.named_children(&mut cursor) {
        if !spec.param_item_nodes.contains(&item.kind()) {
            continue;
        }
        let mut inner = item.walk();
        for child in item.named_children(&mut inner) {
            if child.kind() == "identifier" || child.kind() == "field_identifier" {
                out.push(BindingFacts {
                    name: text_of(child, source),
                    line: line_of(child),
                    short_lived: false,
                    init_shape: None,
                });
            }
        }
        // Dart/Go 的参数里，名字可能出现成 `type name` 顺序
        if item.child_by_field_name("name").is_none() {
            let mut inner = item.walk();
            for child in item.named_children(&mut inner) {
                if child.kind().contains("identifier") && child.kind() != "type_identifier" {
                    out.push(BindingFacts {
                        name: text_of(child, source),
                        line: line_of(child),
                        short_lived: false,
                        init_shape: None,
                    });
                }
            }
        }
    }
}

fn collect_bindings(
    node: Node<'_>,
    source: &str,
    spec: &LangSpec,
    in_loop_target: bool,
    out: &mut Vec<BindingFacts>,
) {
    let kind = node.kind();
    if spec.assignment_nodes.contains(&kind)
        && let Some(left) = node.child_by_field_name(spec.assignment_left_field)
    {
        let mut cursor = left.walk();
        for child in left.named_children(&mut cursor).chain(std::iter::once(left)) {
            if child.kind().contains("identifier") {
                out.push(BindingFacts {
                    name: text_of(child, source),
                    line: line_of(child),
                    short_lived: in_loop_target,
                    init_shape: node
                        .child_by_field_name(spec.assignment_right_field)
                        .map(|right| right.kind().to_string()),
                });
            }
        }
    }
    let loop_target = in_loop_target || spec.loop_nodes.contains(&kind);
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_bindings(child, source, spec, loop_target, out);
    }
}

fn collect_false_conditions(node: Node<'_>, spec: &LangSpec, out: &mut Vec<usize>) {
    if matches!(node.kind(), "if_statement" | "while_statement" | "if_element") && is_false_condition(node, spec) {
        out.push(line_of(node));
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_false_conditions(child, spec, out);
    }
}

fn collect_classes(node: Node<'_>, source: &str, spec: &LangSpec, out: &mut Vec<ClassFacts>) {
    if matches!(node.kind(), "class_declaration" | "class_definition") {
        let name = node
            .child_by_field_name("name")
            .and_then(|child| identifier_text(child, source))
            .unwrap_or_else(|| "<anonymous>".to_string());
        let methods = node
            .child_by_field_name("body")
            .map(|body| {
                let mut cursor = body.walk();
                body.named_children(&mut cursor)
                    .filter(|child| spec.function_nodes.contains(&child.kind()))
                    .count()
            })
            .unwrap_or(0);
        out.push(ClassFacts { name, line: line_of(node), methods });
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_classes(child, source, spec, out);
    }
}

pub fn extract(source: &str, spec: &LangSpec) -> Result<Facts, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&(spec.language)())
        .map_err(|err| format!("{} 语法装不上：{err}", spec.name))?;
    let tree = parser.parse(source, None).ok_or_else(|| format!("{} 解析失败", spec.name))?;
    let root = tree.root_node();

    let mut facts = Facts::default();

    // 顶层函数：只收不在别的函数里的（闭包与内部函数算外层一次）
    let mut cursor = root.walk();
    let top_level: Vec<Node<'_>> = root.named_children(&mut cursor).collect();
    for item in &top_level {
        if spec.function_nodes.contains(&item.kind()) {
            let (function, messages) = function_facts(*item, source, spec);
            let mut function = function;
            function.error_messages = messages;
            facts.functions.push(function);
        }
    }

    collect_classes(root, source, spec, &mut facts.classes);
    for item in &top_level {
        if spec.function_nodes.contains(&item.kind()) {
            collect_parameter_bindings(*item, source, spec, &mut facts.bindings);
        }
    }
    collect_bindings(root, source, spec, false, &mut facts.bindings);
    collect_false_conditions(root, spec, &mut facts.false_conditions);

    facts.top_defs = top_level
        .iter()
        .filter(|item| spec.definition_nodes.contains(&item.kind()))
        .count();

    let lines: Vec<&str> = source.lines().collect();
    for function in &mut facts.functions {
        let start = function.line.saturating_sub(1);
        let end = function.end_line.min(lines.len());
        function.has_comment = lines
            .get(start..end)
            .map(|slice| slice.iter().any(|line| line.trim_start().starts_with(spec.comment_prefix)))
            .unwrap_or(false);
    }

    Ok(facts)
}
