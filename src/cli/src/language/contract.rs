//! 契约：一次读进内存，判断层只认它。
//!
//! 与 `code` 的契约同构——一份契约 = 一类介质的验收标准。查找顺序也一样：
//! `--rules` → 环境变量 → 从当前目录往上找 `src/language/default-contract.toml`。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub const CONTRACT_FILE: &str = "default-contract.toml";
pub const CONTRACT_ENV: &str = "QTFOUNDER_LANGUAGE_CONTRACT";
const CONTRACT_RELATIVE: [&str; 3] = [
    "src/language/default-contract.toml",
    "src/cli/src/language/default-contract.toml",
    "apps/qtfounder/src/cli/src/language/default-contract.toml",
];

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Contract {
    #[serde(default)]
    pub scope: Scope,
    #[serde(default)]
    pub banned: HashMap<String, BannedEntry>,
    #[serde(default)]
    pub required: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub placement: Placement,
    #[serde(default)]
    pub numbers: Numbers,
    #[serde(default)]
    pub syntax: Syntax,
    #[serde(default)]
    pub voice: Voice,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Scope {
    #[serde(default)]
    pub include_default: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BannedEntry {
    #[serde(default)]
    pub advice: String,
    /// 出处原话：给人读的，判定不读它，但出库/复核时要照着它查
    #[serde(default)]
    #[allow(dead_code)]
    pub evidence: String,
    /// 这个词属于哪些语境。文件在 front matter 里声明了其中之一，就不报这个词。
    #[serde(default)]
    pub contexts: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Placement {
    #[serde(default)]
    pub banned_paths: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Numbers {
    #[serde(default)]
    pub denominator_patterns: Vec<String>,
    #[serde(default)]
    pub source_words: Vec<String>,
    #[serde(default)]
    /// 要追分母的单位：写着给人看，判定按正则走，暂不读这张表
    #[allow(dead_code)]
    pub units: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Syntax {
    #[serde(default)]
    pub max_sentence_chars: usize,
    #[serde(default)]
    pub max_sentence_error_chars: usize,
    #[serde(default)]
    pub max_de_chain: usize,
    #[serde(default)]
    pub max_commas: usize,
    #[serde(default)]
    pub max_nominal: usize,
    #[serde(default)]
    pub max_paragraph_chars: usize,
    /// 「由你定」这类把结论推回给读者的收尾。值里写正则。
    #[serde(default)]
    pub push_to_reader_patterns: Vec<String>,
    /// 「三件事：A、B、C」这类先报件数再罗列。值里写正则。
    #[serde(default)]
    pub count_then_list_patterns: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Voice {
    #[serde(default)]
    pub max_internal_ids: usize,
    /// 一段里反引号标识符达到这个数，就认定是列举段（参数表、字段表），不报「没对读者说话」
    #[serde(default)]
    pub min_listed_ids: usize,
    #[serde(default)]
    pub reader_words: Vec<String>,
}

/// 找契约：环境变量 → 当前目录往上找 → 都没有就报错。刻意不内置兜底。
pub fn find_contract() -> Result<PathBuf, String> {
    crate::core::contract::find(CONTRACT_ENV, &CONTRACT_RELATIVE, "language", CONTRACT_FILE)
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
