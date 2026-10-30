//! `data task`：这一个子命令的命令面、实现与加载器。
//!
//! 与另两个子命令同形——各自带模型与加载器，不共用 match 臂。
//! 为什么要拆开：三个子命令是三件不同的事，挤在一个文件里时，
//! 改结果统计要滚过规则与任务包的实现。

use clap::Subcommand;

use serde::Serialize;



use super::{model::Task, read_json_file, tasks_dir};

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

pub(crate) fn run_task(command: TaskCommand) -> Result<(), crate::product::AppError> {
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

