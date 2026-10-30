//! 契约：命名、说明、正文三条口径，一次读进内存，判断层只认它。
//!
//! 与 `docs`／`language` 同构：一份契约 = 一类判定口径。查找顺序也一样：
//! `--rules` → 环境变量 → 从当前目录往上找 `src/agent/default-contract.toml` → 用户级目录。
//!
//! 口径还在试，所以刻意不内置兜底：拿不准的时候就报错，别悄悄按旧标准判。

use std::path::{Path, PathBuf};

use serde::Deserialize;

pub const CONTRACT_FILE: &str = "default-contract.toml";
pub const CONTRACT_ENV: &str = "QTFOUNDER_AGENT_CONTRACT";
const CONTRACT_RELATIVE: [&str; 3] = [
    "src/agent/default-contract.toml",
    "src/cli/src/agent/default-contract.toml",
    "apps/qtfounder/src/cli/src/agent/default-contract.toml",
];

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Contract {
    #[serde(default)]
    pub scope: Scope,
    #[serde(default)]
    pub name: Name,
    #[serde(default)]
    pub description: Description,
    #[serde(default)]
    pub body: Body,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Scope {
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Name {
    #[serde(default = "default_segments")]
    pub segments: usize,
    #[serde(default = "default_name_chars")]
    pub max_chars: usize,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub cli_scopes: Vec<String>,
    #[serde(default)]
    pub noun_suffixes: Vec<String>,
}

impl Default for Name {
    fn default() -> Self {
        Self {
            segments: default_segments(),
            max_chars: default_name_chars(),
            scopes: Vec::new(),
            cli_scopes: Vec::new(),
            noun_suffixes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Description {
    #[serde(default = "default_description_chars")]
    pub max_chars: usize,
    #[serde(default)]
    pub trigger_words: Vec<String>,
}

impl Default for Description {
    fn default() -> Self {
        Self {
            max_chars: default_description_chars(),
            trigger_words: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Body {
    #[serde(default)]
    pub require_heading: bool,
}

fn default_segments() -> usize {
    2
}

fn default_name_chars() -> usize {
    32
}

fn default_description_chars() -> usize {
    1024
}

impl Contract {
    #[cfg(test)]
    pub fn bundled() -> Self {
        Self::parse(include_str!("default-contract.toml"), "内置").expect("内置契约应能解析")
    }

    pub fn load(path: Option<&Path>) -> Result<Self, String> {
        let target = match path {
            Some(given) => {
                if !given.is_file() {
                    return Err(format!("--rules 指的文件不存在：{}", given.display()));
                }
                given.to_path_buf()
            }
            None => find_contract()?,
        };
        let text = std::fs::read_to_string(&target)
            .map_err(|error| format!("契约读不了 {}：{error}", target.display()))?;
        Self::parse(&text, &target.display().to_string())
    }

    fn parse(text: &str, origin: &str) -> Result<Self, String> {
        let contract: Contract =
            toml::from_str(text).map_err(|error| format!("契约 {origin} 格式不对：{error}"))?;
        if contract.name.segments == 0 {
            return Err(format!("契约 {origin} 的 segments 不能是 0"));
        }
        Ok(contract)
    }
}

/// 找契约：环境变量 → 当前目录往上找 → 用户级目录 → 都没有就报错。刻意不内置兜底。
fn find_contract() -> Result<PathBuf, String> {
    crate::core::contract::find(CONTRACT_ENV, &CONTRACT_RELATIVE, "agent", CONTRACT_FILE)
}
