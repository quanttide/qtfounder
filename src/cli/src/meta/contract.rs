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
    /// 资产：入口、长期记忆与归档，共 20 种。
    #[serde(default)]
    pub asset: Vec<Asset>,
    /// 资产之间的关系。
    #[serde(default)]
    pub relation: Vec<Relation>,
    #[serde(default)]
    pub home: Vec<Home>,
    /// 本体论根：这份契约属于哪个域。
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub category: Vec<Category>,
    #[serde(default)]
    pub transition: Vec<Transition>,
    #[serde(default)]
    pub question: Vec<Question>,
    #[serde(default)]
    pub model: Model,
    #[serde(default)]
    pub axiom: Vec<Axiom>,
    #[serde(default)]
    pub property: Vec<Property>,
    #[serde(default)]
    pub paradigm: Vec<Paradigm>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Verdict {
    #[serde(default)]
    pub min_margin: usize,
    #[serde(default)]
    pub min_hits: usize,
    #[serde(default)]
    pub min_home_hits: usize,
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
            min_home_hits: 1,
            max_paragraphs: 5,
            stay: "留在这里".to_string(),
            archive_cues: Vec::new(),
            question_fallback: "它说的是世界怎么运转，还是你下一步要做什么？".to_string(),
        }
    }
}

/// 资产：一个可维护的仓库。入口、长期记忆与归档共用这一形状。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Asset {
    pub name: String,
    #[serde(default)]
    pub label: String,
    /// `source`（context）、`memory`（十八类）、`exit`（archive）。
    #[serde(default)]
    pub role: String,
    pub path: String,
    #[serde(default)]
    pub category: String,
    /// `domain-asset` 或 `asset-domain`：前两段是领域在前还是资产在前。
    #[serde(default)]
    pub layout: String,
}

impl Asset {
    pub fn label(&self) -> &str {
        if self.label.is_empty() {
            &self.name
        } else {
            &self.label
        }
    }

    pub fn is_memory(&self) -> bool {
        self.role == "memory"
    }

    pub fn is_source(&self) -> bool {
        self.role == "source"
    }

    pub fn is_exit(&self) -> bool {
        self.role == "exit"
    }
}

/// 关系：资产之间的迁移边。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Relation {
    /// `intake` 或 `archive`。
    pub kind: String,
    pub from: String,
    pub to: String,
    /// `flip` 交换前两段，`same` 保持。
    #[serde(default)]
    pub transform: String,
}

impl Contract {
    pub fn source(&self) -> Option<&Asset> {
        self.asset.iter().find(|asset| asset.is_source())
    }

    pub fn exit(&self) -> Option<&Asset> {
        self.asset.iter().find(|asset| asset.is_exit())
    }

    pub fn memory(&self) -> Vec<&Asset> {
        self.asset
            .iter()
            .filter(|asset| asset.is_memory())
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Home {
    pub name: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub cues: Vec<String>,
    /// `confirmed` 或 `suspected`，缺省按怀疑算。
    #[serde(default)]
    pub status: String,
}

impl Home {
    pub fn label(&self) -> &str {
        if self.label.is_empty() {
            &self.name
        } else {
            &self.label
        }
    }

    pub fn status_key(&self) -> &'static str {
        status_key(&self.status)
    }

    pub fn status_label(&self) -> &'static str {
        status_label(&self.status)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Category {
    /// 机器可读的标识。
    pub name: String,
    /// 给人看的称呼。
    pub label: String,
    pub path: String,
    #[serde(default)]
    pub family: String,
    #[serde(default)]
    pub cues: Vec<String>,
    #[serde(default)]
    pub description: String,
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
    pub description: String,
    #[serde(default)]
    pub family: Vec<ModelFamily>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ModelFamily {
    pub name: String,
    #[serde(default)]
    pub question: String,
    #[serde(default)]
    pub description: String,
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
    /// 机器可读的标识；uuid 留给 id 字段。
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub why: String,
    /// `confirmed` 或 `suspected`，缺省按怀疑算。
    #[serde(default)]
    pub status: String,
}

impl Axiom {
    pub fn status_key(&self) -> &'static str {
        status_key(&self.status)
    }

    pub fn status_label(&self) -> &'static str {
        status_label(&self.status)
    }
}

/// 标准属性：字段名、所属组与定义。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Property {
    pub name: String,
    #[serde(default)]
    pub group: String,
    pub description: String,
    /// `confirmed` 或 `suspected`，缺省按怀疑算。
    #[serde(default)]
    pub status: String,
}

impl Property {
    pub fn status_key(&self) -> &'static str {
        status_key(&self.status)
    }

    pub fn status_label(&self) -> &'static str {
        status_label(&self.status)
    }
}

/// 范式：一个有名字的范畴描述。对象、态射、合成律三样，归位本身是其中之一。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Paradigm {
    pub name: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub description: String,
    /// `confirmed` 或 `suspected`，缺省按怀疑算。
    #[serde(default)]
    pub status: String,
    /// `categories` 表示对象取 `[[category]]`、态射取 `[[transition]]`；其余按 inline 读。
    #[serde(default)]
    pub source: String,
    /// `transitive`、`partial` 或 `none`，缺省 partial。
    #[serde(default)]
    pub compose: String,
    #[serde(default)]
    pub object: Vec<ParadigmObject>,
    #[serde(default)]
    pub morphism: Vec<ParadigmMorphism>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ParadigmObject {
    pub name: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ParadigmMorphism {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub reason: String,
}

/// 解析后的范式：对象与态射都已展开成同一形状，调用方不必再管 source。
#[derive(Debug, Clone, Default)]
pub struct ResolvedParadigm {
    pub name: String,
    pub label: String,
    pub description: String,
    pub status: String,
    pub source: String,
    pub compose: String,
    pub objects: Vec<ParadigmObject>,
    pub morphisms: Vec<ParadigmMorphism>,
}

impl ResolvedParadigm {
    pub fn label(&self) -> &str {
        if self.label.is_empty() {
            &self.name
        } else {
            &self.label
        }
    }

    pub fn status_key(&self) -> &'static str {
        status_key(&self.status)
    }

    pub fn status_label(&self) -> &'static str {
        status_label(&self.status)
    }

    pub fn compose_key(&self) -> &'static str {
        match self.compose.trim() {
            "transitive" => "transitive",
            "none" => "none",
            _ => "partial",
        }
    }

    pub fn compose_label(&self) -> &'static str {
        match self.compose_key() {
            "transitive" => "传递",
            "none" => "不要求",
            _ => "部分",
        }
    }

    pub fn object(&self, name: &str) -> Option<&ParadigmObject> {
        self.objects.iter().find(|object| object.name == name)
    }
}

/// 确认状态归一：只认 `confirmed`，其余都当怀疑。
pub fn status_key(raw: &str) -> &'static str {
    if raw.trim() == "confirmed" {
        "confirmed"
    } else {
        "suspected"
    }
}

pub fn status_label(raw: &str) -> &'static str {
    if status_key(raw) == "confirmed" {
        "确认"
    } else {
        "怀疑"
    }
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
        if contract.category.is_empty() {
            return Err(format!("契约 {origin} 没有一格：十八格至少得写进来"));
        }
        // 降格目标与分辨句里的格必须是真格，写错了会在判定后才暴露，宁可在读契约时拦下。
        let known: Vec<&str> = contract
            .category
            .iter()
            .map(|category| category.name.as_str())
            .collect();
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
            for category in &contract.category {
                if !families.contains(&category.family.as_str()) {
                    return Err(format!(
                        "契约 {origin} 的模型没有这一族：{}",
                        category.family
                    ));
                }
            }
            for family in &contract.model.family {
                if !contract
                    .category
                    .iter()
                    .any(|category| category.family == family.name)
                {
                    return Err(format!(
                        "契约 {origin} 的模型家族没有对应的格：{}",
                        family.name
                    ));
                }
            }
        }
        let mut axiom_names = HashSet::new();
        for axiom in &contract.axiom {
            if axiom.name.trim().is_empty() {
                return Err(format!("契约 {origin} 的公理缺 name"));
            }
            if !axiom_names.insert(axiom.name.as_str()) {
                return Err(format!("契约 {origin} 的公理 name 重复：{}", axiom.name));
            }
            if axiom.description.trim().is_empty() {
                return Err(format!(
                    "契约 {origin} 的公理没有 description：{}",
                    axiom.name
                ));
            }
            // 只有结论的公理是口号：why 记不住当初的失败，后来的人就无从判断它还算不算前提。
            if axiom.why.trim().is_empty() {
                return Err(format!("契约 {origin} 的公理没有 why：{}", axiom.name));
            }
            let status = axiom.status.trim();
            if !status.is_empty() && status != "confirmed" && status != "suspected" {
                return Err(format!(
                    "契约 {origin} 的公理 status 只能写 confirmed 或 suspected：{}",
                    axiom.name
                ));
            }
        }
        let mut property_names = HashSet::new();
        for property in &contract.property {
            if property.name.trim().is_empty() {
                return Err(format!("契约 {origin} 的属性缺 name"));
            }
            if !property_names.insert(property.name.as_str()) {
                return Err(format!("契约 {origin} 的属性 name 重复：{}", property.name));
            }
            if property.group.trim().is_empty() {
                return Err(format!("契约 {origin} 的属性没有 group：{}", property.name));
            }
            if property.description.trim().is_empty() {
                return Err(format!(
                    "契约 {origin} 的属性没有 description：{}",
                    property.name
                ));
            }
            let status = property.status.trim();
            if !status.is_empty() && status != "confirmed" && status != "suspected" {
                return Err(format!(
                    "契约 {origin} 的属性 status 只能写 confirmed 或 suspected：{}",
                    property.name
                ));
            }
        }
        let mut home_names = HashSet::new();
        for home in &contract.home {
            if home.name.trim().is_empty() {
                return Err(format!("契约 {origin} 的去处缺 name"));
            }
            if !home_names.insert(home.name.as_str()) {
                return Err(format!("契约 {origin} 的去处 name 重复：{}", home.name));
            }
            let status = home.status.trim();
            if !status.is_empty() && status != "confirmed" && status != "suspected" {
                return Err(format!(
                    "契约 {origin} 的去处 status 只能写 confirmed 或 suspected：{}",
                    home.name
                ));
            }
        }
        let mut paradigm_names = HashSet::new();
        for paradigm in &contract.paradigm {
            if paradigm.name.trim().is_empty() {
                return Err(format!("契约 {origin} 的范式缺 name"));
            }
            if !paradigm_names.insert(paradigm.name.as_str()) {
                return Err(format!("契约 {origin} 的范式 name 重复：{}", paradigm.name));
            }
            let status = paradigm.status.trim();
            if !status.is_empty() && status != "confirmed" && status != "suspected" {
                return Err(format!(
                    "契约 {origin} 的范式 status 只能写 confirmed 或 suspected：{}",
                    paradigm.name
                ));
            }
            let source = paradigm.source.trim();
            if !source.is_empty() && source != "categories" && source != "inline" {
                return Err(format!(
                    "契约 {origin} 的范式 source 只能写 categories 或 inline：{}",
                    paradigm.name
                ));
            }
            let compose = paradigm.compose.trim();
            if !compose.is_empty()
                && compose != "transitive"
                && compose != "partial"
                && compose != "none"
            {
                return Err(format!(
                    "契约 {origin} 的范式 compose 只能写 transitive、partial 或 none：{}",
                    paradigm.name
                ));
            }
            let resolved = contract.resolve(paradigm);
            if resolved.objects.is_empty() {
                return Err(format!("契约 {origin} 的范式没有对象：{}", paradigm.name));
            }
            let mut object_names = HashSet::new();
            for object in &resolved.objects {
                if object.name.trim().is_empty() {
                    return Err(format!(
                        "契约 {origin} 的范式 {} 有对象缺 name",
                        paradigm.name
                    ));
                }
                if !object_names.insert(object.name.as_str()) {
                    return Err(format!(
                        "契约 {origin} 的范式 {} 的对象 name 重复：{}",
                        paradigm.name, object.name
                    ));
                }
            }
            for morphism in &resolved.morphisms {
                for end in [&morphism.from, &morphism.to] {
                    if !object_names.contains(end.as_str()) {
                        return Err(format!(
                            "契约 {origin} 的范式 {} 的态射指向不存在的对象：{}",
                            paradigm.name, end
                        ));
                    }
                }
            }
        }
        if bundled && contract.home.is_empty() {
            return Err(format!(
                "契约 {origin} 没有去处：至少写清留在这里之外的去处"
            ));
        }
        Ok(contract)
    }

    pub fn category(&self, name: &str) -> Option<&Category> {
        self.category.iter().find(|category| category.name == name)
    }

    pub fn paradigm_of(&self, name: &str) -> Option<&Paradigm> {
        self.paradigm.iter().find(|paradigm| paradigm.name == name)
    }

    /// 把范式展开成对象与态射：`source = "categories"` 时取格与降格，否则取 inline。
    pub fn resolve(&self, paradigm: &Paradigm) -> ResolvedParadigm {
        let categories = paradigm.source.trim() == "categories";
        let objects = if categories {
            self.category
                .iter()
                .map(|category| ParadigmObject {
                    name: category.name.clone(),
                    label: category.label.clone(),
                    description: category.description.clone(),
                })
                .collect()
        } else {
            paradigm.object.clone()
        };
        let morphisms = if categories {
            self.transition
                .iter()
                .map(|transition| ParadigmMorphism {
                    from: transition.from.clone(),
                    to: transition.to.clone(),
                    reason: transition.reason.clone(),
                })
                .collect()
        } else {
            paradigm.morphism.clone()
        };
        ResolvedParadigm {
            name: paradigm.name.clone(),
            label: paradigm.label.clone(),
            description: paradigm.description.clone(),
            status: paradigm.status.clone(),
            source: if categories {
                "categories".to_string()
            } else {
                "inline".to_string()
            },
            compose: paradigm.compose.clone(),
            objects,
            morphisms,
        }
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
[[category]]
name = "insight"
label = "洞察"
path = "data/insight"
"#;

    fn parse(extra: &str) -> Result<Contract, String> {
        Contract::parse(&format!("{CELL}{extra}"), "测试", false)
    }

    #[test]
    fn 公理缺_why_在读契约时被拦下() {
        let error = parse(
            "\n[[axiom]]\nname = \"one-place\"\ndescription = \"一份内容只有一个正确去处。\"\n",
        )
        .unwrap_err();
        assert!(error.contains("没有 why"), "{error}");
    }

    #[test]
    fn 公理缺_description_在读契约时被拦下() {
        let error =
            parse("\n[[axiom]]\nname = \"one-place\"\ndescription = \"\"\nwhy = \"放错格的代价是没人找得到它。\"\n")
                .unwrap_err();
        assert!(error.contains("没有 description"), "{error}");
    }

    #[test]
    fn 公理状态只能是确认或怀疑() {
        let error = parse(
            "\n[[axiom]]\nname = \"x\"\ndescription = \"甲\"\nwhy = \"乙\"\nstatus = \"maybe\"\n",
        )
        .unwrap_err();
        assert!(error.contains("status"), "{error}");
    }

    #[test]
    fn 公理_name_重复在读契约时被拦下() {
        let error = parse(
            "\n[[axiom]]\nname = \"one-place\"\ndescription = \"甲\"\nwhy = \"乙\"\n\n[[axiom]]\nname = \"one-place\"\ndescription = \"丙\"\nwhy = \"丁\"\n",
        )
        .unwrap_err();
        assert!(error.contains("公理 name 重复"), "{error}");
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
