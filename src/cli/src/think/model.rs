//! 认知模型：`reflect` 用的词汇、取值域与栏位顺序都在这里，判断层只读它。
//!
//! 与 `code`／`language`／`docs` 的契约同构——一份模型 = 一套关于「怎么想」的语法。
//! 查找顺序：`--model` → `QTFOUNDER_THINK_MODEL` → 从当前目录逐级往上找
//! `src/think/default-contract.toml` → `~/.config/qtfounder/think/model.toml` → 内置。
//!
//! 内置兜底是刻意的：`reflect` 是记录本，不该因为找不到配置文件就记不了账。
//! 内置那份与仓库里的 `default-contract.toml` 同源，改模型改文件、重编译即可。
//! `code`／`language`／`docs` 的三份契约没有兜底，因为它们判的是别人的东西，
//! 口径缺失时宁可停手。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use clap::Args;
use serde::{Deserialize, Serialize};

pub const MODEL_FILE: &str = "model.toml";
pub const MODEL_ENV: &str = "QTFOUNDER_THINK_MODEL";
const BUNDLED: &str = include_str!("default-contract.toml");
const RELATIVE: [&str; 2] = [
    "src/think/default-contract.toml",
    "src/cli/src/think/default-contract.toml",
];

/// `think model`：把当前生效的模型摊开看。
#[derive(Args, Debug)]
pub struct ModelArgs {
    /// 指定模型文件；给定时必须存在
    #[arg(long)]
    pub model: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub domains: Vec<String>,
    pub controls: Vec<String>,
    pub order: Vec<String>,
    pub screen: Screen,
    labels: BTreeMap<String, String>,
    questions: BTreeMap<String, String>,
    distortions: Vec<Distortion>,
}

/// 宽筛规则。`load` 用它选候选，与认知模型同一份文件，改词表不用改代码。
#[derive(Debug, Clone)]
pub struct Screen {
    pub signal_words: Vec<String>,
    pub bare_ack: Vec<String>,
    pub min_chars: usize,
}

#[derive(Debug, Clone)]
struct Distortion {
    key: String,
    label: String,
}

#[derive(Debug, Deserialize, Default)]
struct Raw {
    #[serde(default)]
    screen: RawScreen,
    #[serde(default)]
    role: RawRole,
    #[serde(default)]
    fields: RawFields,
    #[serde(default)]
    labels: BTreeMap<String, String>,
    #[serde(default)]
    questions: BTreeMap<String, String>,
    #[serde(default)]
    distortions: Vec<RawDistortion>,
}

#[derive(Debug, Deserialize, Default)]
struct RawScreen {
    #[serde(default)]
    signal_words: Vec<String>,
    #[serde(default)]
    bare_ack: Vec<String>,
    #[serde(default)]
    min_chars: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct RawDistortion {
    key: String,
    label: String,
}

#[derive(Debug, Deserialize, Default)]
struct RawRole {
    #[serde(default)]
    domains: Vec<String>,
    #[serde(default)]
    controls: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct RawFields {
    #[serde(default)]
    order: Vec<String>,
}

impl Model {
    /// 读模型，返回模型与来源（路径或「内置」）。
    pub fn load(path: Option<&Path>) -> Result<(Self, String), String> {
        let Some(target) = find(path)? else {
            let model = Self::parse(BUNDLED, "内置")?;
            return Ok((model, "内置".to_string()));
        };
        let text = std::fs::read_to_string(&target)
            .map_err(|error| format!("模型读不了 {}：{error}", target.display()))?;
        let origin = target.display().to_string();
        let model = Self::parse(&text, &origin)?;
        Ok((model, origin))
    }

    fn parse(text: &str, origin: &str) -> Result<Self, String> {
        let raw: Raw =
            toml::from_str(text).map_err(|error| format!("模型 {origin} 格式不对：{error}"))?;
        if raw.role.domains.is_empty() || raw.role.controls.is_empty() {
            return Err(format!("模型 {origin} 缺 [role] 的 domains 或 controls"));
        }
        if raw.fields.order.is_empty() {
            return Err(format!("模型 {origin} 缺 [fields].order"));
        }
        if raw.distortions.is_empty() {
            return Err(format!("模型 {origin} 缺 [distortions]"));
        }
        if raw.screen.signal_words.is_empty() {
            return Err(format!("模型 {origin} 缺 [screen].signal_words"));
        }
        let min_chars = raw
            .screen
            .min_chars
            .ok_or_else(|| format!("模型 {origin} 缺 [screen].min_chars"))?;
        Ok(Model {
            domains: raw.role.domains,
            controls: raw.role.controls,
            order: raw.fields.order,
            screen: Screen {
                signal_words: raw.screen.signal_words,
                bare_ack: raw.screen.bare_ack,
                min_chars,
            },
            labels: raw.labels,
            questions: raw.questions,
            distortions: raw
                .distortions
                .into_iter()
                .map(|item| Distortion {
                    key: item.key,
                    label: item.label,
                })
                .collect(),
        })
    }

    /// 内置那份，测试用：不读环境变量、不看当前目录。
    #[cfg(test)]
    pub fn bundled() -> Self {
        Self::parse(BUNDLED, "内置").expect("内置模型应能解析")
    }

    /// 中文标签与英文键互为别名，落库统一用标签。
    pub fn distortion_label(&self, raw: &str) -> Option<&str> {
        let key = raw.trim();
        self.distortions
            .iter()
            .find(|item| key.eq_ignore_ascii_case(&item.key) || key == item.label)
            .map(|item| item.label.as_str())
    }

    pub fn distortion_labels(&self) -> Vec<&str> {
        self.distortions
            .iter()
            .map(|item| item.label.as_str())
            .collect()
    }

    pub fn label(&self, field: &str) -> Option<&str> {
        self.labels.get(field).map(String::as_str)
    }

    pub fn question(&self, field: &str) -> Option<&str> {
        self.questions.get(field).map(String::as_str)
    }

    pub fn complete_question(&self) -> &str {
        self.questions
            .get("complete")
            .map(String::as_str)
            .unwrap_or("到复看日回填结果：实际发生的事，比任何推理都算数。")
    }
}

/// 找模型文件；都找不到给 `None`，由调用方用内置那份。
fn find(path: Option<&Path>) -> Result<Option<PathBuf>, String> {
    if let Some(given) = path {
        if given.is_file() {
            return Ok(Some(given.to_path_buf()));
        }
        return Err(format!("--model 指的文件不存在：{}", given.display()));
    }
    if let Ok(from_env) = std::env::var(MODEL_ENV)
        && !from_env.is_empty()
    {
        let candidate = PathBuf::from(&from_env);
        if candidate.is_file() {
            return Ok(Some(candidate));
        }
        return Err(format!("{MODEL_ENV} 指向的模型不存在：{from_env}"));
    }
    let start = std::env::current_dir().map_err(|error| format!("读不到当前目录：{error}"))?;
    let mut dir = start.as_path();
    loop {
        for relative in RELATIVE {
            let candidate = dir.join(relative);
            if candidate.is_file() {
                return Ok(Some(candidate));
            }
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let user_level = Path::new(&home).join(format!(".config/qtfounder/think/{MODEL_FILE}"));
        if user_level.is_file() {
            return Ok(Some(user_level));
        }
    }
    Ok(None)
}

#[derive(Serialize)]
struct ModelView<'a> {
    source: &'a str,
    screen: ScreenView<'a>,
    role: RoleView<'a>,
    fields: &'a [String],
    labels: Vec<Entry<'a>>,
    questions: Vec<Entry<'a>>,
    distortions: Vec<Entry<'a>>,
}

#[derive(Serialize)]
struct ScreenView<'a> {
    signal_words: &'a [String],
    bare_ack: &'a [String],
    min_chars: usize,
}

#[derive(Serialize)]
struct RoleView<'a> {
    domains: &'a [String],
    controls: &'a [String],
}

#[derive(Serialize)]
struct Entry<'a> {
    key: &'a str,
    value: &'a str,
}

pub fn run(args: &ModelArgs) -> Result<i32, String> {
    let (model, origin) = Model::load(args.model.as_deref())?;
    if args.json {
        let view = ModelView {
            source: &origin,
            screen: ScreenView {
                signal_words: &model.screen.signal_words,
                bare_ack: &model.screen.bare_ack,
                min_chars: model.screen.min_chars,
            },
            role: RoleView {
                domains: &model.domains,
                controls: &model.controls,
            },
            fields: &model.order,
            labels: entries(&model.labels),
            questions: entries(&model.questions),
            distortions: model
                .distortions
                .iter()
                .map(|item| Entry {
                    key: &item.key,
                    value: &item.label,
                })
                .collect(),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&view)
                .map_err(|error| format!("JSON 序列化失败：{error}"))?
        );
        return Ok(0);
    }
    println!("认知模型：{origin}");
    println!(
        "宽筛：{} 个信号词，无信号时短于 {} 字不进候选",
        model.screen.signal_words.len(),
        model.screen.min_chars
    );
    println!("归属：{}", model.domains.join(" / "));
    println!("控制：{}", model.controls.join(" / "));
    println!("栏位顺序：{}", model.order.join(" → "));
    println!("认知扭曲：{}", model.distortion_labels().join("、"));
    if let Some(complete) = model.question("complete") {
        println!("收尾问：{complete}");
    }
    Ok(0)
}

fn entries(map: &BTreeMap<String, String>) -> Vec<Entry<'_>> {
    map.iter()
        .map(|(key, value)| Entry { key, value })
        .collect()
}
