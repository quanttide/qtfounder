//! `qtfounder infra`：把要周期跑或事件触发的事登记下来，并在事件来时派发。
//!
//! CLI 自己跑不了周期——它被调用就退出。所以这里管两件事：**一张订阅表**，和**把触发
//! 接到外面去的那行文本**（`emit`）。解释权一律在驱动方：今天是 git 钩子与 crontab，
//! 将来是云端事件总线；换驱动方不动表，也不动命令面。
//!
//! 刻意不做三件：不写用户的 crontab、不动仓里已有的 `.githooks`、不解释事件名与周期串。
//! 前两件都是往外系统里伸手，要单独一次决定；第三件一解释就把驱动方的选择焊死了。

pub mod store;

use std::process::Command;

use clap::{Args, Subcommand};

use crate::core::exit;
use store::{Subscription, Trigger};

/// 订阅登记与派发：周期性的、事件触发的，都登记在同一张表里。
#[derive(Args, Debug)]
pub struct InfraArgs {
    #[command(subcommand)]
    pub command: InfraCommand,
}

#[derive(Subcommand, Debug)]
pub enum InfraCommand {
    /// 登记一条订阅；名字在表内唯一
    Add {
        /// 订阅名
        name: String,
        /// 盯一个事件（事件名原样交给驱动方）
        #[arg(long, conflicts_with = "every")]
        on: Option<String>,
        /// 按周期跑（周期串原样交给驱动方，如 `*/5 * * * *`）
        #[arg(long, conflicts_with = "on")]
        every: Option<String>,
        /// 要跑的命令，经 `sh -c` 执行
        #[arg(long)]
        run: String,
    },
    /// 列出全部订阅
    List {
        /// 输出 JSON
        #[arg(long)]
        json: bool,
    },
    /// 删掉一条订阅
    Remove {
        /// 订阅名
        name: String,
    },
    /// 跑一条订阅，返回它的退出码
    Run {
        /// 订阅名
        name: String,
    },
    /// 事件来了：跑所有盯这个事件且已启用的订阅
    Fire {
        /// 事件名
        event: String,
    },
    /// 打印这条订阅怎么接到驱动方（git 钩子片段或 crontab 行），不写任何东西
    Emit {
        /// 订阅名
        name: String,
    },
}

pub fn run(args: &InfraArgs) -> Result<i32, String> {
    match &args.command {
        InfraCommand::Add {
            name,
            on,
            every,
            run,
        } => add(name, on.as_deref(), every.as_deref(), run),
        InfraCommand::List { json } => list(*json),
        InfraCommand::Remove { name } => remove(name),
        InfraCommand::Run { name } => run_one(name),
        InfraCommand::Fire { event } => fire(event),
        InfraCommand::Emit { name } => emit(name),
    }
}

fn trigger_of(on: Option<&str>, every: Option<&str>) -> Result<Trigger, String> {
    match (on, every) {
        (Some(event), None) => Ok(Trigger::Event {
            name: event.to_string(),
        }),
        (None, Some(spec)) => Ok(Trigger::Every {
            spec: spec.to_string(),
        }),
        (Some(_), Some(_)) => Err("--on 与 --every 只能给一个：一条订阅一种触发".to_string()),
        (None, None) => Err("要给 --on <事件> 或 --every <周期>，二选一".to_string()),
    }
}

fn add(name: &str, on: Option<&str>, every: Option<&str>, run: &str) -> Result<i32, String> {
    if name.trim().is_empty() {
        return Err("订阅名不能为空".to_string());
    }
    if run.trim().is_empty() {
        return Err("命令不能为空".to_string());
    }
    let trigger = match trigger_of(on, every) {
        Ok(trigger) => trigger,
        Err(error) => return Err(error),
    };

    let path = store::registry_path();
    let mut registry = match store::load(&path) {
        Ok(registry) => registry,
        Err(error) => return Err(error),
    };
    if registry.get(name).is_some() {
        return Err(
            format!("已有同名订阅「{name}」；要改就先 remove，再 add"),
        );
    }
    registry.subscriptions.push(Subscription {
        name: name.to_string(),
        trigger,
        run: run.to_string(),
        enabled: true,
    });
    if let Err(error) = store::save(&path, &registry) {
        return Err(error);
    }
    println!("记下订阅「{name}」：{}", describe(&registry.subscriptions.last().unwrap().trigger));
    println!("表在 {}", path.display());
    Ok(exit::OK)
}

fn list(json: bool) -> Result<i32, String> {
    let path = store::registry_path();
    let registry = match store::load(&path) {
        Ok(registry) => registry,
        Err(error) => return Err(error),
    };
    if json {
        let body = serde_json::to_string_pretty(&serde_json::json!({
            "path": path.display().to_string(),
            "subscriptions": registry.subscriptions,
        }))
        .map_err(|err| err.to_string())?;
        println!("{body}");
        return Ok(exit::OK);
    }
    println!("表：{}", path.display());
    if registry.subscriptions.is_empty() {
        println!("一条订阅都没有；`qtfounder infra add <名字> --on <事件> --run <命令>` 记一条");
        return Ok(exit::OK);
    }
    println!();
    for item in &registry.subscriptions {
        let state = if item.enabled { "启用" } else { "停用" };
        println!("{}  {}  {}", item.name, describe(&item.trigger), state);
        println!("    跑：{}", item.run);
    }
    Ok(exit::OK)
}

fn remove(name: &str) -> Result<i32, String> {
    let path = store::registry_path();
    let mut registry = match store::load(&path) {
        Ok(registry) => registry,
        Err(error) => return Err(error),
    };
    let before = registry.subscriptions.len();
    registry.subscriptions.retain(|item| item.name != name);
    if registry.subscriptions.len() == before {
        eprintln!("没有订阅「{name}」；`infra list` 看有哪些");
        return Ok(exit::NOT_FOUND);
    }
    if let Err(error) = store::save(&path, &registry) {
        return Err(error);
    }
    println!("删了订阅「{name}」");
    Ok(exit::OK)
}

fn run_one(name: &str) -> Result<i32, String> {
    let registry = match store::load(&store::registry_path()) {
        Ok(registry) => registry,
        Err(error) => return Err(error),
    };
    let Some(item) = registry.get(name) else {
        eprintln!("没有订阅「{name}」；`infra list` 看有哪些");
        return Ok(exit::NOT_FOUND);
    };
    if !should_run(item) {
        println!("订阅「{name}」已停用，不跑");
        return Ok(exit::OK);
    }
    Ok(execute(item))
}

fn fire(event: &str) -> Result<i32, String> {
    let registry = match store::load(&store::registry_path()) {
        Ok(registry) => registry,
        Err(error) => return Err(error),
    };
    let listeners = registry.listening(event);
    if listeners.is_empty() {
        // 没人听不是错：事件可能在装订阅之前就发了。
        println!("事件「{event}」没有订阅者");
        return Ok(exit::OK);
    }
    let mut codes = Vec::with_capacity(listeners.len());
    for item in listeners {
        codes.push(execute(item));
    }
    Ok(fire_exit(codes))
}

/// `fire` 的退出码：第一条失败的退码；全过或压根没人跑退 0。
///
/// 「没人听退 0」是摘闸门的退路——订阅表被清空后，钩子应该放行而不是拦人。
pub(super) fn fire_exit(codes: impl IntoIterator<Item = i32>) -> i32 {
    codes
        .into_iter()
        .find(|code| *code != exit::OK)
        .unwrap_or(exit::OK)
}

/// `emit` 打的两行：接线注释与驱动方那一行。
/// 抽出来是为了能断言——它就是 emit 的全部交付物。
pub(super) fn emit_lines(item: &Subscription) -> Vec<String> {
    match &item.trigger {
        Trigger::Event { name: event } => vec![
            format!("# git 钩子片段：追加进 .githooks/{event}（本仓已有钩子管理，不自动改）"),
            format!("exec qtfounder infra fire {event}"),
        ],
        Trigger::Every { spec } => vec![
            "# crontab 行：`crontab -e` 加进去（不自动写你的 crontab）".to_string(),
            format!("{spec} qtfounder infra run {}", item.name),
        ],
    }
}

/// `run` 的该不该跑：停用的跳过，但不删——停用不是删除。
pub(super) fn should_run(item: &Subscription) -> bool {
    item.enabled
}

fn execute(item: &Subscription) -> i32 {
    println!("跑订阅「{}」：{}", item.name, item.run);
    match Command::new("sh").arg("-c").arg(&item.run).status() {
        Ok(status) => status.code().unwrap_or(exit::FINDINGS),
        Err(error) => {
            eprintln!("跑不了「{}」：{error}", item.run);
            exit::FINDINGS
        }
    }
}

fn emit(name: &str) -> Result<i32, String> {
    let registry = match store::load(&store::registry_path()) {
        Ok(registry) => registry,
        Err(error) => return Err(error),
    };
    let Some(item) = registry.get(name) else {
        eprintln!("没有订阅「{name}」；`infra list` 看有哪些");
        return Ok(exit::NOT_FOUND);
    };
    for line in emit_lines(item) {
        println!("{line}");
    }
    Ok(exit::OK)
}

fn describe(trigger: &Trigger) -> String {
    match trigger {
        Trigger::Event { name } => format!("事件 {name}"),
        Trigger::Every { spec } => format!("周期 {spec}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(name: &str) -> Trigger {
        Trigger::Event {
            name: name.to_string(),
        }
    }

    fn sub(name: &str, trigger: Trigger, enabled: bool) -> Subscription {
        Subscription {
            name: name.to_string(),
            trigger,
            run: "true".to_string(),
            enabled,
        }
    }

    #[test]
    fn 全过或没人跑都退零() {
        assert_eq!(fire_exit(std::iter::empty()), exit::OK, "没人听不是错");
        assert_eq!(fire_exit([0, 0, 0]), exit::OK);
    }

    #[test]
    fn 第一条失败说了算() {
        // 中间夹着成功的也照样退第一条失败的码——不取最后一条，也不求和。
        assert_eq!(fire_exit([0, 3, 2]), 3);
        assert_eq!(fire_exit([2, 3]), 2);
    }

    #[test]
    fn 停用的不跑但也不删() {
        let on = sub("a", event("pre-commit"), true);
        let off = sub("b", event("pre-commit"), false);
        assert!(should_run(&on));
        assert!(!should_run(&off), "停用不是删除：记录还在，只是不派发");
    }

    #[test]
    fn emit事件给钩子片段() {
        let item = sub("gate", event("pre-commit"), true);
        let lines = emit_lines(&item);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains(".githooks/pre-commit"), "注释要说清贴哪：{}", lines[0]);
        assert_eq!(lines[1], "exec qtfounder infra fire pre-commit");
    }

    #[test]
    fn emit周期给crontab行() {
        let item = sub("nightly", Trigger::Every { spec: "0 3 * * *".into() }, true);
        let lines = emit_lines(&item);
        assert!(lines[0].contains("crontab -e"), "注释要说清怎么贴：{}", lines[0]);
        assert_eq!(lines[1], "0 3 * * * qtfounder infra run nightly");
    }

    #[test]
    fn emit不替人接线() {
        // emit 只产文本，不写 crontab、不动钩子文件——立场写在 dev-guide/local-and-cloud.md。
        for item in [
            sub("a", event("pre-commit"), true),
            sub("b", Trigger::Every { spec: "* * * * *".into() }, true),
        ] {
            for line in emit_lines(&item) {
                assert!(!line.contains(">>") && !line.contains("crontab -e |"),
                    "emit 里出现重定向，它不该替人写：{line}");
            }
        }
    }
}
