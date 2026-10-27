//! `qtfounder git hook run`：钩子脚本的兜底实现。
//!
//! 脚本只负责把工作区根传进来，判定与动作都在这里：
//! `pre-commit` 只拦这一笔新引入的用词错误；`post-merge` 与 `post-checkout`
//! 把子模块对齐到指针提交；`pre-push` 查子模块有没有断掉的指针。

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Args, ValueEnum};

use crate::language::contract::{self, Contract};
use crate::language::model::{Finding, Severity, Verdict};
use crate::language::scan;

use super::common::{HOOKS_DIR, display_name, git, git_command, resolve_root};
use super::submodule;

/// 能执行的钩子。
#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum HookName {
    PreCommit,
    PostMerge,
    PostCheckout,
    PrePush,
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// 要跑哪个钩子
    pub name: HookName,
    /// 工作区根（钩子目录所在）；不给就用当前仓
    #[arg(long)]
    pub root: Option<PathBuf>,
}

pub fn run(args: &RunArgs) -> Result<i32, String> {
    if matches!(args.name, HookName::PostMerge | HookName::PostCheckout)
        && std::env::var("QT_SUBMODULE_SYNC").as_deref() == Ok("1")
    {
        return Ok(0);
    }

    // 钩子作用在当前仓；脚本传来的 --root 只用来找契约与面板日志。
    let repo = resolve_root(None)?;
    let workspace = match &args.root {
        Some(path) => std::fs::canonicalize(path)
            .map_err(|error| format!("读不到 {}：{error}", path.display()))?,
        None => repo.clone(),
    };

    match args.name {
        HookName::PreCommit => pre_commit(&repo, &workspace),
        HookName::PostMerge | HookName::PostCheckout => {
            sync_submodules(&repo);
            Ok(0)
        }
        HookName::PrePush => pre_push(&repo),
    }
}

fn pre_commit(repo: &Path, workspace: &Path) -> Result<i32, String> {
    let contract = match load_contract(workspace) {
        Ok(contract) => contract,
        Err(note) => {
            eprintln!("闸门：{note}，这次不查");
            return Ok(0);
        }
    };

    let listing = git(
        repo,
        &[
            "-c",
            "core.quotePath=false",
            "diff",
            "--cached",
            "--name-only",
            "--diff-filter=ACMR",
            "--",
            "*.md",
        ],
    )?;

    let base = TempBase::new()?;
    let mut blocked = Vec::new();
    let mut suspects = Vec::new();
    for path in listing.lines().filter(|line| !line.is_empty()) {
        if skipped(path) {
            continue;
        }
        let full = repo.join(path);
        if !full.is_file() {
            continue;
        }
        let current = scan::check_file(&full, &contract, Some(repo));
        let baseline = base.findings(repo, path, &contract);
        let new = new_findings(&current, &baseline);
        let suspect_count = current
            .iter()
            .filter(|finding| finding.severity == Severity::Suspect)
            .count();
        if suspect_count > 0 {
            suspects.push((path.to_string(), suspect_count));
        }
        if !new.is_empty() {
            blocked.push((path.to_string(), new));
        }
    }

    for (path, count) in &suspects {
        log_suspect(workspace, path, *count);
    }

    if blocked.is_empty() {
        return Ok(0);
    }
    for (path, findings) in &blocked {
        eprintln!("闸门拦下：{path}");
        for (kind, advice) in findings {
            eprintln!("  {kind}：{advice}");
        }
    }
    eprintln!("闸门：改到过再提交；确属误判，用 git commit --no-verify。");
    Ok(1)
}

/// 把 HEAD 里的旧内容写到同名的临时路径，落点检查才和当前文件对得上。
struct TempBase {
    dir: PathBuf,
}

impl TempBase {
    fn new() -> Result<Self, String> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("qtfounder-gate-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&dir)
            .map_err(|error| format!("建不了临时目录 {}：{error}", dir.display()))?;
        Ok(Self { dir })
    }

    fn findings(&self, repo: &Path, rel: &str, contract: &Contract) -> Vec<Finding> {
        let spec = format!("HEAD:{rel}");
        let Ok(output) = git_command(repo).args(["show", &spec]).output() else {
            return Vec::new();
        };
        if !output.status.success() {
            return Vec::new();
        }
        let path = self.dir.join(rel);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::write(&path, &output.stdout).is_err() {
            return Vec::new();
        }
        scan::check_file(&path, contract, Some(&self.dir))
    }
}

impl Drop for TempBase {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn load_contract(workspace: &Path) -> Result<Contract, String> {
    if let Ok(path) = std::env::var(contract::CONTRACT_ENV)
        && !path.is_empty()
    {
        return Contract::load(Some(Path::new(&path)));
    }
    for relative in [
        "apps/qtfounder/src/cli/src/language/default-contract.toml",
        "src/cli/src/language/default-contract.toml",
        "src/language/default-contract.toml",
    ] {
        let candidate = workspace.join(relative);
        if candidate.is_file() {
            return Contract::load(Some(&candidate));
        }
    }
    Contract::load(None)
}

/// 契约里排除的是「不是给人反复读的」：流水、归档、示例、变更日志。
pub(super) fn skipped(path: &str) -> bool {
    let wrapped = format!("/{path}");
    if wrapped.ends_with("/CHANGELOG.md") {
        return true;
    }
    [
        "/node_modules/",
        "/_build/",
        "/target/",
        "/archive/",
        "/journal/",
        "/report/",
        "/assets/",
        "/examples/",
    ]
    .iter()
    .any(|needle| wrapped.contains(needle))
}

/// 只收这一笔新引入的：同一处问题在 HEAD 里已经有，算存量。
pub(super) fn new_findings(current: &[Finding], baseline: &[Finding]) -> Vec<(String, String)> {
    let base = counts(baseline);
    let mut seen: HashMap<(String, String), usize> = HashMap::new();
    let mut new = Vec::new();
    for finding in current {
        if !blocks(finding) {
            continue;
        }
        let key = (finding.kind.key().to_string(), finding.advice.clone());
        let count = seen.entry(key.clone()).or_insert(0);
        *count += 1;
        if *count > base.get(&key).copied().unwrap_or(0) && !new.contains(&key) {
            new.push(key);
        }
    }
    new
}

fn counts(findings: &[Finding]) -> HashMap<(String, String), usize> {
    let mut counts = HashMap::new();
    for finding in findings.iter().filter(|finding| blocks(finding)) {
        *counts
            .entry((finding.kind.key().to_string(), finding.advice.clone()))
            .or_insert(0) += 1;
    }
    counts
}

fn blocks(finding: &Finding) -> bool {
    finding.severity == Severity::Error && finding.verdict == Verdict::Real && finding.line != 0
}

fn log_suspect(workspace: &Path, path: &str, count: usize) {
    let dir = workspace.join(HOOKS_DIR);
    if !dir.is_dir() {
        return;
    }
    let stamp = Command::new("date")
        .arg("+%F %T")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_default();
    let line = format!("{stamp} | {path} | language | 嫌疑 {count} 处\n");
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("passed.log"))
    {
        let _ = file.write_all(line.as_bytes());
    }
}

fn sync_submodules(repo: &Path) {
    let _ = git(repo, &["submodule", "sync", "--recursive"]);
    let output = git_command(repo)
        .args(["submodule", "update", "--init", "--recursive"])
        .env("QT_SUBMODULE_SYNC", "1")
        .output();
    match output {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout);
            if !text.trim().is_empty() {
                print!("{text}");
            }
        }
        Ok(output) => {
            eprintln!(
                "子模块同步失败：先处理子模块里的本地改动，再跑 git submodule update --init --recursive"
            );
            let text = String::from_utf8_lossy(&output.stderr);
            if !text.trim().is_empty() {
                eprintln!("{}", text.trim());
            }
        }
        Err(error) => eprintln!("子模块同步失败：跑不了 git：{error}"),
    }
}

fn pre_push(repo: &Path) -> Result<i32, String> {
    let report = submodule::inspect(repo, true)?;
    if report.blocking == 0 {
        return Ok(0);
    }
    for item in &report.repos {
        let labels = item.blocking_labels();
        if !labels.is_empty() {
            eprintln!("{}：{}", display_name(&item.path), labels.join("；"));
        }
    }
    eprintln!("pre-push：先推送子模块或修正指针，再推父仓。");
    Ok(1)
}
