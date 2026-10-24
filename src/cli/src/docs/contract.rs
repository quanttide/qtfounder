//! 契约：一次读进内存，判定层只认它。
//!
//! 与 `language` 的契约同构——一份契约 = 一类介质的验收标准。查找顺序也一样：
//! `--rules` → 环境变量 → 从当前目录往上找 `src/docs/default-contract.toml` → 用户级目录。

use std::path::{Path, PathBuf};

use serde::Deserialize;

pub const CONTRACT_FILE: &str = "default-contract.toml";
pub const CONTRACT_ENV: &str = "QTFOUNDER_DOCS_CONTRACT";
const CONTRACT_RELATIVE: [&str; 2] = [
    "src/docs/default-contract.toml",
    "src/cli/src/docs/default-contract.toml",
];

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Contract {
    #[serde(default)]
    pub scope: Scope,
    #[serde(default)]
    pub table: Table,
    #[serde(default)]
    pub list: List,
    #[serde(default)]
    pub density: Density,
    #[serde(default)]
    pub bold: Bold,
    #[serde(default)]
    pub rule: Rule,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Scope {
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Table {
    #[serde(default)]
    pub max_columns: usize,
    #[serde(default)]
    pub max_cell_chars: usize,
    #[serde(default)]
    pub max_rows: usize,
    #[serde(default)]
    pub require_header: bool,
    #[serde(default)]
    pub flag_two_column: bool,
    #[serde(default)]
    pub min_rows: usize,
    #[serde(default)]
    pub sparse_ratio: f64,
    #[serde(default)]
    pub layout_headers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct List {
    /// 单项超过这个字数：本该是段落
    #[serde(default)]
    pub max_item_chars: usize,
    /// 三段式列表项（`- **要点**：解释`）占比上限
    #[serde(default)]
    pub label_ratio: f64,
    /// 少于这个项数不算结构块，一条两条本来就是一句话
    #[serde(default)]
    pub min_items: usize,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Density {
    /// 结构块（表格 + 列表）字数占正文比例上限
    #[serde(default)]
    pub max_block_ratio: f64,
    /// 没有过渡段落的连续结构块占比上限
    #[serde(default)]
    pub max_run_ratio: f64,
    /// 多长的段落才算把两串结构块隔开
    #[serde(default)]
    pub min_paragraph_chars: usize,
    /// 文档总字数低于它就不判
    #[serde(default)]
    pub min_chars: usize,
}

/// 加粗不超过几处：整句加粗、同行并列加粗、满篇重点都是 AI 的常见毛病。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Bold {
    /// 每千字加粗处数上限
    #[serde(default)]
    pub max_per_1000: usize,
    /// 同一行并列加粗处数上限
    #[serde(default)]
    pub max_per_line: usize,
    /// 单个加粗内的字数上限：超过就是把整句加粗了
    #[serde(default)]
    pub max_span_chars: usize,
    /// 文档低于这个字数就不算密度，短文档几处加粗就能把比例拉满
    #[serde(default)]
    pub min_chars: usize,
}

/// 分割线：分章靠标题，不靠横线。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Rule {
    /// 全文分割线处数上限
    #[serde(default)]
    pub max: usize,
    /// 紧挨标题的横线只是装饰，单独报
    #[serde(default)]
    pub flag_between_sections: bool,
}

/// 找契约：环境变量 → 当前目录往上找 → 用户级目录 → 都没有就报错。刻意不内置兜底。
pub fn find_contract() -> Result<PathBuf, String> {
    if let Ok(from_env) = std::env::var(CONTRACT_ENV)
        && !from_env.is_empty()
    {
        let path = PathBuf::from(&from_env);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!("{CONTRACT_ENV} 指向的契约不存在：{from_env}"));
    }
    let start = std::env::current_dir().map_err(|error| format!("读不到当前目录：{error}"))?;
    let mut dir = start.as_path();
    loop {
        for relative in CONTRACT_RELATIVE {
            let candidate = dir.join(relative);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let user_level = Path::new(&home).join(format!(".config/qtfounder/docs/{CONTRACT_FILE}"));
        if user_level.is_file() {
            return Ok(user_level);
        }
    }
    Err(format!(
        "找不到默认契约 {CONTRACT_FILE}：用 --rules 指定，或设 {CONTRACT_ENV}，或放在 src/docs/{CONTRACT_FILE}，或装到 ~/.config/qtfounder/docs/{CONTRACT_FILE}"
    ))
}

impl Contract {
    pub fn load(path: Option<&Path>) -> Result<Self, String> {
        let target = match path {
            Some(given) => given.to_path_buf(),
            None => find_contract()?,
        };
        let text = std::fs::read_to_string(&target)
            .map_err(|error| format!("契约读不了 {}：{error}", target.display()))?;
        toml::from_str(&text)
            .map_err(|error| format!("契约 {} 格式不对：{error}", target.display()))
    }
}
