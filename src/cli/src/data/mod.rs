//! `qtfounder data`：标注规则、任务包与标注结果的存取。
//!
//! INTENT:
//!   核心目标：把标注要用的规则、任务包与结果存成三份能对上的 JSON，位置稳定
//!   边界：不做标注界面、不调模型、不搬文件；studio 是设计端，标注端只导结果
//!   约束：存储位置必须与 studio 一致，一律走 XDG；QTFOUNDER_DATA 优先，供测试与临时切换
//!   假设：读写的是同一批人，id 由写入方生成，不并发
//!   不确定：结果回流要不要加校验——现在只验格式，不验它对得上哪份任务包
//!
//! 子命令按领域拆文件：`rule`／`task`／`result` 各带自己的模型与加载器，
//! 本文件只留命令面、派发与三者共用的存储原语。
//!
//! 错误信封同 `product`：业务错误一律 JSON 写 stderr，用法错退 2，未找到退 3。

pub mod model;
pub mod result;
pub mod rule;
pub mod task;

#[cfg(test)]
mod tests;

pub use model::*;

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::product::store;

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
        command: rule::RuleCommand,
    },
    /// 任务包：规则快照加待标单元，标注端的输入
    Task {
        #[command(subcommand)]
        command: task::TaskCommand,
    },
    /// 标注结果：list 看分布，import 收标注端导出的 JSONL
    Result {
        #[command(subcommand)]
        command: result::ResultCommand,
    },
}

pub fn run(command: DataCommand) -> Result<(), crate::product::AppError> {
    match command {
        DataCommand::Rule { command } => rule::run_rule(command),
        DataCommand::Task { command } => task::run_task(command),
        DataCommand::Result { command } => result::run_result(command),
    }
}

// ---------- rule ----------
