//! 资产契约：资产图式的事实源——二十种资产、迁入边、每类资产的写法与资产领域公理。
//!
//! 数据源是本目录的 `default-contract.toml`。`meta` 子命令淘汰后，这张表从归位契约
//! 抽出来独立成活：资产图式不随归位判断一起死。契约查找走共用三层——`--rules` →
//! 环境变量 → 从当前目录向上找 `src/asset/default-contract.toml` → 用户级目录，
//! 没有内置兜底，缺了就停手报错。

use std::path::{Path, PathBuf};

use clap::Args;
use serde::Deserialize;
use serde_json::json;

use crate::core::contract::find;

pub const CONTRACT_FILE: &str = "default-contract.toml";
pub const CONTRACT_ENV: &str = "QTFOUNDER_ASSET_CONTRACT";
const CONTRACT_RELATIVE: [&str; 3] = [
    "src/asset/default-contract.toml",
    "src/cli/src/asset/default-contract.toml",
    "apps/qtfounder/src/cli/src/asset/default-contract.toml",
];

// ───────────────────────── 契约类型 ─────────────────────────

/// 资产契约：`[[asset]]`、`[[relation]]`、`[[category]]` 与资产领域公理四段。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Contract {
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub asset: Vec<Asset>,
    #[serde(default)]
    pub relation: Vec<Relation>,
    #[serde(default)]
    pub category: Vec<Category>,
    #[serde(default)]
    pub axiom: Vec<Axiom>,
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
    /// 这一位置收什么。两端写在这里；十八类的内容写在 `[[category]]`。
    #[serde(default)]
    pub description: String,
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

/// 一类资产：边界、分档与写法。
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

/// 资产领域公理：管这个领域具体怎么运转的前提。
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
    /// `asset`；保留字段以兼容契约写法。
    #[serde(default)]
    pub domain: String,
}

impl Axiom {
    pub fn status_key(&self) -> &'static str {
        status_key(&self.status)
    }

    pub fn status_label(&self) -> &'static str {
        status_label(&self.status)
    }
}

/// `confirmed` 或缺省/未知 → `confirmed` 才算确认，其余按怀疑。
pub fn status_key(status: &str) -> &'static str {
    if status.trim() == "confirmed" {
        "confirmed"
    } else {
        "suspected"
    }
}

pub fn status_label(status: &str) -> &'static str {
    match status_key(status) {
        "confirmed" => "确认",
        _ => "怀疑",
    }
}

impl Contract {
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

    /// 解析并拦错：domain 必须是 asset（或空），类别非空。
    pub fn parse(text: &str, origin: &str) -> Result<Self, String> {
        let contract: Contract =
            toml::from_str(text).map_err(|error| format!("契约 {origin} 格式不对：{error}"))?;
        if !contract.domain.is_empty() && contract.domain.trim() != "asset" {
            return Err(format!(
                "契约 {origin} 的 domain 是「{}」，资产契约只认 asset",
                contract.domain
            ));
        }
        if contract.category.is_empty() {
            return Err(format!("契约 {origin} 没有一类资产：至少得写进来"));
        }
        for axiom in &contract.axiom {
            if !axiom.domain.is_empty() && axiom.domain.trim() != "asset" {
                return Err(format!(
                    "契约 {origin} 的公理 {} 打的是「{}」，资产契约只收 asset",
                    axiom.name, axiom.domain
                ));
            }
        }
        Ok(contract)
    }

    /// 测试与示例用的内置契约：编译期读本目录的表。
    #[cfg(test)]
    pub fn bundled() -> Self {
        Self::parse(include_str!("default-contract.toml"), "内置").expect("内置契约应能解析")
    }

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

    pub fn category(&self, name: &str) -> Option<&Category> {
        self.category.iter().find(|category| category.name == name)
    }
}

fn find_contract() -> Result<PathBuf, String> {
    find(CONTRACT_ENV, &CONTRACT_RELATIVE, "asset", CONTRACT_FILE)
}

// ───────────────────────── 命令：asset contract ─────────────────────────

#[derive(Args, Debug)]
pub struct ContractArgs {
    /// 只看一类资产
    pub name: Option<String>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &ContractArgs, rules: Option<&Path>) -> Result<i32, String> {
    let contract = Contract::load(rules)?;
    let categories: Vec<&Category> = match &args.name {
        Some(name) => vec![category_of(&contract, name)?],
        None => contract.category.iter().collect(),
    };
    if args.json {
        print_json(&categories);
    } else {
        print_text(&categories);
    }
    Ok(0)
}

fn category_of<'a>(contract: &'a Contract, name: &str) -> Result<&'a Category, String> {
    contract.category(name).ok_or_else(|| {
        let names: Vec<&str> = contract
            .category
            .iter()
            .map(|category| category.name.as_str())
            .collect();
        format!("没有这类资产：{name}；可选：{}", names.join("、"))
    })
}

fn grade_line(category: &Category) -> String {
    if category.grade.is_empty() {
        return String::new();
    }
    category
        .grade
        .iter()
        .map(|grade| {
            let cues = if grade.cues.is_empty() {
                "默认".to_string()
            } else {
                grade.cues.join("、")
            };
            format!("{}（{cues}）", grade.label)
        })
        .collect::<Vec<String>>()
        .join("；")
}

fn print_text(categories: &[&Category]) {
    for (index, category) in categories.iter().enumerate() {
        if index > 0 {
            println!();
        }
        println!(
            "{}  {}  {}  {}",
            category.name, category.label, category.path, category.family
        );
        if !category.description.is_empty() {
            println!("  边界：{}", category.description);
        }
        let grades = grade_line(category);
        if !grades.is_empty() {
            println!("  分档：{grades}");
        }
        if !category.skeleton.trim().is_empty() {
            println!("  写法：");
            for line in category.skeleton.lines() {
                if line.is_empty() {
                    println!();
                } else {
                    println!("    {line}");
                }
            }
        }
    }
}

fn print_json(categories: &[&Category]) {
    let items: Vec<serde_json::Value> = categories
        .iter()
        .map(|category| {
            let grades: Vec<serde_json::Value> = category
                .grade
                .iter()
                .map(|grade| {
                    json!({
                        "label": grade.label,
                        "cues": grade.cues,
                        "fallback": grade.fallback,
                    })
                })
                .collect();
            json!({
                "name": category.name,
                "label": category.label,
                "path": category.path,
                "family": category.family,
                "description": category.description,
                "skeleton": category.skeleton,
                "grades": grades,
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_string())
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 每类资产都有契约() {
        let contract = Contract::bundled();
        for category in &contract.category {
            assert!(!category.name.is_empty());
            assert!(!category.path.is_empty());
            assert!(!category.description.is_empty(), "{} 缺边界", category.name);
            assert!(
                !category.skeleton.trim().is_empty(),
                "{} 缺写法",
                category.name
            );
        }
    }

    #[test]
    fn 未知资产会报可选项() {
        let contract = Contract::bundled();
        let error = category_of(&contract, "nope").unwrap_err();
        assert!(error.contains("report"));
    }

    #[test]
    fn 分档写明默认项() {
        let contract = Contract::bundled();
        let insight = contract.category("insight").unwrap();
        let line = grade_line(insight);
        assert!(line.contains("假说"));
        assert!(line.contains("默认"));
    }

    /// 资产表本体：二十种资产、三类关系、十八类资产格、五条资产领域公理都在。
    #[test]
    fn 资产图式四段齐全() {
        let contract = Contract::bundled();
        assert_eq!(contract.domain.trim(), "asset");
        assert_eq!(contract.asset.len(), 20);
        assert_eq!(contract.relation.len(), 3);
        assert_eq!(contract.category.len(), 18);
        assert_eq!(contract.axiom.len(), 5);
        assert!(contract.source().is_some());
        assert!(contract.exit().is_some());
    }
}
