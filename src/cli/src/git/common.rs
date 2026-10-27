//! `git` 各子命令共用的仓发现与 Git 调用。

use std::path::{Path, PathBuf};
use std::process::Command;

/// 钩子目录的固定名字。
pub(super) const HOOKS_DIR: &str = ".githooks";

/// 一个要处理的仓：根自己，或某个子模块。
#[derive(Debug, Clone)]
pub(super) struct Repo {
    pub(super) dir: PathBuf,
    pub(super) label: String,
    pub(super) initialized: bool,
}

pub(super) fn resolve_root(root: Option<&Path>) -> Result<PathBuf, String> {
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

pub(super) fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
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
pub(super) fn repos(root: &Path) -> Result<Vec<Repo>, String> {
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

pub(super) fn configured_path(value: &str, repo: &Path) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        repo.join(path)
    }
}

pub(super) fn same_path(left: &Path, right: &Path) -> bool {
    match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

/// `.githooks` 里除日志外的文件都应当是脚本，且可执行。
#[cfg(unix)]
pub(super) fn non_executable(dir: &Path) -> Vec<String> {
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
pub(super) fn non_executable(_dir: &Path) -> Vec<String> {
    Vec::new()
}

pub(super) fn display_name(label: &str) -> String {
    if label == "." {
        "主仓".to_string()
    } else {
        label.to_string()
    }
}
