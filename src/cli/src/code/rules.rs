//! 阈值全在 `data/rules/code.toml`。改口径不碰代码。

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

/// 内嵌默认规则表：装成二进制后不必附带外部文件。
const DEFAULT_RULES: &str = include_str!("../../data/rules/code.toml");

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

impl Default for Rules {
    fn default() -> Self {
        toml::from_str(DEFAULT_RULES).expect("内嵌规则表必须是合法 TOML")
    }
}

impl Rules {
    /// 从文件读：给了路径就用它，否则用内嵌默认表。
    pub fn load(path: Option<&Path>) -> Result<Self, String> {
        match path {
            None => Ok(Rules::default()),
            Some(target) => {
                let text = std::fs::read_to_string(target)
                    .map_err(|err| format!("规则表读不了 {}：{err}", target.display()))?;
                toml::from_str(&text).map_err(|err| format!("规则表格式不对：{err}"))
            }
        }
    }

    pub fn is_whitelisted(&self, name: &str) -> bool {
        self.lexical.whitelist.iter().any(|item| item == name)
    }
}
