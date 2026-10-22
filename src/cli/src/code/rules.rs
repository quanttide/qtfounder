//! 阈值全在 `src/code/default-contract.toml`（本工具的软件工程契约，与判定代码放一起）。改口径不碰代码。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// 契约文件名：一份显性的配置文件，不是编译进二进制的常量。
pub const CONTRACT_FILE: &str = "default-contract.toml";
/// 指定契约的环境变量。
pub const CONTRACT_ENV: &str = "QTFOUNDER_CODE_CONTRACT";
/// 相对路径：在仓里跑时用它。
const CONTRACT_RELATIVE: [&str; 2] = ["src/code/default-contract.toml", "src/cli/src/code/default-contract.toml"];

#[derive(Debug, Clone, Deserialize)]
pub struct Rules {
    #[serde(default)]
    pub lexical: LexicalRules,
    #[serde(default)]
    pub syntax: SyntaxRules,
    #[serde(default)]
    pub module: ModuleRules,
    #[serde(default)]
    pub pragmatic: PragmaticRules,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LexicalRules {
    #[serde(default = "default_vague_names")]
    pub vague_names: Vec<String>,
    #[serde(default = "default_min_len")]
    pub min_len: usize,
    #[serde(default = "default_whitelist")]
    pub whitelist: Vec<String>,
    #[serde(default)]
    pub abbreviations: HashMap<String, String>,
    #[serde(default)]
    pub synonym_groups: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SyntaxRules {
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    #[serde(default = "default_max_function_lines")]
    pub max_function_lines: usize,
    #[serde(default = "default_max_params")]
    pub max_params: usize,
    #[serde(default = "default_max_cognitive")]
    pub max_cognitive: usize,
    #[serde(default = "default_max_chain")]
    pub max_chain: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModuleRules {
    #[serde(default = "default_max_top_defs")]
    pub max_top_defs: usize,
    #[serde(default = "default_max_methods")]
    pub max_methods: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PragmaticRules {
    #[serde(default = "default_error_min_chars")]
    pub error_msg_min_chars: usize,
    #[serde(default = "default_missing_why_lines")]
    pub missing_why_lines: usize,
}

fn default_vague_names() -> Vec<String> {
    ["data", "temp", "tmp", "flag", "val", "value", "res", "obj", "info", "thing", "stuff", "foo", "bar"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}
fn default_min_len() -> usize {
    3
}
fn default_whitelist() -> Vec<String> {
    [
        "id", "db", "os", "io", "re", "ai", "ui", "tz", "ts", "ok", "ms", "el", "it", "fp", "fs", "fn", "kv", "ex",
        "ev", "ci", "cd", "to", "of", "is", "in", "on", "by", "or", "up", "i", "j", "k", "x", "y", "z", "a", "b",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}
fn default_max_depth() -> usize {
    3
}
fn default_max_function_lines() -> usize {
    60
}
fn default_max_params() -> usize {
    5
}
fn default_max_cognitive() -> usize {
    15
}
fn default_max_chain() -> usize {
    4
}
fn default_max_top_defs() -> usize {
    15
}
fn default_max_methods() -> usize {
    15
}
fn default_error_min_chars() -> usize {
    12
}
fn default_missing_why_lines() -> usize {
    25
}

impl Default for LexicalRules {
    fn default() -> Self {
        Self {
            vague_names: default_vague_names(),
            min_len: default_min_len(),
            whitelist: default_whitelist(),
            abbreviations: HashMap::new(),
            synonym_groups: Vec::new(),
        }
    }
}

impl Default for SyntaxRules {
    fn default() -> Self {
        Self {
            max_depth: default_max_depth(),
            max_function_lines: default_max_function_lines(),
            max_params: default_max_params(),
            max_cognitive: default_max_cognitive(),
            max_chain: default_max_chain(),
        }
    }
}

impl Default for ModuleRules {
    fn default() -> Self {
        Self { max_top_defs: default_max_top_defs(), max_methods: default_max_methods() }
    }
}

impl Default for PragmaticRules {
    fn default() -> Self {
        Self {
            error_msg_min_chars: default_error_min_chars(),
            missing_why_lines: default_missing_why_lines(),
        }
    }
}

/// 找默认契约：环境变量 → 当前目录往上找 → 都没有就报错。
///
/// 刻意**不**内置兜底：契约是显性文件，找不到就明说，而不是偷偷用一份藏在二进制里的阈值。
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

    let start = std::env::current_dir().map_err(|err| format!("读不到当前目录：{err}"))?;
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
    Err(format!(
        "找不到默认契约 {CONTRACT_FILE}：用 --rules 指定，或设 {CONTRACT_ENV}，或放在 src/code/{CONTRACT_FILE}"
    ))
}

impl Rules {
    /// 读契约：给了路径就用它，否则按 [`find_contract`] 查找。
    pub fn load(path: Option<&Path>) -> Result<Self, String> {
        let target = match path {
            Some(given) => given.to_path_buf(),
            None => find_contract()?,
        };
        let text = std::fs::read_to_string(&target)
            .map_err(|err| format!("契约读不了 {}：{err}", target.display()))?;
        toml::from_str(&text).map_err(|err| format!("契约 {} 格式不对：{err}", target.display()))
    }

    pub fn is_whitelisted(&self, name: &str) -> bool {
        self.lexical.whitelist.iter().any(|item| item == name)
    }
}
