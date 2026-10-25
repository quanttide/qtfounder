//! `qtfounder skill`：把二进制里嵌的 Skill 装进目标仓。
//!
//! 与另外几个子命令不同，它不改任何内容，只搬运：canonical 落点是
//! `<root>/.agents/skills/<id>`，同时维护 `.agents/.skill-lock.json`。
//! 嵌入清单由 `build.rs` 生成，来源是主仓的 `.agents/skills`。

pub mod registry;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use quanttide_founder::agent::skills::{
    self, EmbeddedSkill, InstallOptions, Mode, Status, install_report,
};

/// `skill` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum SkillCommand {
    /// 看嵌了哪些 Skill，目标仓装到哪一步
    List(ListArgs),
    /// 装一个或全部到目标仓
    Install(InstallArgs),
    /// 只查未装与不一致；有就退 1
    Check(ListArgs),
}

/// `skill` 入口。
#[derive(Args, Debug)]
pub struct SkillArgs {
    #[command(subcommand)]
    pub command: SkillCommand,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// 哪个仓的安装；不给就看系统位置（$HOME/.agents/skills）
    #[arg(long)]
    pub root: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Skill id；不给就装全部
    pub id: Option<String>,
    /// 装到哪个仓；不给就装到系统位置（$HOME/.agents/skills）
    #[arg(long)]
    pub root: Option<PathBuf>,
    /// 复制而不是链接
    #[arg(long)]
    pub copy: bool,
    /// 同时投递到 .claude/skills
    #[arg(long)]
    pub claude: bool,
    /// 先清掉同名目录再装
    #[arg(long)]
    pub force: bool,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &SkillArgs) -> Result<i32, String> {
    match &args.command {
        SkillCommand::List(list) => report(list, false),
        SkillCommand::Check(list) => report(list, true),
        SkillCommand::Install(command) => install(command),
    }
}

/// `list` 与 `check` 共用一张表，区别只在报什么、怎么退出。
fn report(args: &ListArgs, check: bool) -> Result<i32, String> {
    let root = resolve_root(args.root.as_deref())?;
    let entries = registry::all();

    if entries.is_empty() {
        let note =
            "这个二进制没嵌 Skill：构建时 skills/ 是空的，或 QTFOUNDER_SKILLS_DIR 指错了目录";
        if args.json {
            println!(
                "{}",
                serde_json::json!({
                    "source": serde_json::Value::Null,
                    "root": root,
                    "skills": [],
                    "note": note,
                })
            );
        } else {
            println!("{note}");
        }
        return Ok(if check { 2 } else { 0 });
    }

    let mut rows = Vec::new();
    for entry in entries {
        let description = skills::describe(entry)
            .map(|summary| summary.description)
            .unwrap_or_default();
        rows.push((
            entry.id.to_string(),
            description,
            skills::status(entry, &root),
        ));
    }

    if args.json {
        let items: Vec<serde_json::Value> = rows
            .iter()
            .map(|(id, description, status)| {
                serde_json::json!({
                    "id": id,
                    "description": description,
                    "status": status.key(),
                    "path": skills::skill_path(&root, id),
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::json!({
                "source": registry::source(),
                "root": root,
                "skills": items,
            })
        );
    } else {
        let width = rows
            .iter()
            .map(|(id, _, _)| id.chars().count())
            .max()
            .unwrap_or(0);
        println!(
            "嵌入 {} 个 Skill（源：{}）",
            rows.len(),
            registry::source().unwrap_or("未标注")
        );
        println!("安装位置：{}", root.display());
        println!();
        for (id, _, status) in &rows {
            if check && *status == Status::Current {
                continue;
            }
            println!("{id:<width$}  {}", status.label(), width = width);
        }
        println!();
        let current = rows
            .iter()
            .filter(|(_, _, s)| *s == Status::Current)
            .count();
        let stale = rows.iter().filter(|(_, _, s)| *s == Status::Stale).count();
        let missing = rows
            .iter()
            .filter(|(_, _, s)| *s == Status::Missing)
            .count();
        if check {
            if current == rows.len() {
                println!("{} 个 Skill 都与嵌入版本一致", rows.len());
            } else {
                println!("{stale} 个不一致、{missing} 个未装；跑 qtfounder skill install 补齐");
            }
        } else {
            println!("已是最新 {current}，不一致 {stale}，未安装 {missing}");
        }
    }

    if check {
        let clean = rows.iter().all(|(_, _, status)| *status == Status::Current);
        return Ok(if clean { 0 } else { 1 });
    }
    Ok(0)
}

fn install(args: &InstallArgs) -> Result<i32, String> {
    let root = resolve_root(args.root.as_deref())?;
    let entries = registry::all();
    if entries.is_empty() {
        return Err(
            "这个二进制没嵌 Skill：构建时 skills/ 是空的，或 QTFOUNDER_SKILLS_DIR 指错了目录"
                .to_string(),
        );
    }

    let targets: Vec<&EmbeddedSkill> = match &args.id {
        Some(id) => {
            let found = registry::find(id).ok_or_else(|| {
                let ids: Vec<&str> = entries.iter().map(|entry| entry.id).collect();
                format!("没有嵌入「{id}」，可用：{}", ids.join("／"))
            })?;
            vec![found]
        }
        None => entries.iter().collect(),
    };

    let mut options = InstallOptions::new(&root);
    options.mode = if args.copy { Mode::Copy } else { Mode::Link };
    options.force = args.force;
    if args.claude {
        options.target_dirs.push(root.join(".claude/skills"));
    }

    let mut installed = Vec::new();
    let mut failed = Vec::new();
    let mut warnings = Vec::new();
    for entry in targets {
        match install_report(entry, &options) {
            Ok((item, note)) => {
                installed.push(item);
                if let Some(note) = note
                    && !warnings.contains(&note)
                {
                    warnings.push(note);
                }
            }
            Err(error) => failed.push((entry.id.to_string(), error)),
        }
    }
    for warning in &warnings {
        eprintln!("! {warning}");
    }

    if args.json {
        println!(
            "{}",
            serde_json::json!({
                "root": root,
                "installed": installed
                    .iter()
                    .map(|item| serde_json::json!({
                        "id": item.id,
                        "path": item.path,
                        "symlink_failed": item.symlink_failed,
                    }))
                    .collect::<Vec<serde_json::Value>>(),
                "failed": failed
                    .iter()
                    .map(|(id, error)| serde_json::json!({"id": id, "error": error}))
                    .collect::<Vec<serde_json::Value>>(),
            })
        );
    } else {
        for item in &installed {
            let note = if item.symlink_failed {
                "（链接不可用，已复制）"
            } else {
                ""
            };
            println!("✓ {} → {}{note}", item.id, item.path.display());
        }
        for (id, error) in &failed {
            eprintln!("✗ {id}：{error}");
        }
        println!();
        println!("装了 {} 个，失败 {} 个。", installed.len(), failed.len());
    }

    Ok(if failed.is_empty() { 0 } else { 2 })
}

/// 不给 `--root` 就装到系统位置：`$HOME`，落到 `$HOME/.agents/skills`。
fn resolve_root(root: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(path) = root {
        return Ok(path.to_path_buf());
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "装到系统位置要读 HOME，但没读到；用 --root 指定目标仓".to_string())
}
