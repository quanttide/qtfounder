//! 契约：一次读进内存，判断层只认它。
//!
//! 与 `language`、`docs` 的契约同构——一份契约 = 一类介质的验收标准。查找顺序也一样：
//! `--rules` → 环境变量 → 从当前目录往上找 `src/write/default-contract.toml` → 用户级目录。
//!
//! 表达审计先看三件事：体裁决定骨架与禁区，读者决定要交代多少背景，
//! 主题决定哪些词必须先定义。三层都可以在 front matter 里声明，也可以用命令行覆盖。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::Deserialize;

pub const CONTRACT_FILE: &str = "default-contract.toml";
pub const CONTRACT_ENV: &str = "QTFOUNDER_WRITE_CONTRACT";
const CONTRACT_RELATIVE: [&str; 3] = [
    "src/write/default-contract.toml",
    "src/cli/src/write/default-contract.toml",
    "apps/qtfounder/src/cli/src/write/default-contract.toml",
];

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Contract {
    #[serde(default)]
    pub scope: Scope,
    #[serde(default)]
    pub empty: Empty,
    #[serde(default)]
    pub slogan: Slogan,
    #[serde(default)]
    pub reference: Reference,
    #[serde(default)]
    pub structure: Structure,
    #[serde(default)]
    pub genre: BTreeMap<String, Genre>,
    #[serde(default)]
    pub audience: BTreeMap<String, Audience>,
    #[serde(default)]
    pub topic: BTreeMap<String, Topic>,
    /// 读者评审的交付纪律；缺省为空，`write review` 会提示契约里没有。
    #[serde(default)]
    pub review: Vec<ReviewRule>,
}

/// 一条读者评审纪律：从读者角度评的形状与禁忌。
#[derive(Debug, Clone, Deserialize)]
pub struct ReviewRule {
    pub name: String,
    pub description: String,
    pub why: String,
    #[serde(default)]
    pub example: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Scope {
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// 空话：删掉不影响意思，或需要说清具体指什么。按整篇密度判，不逐句报。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Empty {
    #[serde(default)]
    pub words: Vec<String>,
    #[serde(default)]
    pub max_per_1000: usize,
    #[serde(default)]
    pub min_chars: usize,
    #[serde(default)]
    pub advice: String,
}

/// 口号：先否定再肯定、只给条件不给反例、用递进代替论证。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Slogan {
    #[serde(default)]
    pub patterns: Vec<Pattern>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Pattern {
    pub pattern: String,
    pub advice: String,
}

/// 指路：正文里让读者去看另一份文档。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Reference {
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub advice: String,
}

/// 结构：标题层级与长文分节。所有体裁共用。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Structure {
    #[serde(default)]
    pub require_heading: bool,
    #[serde(default)]
    pub max_h1: usize,
    #[serde(default)]
    pub max_heading_jump: usize,
    #[serde(default)]
    pub min_chars_per_section: usize,
}

/// 体裁：这类文本要有的节，以及不该出现的内容。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Genre {
    #[serde(default)]
    pub description: String,
    /// 路径里出现这些词就推断成这个体裁
    #[serde(default)]
    pub path_hints: Vec<String>,
    #[serde(default)]
    pub required: Vec<Requirement>,
    /// 这个体裁不该出现的词
    #[serde(default)]
    pub forbid: Vec<String>,
    #[serde(default)]
    pub min_h2: usize,
    #[serde(default)]
    pub max_h2: usize,
}

/// 一节：标题里出现 `any_of` 任一词就算有。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Requirement {
    pub name: String,
    #[serde(default)]
    pub any_of: Vec<String>,
}

/// 读者：决定内部标识能密到什么程度，以及要不要对读者说话。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Audience {
    #[serde(default)]
    pub description: String,
    /// 一段里不同的反引号标识超过几个就报
    #[serde(default)]
    pub max_internal_ids: usize,
    /// 对读者说话的词；硬性读者要有至少一个
    #[serde(default)]
    pub reader_words: Vec<String>,
    #[serde(default)]
    pub require_reader_words: bool,
}

/// 主题：这篇讲什么，哪些词必须先定义。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Topic {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub terms: Vec<String>,
    #[serde(default)]
    pub require_definition: bool,
}

impl Contract {
    pub fn load(path: Option<&Path>) -> Result<Self, String> {
        let target = match path {
            Some(given) => given.to_path_buf(),
            None => find_contract()?,
        };
        let text = std::fs::read_to_string(&target)
            .map_err(|error| format!("契约读不了 {}：{error}", target.display()))?;
        let contract: Self = toml::from_str(&text)
            .map_err(|error| format!("契约 {} 格式不对：{error}", target.display()))?;
        contract.validate()?;
        Ok(contract)
    }

    /// 正则写不对时在读契约阶段就报错，避免规则静默失效。
    fn validate(&self) -> Result<(), String> {
        for item in &self.slogan.patterns {
            Regex::new(&item.pattern).map_err(|error| {
                format!("契约里的 slogan 正则写不对：{}（{error}）", item.pattern)
            })?;
        }
        for pattern in &self.reference.patterns {
            Regex::new(pattern)
                .map_err(|error| format!("契约里的 reference 正则写不对：{pattern}（{error}）"))?;
        }
        Ok(())
    }

    /// 没声明体裁时按文件名推断。`default` 不参与推断。
    pub fn genre_for_path(&self, path: &str) -> Option<String> {
        let file = path.rsplit('/').next().unwrap_or(path).to_lowercase();
        self.genre
            .iter()
            .filter(|(name, _)| name.as_str() != "default")
            .find(|(_, profile)| {
                profile
                    .path_hints
                    .iter()
                    .any(|hint| file.contains(&hint.to_lowercase()))
            })
            .map(|(name, _)| name.clone())
    }
}

/// 找契约：环境变量 → 当前目录往上找 → 用户级目录 → 都没有就报错。刻意不内置兜底。
pub fn find_contract() -> Result<PathBuf, String> {
    crate::core::contract::find(CONTRACT_ENV, &CONTRACT_RELATIVE, "write", CONTRACT_FILE)
}
