//! `data result`：这一个子命令的命令面、实现与加载器。
//!
//! 与另两个子命令同形——各自带模型与加载器，不共用 match 臂。
//! 为什么要拆开：三个子命令是三件不同的事，挤在一个文件里时，
//! 改结果统计要滚过规则与任务包的实现。

use clap::Subcommand;

use serde::Serialize;

use std::path::PathBuf;

use crate::product::store;

use super::{model::ResultLine, tally, results_dir};

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

pub(crate) fn run_result(command: ResultCommand) -> Result<(), crate::product::AppError> {
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

