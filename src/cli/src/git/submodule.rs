//! `qtfounder git submodule`：看子模块的指针、改动与推送状态。
//!
//! 只读，不改工作区。`pre-push` 复用同一份判定，只取其中会挡推送的几项。

use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Args, Subcommand};
use serde_json::json;

use super::common::{display_name, git, repos, resolve_root};

/// `git submodule` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum SubmoduleCommand {
    /// 查指针、未提交改动与推送状态；有问题退 1
    Check(CheckArgs),
}

#[derive(Args, Debug)]
pub struct SubmoduleArgs {
    #[command(subcommand)]
    pub command: SubmoduleCommand,
}

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// 在哪个仓里操作；不给就用当前目录所在的仓
    #[arg(long)]
    pub root: Option<PathBuf>,
    /// 只查推送安全：未推送的提交、指针不在远端
    #[arg(long)]
    pub pushed: bool,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &SubmoduleArgs) -> Result<i32, String> {
    match &args.command {
        SubmoduleCommand::Check(args) => check(args),
    }
}

pub(super) struct Report {
    pub(super) repos: Vec<RepoReport>,
    /// 有推送风险（未推送提交、指针不在远端）的子模块数。
    pub(super) blocking: usize,
}

pub(super) struct RepoReport {
    pub(super) path: String,
    pub(super) head: Option<String>,
    pub(super) recorded: Option<String>,
    pub(super) issues: Vec<Issue>,
}

impl RepoReport {
    pub(super) fn blocking_labels(&self) -> Vec<String> {
        self.issues
            .iter()
            .filter(|issue| issue.blocking())
            .map(Issue::label)
            .collect()
    }

    fn labels(&self, pushed_only: bool) -> Vec<String> {
        self.issues
            .iter()
            .filter(|issue| !pushed_only || issue.blocking())
            .map(Issue::label)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Issue {
    Uninitialized,
    Drift,
    Dirty,
    Unpushed(usize),
    PointerUnpushed,
    PointerMissing,
}

impl Issue {
    fn key(&self) -> &'static str {
        match self {
            Self::Uninitialized => "uninitialized",
            Self::Drift => "drift",
            Self::Dirty => "dirty",
            Self::Unpushed(_) => "unpushed",
            Self::PointerUnpushed => "pointer-unpushed",
            Self::PointerMissing => "pointer-missing",
        }
    }

    fn label(&self) -> String {
        match self {
            Self::Uninitialized => "未初始化".to_string(),
            Self::Drift => "工作区与指针不一致".to_string(),
            Self::Dirty => "有未提交改动".to_string(),
            Self::Unpushed(count) => format!("未推送 {count} 个提交"),
            Self::PointerUnpushed => "指针提交不在远端".to_string(),
            Self::PointerMissing => "找不到指针提交".to_string(),
        }
    }

    fn detail(&self) -> Option<serde_json::Value> {
        match self {
            Self::Unpushed(count) => Some(json!(count)),
            _ => None,
        }
    }

    fn blocking(&self) -> bool {
        matches!(
            self,
            Self::Unpushed(_) | Self::PointerUnpushed | Self::PointerMissing
        )
    }
}

/// 查根仓下的所有子模块。`pushed_only` 时只算会挡推送的几项。
pub(super) fn inspect(root: &Path, pushed_only: bool) -> Result<Report, String> {
    let all = repos(root)?;
    let mut reports = Vec::new();
    let mut blocking = 0;

    for repo in all.iter().filter(|repo| repo.label != ".") {
        let mut issues = Vec::new();
        let (head, recorded) = if repo.initialized {
            (
                git(&repo.dir, &["rev-parse", "HEAD"])
                    .ok()
                    .map(|text| text.trim().to_string()),
                recorded_sha(&repo.dir),
            )
        } else {
            issues.push(Issue::Uninitialized);
            (None, None)
        };

        if repo.initialized {
            if !pushed_only {
                if let (Some(head), Some(recorded)) = (&head, &recorded)
                    && head != recorded
                {
                    issues.push(Issue::Drift);
                }
                if dirty(&repo.dir) {
                    issues.push(Issue::Dirty);
                }
            }
            let count = unpushed(&repo.dir);
            if count > 0 {
                issues.push(Issue::Unpushed(count));
            }
            match &recorded {
                Some(sha) => match pointer_state(&repo.dir, sha) {
                    PointerState::Pushed => {}
                    PointerState::Unpushed => issues.push(Issue::PointerUnpushed),
                    PointerState::Missing => issues.push(Issue::PointerMissing),
                },
                None => issues.push(Issue::PointerMissing),
            }
        }

        if issues.iter().any(Issue::blocking) {
            blocking += 1;
        }
        reports.push(RepoReport {
            path: repo.label.clone(),
            head,
            recorded,
            issues,
        });
    }

    Ok(Report {
        repos: reports,
        blocking,
    })
}

enum PointerState {
    Pushed,
    Unpushed,
    Missing,
}

fn pointer_state(dir: &Path, sha: &str) -> PointerState {
    let probe = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["cat-file", "-e", &format!("{sha}^{{commit}}")])
        .output();
    match probe {
        Ok(output) if output.status.success() => {}
        _ => return PointerState::Missing,
    }
    match git(dir, &["branch", "-r", "--contains", sha]) {
        Ok(text) if !text.trim().is_empty() => PointerState::Pushed,
        _ => PointerState::Unpushed,
    }
}

/// 父仓（或上一级子模块）HEAD 里记的指针提交。
fn recorded_sha(dir: &Path) -> Option<String> {
    let parent = parent_repo(dir)?;
    let relative = dir.strip_prefix(parent).ok()?;
    let output = Command::new("git")
        .arg("-C")
        .arg(parent)
        .args(["ls-tree", "-z", "HEAD", "--"])
        .arg(relative)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let entry = text.split('\0').next()?;
    let (meta, _) = entry.split_once('\t')?;
    meta.split_whitespace().nth(2).map(str::to_string)
}

/// 往上找到最近的仓根；子模块目录自己的 `.git` 是文件，不算父仓。
fn parent_repo(dir: &Path) -> Option<&Path> {
    let mut current = dir.parent()?;
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        current = current.parent()?;
    }
}

fn dirty(dir: &Path) -> bool {
    git(dir, &["status", "--porcelain"])
        .map(|text| !text.trim().is_empty())
        .unwrap_or(false)
}

fn unpushed(dir: &Path) -> usize {
    git(dir, &["rev-list", "--count", "HEAD", "--not", "--remotes"])
        .ok()
        .and_then(|text| text.trim().parse().ok())
        .unwrap_or(0)
}

fn check(args: &CheckArgs) -> Result<i32, String> {
    let root = resolve_root(args.root.as_deref())?;
    let report = inspect(&root, args.pushed)?;
    let flagged: Vec<&RepoReport> = report
        .repos
        .iter()
        .filter(|repo| !repo.labels(args.pushed).is_empty())
        .collect();
    let ok = if args.pushed {
        report.blocking == 0
    } else {
        flagged.is_empty()
    };

    if args.json {
        let items: Vec<serde_json::Value> = report
            .repos
            .iter()
            .map(|repo| {
                let issues: Vec<serde_json::Value> = repo
                    .issues
                    .iter()
                    .filter(|issue| !args.pushed || issue.blocking())
                    .map(|issue| {
                        json!({
                            "kind": issue.key(),
                            "detail": issue.detail(),
                        })
                    })
                    .collect();
                json!({
                    "path": repo.path,
                    "head": repo.head,
                    "recorded": repo.recorded,
                    "issues": issues,
                })
            })
            .collect();
        println!(
            "{}",
            json!({
                "root": root.display().to_string(),
                "pushed": args.pushed,
                "ok": ok,
                "submodules": items,
            })
        );
    } else if report.repos.is_empty() {
        println!("没有子模块。");
    } else {
        let width = report
            .repos
            .iter()
            .map(|repo| display_name(&repo.path).chars().count())
            .max()
            .unwrap_or(0)
            .max(4);
        for repo in &flagged {
            println!(
                "{:<width$}  {}",
                display_name(&repo.path),
                repo.labels(args.pushed).join("；"),
                width = width
            );
        }
        if !flagged.is_empty() {
            println!();
        }
        println!(
            "共 {} 个子模块：正常 {}，有问题 {}",
            report.repos.len(),
            report.repos.len() - flagged.len(),
            flagged.len()
        );
    }

    Ok(if ok { 0 } else { 1 })
}
