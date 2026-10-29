//! `qtfounder execute`：待办的收与抽。
//!
//! `execute collect` 从协作上下文里收待办进池子，`execute draw` 抽一件：
//! 没有优先级信息就均匀随机，有就按优先级加权。抽走的进 `drawn` 留痕，
//! 不自动放回——抽中就做，不做手改记录库文件。
//!
//! 存储复用 `product` 的数据目录与 id 方案（`execute.json`），错误信封同
//! `product`：业务错误一律 JSON 写 stderr，用法错退 2，未找到退 3。

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

use crate::product::AppError;
use crate::product::store;

/// 一件待办。`priority` 是 draw 的抽签权重，收下时就定好，缺省 1。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Todo {
    pub id: String,
    pub text: String,
    pub priority: u32,
    pub at: String,
}

/// 抽走的待办：多一个抽走时间，留在池子文件里作痕。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawnTodo {
    #[serde(flatten)]
    pub todo: Todo,
    pub drawn_at: String,
}

/// `execute.json` 的全部内容：待抽的池子 + 抽走的历史。
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Pool {
    #[serde(default)]
    pub pending: Vec<Todo>,
    #[serde(default)]
    pub drawn: Vec<DrawnTodo>,
}

/// `execute` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum ExecuteCommand {
    /// 收待办进池子：一句一件，或 --stdin 批量
    Collect {
        /// 待办，一句一件；批量走 --stdin
        texts: Vec<String>,
        /// 这一批的抽签权重，越大越容易抽中（缺省 1）
        #[arg(long)]
        priority: Option<u32>,
        /// 从 stdin 读一批：{"todos":[{"text":"…","priority":2},…]}
        #[arg(long)]
        stdin: bool,
    },
    /// 抽一件：没有优先级就随机，有就按优先级加权
    Draw,
}

/// 待办入口。
#[derive(Args, Debug)]
pub struct ExecuteArgs {
    #[command(subcommand)]
    pub command: ExecuteCommand,
    /// 输出 JSON
    #[arg(long, global = true)]
    pub json: bool,
}

fn data_file() -> std::path::PathBuf {
    store::data_dir().join("execute.json")
}

impl Pool {
    /// 读池子文件；文件不存在按空池处理。
    pub fn load() -> Result<Self, AppError> {
        let path = data_file();
        match std::fs::read_to_string(&path) {
            Ok(text) if text.trim().is_empty() => Ok(Pool::default()),
            Ok(text) => serde_json::from_str(&text).map_err(|err| {
                AppError::usage(format!("池子文件解析失败 {}：{err}", path.display()))
            }),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Pool::default()),
            Err(err) => Err(AppError::usage(format!(
                "无法读取 {}：{err}",
                path.display()
            ))),
        }
    }

    /// 写池子文件（先建目录）。
    pub fn save(&self) -> Result<(), AppError> {
        let path = data_file();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                AppError::usage(format!("无法创建目录 {}：{err}", parent.display()))
            })?;
        }
        let mut text = serde_json::to_string_pretty(self)
            .map_err(|err| AppError::usage(format!("池子序列化失败：{err}")))?;
        text.push('\n');
        std::fs::write(&path, text)
            .map_err(|err| AppError::usage(format!("无法写入 {}：{err}", path.display())))
    }

    /// 按 id 从池子里抽走一件：移出 `pending`、留痕进 `drawn`，返回抽走的那件。
    pub fn take(&mut self, index: usize) -> DrawnTodo {
        let todo = self.pending.remove(index);
        let drawn = DrawnTodo {
            drawn_at: store::now_iso8601(),
            todo,
        };
        self.drawn.push(drawn.clone());
        drawn
    }
}

/// 抽签权重列表里按 roll 取一件。纯函数，roll 由调用方给，便于钉住。
///
/// 权重和为零是调用方的错（空池应先报错），这里除不出去，返回末位兜底。
pub fn pick_index(priorities: &[u32], roll: u64) -> usize {
    let total: u64 = priorities.iter().map(|p| u64::from(*p)).sum();
    if total == 0 {
        return priorities.len().saturating_sub(1);
    }
    let mut target = roll % total;
    for (index, priority) in priorities.iter().enumerate() {
        if target < u64::from(*priority) {
            return index;
        }
        target -= u64::from(*priority);
    }
    priorities.len() - 1
}

/// 一次抽签：时间做种子拧一个 xorshift，抽签不需要密码学随机。
fn roll(bound: u64) -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let mut state = nanos
        ^ COUNTER
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_mul(0x9E37_79B9_7F4A_7C15);
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state % bound.max(1)
}

pub fn run(command: ExecuteCommand, json: bool) -> Result<(), AppError> {
    match command {
        ExecuteCommand::Collect {
            texts,
            priority,
            stdin,
        } => collect(&texts, priority, stdin, json),
        ExecuteCommand::Draw => draw(json),
    }
}

// ---------- collect ----------

/// stdin 批量的形状。
#[derive(Debug, Deserialize)]
struct CollectInput {
    todos: Vec<TodoInput>,
}

#[derive(Debug, Deserialize)]
struct TodoInput {
    text: String,
    #[serde(default)]
    priority: Option<u32>,
}

/// 收待办：同文本已在池子里的跳过，回执报收了几件、去重几件。
fn collect(
    texts: &[String],
    priority: Option<u32>,
    from_stdin: bool,
    json: bool,
) -> Result<(), AppError> {
    let inputs: Vec<TodoInput> = if from_stdin {
        if !texts.is_empty() {
            return Err(AppError::usage("位置参数与 --stdin 只能给一个".to_string()));
        }
        let mut buffer = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buffer)
            .map_err(|err| AppError::usage(format!("读 stdin 失败：{err}")))?;
        let input: CollectInput = serde_json::from_str(&buffer)
            .map_err(|err| AppError::usage(format!("stdin 不是合法 JSON：{err}")))?;
        input.todos
    } else {
        if texts.is_empty() {
            return Err(AppError::usage(
                "给一句话收一件，批量用 --stdin".to_string(),
            ));
        }
        texts
            .iter()
            .map(|text| TodoInput {
                text: text.clone(),
                priority,
            })
            .collect()
    };

    if inputs.is_empty() {
        return Err(AppError::usage("一件待办都没有".to_string()));
    }

    let mut pool = Pool::load()?;
    let mut collected: Vec<Todo> = Vec::new();
    let mut deduplicated = 0usize;
    for input in inputs {
        let text = input.text.trim().to_string();
        if text.is_empty() {
            return Err(AppError::usage("待办文本不能为空".to_string()));
        }
        let priority = input.priority.unwrap_or(1);
        if priority == 0 {
            return Err(AppError::usage(
                "优先级从 1 起：0 的签永远抽不中".to_string(),
            ));
        }
        if pool.pending.iter().any(|todo| todo.text == text) {
            deduplicated += 1;
            continue;
        }
        let todo = Todo {
            id: store::new_id("tdo", |id| {
                pool.pending.iter().any(|todo| todo.id == id)
                    || pool.drawn.iter().any(|drawn| drawn.todo.id == id)
            }),
            text,
            priority,
            at: store::now_iso8601(),
        };
        pool.pending.push(todo.clone());
        collected.push(todo);
    }
    pool.save()?;

    let pending = pool.pending.len();
    if json {
        let items: Vec<String> = collected
            .iter()
            .map(|todo| {
                format!(
                    "    {{\"id\":\"{}\",\"text\":\"{}\",\"priority\":{},\"at\":\"{}\"}}",
                    json_escape(&todo.id),
                    json_escape(&todo.text),
                    todo.priority,
                    json_escape(&todo.at)
                )
            })
            .collect();
        println!(
            "{{\n  \"collected\": [\n{}\n  ],\n  \"deduplicated\": {},\n  \"pending\": {}\n}}",
            items.join(",\n"),
            deduplicated,
            pending
        );
        return Ok(());
    }

    for todo in &collected {
        println!(
            "已收 {}（优先级 {}）：{}",
            todo.id, todo.priority, todo.text
        );
    }
    if deduplicated > 0 {
        println!("去重 {deduplicated} 件（池子里已有同文本的签）");
    }
    println!("池里待抽 {pending} 件");
    Ok(())
}

// ---------- draw ----------

/// 抽一件：空池报未找到（退 3），否则按优先级加权抽签。
fn draw(json: bool) -> Result<(), AppError> {
    let mut pool = Pool::load()?;
    if pool.pending.is_empty() {
        return Err(AppError::NotFound {
            message: "待办池是空的".to_string(),
            hint: "先跑 qtfounder execute collect 收几件".to_string(),
        });
    }
    let priorities: Vec<u32> = pool.pending.iter().map(|todo| todo.priority).collect();
    let index = pick_index(&priorities, roll(u64::from(priorities.iter().sum::<u32>())));
    let drawn = pool.take(index);
    let pending = pool.pending.len();
    pool.save()?;

    if json {
        println!(
            "{{\n  \"id\": \"{}\",\n  \"text\": \"{}\",\n  \"priority\": {},\n  \"at\": \"{}\",\n  \"drawn_at\": \"{}\",\n  \"pending\": {}\n}}",
            json_escape(&drawn.todo.id),
            json_escape(&drawn.todo.text),
            drawn.todo.priority,
            json_escape(&drawn.todo.at),
            json_escape(&drawn.drawn_at),
            pending
        );
        return Ok(());
    }
    println!(
        "抽中 {}（优先级 {}）：{}",
        drawn.todo.id, drawn.todo.priority, drawn.todo.text
    );
    if pending == 0 {
        println!("池子抽空了");
    } else {
        println!("池里还剩 {pending} 件");
    }
    Ok(())
}

fn json_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests;
