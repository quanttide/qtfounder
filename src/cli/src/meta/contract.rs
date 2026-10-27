//! 归位契约：十八格、十个领域、降格规则与分辨句，一次读进内存，判定层只认它。
//!
//! 与 `docs`／`language` 的契约同构——一份契约 = 一类判定规则。查找顺序也一样：
//! `--rules` → 环境变量 → 从当前目录往上找 `src/meta/default-contract.toml` → 用户级目录。
//!
//! 刻意不内置兜底：这份契约是十八格的目录本身，缺了就无从判断归属，
//! 与其猜一个去处，不如停手报错。

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub const CONTRACT_FILE: &str = "default-contract.toml";
pub const CONTRACT_ENV: &str = "QTFOUNDER_META_CONTRACT";
const CONTRACT_RELATIVE: [&str; 3] = [
    "src/meta/default-contract.toml",
    "src/cli/src/meta/default-contract.toml",
    "apps/qtfounder/src/cli/src/meta/default-contract.toml",
];

#[derive(Debug, Clone, Deserialize)]
pub struct Contract {
    #[serde(default)]
    pub verdict: Verdict,
    #[serde(default)]
    pub domain: Vec<Domain>,
    #[serde(default)]
    pub cell: Vec<Cell>,
    #[serde(default)]
    pub transition: Vec<Transition>,
    #[serde(default)]
    pub question: Vec<Question>,
    #[serde(default)]
    pub model: Model,
    #[serde(default)]
    pub axiom: Vec<Axiom>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Verdict {
    #[serde(default)]
    pub min_margin: usize,
    #[serde(default)]
    pub min_hits: usize,
    #[serde(default)]
    pub min_domain_hits: usize,
    #[serde(default)]
    pub max_paragraphs: usize,
    #[serde(default)]
    pub stay: String,
    #[serde(default)]
    pub archive_cues: Vec<String>,
    #[serde(default)]
    pub question_fallback: String,
}

impl Default for Verdict {
    fn default() -> Self {
        Self {
            min_margin: 1,
            min_hits: 1,
            min_domain_hits: 1,
            max_paragraphs: 5,
            stay: "留在这里".to_string(),
            archive_cues: Vec::new(),
            question_fallback: "它说的是世界怎么运转，还是你下一步要做什么？".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Domain {
    pub name: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub cues: Vec<String>,
}

impl Domain {
    pub fn label(&self) -> &str {
        if self.label.is_empty() {
            &self.name
        } else {
            &self.label
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Cell {
    pub key: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub family: String,
    #[serde(default)]
    pub cues: Vec<String>,
    #[serde(default)]
    pub boundary: String,
    #[serde(default)]
    pub skeleton: String,
    #[serde(default)]
    pub grade: Vec<Grade>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Grade {
    pub label: String,
    #[serde(default)]
    pub cues: Vec<String>,
    #[serde(default)]
    pub fallback: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Transition {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub require_any: Vec<String>,
    #[serde(default)]
    pub lack_any: Vec<String>,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Question {
    #[serde(default)]
    pub pair: Vec<String>,
    #[serde(default)]
    pub ask: String,
}

/// 本体模型：两族九宫格的轴。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Model {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub family: Vec<ModelFamily>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ModelFamily {
    pub name: String,
    #[serde(default)]
    pub question: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub axis: Vec<Axis>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Axis {
    pub name: String,
    #[serde(default)]
    pub values: Vec<String>,
}

/// 公理：归位时不重新论证的前提。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Axiom {
    pub id: String,
    pub statement: String,
    #[serde(default)]
    pub why: String,
}

impl Contract {
    #[cfg(test)]
    pub fn bundled() -> Self {
        Self::parse(include_str!("default-contract.toml"), "内置", true).expect("内置契约应能解析")
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
        Self::parse(&text, &target.display().to_string(), false)
    }

    fn parse(text: &str, origin: &str, bundled: bool) -> Result<Self, String> {
        let contract: Contract =
            toml::from_str(text).map_err(|error| format!("契约 {origin} 格式不对：{error}"))?;
        if contract.cell.is_empty() {
            return Err(format!("契约 {origin} 没有一格：十八格至少得写进来"));
        }
        // 降格目标与分辨句里的格必须是真格，写错了会在判定后才暴露，宁可在读契约时拦下。
        let known: Vec<&str> = contract.cell.iter().map(|cell| cell.key.as_str()).collect();
        for transition in &contract.transition {
            for key in [&transition.from, &transition.to] {
                if !known.contains(&key.as_str()) {
                    return Err(format!("契约 {origin} 的降格规则指向不存在的格：{key}"));
                }
            }
        }
        for question in &contract.question {
            if question.pair.len() != 2 {
                return Err(format!("契约 {origin} 的分辨句要成对给两格"));
            }
            for key in &question.pair {
                if !known.contains(&key.as_str()) {
                    return Err(format!("契约 {origin} 的分辨句指向不存在的格：{key}"));
                }
            }
        }
        if !contract.model.family.is_empty() {
            let families: Vec<&str> = contract
                .model
                .family
                .iter()
                .map(|family| family.name.as_str())
                .collect();
            for cell in &contract.cell {
                if !families.contains(&cell.family.as_str()) {
                    return Err(format!("契约 {origin} 的模型没有这一族：{}", cell.family));
                }
            }
            for family in &contract.model.family {
                if !contract.cell.iter().any(|cell| cell.family == family.name) {
                    return Err(format!(
                        "契约 {origin} 的模型家族没有对应的格：{}",
                        family.name
                    ));
                }
            }
        }
        let mut axiom_ids = HashSet::new();
        for axiom in &contract.axiom {
            if axiom.id.trim().is_empty() {
                return Err(format!("契约 {origin} 的公理缺 id"));
            }
            if !axiom_ids.insert(axiom.id.as_str()) {
                return Err(format!("契约 {origin} 的公理 id 重复：{}", axiom.id));
            }
            if axiom.statement.trim().is_empty() {
                return Err(format!("契约 {origin} 的公理没有 statement：{}", axiom.id));
            }
            // 只有结论的公理是口号：why 记不住当初的失败，后来的人就无从判断它还算不算前提。
            if axiom.why.trim().is_empty() {
                return Err(format!("契约 {origin} 的公理没有 why：{}", axiom.id));
            }
        }
        if bundled && contract.domain.is_empty() {
            return Err(format!(
                "契约 {origin} 没有领域：至少写清留在这里之外的去处"
            ));
        }
        Ok(contract)
    }

    pub fn cell(&self, key: &str) -> Option<&Cell> {
        self.cell.iter().find(|cell| cell.key == key)
    }

    pub fn question(&self, first: &str, second: &str) -> &str {
        self.question
            .iter()
            .find(|question| {
                question.pair.len() == 2
                    && ((question.pair[0] == first && question.pair[1] == second)
                        || (question.pair[0] == second && question.pair[1] == first))
            })
            .map(|question| question.ask.as_str())
            .filter(|ask| !ask.is_empty())
            .unwrap_or(self.verdict.question_fallback.as_str())
    }
}

/// 找契约：环境变量 → 当前目录往上找 → 用户级目录 → 都没有就报错。刻意不内置兜底。
#[cfg(test)]
mod tests {
    use super::Contract;

    const CELL: &str = r#"
[[cell]]
key = "insight"
name = "洞察"
path = "data/insight"
"#;

    fn parse(extra: &str) -> Result<Contract, String> {
        Contract::parse(&format!("{CELL}{extra}"), "测试", false)
    }

    #[test]
    fn 公理缺_why_在读契约时被拦下() {
        let error =
            parse("\n[[axiom]]\nid = \"one-place\"\nstatement = \"一份内容只有一个正确去处。\"\n")
                .unwrap_err();
        assert!(error.contains("没有 why"), "{error}");
    }

    #[test]
    fn 公理缺_statement_在读契约时被拦下() {
        let error =
            parse("\n[[axiom]]\nid = \"one-place\"\nstatement = \"\"\nwhy = \"放错格的代价是没人找得到它。\"\n")
                .unwrap_err();
        assert!(error.contains("没有 statement"), "{error}");
    }

    #[test]
    fn 公理_id_重复在读契约时被拦下() {
        let error = parse(
            "\n[[axiom]]\nid = \"one-place\"\nstatement = \"甲\"\nwhy = \"乙\"\n\n[[axiom]]\nid = \"one-place\"\nstatement = \"丙\"\nwhy = \"丁\"\n",
        )
        .unwrap_err();
        assert!(error.contains("公理 id 重复"), "{error}");
    }
}

fn find_contract() -> Result<PathBuf, String> {
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
        let user_level = Path::new(&home).join(format!(".config/qtfounder/meta/{CONTRACT_FILE}"));
        if user_level.is_file() {
            return Ok(user_level);
        }
    }
    Err(format!(
        "找不到默认契约 {CONTRACT_FILE}：用 --rules 指定，或设 {CONTRACT_ENV}，或放在 src/meta/{CONTRACT_FILE}，或装到 ~/.config/qtfounder/meta/{CONTRACT_FILE}"
    ))
}
