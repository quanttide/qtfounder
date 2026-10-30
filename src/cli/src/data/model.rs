//! 标注的三份数据形状：规则、任务包、结果，以及规则校验与结果统计。
//!
//! 模型与存储原语分家：这里只管「一份数据长什么样、算不算合法」，
//! 不碰路径（路径在 `super`）也不做 IO（IO 在 `super::read_json_file`）。


use serde::{Deserialize, Serialize};


/// 一个标签：机器名 + 显示名 + 定义与正反例（设计规则时用来把话说清楚）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleLabel {
    /// 机器可读标识，如 `fact`；结果里按它计数。
    pub name: String,
    /// 界面显示名，如「事实」。
    pub label: String,
    /// 一句话定义：什么算这个标签。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub define: Option<String>,
    /// 正例一句。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yes: Option<String>,
    /// 反例一句。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no: Option<String>,
}

/// 标注对象：哪来的数据、怎么切段。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleSource {
    /// 数据源种类，当前只有 `journal`。
    pub kind: String,
    /// 切段方式，当前只有 `blank-line`（空行与 `---`）。
    pub split: String,
}

/// 一条标注规则。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub source: RuleSource,
    pub labels: Vec<RuleLabel>,
    /// 每批抽多少单元。
    pub batch: usize,
    pub at: String,
}

/// 任务包里的一个待标单元。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Unit {
    /// 单元 id，如 `data/journal/write/2026-10/2026-10-13.md#2`。
    pub id: String,
    /// journal 下的域目录名。
    pub domain: String,
    /// 正文。
    pub text: String,
}

/// 任务包：规则快照 + 待标单元，标注端（lite）只认它。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub rule_id: String,
    /// 规则快照：任务不随规则后续修改而变。
    pub rule: Rule,
    pub units: Vec<Unit>,
    pub at: String,
}

/// 一条标注结果（`results/<task>.jsonl` 的一行）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultLine {
    pub task: String,
    pub unit: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
}

/// 规则校验：设计工具与 CLI 共用同一把尺子。
pub fn validate_rule(rule: &Rule) -> Result<(), crate::product::AppError> {
    if rule.name.trim().is_empty() {
        return Err(crate::product::AppError::usage("规则名不能为空"));
    }
    if rule.batch == 0 {
        return Err(crate::product::AppError::usage("batch 至少为 1"));
    }
    if rule.labels.is_empty() {
        return Err(crate::product::AppError::usage("至少要有一个标签"));
    }
    for label in &rule.labels {
        if label.name.trim().is_empty() || label.label.trim().is_empty() {
            return Err(crate::product::AppError::usage(
                "标签的 name（机器名）与 label（显示名）都不能为空",
            ));
        }
    }
    let mut seen = std::collections::HashSet::new();
    for label in &rule.labels {
        if !seen.insert(label.name.as_str()) {
            return Err(crate::product::AppError::usage(format!(
                "标签机器名重复：{}",
                label.name
            )));
        }
    }
    Ok(())
}

/// 一个任务的统计：总数与各标签条数。
pub struct TaskTally {
    pub task: String,
    pub total: usize,
    pub labels: Vec<(String, usize)>,
}

/// 把结果行按任务聚合成统计，任务内标签按条数降序、同数按名升序。
pub fn tally(results: &[ResultLine]) -> Vec<TaskTally> {
    let mut by_task: std::collections::BTreeMap<String, std::collections::BTreeMap<String, usize>> =
        std::collections::BTreeMap::new();
    for line in results {
        *by_task
            .entry(line.task.clone())
            .or_default()
            .entry(line.label.clone())
            .or_default() += 1;
    }
    by_task
        .into_iter()
        .map(|(task, labels)| {
            let total = labels.values().sum();
            let mut labels: Vec<(String, usize)> = labels.into_iter().collect();
            labels.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            TaskTally {
                task,
                total,
                labels,
            }
        })
        .collect()
}

