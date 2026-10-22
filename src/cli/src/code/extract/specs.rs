//! 各语言的语法树口径：节点名与下钻路径。
//!
//! 加一个语言就是加一份 [`LangSpec`]——提取器是通用的，见 `super::tree_sitter_extractor`。
//! 这份文件就是「一条验收标准管多语言」的配置面。

use tree_sitter::Language;

pub struct LangSpec {
    /// 语法标识，报错时用
    pub name: &'static str,
    /// 装语法
    pub language: fn() -> Language,
    /// 哪些节点算「函数」
    pub function_nodes: &'static [&'static str],
    /// 从函数节点走到名字的下钻路径（Go 的 `parameter_list` 直接挂在函数上，Dart 的要先进 signature）
    pub name_path: &'static [&'static str],
    /// 参数列表所在字段名
    pub params_field: &'static str,
    /// 参数列表里，哪些节点算「一个参数条目」
    pub param_item_nodes: &'static [&'static str],
    /// 函数体字段名
    pub body_field: &'static str,
    /// 算作嵌套的节点
    pub nesting_nodes: &'static [&'static str],
    /// 常量假条件的关键字节点
    pub false_nodes: &'static [&'static str],
    /// 「一条语句带一个字符串」的节点：错误信息候选
    pub error_nodes: &'static [&'static str],
    /// 顶层定义节点（判「一个文件讲几件事」）
    pub definition_nodes: &'static [&'static str],
    /// 赋值节点与其右值字段（判绑定与寿命）
    pub assignment_nodes: &'static [&'static str],
    pub assignment_left_field: &'static str,
    pub assignment_right_field: &'static str,
    /// for/range 循环节点（其目标算短命）
    pub loop_nodes: &'static [&'static str],
    /// 行注释前缀
    pub comment_prefix: &'static str,
}

pub fn go() -> LangSpec {
    LangSpec {
        name: "go",
        language: || tree_sitter_go::LANGUAGE.into(),
        function_nodes: &["function_declaration", "method_declaration", "func_literal"],
        name_path: &[],
        params_field: "parameters",
        param_item_nodes: &["parameter_declaration", "variadic_parameter_declaration"],
        body_field: "body",
        nesting_nodes: &[
            "if_statement",
            "for_statement",
            "expression_switch_statement",
            "type_switch_statement",
            "select_statement",
        ],
        false_nodes: &["false"],
        error_nodes: &["call_expression"],
        definition_nodes: &[
            "function_declaration",
            "method_declaration",
            "type_declaration",
            "var_declaration",
            "const_declaration",
        ],
        assignment_nodes: &["short_var_declaration", "assignment_statement"],
        assignment_left_field: "left",
        assignment_right_field: "right",
        loop_nodes: &["for_statement"],
        comment_prefix: "//",
    }
}

pub fn typescript() -> LangSpec {
    LangSpec {
        name: "typescript",
        language: || tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        function_nodes: &[
            "function_declaration",
            "method_definition",
            "arrow_function",
        ],
        name_path: &[],
        params_field: "parameters",
        param_item_nodes: &["required_parameter", "optional_parameter", "rest_pattern"],
        body_field: "body",
        nesting_nodes: &[
            "if_statement",
            "for_statement",
            "for_in_statement",
            "while_statement",
            "switch_statement",
            "try_statement",
        ],
        false_nodes: &["false"],
        error_nodes: &["throw_statement"],
        definition_nodes: &[
            "function_declaration",
            "class_declaration",
            "interface_declaration",
            "type_alias_declaration",
            "lexical_declaration",
        ],
        assignment_nodes: &["variable_declarator"],
        assignment_left_field: "name",
        assignment_right_field: "value",
        loop_nodes: &["for_statement", "for_in_statement"],
        comment_prefix: "//",
    }
}

pub fn dart() -> LangSpec {
    LangSpec {
        name: "dart",
        language: || tree_sitter_dart::LANGUAGE.into(),
        function_nodes: &["function_declaration", "method_declaration"],
        // Dart 的 name 与 parameters 都在 signature 里
        name_path: &["signature"],
        params_field: "parameters",
        param_item_nodes: &["formal_parameter"],
        body_field: "body",
        nesting_nodes: &[
            "if_statement",
            "for_statement",
            "while_statement",
            "switch_statement",
            "try_statement",
        ],
        false_nodes: &["false"],
        error_nodes: &["throw_expression", "throw_statement"],
        definition_nodes: &[
            "function_declaration",
            "class_definition",
            "mixin_declaration",
            "extension_declaration",
            "enum_declaration",
        ],
        assignment_nodes: &["initialized_variable_definition", "variable_declaration"],
        assignment_left_field: "name",
        assignment_right_field: "value",
        loop_nodes: &["for_statement"],
        comment_prefix: "//",
    }
}
