//! `qtfounder git hook`：装、查、卸工作区的 Git 钩子。
//!
//! 钩子目录固定在 `<root>/.githooks`，主仓与各层子模块共用同一份。
//! 只改 Git 配置，不改钩子内容；脚本缺失或不可执行只报不改。

use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Args, Subcommand};
use serde_json::json;

/// 钩子目录的固定名字。
const HOOKS_DIR: &str = ".githooks";

/// `git hook` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum HookCommand {
    /// 把主仓与子模块的 core.hooksPath 指到 .githooks
    Install(InstallArgs),
    /// 查各仓的配置有没有漂移；有就退 1
    Status(StatusArgs),
    /// 取消指向本仓 .githooks 的 core.hooksPath
    Uninstall(UninstallArgs),
}

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// 在哪个仓里操作；不给就用当前目录所在的仓
    #[arg(long)]
    pub root: Option<PathBuf>,
    /// 已指向别处时也覆盖
    #[arg(long)]
    pub force: bool,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// 在哪个仓里操作；不给就用当前目录所在的仓
    #[arg(long)]
    pub root: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// 在哪个仓里操作；不给就用当前目录所在的仓
    #[arg(long)]
    pub root: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct HookArgs {
    #[command(subcommand)]
    pub command: HookCommand,
}

pub fn run(args: &HookArgs) -> Result<i32, String> {
    match &args.command {
        HookCommand::Install(args) => install(args),
        HookCommand::Status(args) => status(args),
        HookCommand::Uninstall(args) => uninstall(args),
    }
}

/// 一个要处理的仓：根自己，或某个子模块。
#[derive(Debug, Clone)]
pub(super) struct Repo {
    pub(super) dir: PathBuf,
    pub(super) label: String,
    pub(super) initialized: bool,
}

/// 一个仓的钩子配置处在哪一步。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum State {
    Ok,
    Unset,
    Other(String),
    Broken,
    NotExecutable(Vec<String>),
    Uninitialized,
}

impl State {
    fn key(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Unset => "unset",
            Self::Other(_) => "other",
            Self::Broken => "broken",
            Self::NotExecutable(_) => "not_executable",
            Self::Uninitialized => "uninitialized",
        }
    }

    fn label(&self) -> String {
        match self {
            Self::Ok => "一致".to_string(),
            Self::Unset => "未安装".to_string(),
            Self::Other(path) => format!("指向 {path}"),
            Self::Broken => "钩子目录不存在".to_string(),
            Self::NotExecutable(scripts) => format!("不可执行：{}", scripts.join("、")),
            Self::Uninitialized => "未初始化".to_string(),
        }
    }

    fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }
}

fn resolve_root(root: Option<&Path>) -> Result<PathBuf, String> {
    let dir = root.unwrap_or(Path::new("."));
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|error| format!("跑不了 git：{error}"))?;
    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!("{} 不在 Git 仓里：{message}", dir.display()));
    }
    let top = String::from_utf8_lossy(&output.stdout).trim().to_string();
    std::fs::canonicalize(&top).map_err(|error| format!("读不到 {}：{error}", top))
}

fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|error| format!("跑不了 git：{error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// 根仓自己，加上各层子模块。
fn repos(root: &Path) -> Result<Vec<Repo>, String> {
    let mut repos = vec![Repo {
        dir: root.to_path_buf(),
        label: ".".to_string(),
        initialized: true,
    }];
    let listing = git(
        root,
        &[
            "-c",
            "core.quotePath=false",
            "submodule",
            "status",
            "--recursive",
        ],
    )?;
    for line in listing.lines() {
        if let Some(repo) = parse_submodule_line(root, line) {
            repos.push(repo);
        }
    }
    Ok(repos)
}

/// 解析 `git submodule status` 的一行：状态符、提交、路径、可选的描述。
pub(super) fn parse_submodule_line(root: &Path, line: &str) -> Option<Repo> {
    let line = line.trim_end();
    let mut chars = line.chars();
    let flag = chars.next()?;
    if !matches!(flag, ' ' | '+' | '-' | 'U') {
        return None;
    }
    let rest = &line[flag.len_utf8()..];
    let space = rest.find(' ')?;
    let path = &rest[space + 1..];
    let path = match path.rfind(" (") {
        Some(cut) => &path[..cut],
        None => path,
    };
    if path.is_empty() {
        return None;
    }
    Some(Repo {
        dir: root.join(path),
        label: path.to_string(),
        initialized: flag != '-',
    })
}

fn hooks_path(repo: &Path) -> Result<Option<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["config", "--get", "core.hooksPath"])
        .output()
        .map_err(|error| format!("跑不了 git：{error}"))?;
    if output.status.success() {
        return Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ));
    }
    if output.status.code() == Some(1) {
        return Ok(None);
    }
    Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
}

fn set_hooks_path(repo: &Path, value: &Path) -> Result<(), String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["config", "core.hooksPath"])
        .arg(value)
        .output()
        .map_err(|error| format!("跑不了 git：{error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn unset_hooks_path(repo: &Path) -> Result<(), String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["config", "--unset-all", "core.hooksPath"])
        .output()
        .map_err(|error| format!("跑不了 git：{error}"))?;
    match output.status.code() {
        // 5：原本就没有这个键，等于已经卸下。
        Some(0) | Some(5) => Ok(()),
        _ => Err(String::from_utf8_lossy(&output.stderr).trim().to_string()),
    }
}

fn configured_path(value: &str, repo: &Path) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        repo.join(path)
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

/// `.githooks` 里除日志外的文件都应当是脚本，且可执行。
#[cfg(unix)]
fn non_executable(dir: &Path) -> Vec<String> {
    use std::os::unix::fs::PermissionsExt;

    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().is_some_and(|extension| extension == "log") {
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.permissions().mode() & 0o111 == 0 {
            found.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    found.sort();
    found
}

#[cfg(not(unix))]
fn non_executable(_dir: &Path) -> Vec<String> {
    Vec::new()
}

fn inspect(repo: &Repo, managed: &Path, managed_exists: bool) -> Result<State, String> {
    if !repo.initialized {
        return Ok(State::Uninitialized);
    }
    let Some(value) = hooks_path(&repo.dir)? else {
        return Ok(State::Unset);
    };
    let configured = configured_path(&value, &repo.dir);
    if !same_path(&configured, managed) {
        return Ok(State::Other(value));
    }
    if !managed_exists {
        return Ok(State::Broken);
    }
    let bad = non_executable(managed);
    if bad.is_empty() {
        Ok(State::Ok)
    } else {
        Ok(State::NotExecutable(bad))
    }
}

fn display_name(label: &str) -> String {
    if label == "." {
        "主仓".to_string()
    } else {
        label.to_string()
    }
}

fn status(args: &StatusArgs) -> Result<i32, String> {
    let root = resolve_root(args.root.as_deref())?;
    let managed = root.join(HOOKS_DIR);
    let managed_exists = managed.is_dir();
    let repos = repos(&root)?;

    let mut rows = Vec::new();
    for repo in &repos {
        let state = inspect(repo, &managed, managed_exists)?;
        rows.push((repo.clone(), state));
    }
    let ok = rows.iter().all(|(_, state)| state.is_ok());

    if args.json {
        let items: Vec<serde_json::Value> = rows
            .iter()
            .map(|(repo, state)| {
                let detail = match state {
                    State::Other(path) => Some(path.clone()),
                    State::NotExecutable(scripts) => Some(scripts.join("、")),
                    _ => None,
                };
                json!({
                    "path": repo.label,
                    "state": state.key(),
                    "detail": detail,
                })
            })
            .collect();
        println!(
            "{}",
            json!({
                "root": root.display().to_string(),
                "hooks": managed.display().to_string(),
                "ok": ok,
                "repos": items,
            })
        );
    } else {
        let names: Vec<String> = rows
            .iter()
            .map(|(repo, _)| display_name(&repo.label))
            .collect();
        let width = names
            .iter()
            .map(|name| name.chars().count())
            .max()
            .unwrap_or(0)
            .max(4);
        println!("hooks 目录：{}", managed.display());
        for ((_, state), name) in rows.iter().zip(names.iter()) {
            println!("{name:<width$}  {}", state.label(), width = width);
        }
        println!();
        let ok_count = rows.iter().filter(|(_, state)| state.is_ok()).count();
        println!(
            "共 {} 个仓：一致 {}，漂移 {}",
            rows.len(),
            ok_count,
            rows.len() - ok_count
        );
    }

    Ok(if ok { 0 } else { 1 })
}

fn install(args: &InstallArgs) -> Result<i32, String> {
    let root = resolve_root(args.root.as_deref())?;
    let managed = root.join(HOOKS_DIR);
    if !managed.is_dir() {
        return Err(format!("找不到 {}；钩子目录不存在", managed.display()));
    }
    let bad = non_executable(&managed);
    if !bad.is_empty() {
        return Err(format!(
            "钩子脚本不可执行：{}；先 chmod +x {}",
            bad.join("、"),
            managed.display()
        ));
    }

    let repos = repos(&root)?;
    let mut installed = Vec::new();
    let mut skipped = Vec::new();
    let mut blocked = 0;
    for repo in &repos {
        if !repo.initialized {
            skipped.push((repo.label.clone(), "未初始化".to_string()));
            continue;
        }
        if let Some(value) = hooks_path(&repo.dir)? {
            let configured = configured_path(&value, &repo.dir);
            if !same_path(&configured, &managed) && !args.force {
                skipped.push((repo.label.clone(), format!("已指向 {value}")));
                blocked += 1;
                continue;
            }
        }
        set_hooks_path(&repo.dir, &managed)?;
        installed.push(repo.label.clone());
    }

    if args.json {
        println!(
            "{}",
            json!({
                "root": root.display().to_string(),
                "hooks": managed.display().to_string(),
                "installed": installed,
                "skipped": skipped
                    .iter()
                    .map(|(path, reason)| json!({"path": path, "reason": reason}))
                    .collect::<Vec<serde_json::Value>>(),
            })
        );
    } else {
        let width = installed
            .iter()
            .chain(skipped.iter().map(|(path, _)| path))
            .map(|path| display_name(path).chars().count())
            .max()
            .unwrap_or(0)
            .max(4);
        println!("hooks 目录：{}", managed.display());
        for path in &installed {
            println!("{:<width$}  已装", display_name(path), width = width);
        }
        for (path, reason) in &skipped {
            println!(
                "{:<width$}  跳过：{reason}",
                display_name(path),
                width = width
            );
        }
        println!();
        println!(
            "共 {} 个仓：装 {}，跳过 {}",
            repos.len(),
            installed.len(),
            skipped.len()
        );
    }

    Ok(if blocked == 0 { 0 } else { 1 })
}

fn uninstall(args: &UninstallArgs) -> Result<i32, String> {
    let root = resolve_root(args.root.as_deref())?;
    let managed = root.join(HOOKS_DIR);
    let repos = repos(&root)?;

    let mut removed = Vec::new();
    let mut absent = Vec::new();
    let mut kept = Vec::new();
    for repo in &repos {
        if !repo.initialized {
            kept.push((repo.label.clone(), "未初始化".to_string()));
            continue;
        }
        match hooks_path(&repo.dir)? {
            None => absent.push(repo.label.clone()),
            Some(value) => {
                let configured = configured_path(&value, &repo.dir);
                if same_path(&configured, &managed) {
                    unset_hooks_path(&repo.dir)?;
                    removed.push(repo.label.clone());
                } else {
                    kept.push((repo.label.clone(), format!("指向 {value}")));
                }
            }
        }
    }

    if args.json {
        println!(
            "{}",
            json!({
                "root": root.display().to_string(),
                "hooks": managed.display().to_string(),
                "removed": removed,
                "absent": absent,
                "kept": kept
                    .iter()
                    .map(|(path, reason)| json!({"path": path, "reason": reason}))
                    .collect::<Vec<serde_json::Value>>(),
            })
        );
    } else {
        let width = removed
            .iter()
            .chain(absent.iter())
            .chain(kept.iter().map(|(path, _)| path))
            .map(|path| display_name(path).chars().count())
            .max()
            .unwrap_or(0)
            .max(4);
        println!("hooks 目录：{}", managed.display());
        for path in &removed {
            println!("{:<width$}  已卸下", display_name(path), width = width);
        }
        for path in &absent {
            println!("{:<width$}  原本没装", display_name(path), width = width);
        }
        for (path, reason) in &kept {
            println!(
                "{:<width$}  保留：{reason}",
                display_name(path),
                width = width
            );
        }
        println!();
        println!(
            "共 {} 个仓：卸下 {}，原本没装 {}，保留 {}",
            repos.len(),
            removed.len(),
            absent.len(),
            kept.len()
        );
    }

    Ok(0)
}
