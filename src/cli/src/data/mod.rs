//! `qtfounder data`：标注规则、任务包与标注结果的存取。
//!
//! 存储在 XDG 数据目录（`<data>/data/` 下 `rules/`、`tasks/`、`results/`
//! 三个子目录），与 studio 的规则设计工具、lite 的标注导出共用同一份数据。
//! 与 `product`／`execute` 不同：共享存储必须位置稳定，本命令组不区分
//! 开发与发布构建，一律走 XDG；`QTFOUNDER_DATA` 仍优先，供测试与临时切换。
//!
//! 错误信封同 `product`：业务错误一律 JSON 写 stderr，用法错退 2，未找到退 3。

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

use crate::product::store;

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

/// 存储根：`QTFOUNDER_DATA` → `$XDG_DATA_HOME/qtfounder` → `~/.local/share/qtfounder`。
pub fn data_home() -> PathBuf {
    data_home_from(
        std::env::var_os("QTFOUNDER_DATA").map(PathBuf::from),
        std::env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}

/// 纯函数形式的根解析，三个入参都显式传入，便于把分支都钉住。
pub fn data_home_from(
    override_dir: Option<PathBuf>,
    xdg_data_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> PathBuf {
    if let Some(overridden) = override_dir.filter(|path| !path.as_os_str().is_empty()) {
        return overridden;
    }
    if let Some(xdg) = xdg_data_home.filter(|path| !path.as_os_str().is_empty()) {
        return xdg.join(store::APP_DIR);
    }
    match home.filter(|path| !path.as_os_str().is_empty()) {
        Some(home) => home.join(".local").join("share").join(store::APP_DIR),
        None => PathBuf::from(store::APP_DIR),
    }
}

fn rules_dir() -> PathBuf {
    data_home().join("data").join("rules")
}

fn tasks_dir() -> PathBuf {
    data_home().join("data").join("tasks")
}

fn results_dir() -> PathBuf {
    data_home().join("data").join("results")
}

fn read_json_file<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> Result<T, crate::product::AppError> {
    let text = std::fs::read_to_string(path).map_err(|err| {
        crate::product::AppError::usage(format!("无法读取 {}：{err}", path.display()))
    })?;
    serde_json::from_str(&text).map_err(|err| {
        crate::product::AppError::usage(format!("{} 解析失败：{err}", path.display()))
    })
}

fn write_json_file(path: &Path, value: &impl Serialize) -> Result<(), crate::product::AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            crate::product::AppError::usage(format!("无法创建目录 {}：{err}", parent.display()))
        })?;
    }
    let mut text = serde_json::to_string_pretty(value)
        .map_err(|err| crate::product::AppError::usage(format!("序列化失败：{err}")))?;
    text.push('\n');
    std::fs::write(path, text).map_err(|err| {
        crate::product::AppError::usage(format!("无法写入 {}：{err}", path.display()))
    })
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

/// `data` 入口。
#[derive(Args, Debug)]
pub struct DataArgs {
    #[command(subcommand)]
    pub command: DataCommand,
}

/// `data` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum DataCommand {
    /// 标注规则：list 看有哪些，show 看一条，save 写一条
    Rule {
        #[command(subcommand)]
        command: RuleCommand,
    },
    /// 任务包：规则快照加待标单元，标注端的输入
    Task {
        #[command(subcommand)]
        command: TaskCommand,
    },
    /// 标注结果：list 看分布，import 收标注端导出的 JSONL
    Result {
        #[command(subcommand)]
        command: ResultCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum RuleCommand {
    /// 列出全部规则
    List {
        #[arg(long)]
        json: bool,
    },
    /// 看一条规则的全文
    Show {
        /// 规则 id，`rul_` 开头
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// 写入一条规则（stdin 给 JSON）；带已有 id 则覆盖，不带则新建
    Save {
        /// stdin 形状：{"name":"…","source":{"kind":"journal","split":"blank-line"},"labels":[{"name":"fact","label":"事实"}],"batch":20}
        #[arg(long)]
        stdin: bool,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum TaskCommand {
    /// 列出全部任务包
    List {
        #[arg(long)]
        json: bool,
    },
    /// 看一个任务包（规则快照 + 单元清单）
    Show {
        /// 任务 id，`tsk_` 开头
        id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ResultCommand {
    /// 按标签统计标注结果
    List {
        /// 只统计某个任务
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// 导入标注端导出的 JSONL，按行里的 task（或 --task）归档
    Import {
        /// 导出文件路径
        file: PathBuf,
        /// 文件里没写 task 时用它兜底
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

pub fn run(command: DataCommand) -> Result<(), crate::product::AppError> {
    match command {
        DataCommand::Rule { command } => run_rule(command),
        DataCommand::Task { command } => run_task(command),
        DataCommand::Result { command } => run_result(command),
    }
}

// ---------- rule ----------

fn run_rule(command: RuleCommand) -> Result<(), crate::product::AppError> {
    match command {
        RuleCommand::List { json } => {
            let rules = load_all_rules()?;
            if json {
                #[derive(Serialize)]
                struct Row<'a> {
                    id: &'a str,
                    name: &'a str,
                    labels: usize,
                    batch: usize,
                }
                let rows: Vec<Row> = rules
                    .iter()
                    .map(|rule| Row {
                        id: &rule.id,
                        name: &rule.name,
                        labels: rule.labels.len(),
                        batch: rule.batch,
                    })
                    .collect();
                let payload = serde_json::json!({ "rules": rows });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_default()
                );
                return Ok(());
            }
            if rules.is_empty() {
                println!("暂无规则，先在 studio 设计一条，或 qtfounder data rule save --stdin");
                return Ok(());
            }
            for rule in &rules {
                println!(
                    "{}  {}  {} 个标签  批次 {}",
                    rule.id,
                    rule.name,
                    rule.labels.len(),
                    rule.batch
                );
            }
            Ok(())
        }
        RuleCommand::Show { id, json } => {
            let rule = load_rule(&id)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&rule).unwrap_or_default()
                );
                return Ok(());
            }
            println!("{}  {}", rule.id, rule.name);
            println!(
                "来源 {} / 切段 {}，批次 {}",
                rule.source.kind, rule.source.split, rule.batch
            );
            for label in &rule.labels {
                println!("  {}（{}）", label.label, label.name);
                if let Some(define) = &label.define {
                    println!("    定义：{define}");
                }
                if let Some(yes) = &label.yes {
                    println!("    正例：{yes}");
                }
                if let Some(no) = &label.no {
                    println!("    反例：{no}");
                }
            }
            Ok(())
        }
        RuleCommand::Save { stdin, json } => {
            if !stdin {
                return Err(crate::product::AppError::usage(
                    "用 --stdin 给规则 JSON".to_string(),
                ));
            }
            let mut buffer = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut buffer)
                .map_err(|err| crate::product::AppError::usage(format!("读 stdin 失败：{err}")))?;

            #[derive(Deserialize)]
            struct RuleInput {
                #[serde(default)]
                id: Option<String>,
                name: String,
                source: RuleSource,
                labels: Vec<RuleLabel>,
                #[serde(default = "default_batch")]
                batch: usize,
            }
            fn default_batch() -> usize {
                20
            }

            let input: RuleInput = serde_json::from_str(&buffer).map_err(|err| {
                crate::product::AppError::usage(format!("stdin 不是合法 JSON：{err}"))
            })?;
            let mut rules = load_all_rules()?;
            let id = match input.id {
                Some(id) => {
                    if !rules.iter().any(|rule| rule.id == id) {
                        return Err(crate::product::AppError::NotFound {
                            message: format!("{id} 不存在"),
                            hint: "新建规则不要带 id，改由工具生成".to_string(),
                        });
                    }
                    id
                }
                None => store::new_id("rul", |candidate| {
                    rules.iter().any(|rule| rule.id == candidate)
                }),
            };
            let rule = Rule {
                id: id.clone(),
                name: input.name,
                source: input.source,
                labels: input.labels,
                batch: input.batch,
                at: store::now_iso8601(),
            };
            validate_rule(&rule)?;
            let path = rules_dir().join(format!("{id}.json"));
            write_json_file(&path, &rule)?;
            rules.retain(|existing| existing.id != id);
            rules.push(rule.clone());
            if json {
                let payload = serde_json::json!({
                    "id": rule.id,
                    "path": path.display().to_string(),
                    "labels": rule.labels.len(),
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_default()
                );
                return Ok(());
            }
            println!(
                "已保存 {}（{} 个标签）：{}",
                rule.id,
                rule.labels.len(),
                rule.name
            );
            println!("{}", path.display());
            Ok(())
        }
    }
}

fn load_rule(id: &str) -> Result<Rule, crate::product::AppError> {
    let path = rules_dir().join(format!("{id}.json"));
    if !path.exists() {
        return Err(crate::product::AppError::NotFound {
            message: format!("{id} 不存在"),
            hint: "用 qtfounder data rule list 查看".to_string(),
        });
    }
    read_json_file(&path)
}

fn load_all_rules() -> Result<Vec<Rule>, crate::product::AppError> {
    let dir = rules_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut rules = vec![];
    for entry in std::fs::read_dir(&dir).map_err(|err| {
        crate::product::AppError::usage(format!("无法读取 {}：{err}", dir.display()))
    })? {
        let entry = entry.map_err(|err| {
            crate::product::AppError::usage(format!("无法读取 {}：{err}", dir.display()))
        })?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let rule: Rule = read_json_file(&path)?;
        rules.push(rule);
    }
    rules.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(rules)
}

// ---------- task ----------

fn run_task(command: TaskCommand) -> Result<(), crate::product::AppError> {
    match command {
        TaskCommand::List { json } => {
            let tasks = load_all_tasks()?;
            if json {
                #[derive(Serialize)]
                struct Row<'a> {
                    id: &'a str,
                    rule_id: &'a str,
                    units: usize,
                    at: &'a str,
                }
                let rows: Vec<Row> = tasks
                    .iter()
                    .map(|task| Row {
                        id: &task.id,
                        rule_id: &task.rule_id,
                        units: task.units.len(),
                        at: &task.at,
                    })
                    .collect();
                let payload = serde_json::json!({ "tasks": rows });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_default()
                );
                return Ok(());
            }
            if tasks.is_empty() {
                println!("暂无任务包；在 studio 里选一条规则导出任务包");
                return Ok(());
            }
            for task in &tasks {
                println!(
                    "{}  {}  {} 个单元  {}",
                    task.id,
                    task.rule_id,
                    task.units.len(),
                    task.at
                );
            }
            Ok(())
        }
        TaskCommand::Show { id, json } => {
            let task = load_task(&id)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&task).unwrap_or_default()
                );
                return Ok(());
            }
            println!(
                "{}  规则 {}（{}）  {} 个单元",
                task.id,
                task.rule_id,
                task.rule.name,
                task.units.len()
            );
            for unit in &task.units {
                let preview: String = unit.text.chars().take(40).collect();
                println!("  {}  {preview}", unit.id);
            }
            Ok(())
        }
    }
}

fn load_task(id: &str) -> Result<Task, crate::product::AppError> {
    let path = tasks_dir().join(format!("{id}.json"));
    if !path.exists() {
        return Err(crate::product::AppError::NotFound {
            message: format!("{id} 不存在"),
            hint: "用 qtfounder data task list 查看".to_string(),
        });
    }
    read_json_file(&path)
}

fn load_all_tasks() -> Result<Vec<Task>, crate::product::AppError> {
    let dir = tasks_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut tasks = vec![];
    for entry in std::fs::read_dir(&dir).map_err(|err| {
        crate::product::AppError::usage(format!("无法读取 {}：{err}", dir.display()))
    })? {
        let entry = entry.map_err(|err| {
            crate::product::AppError::usage(format!("无法读取 {}：{err}", dir.display()))
        })?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let task: Task = read_json_file(&path)?;
        tasks.push(task);
    }
    tasks.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(tasks)
}

// ---------- result ----------

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

fn load_results(task_filter: Option<&str>) -> Result<Vec<ResultLine>, crate::product::AppError> {
    let dir = results_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut lines = vec![];
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|err| {
            crate::product::AppError::usage(format!("无法读取 {}：{err}", dir.display()))
        })?
        .map(|entry| {
            entry.map(|entry| entry.path()).map_err(|err| {
                crate::product::AppError::usage(format!("无法读取 {}：{err}", dir.display()))
            })
        })
        .collect::<Result<_, _>>()?;
    entries.sort();
    for path in entries {
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if let Some(filter) = task_filter {
            if stem != filter {
                continue;
            }
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }
        let text = std::fs::read_to_string(&path).map_err(|err| {
            crate::product::AppError::usage(format!("无法读取 {}：{err}", path.display()))
        })?;
        for (index, row) in text.lines().enumerate() {
            if row.trim().is_empty() {
                continue;
            }
            let line: ResultLine = serde_json::from_str(row).map_err(|err| {
                crate::product::AppError::usage(format!(
                    "{} 第 {} 行解析失败：{err}",
                    path.display(),
                    index + 1
                ))
            })?;
            lines.push(line);
        }
    }
    Ok(lines)
}

fn run_result(command: ResultCommand) -> Result<(), crate::product::AppError> {
    match command {
        ResultCommand::List { task, json } => {
            if let Some(task) = &task {
                let dir = results_dir();
                if !dir.join(format!("{task}.jsonl")).exists() {
                    return Err(crate::product::AppError::NotFound {
                        message: format!("任务 {task} 没有结果"),
                        hint: "用 qtfounder data result list 查看有哪些任务".to_string(),
                    });
                }
            }
            let results = load_results(task.as_deref())?;
            let tallies = tally(&results);
            if json {
                #[derive(Serialize)]
                struct Row<'a> {
                    task: &'a str,
                    total: usize,
                    labels: std::collections::BTreeMap<&'a str, usize>,
                }
                let rows: Vec<Row> = tallies
                    .iter()
                    .map(|tally| Row {
                        task: &tally.task,
                        total: tally.total,
                        labels: tally
                            .labels
                            .iter()
                            .map(|(name, count)| (name.as_str(), *count))
                            .collect(),
                    })
                    .collect();
                let payload = serde_json::json!({ "tasks": rows });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_default()
                );
                return Ok(());
            }
            if tallies.is_empty() {
                println!("暂无标注结果");
                return Ok(());
            }
            for tally in tallies {
                let labels: Vec<String> = tally
                    .labels
                    .iter()
                    .map(|(name, count)| format!("{name} {count}"))
                    .collect();
                println!(
                    "{}  共 {} 条：{}",
                    tally.task,
                    tally.total,
                    labels.join("、")
                );
            }
            Ok(())
        }
        ResultCommand::Import { file, task, json } => {
            if !file.exists() {
                return Err(crate::product::AppError::NotFound {
                    message: format!("文件不存在：{}", file.display()),
                    hint: "给标注端导出的 JSONL 路径".to_string(),
                });
            }
            let text = std::fs::read_to_string(&file).map_err(|err| {
                crate::product::AppError::usage(format!("无法读取 {}：{err}", file.display()))
            })?;
            let mut imported = 0usize;
            let mut by_task: std::collections::BTreeMap<String, Vec<String>> =
                std::collections::BTreeMap::new();
            for (index, row) in text.lines().enumerate() {
                if row.trim().is_empty() {
                    continue;
                }
                let mut line: ResultLine = serde_json::from_str(row).map_err(|err| {
                    crate::product::AppError::usage(format!(
                        "第 {} 行不是合法 JSON：{err}",
                        index + 1
                    ))
                })?;
                if let Some(task) = &task {
                    line.task = task.clone();
                }
                if line.task.trim().is_empty() {
                    return Err(crate::product::AppError::usage(format!(
                        "第 {} 行缺 task 字段，或用 --task 指定",
                        index + 1
                    )));
                }
                if line.unit.trim().is_empty() || line.label.trim().is_empty() {
                    return Err(crate::product::AppError::usage(format!(
                        "第 {} 行缺 unit 或 label 字段",
                        index + 1
                    )));
                }
                if line.at.is_none() {
                    line.at = Some(store::now_iso8601());
                }
                by_task.entry(line.task.clone()).or_default().push(
                    serde_json::to_string(&line).map_err(|err| {
                        crate::product::AppError::usage(format!("序列化失败：{err}"))
                    })?,
                );
                imported += 1;
            }
            if imported == 0 {
                return Err(crate::product::AppError::usage("文件里一行结果都没有"));
            }
            let mut files = vec![];
            for (task, rows) in by_task {
                let path = results_dir().join(format!("{task}.jsonl"));
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).map_err(|err| {
                        crate::product::AppError::usage(format!(
                            "无法创建目录 {}：{err}",
                            parent.display()
                        ))
                    })?;
                }
                use std::io::Write as _;
                let mut handle = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .map_err(|err| {
                        crate::product::AppError::usage(format!(
                            "无法写入 {}：{err}",
                            path.display()
                        ))
                    })?;
                for row in rows {
                    writeln!(handle, "{row}").map_err(|err| {
                        crate::product::AppError::usage(format!("写入失败：{err}"))
                    })?;
                }
                files.push(path.display().to_string());
            }
            if json {
                let payload = serde_json::json!({
                    "imported": imported,
                    "files": files,
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_default()
                );
                return Ok(());
            }
            println!("已导入 {imported} 条");
            for path in files {
                println!("{path}");
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;
