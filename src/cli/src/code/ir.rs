//! 中间表示：判定层只认这层事实。
//!
//! 一个语言写一个提取器，四层验收标准只写一遍。要加新语言，只需让它交出这个结构，
//! 判定层一行不改——这是「一份验收标准管多语言」能否成立的唯一验收。

/// 一个文件摊出来的事实。
#[derive(Debug, Default, Clone)]
pub struct Facts {
    pub functions: Vec<FunctionFacts>,
    pub classes: Vec<ClassFacts>,
    pub bindings: Vec<BindingFacts>,
    /// 条件恒为假的语句行号（`if false` / `while False`）
    pub false_conditions: Vec<usize>,
    /// 顶层定义数（函数、结构体/类、枚举、trait/接口、impl）
    pub top_defs: usize,
}

#[derive(Debug, Default, Clone)]
pub struct FunctionFacts {
    pub name: String,
    pub line: usize,
    pub end_line: usize,
    pub params: usize,
    /// 文档注释原文（没有就是空串）
    pub doc: String,
    /// 最深嵌套层数与其所在行
    pub max_depth: usize,
    pub max_depth_line: usize,
    /// 认知负荷（嵌套加权累加）
    pub cognitive: usize,
    /// 函数体里有没有注释
    pub has_comment: bool,
    /// 方法调用链：(行号, 链长)
    pub chains: Vec<(usize, usize)>,
    /// 错误信息候选：(行号, 内容)——只收「会显示给人看的报错」，不是所有字符串
    pub error_messages: Vec<(usize, String)>,
}

impl FunctionFacts {
    pub fn lines(&self) -> usize {
        self.end_line.saturating_sub(self.line) + 1
    }
}

#[derive(Debug, Default, Clone)]
pub struct ClassFacts {
    pub name: String,
    pub line: usize,
    pub methods: usize,
}

#[derive(Debug, Default, Clone)]
pub struct BindingFacts {
    pub name: String,
    pub line: usize,
    /// 只活几行：循环变量、闭包参数、match 分支绑定
    pub short_lived: bool,
    /// 初始化表达式的构造器名（`vec`、`HashMap`、`String`），判名实不符用
    pub init_shape: Option<String>,
}

impl Facts {
    pub fn function_names(&self) -> Vec<String> {
        self.functions.iter().map(|function| function.name.clone()).collect()
    }
}
