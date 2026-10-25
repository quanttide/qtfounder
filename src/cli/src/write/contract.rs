//! 契约：一次读进内存，判断层只认它。
//!
//! 与 `language`、`docs` 的契约同构——一份契约 = 一类介质的验收标准。查找顺序也一样：
//! `--rules` → 环境变量 → 从当前目录往上找 `src/write/default-contract.toml` → 用户级目录。

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
    /// 每千字命中上限
    #[serde(default)]
    pub max_per_1000: usize,
    /// 文档低于这个字数不判
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

/// 结构：标题层级与长文分节。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Structure {
    #[serde(default)]
    pub require_heading: bool,
    /// 一篇允许的一级标题数
    #[serde(default)]
    pub max_h1: usize,
    /// 标题一次最多降几级
    #[serde(default)]
    pub max_heading_jump: usize,
    /// 超过这个字数还没有二级标题，报「一段到底」
    #[serde(default)]
    pub min_chars_per_section: usize,
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
        let user_level = Path::new(&home).join(format!(".config/qtfounder/write/{CONTRACT_FILE}"));
        if user_level.is_file() {
            return Ok(user_level);
        }
    }
    Err(format!(
        "找不到默认契约 {CONTRACT_FILE}：用 --rules 指定，或设 {CONTRACT_ENV}，或放在 src/write/{CONTRACT_FILE}，或装到 ~/.config/qtfounder/write/{CONTRACT_FILE}"
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
}
