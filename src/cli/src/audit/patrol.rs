//! 存量巡检：把历史快照按今天的契约重判一遍。
//!
//! 体检回答「现在还有多少」，巡检回答「当年漏过去多少、后来有多少被人改掉」。
//! 做法是抽一批提交，各自导出成一棵树，用当前契约重判，再和今天逐条比。
//!
//! 文件改名或删除时，旧命中随之清零，这不算有人改过。所以比对分三档：
//! 随文件消失、在留存文件里被改掉、今天仍在。第三档是仍然背着的存量。
//!
//! 巡检很慢：每个仓默认抽 12 个提交，每个提交导出一次、扫两遍。只在需要基线时跑。

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use walkdir::WalkDir;

use crate::docs::contract::Contract as DocsContract;
use crate::docs::model::Severity as DocsSeverity;
use crate::docs::scan::scan as scan_docs;
use crate::language::contract::Contract as LanguageContract;
use crate::language::model::Severity as LanguageSeverity;
use crate::language::scan::scan as scan_language;

/// 一条命中：同一个文件、同一条规则、同一种改法，算一条。
type Key = (String, String, String);

#[derive(Default)]
pub struct Report {
    pub repos: usize,
    pub snapshots: usize,
    pub seen: usize,
    pub alive: usize,
    pub file_gone: usize,
    pub fixed: usize,
    pub terminology_gone: usize,
    pub terminology_fixed: usize,
    pub sampled: usize,
    pub targeted: usize,
    pub incidental: usize,
    pub today_errors: usize,
}

#[derive(Clone)]
struct Repo {
    /// 相对根目录的路径，空串表示根自己就是仓
    prefix: String,
    dir: PathBuf,
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

/// 找出根目录下的所有 git 仓：根自己，加上各层子模块。
fn discover(root: &Path) -> Vec<Repo> {
    let mut repos = Vec::new();
    for entry in WalkDir::new(root)
        .max_depth(6)
        .into_iter()
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            entry.depth() == 0
                || !(name == ".git"
                    || name == "target"
                    || name == "node_modules"
                    || name == "_build")
        })
        .flatten()
    {
        if !entry.file_type().is_dir() || !entry.path().join(".git").exists() {
            continue;
        }
        let prefix = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(Path::new(""))
            .to_string_lossy()
            .to_string();
        repos.push(Repo {
            prefix,
            dir: entry.path().to_path_buf(),
        });
    }
    repos.sort_by(|a, b| a.prefix.cmp(&b.prefix));
    repos
}

/// 按提交数均匀抽样：两端必取，中间等距。
fn pick_commits(dir: &Path, samples: usize) -> Result<Vec<(String, String)>, String> {
    let log = git(dir, &["log", "--reverse", "--format=%H %as", "HEAD"])?;
    let rows: Vec<(String, String)> = log
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            Some((parts.next()?.to_string(), parts.next()?.to_string()))
        })
        .collect();
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    if samples < 2 || rows.len() <= samples {
        return Ok(rows);
    }
    let mut picked = Vec::new();
    for slot in 0..samples {
        let index = slot * (rows.len() - 1) / (samples - 1);
        let row = rows[index].clone();
        if picked.last().map(|last: &(String, String)| &last.0) != Some(&row.0) {
            picked.push(row);
        }
    }
    Ok(picked)
}

fn temp_root() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("qtfounder-patrol-{}-{stamp}", std::process::id()))
}

/// 把一个提交导出到 `dest`。路径保持根目录相对，契约里的 exclude 才对得上。
fn export(repo: &Repo, sha: &str, workspace: &Path) -> Result<PathBuf, String> {
    let dest = if repo.prefix.is_empty() {
        workspace.to_path_buf()
    } else {
        workspace.join(&repo.prefix)
    };
    fs::create_dir_all(&dest).map_err(|error| format!("建不了临时目录：{error}"))?;
    let tar = workspace.join("snapshot.tar");
    git(
        &repo.dir,
        &["archive", "--format=tar", "-o", &tar.to_string_lossy(), sha],
    )?;
    let status = Command::new("tar")
        .arg("-xf")
        .arg(&tar)
        .arg("-C")
        .arg(&dest)
        .status()
        .map_err(|error| format!("解不开快照：{error}"))?;
    let _ = fs::remove_file(&tar);
    if !status.success() {
        return Err(format!("解不开快照：{sha}"));
    }
    Ok(dest)
}

fn clear(dir: &Path) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            let _ = fs::remove_dir_all(&path);
        } else {
            let _ = fs::remove_file(&path);
        }
    }
}

/// 扫一棵树，取出真问题，按「文件 + 规则 + 改法」归并。
fn keys(root: &Path, language: &LanguageContract, docs: &DocsContract) -> HashSet<Key> {
    let mut keys = HashSet::new();
    let (findings, _, _) = scan_language(root, language, "default");
    for finding in findings {
        if finding.severity != LanguageSeverity::Error
            || finding.verdict.is_false_positive()
            || finding.line == 0
        {
            continue;
        }
        keys.insert((finding.path, finding.kind.key().to_string(), finding.advice));
    }
    let (findings, _, _) = scan_docs(root, docs);
    for finding in findings {
        if finding.severity != DocsSeverity::Error || finding.line == 0 {
            continue;
        }
        keys.insert((finding.path, finding.kind.key().to_string(), finding.advice));
    }
    keys
}

/// 从改法里取出被禁的那个词：「判定」→ 判断、看出。
fn banned_word(advice: &str) -> Option<&str> {
    let rest = advice.strip_prefix('「')?;
    let end = rest.find('」')?;
    Some(&rest[..end])
}

/// 专门清理的提交，信息里通常带这几个词；其余算改写顺手带走。
fn looks_targeted(message: &str) -> bool {
    ["术语", "用词", "说法", "语用", "命名"]
        .iter()
        .any(|word| message.contains(word))
}

fn repo_of<'a>(path: &str, repos: &'a [Repo]) -> Option<&'a Repo> {
    let mut best: Option<&Repo> = None;
    for repo in repos {
        let hit = repo.prefix.is_empty()
            || path == repo.prefix
            || path.starts_with(&format!("{}/", repo.prefix));
        if hit
            && best
                .map(|b| repo.prefix.len() > b.prefix.len())
                .unwrap_or(true)
        {
            best = Some(repo);
        }
    }
    best
}

/// 回溯一条消失的命中：取最近一次改动该词的提交，按提交信息分类。
fn classify(key: &Key, repos: &[Repo]) -> Option<bool> {
    let repo = repo_of(&key.0, repos)?;
    let relative = if repo.prefix.is_empty() {
        key.0.clone()
    } else {
        key.0
            .strip_prefix(&format!("{}/", repo.prefix))?
            .to_string()
    };
    let word = banned_word(&key.2)?;
    let log = git(
        &repo.dir,
        &[
            "--no-pager",
            "log",
            "-S",
            word,
            "--format=%s",
            "--",
            &relative,
        ],
    )
    .ok()?;
    let message = log.lines().next()?.trim();
    Some(looks_targeted(message))
}

pub fn run(
    roots: &[PathBuf],
    samples: usize,
    language: &LanguageContract,
    docs: &DocsContract,
    quiet: bool,
) -> Result<Report, String> {
    let mut report = Report::default();
    let mut seen: HashSet<Key> = HashSet::new();
    let mut alive: HashSet<Key> = HashSet::new();
    let mut alive_paths: HashSet<String> = HashSet::new();
    let mut all_repos: Vec<Repo> = Vec::new();

    for root in roots {
        if !root.is_dir() {
            continue;
        }
        let repos = discover(root);
        report.repos += repos.len();
        all_repos.extend(repos.iter().cloned());

        // 今天：根下一次扫完，路径与快照口径一致。
        let (findings, _, _) = scan_language(root, language, "default");
        for finding in findings {
            if finding.severity == LanguageSeverity::Error
                && !finding.verdict.is_false_positive()
                && finding.line != 0
            {
                report.today_errors += 1;
                alive_paths.insert(finding.path.clone());
                alive.insert((finding.path, finding.kind.key().to_string(), finding.advice));
            }
        }
        let (findings, _, _) = scan_docs(root, docs);
        for finding in findings {
            if finding.severity == DocsSeverity::Error && finding.line != 0 {
                report.today_errors += 1;
                alive_paths.insert(finding.path.clone());
                alive.insert((finding.path, finding.kind.key().to_string(), finding.advice));
            }
        }

        let workspace = temp_root();
        for repo in &repos {
            let commits = match pick_commits(&repo.dir, samples) {
                Ok(commits) => commits,
                Err(_) => continue,
            };
            let name = if repo.prefix.is_empty() {
                ".".to_string()
            } else {
                repo.prefix.clone()
            };
            if !quiet {
                eprintln!("巡检 {name}：{} 次提交", commits.len());
            }
            for (sha, _date) in &commits {
                let dest = match export(repo, sha, &workspace) {
                    Ok(dest) => dest,
                    Err(_) => continue,
                };
                report.snapshots += 1;
                seen.extend(keys(&workspace, language, docs));
                clear(&dest);
            }
        }
        let _ = fs::remove_dir_all(&workspace);
    }

    report.seen = seen.len();
    report.alive = seen.intersection(&alive).count();

    let mut gone: Vec<Key> = seen.difference(&alive).cloned().collect();
    gone.sort();
    for key in &gone {
        if alive_paths.contains(&key.0) {
            report.fixed += 1;
            if key.1 == "banned" {
                report.terminology_fixed += 1;
            }
        } else {
            report.file_gone += 1;
        }
    }
    report.terminology_gone = gone.iter().filter(|key| key.1 == "banned").count();

    // 抽检：只看留在留存文件里的术语，最多 40 条，够看出比例。
    let candidates: Vec<&Key> = gone
        .iter()
        .filter(|key| key.1 == "banned" && alive_paths.contains(&key.0))
        .take(40)
        .collect();
    for key in candidates {
        match classify(key, &all_repos) {
            Some(true) => {
                report.sampled += 1;
                report.targeted += 1;
            }
            Some(false) => {
                report.sampled += 1;
                report.incidental += 1;
            }
            None => {}
        }
    }

    Ok(report)
}

pub fn print_report(report: &Report) {
    println!("量潮创始人 · 存量巡检\n");
    println!(
        "重判了 {} 个仓、{} 次快照。\n",
        report.repos, report.snapshots
    );
    println!(
        "历史上有 {} 处命中进过库，今天还在 {} 处。",
        report.seen, report.alive
    );
    println!(
        "消失的 {} 处：{} 处随文件消失，{} 处在留存文件里被改掉。\n",
        report.file_gone + report.fixed,
        report.file_gone,
        report.fixed
    );
    if report.sampled > 0 {
        println!(
            "术语命中消失 {} 处，其中 {} 处改在留存文件里。抽检 {} 条：{} 条由术语清理提交改掉，{} 条随改写消失。",
            report.terminology_gone,
            report.terminology_fixed,
            report.sampled,
            report.targeted,
            report.incidental
        );
    }
    println!(
        "\n按条数算，今天的体检还有 {} 处真问题；上面按「同一文件同一规则」算，同一个词在多行命中只计一次。",
        report.today_errors
    );
}

pub fn render_json(report: &Report) {
    println!(
        "{{\n  \"repos\": {},\n  \"snapshots\": {},\n  \"seen\": {},\n  \"alive\": {},\n  \"file_gone\": {},\n  \"fixed\": {},\n  \"terminology_gone\": {},\n  \"terminology_fixed\": {},\n  \"sampled\": {},\n  \"targeted\": {},\n  \"incidental\": {},\n  \"today_errors\": {}\n}}",
        report.repos,
        report.snapshots,
        report.seen,
        report.alive,
        report.file_gone,
        report.fixed,
        report.terminology_gone,
        report.terminology_fixed,
        report.sampled,
        report.targeted,
        report.incidental,
        report.today_errors
    );
}

#[cfg(test)]
mod tests {
    use super::{Repo, banned_word, looks_targeted, repo_of};
    use std::path::PathBuf;

    #[test]
    fn takes_banned_word_from_advice() {
        assert_eq!(banned_word("「判定」→ 判断、看出"), Some("判定"));
        assert_eq!(banned_word("一行 2 处加粗（上限 1）"), None);
    }

    #[test]
    fn recognizes_targeted_cleanup() {
        assert!(looks_targeted("docs: 术语改按用户说法"));
        assert!(looks_targeted("docs(cli): 文档按自查结果修正术语与长句"));
        assert!(!looks_targeted("feat: 新增工作论文《人机协作的两项成本》"));
    }

    #[test]
    fn prefers_the_most_specific_repo() {
        let repos = vec![
            Repo {
                prefix: String::new(),
                dir: PathBuf::from("/tmp/root"),
            },
            Repo {
                prefix: "docs/bylaw".to_string(),
                dir: PathBuf::from("/tmp/root/docs/bylaw"),
            },
        ];
        let repo = repo_of("docs/bylaw/intro/index.md", &repos).unwrap();
        assert_eq!(repo.prefix, "docs/bylaw");
        let repo = repo_of("CONTRIBUTING.md", &repos).unwrap();
        assert_eq!(repo.prefix, "");
    }
}
