//! 资产维护：把 context 整理成「领域 / 资产」，再按关系搬到长期记忆仓。
//!
//! context 与 archive 是「领域 / 资产」两级；长期记忆是资产一级仓库、领域一级文件夹。
//! 移动时交换前两段：`context/<领域>/<资产>/<余下>` → `<资产仓>/<领域>/<余下>`。
//! `--check` 只整理与校验，`--dry-run` 预览，默认移动。

use std::fs;
use std::path::{Path, PathBuf};

use clap::Args;
use serde_json::json;
use walkdir::WalkDir;

use crate::meta::contract::{Asset, Contract};

const SCAFFOLD: [&str; 7] = [
    "README.md",
    "CHANGELOG.md",
    "AGENTS.md",
    "CONTRIBUTING.md",
    "LICENSE",
    "myst.yml",
    ".gitignore",
];

#[derive(Args, Debug)]
pub struct MaintainArgs {
    /// 仓根；context 与资产的相对路径按它算
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    /// 只整理与校验，不移动
    #[arg(long)]
    pub check: bool,
    /// 预览要移动的文件，不落盘
    #[arg(long)]
    pub dry_run: bool,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Default)]
pub struct Plan {
    pub moves: Vec<Step>,
    pub conflicts: Vec<String>,
    pub unclassified: Vec<String>,
    pub today: Vec<String>,
}

#[derive(Debug)]
pub struct Step {
    pub from: PathBuf,
    pub to: PathBuf,
    pub asset: String,
    pub asset_label: String,
    pub category: String,
    pub domain: String,
}

pub fn run(args: &MaintainArgs, rules: Option<&Path>) -> Result<i32, String> {
    let contract = Contract::load(rules)?;
    let plan = plan(&contract, &args.root)?;
    let nested = nested_assets(&contract, &args.root);
    let moving = !args.check && !args.dry_run;
    if moving {
        execute(&plan)?;
    }

    if args.json {
        print_json(args, &plan, &nested, moving);
    } else {
        print_text(args, &plan, &nested, moving);
    }

    let issues = !plan.conflicts.is_empty() || !plan.unclassified.is_empty() || !nested.is_empty();
    let pending = args.check && !plan.moves.is_empty();
    Ok(if issues || pending { 1 } else { 0 })
}

/// 扫 context，算出要移动的文件。只算，不动。
pub fn plan(contract: &Contract, root: &Path) -> Result<Plan, String> {
    plan_on(contract, root, &today())
}

/// 指定「今天」的日期，供测试固定时间。
pub fn plan_on(contract: &Contract, root: &Path, today: &str) -> Result<Plan, String> {
    let source = contract
        .source()
        .ok_or_else(|| "契约缺少 source 资产：context".to_string())?;
    contract
        .exit()
        .ok_or_else(|| "契约缺少 exit 资产：archive".to_string())?;
    if source.layout != "domain-asset" {
        return Err(format!(
            "{} 的 layout 应为 domain-asset，实为 {}",
            source.name, source.layout
        ));
    }
    let intake = contract
        .relation
        .iter()
        .find(|relation| relation.kind == "intake" && relation.from == source.name)
        .ok_or_else(|| format!("契约缺少 {} → memory 的 intake 关系", source.name))?;
    if intake.to != "memory" {
        return Err(format!(
            "{} → memory 的 intake 目标应为 memory，实为 {}",
            source.name, intake.to
        ));
    }
    let flip = intake.transform == "flip";
    if !flip {
        return Err(format!(
            "{} → memory 的 transform 应为 flip，实为 {}",
            source.name, intake.transform
        ));
    }
    let context = root.join(&source.path);
    let memories = contract.memory();
    let mut plan = Plan::default();
    if !context.exists() {
        return Ok(plan);
    }

    for entry in WalkDir::new(&context).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = path.strip_prefix(&context).unwrap_or(path);
        if rel.components().any(|part| part.as_os_str() == ".git") {
            continue;
        }
        let segments = segments(rel);
        if segments.len() < 3 {
            let name = segments.last().map(String::as_str).unwrap_or("");
            if !SCAFFOLD.contains(&name) && name != ".gitignore" {
                plan.unclassified.push(rel.display().to_string());
            }
            continue;
        }
        let Some((domain, asset, rest)) = classify(&segments, &memories) else {
            plan.unclassified.push(rel.display().to_string());
            continue;
        };
        if rest.len() == 1 && SCAFFOLD.contains(&rest[0].as_str()) {
            plan.unclassified.push(rel.display().to_string());
            continue;
        }
        if is_today(&rest, today) {
            plan.today.push(rel.display().to_string());
            continue;
        }
        let mut target = root.join(&asset.path).join(&domain);
        for part in &rest {
            target.push(part);
        }
        if target.exists() {
            plan.conflicts
                .push(format!("{} -> {}", rel.display(), target.display()));
            continue;
        }
        plan.moves.push(Step {
            from: path.to_path_buf(),
            to: target,
            asset: asset.name.clone(),
            asset_label: asset.label().to_string(),
            category: asset.category.clone(),
            domain,
        });
    }

    Ok(plan)
}

/// 前两段是「领域 / 资产」还是「资产 / 领域」，两种都认；资产套资产不算。
fn classify<'a>(
    segments: &[String],
    memories: &'a [&'a Asset],
) -> Option<(String, &'a Asset, Vec<String>)> {
    let first = memory_named(memories, &segments[0]);
    let second = memory_named(memories, &segments[1]);

    match (first, second) {
        (Some(asset), None) => Some((segments[1].clone(), asset, segments[2..].to_vec())),
        (None, Some(asset)) => Some((segments[0].clone(), asset, segments[2..].to_vec())),
        _ => None,
    }
}

fn memory_named<'a>(memories: &'a [&'a Asset], name: &str) -> Option<&'a Asset> {
    memories.iter().copied().find(|asset| asset.name == name)
}

fn segments(rel: &Path) -> Vec<String> {
    rel.components()
        .filter_map(|part| part.as_os_str().to_str().map(str::to_string))
        .collect()
}

fn execute(plan: &Plan) -> Result<(), String> {
    for step in &plan.moves {
        move_file(&step.from, &step.to)?;
    }
    Ok(())
}

/// `rest` 是单个今天的日期文件时留在 context——当天还要继续录入。
fn is_today(rest: &[String], today: &str) -> bool {
    rest.len() == 1 && rest[0].trim_end_matches(".md") == today
}

/// 今天的日期，`YYYY-MM-DD`（UTC）。不引依赖，用 Unix 纪元天数换算。
fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    civil_from_days((secs / 86_400) as i64)
}

/// Hinnant 的 `civil_from_days`：纪元天数 → 年月日。
fn civil_from_days(days: i64) -> String {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 {
        yoe + era * 400 + 1
    } else {
        yoe + era * 400
    };
    format!("{year:04}-{month:02}-{day:02}")
}

fn move_file(from: &Path, to: &Path) -> Result<(), String> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("建不了目录 {}：{error}", parent.display()))?;
    }
    if fs::rename(from, to).is_ok() {
        return Ok(());
    }
    fs::copy(from, to)
        .map_err(|error| format!("搬不动 {} -> {}：{error}", from.display(), to.display()))?;
    fs::remove_file(from).map_err(|error| format!("删不掉 {}：{error}", from.display()))?;
    Ok(())
}

/// 长期记忆的顶层应当是领域；顶层出现资产名，说明还没做二次整理。
fn nested_assets(contract: &Contract, root: &Path) -> Vec<String> {
    let memories = contract.memory();
    let names: Vec<&str> = memories.iter().map(|asset| asset.name.as_str()).collect();
    let mut nested = Vec::new();
    for asset in memories {
        let dir = root.join(&asset.path);
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let is_dir = entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false);
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if is_dir && names.contains(&name.as_str()) {
                nested.push(format!("{}{}/ 是资产名，不是领域", asset.path, name));
            }
        }
    }
    nested
}

fn print_text(args: &MaintainArgs, plan: &Plan, nested: &[String], moving: bool) {
    println!("根：{}", args.root.display());
    if args.dry_run {
        println!("预览，不落盘");
    }
    if args.check {
        println!("只整理与校验，不移动");
    }

    let label = if moving { "已移动" } else { "待移动" };
    println!("{label} {}", plan.moves.len());
    for step in &plan.moves {
        let from = step.from.display();
        let to = step.to.display();
        println!("  {from} -> {to}");
    }

    println!("当天不移动 {}", plan.today.len());
    for path in &plan.today {
        println!("  {path}");
    }

    println!("目标已存在 {}", plan.conflicts.len());
    for conflict in &plan.conflicts {
        println!("  {conflict}");
    }

    println!("无法归位 {}", plan.unclassified.len());
    for path in &plan.unclassified {
        println!("  {path}");
    }

    println!("目标布局异常 {}", nested.len());
    for line in nested {
        println!("  {line}");
    }
}

fn print_json(args: &MaintainArgs, plan: &Plan, nested: &[String], moving: bool) {
    let steps: Vec<serde_json::Value> = plan
        .moves
        .iter()
        .map(|step| {
            json!({
                "from": step.from.display().to_string(),
                "to": step.to.display().to_string(),
                "asset": step.asset,
                "asset_label": step.asset_label,
                "category": step.category,
                "domain": step.domain,
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "root": args.root.display().to_string(),
            "moved": moving,
            "dry_run": args.dry_run,
            "check": args.check,
            "steps": steps,
            "today": plan.today,
            "conflicts": plan.conflicts,
            "unclassified": plan.unclassified,
            "nested": nested,
        }))
        .unwrap_or_else(|_| "{}".to_string())
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("data/context/life/insight")).unwrap();
        fs::create_dir_all(root.join("data/context/handbook/life")).unwrap();
        fs::create_dir_all(root.join("data/context/default")).unwrap();
        fs::create_dir_all(root.join("data/context/work/journal")).unwrap();
        fs::create_dir_all(root.join("data/insight")).unwrap();
        fs::create_dir_all(root.join("docs/handbook")).unwrap();
        fs::create_dir_all(root.join("data/journal")).unwrap();
        fs::write(root.join("data/context/life/insight/a.md"), "甲").unwrap();
        fs::write(root.join("data/context/handbook/life/b.md"), "乙").unwrap();
        fs::write(root.join("data/context/default/README.md"), "说明").unwrap();
        fs::write(root.join("data/context/work/journal/2026-10-27.md"), "丙").unwrap();
        dir
    }

    #[test]
    fn 领域在前与资产在前都认() {
        let dir = fixture();
        let contract = Contract::bundled();
        let plan = plan(&contract, dir.path()).unwrap();
        let targets: Vec<String> = plan
            .moves
            .iter()
            .map(|step| step.to.display().to_string())
            .collect();
        assert!(
            targets
                .iter()
                .any(|path| path.ends_with("data/insight/life/a.md"))
        );
        assert!(
            targets
                .iter()
                .any(|path| path.ends_with("docs/handbook/life/b.md"))
        );
        assert!(
            targets
                .iter()
                .any(|path| path.ends_with("data/journal/work/2026-10-27.md"))
        );
        assert!(
            !plan
                .unclassified
                .iter()
                .any(|path| path.ends_with("default/README.md"))
        );
    }

    #[test]
    fn 默认移动且不覆盖() {
        let dir = fixture();
        let contract = Contract::bundled();
        let plan = plan(&contract, dir.path()).unwrap();
        execute(&plan).unwrap();
        assert!(dir.path().join("data/insight/life/a.md").exists());
        assert!(dir.path().join("docs/handbook/life/b.md").exists());
        assert!(dir.path().join("data/journal/work/2026-10-27.md").exists());
        assert!(!dir.path().join("data/context/life/insight/a.md").exists());
    }

    #[test]
    fn 目标已存在时报冲突() {
        let dir = fixture();
        fs::create_dir_all(dir.path().join("data/insight/life")).unwrap();
        fs::write(dir.path().join("data/insight/life/a.md"), "旧").unwrap();
        let contract = Contract::bundled();
        let plan = plan(&contract, dir.path()).unwrap();
        assert!(plan.conflicts.iter().any(|line| line.contains("a.md")));
    }

    #[test]
    fn 当天的日志不移动() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("data/context/default/journal")).unwrap();
        fs::write(
            root.join("data/context/default/journal/2026-10-30.md"),
            "今",
        )
        .unwrap();
        fs::write(
            root.join("data/context/default/journal/2026-10-29.md"),
            "昨",
        )
        .unwrap();
        let contract = Contract::bundled();
        let plan = plan_on(&contract, root, "2026-10-30").unwrap();
        assert!(
            plan.today
                .iter()
                .any(|path| path.ends_with("2026-10-30.md"))
        );
        assert!(
            !plan
                .moves
                .iter()
                .any(|step| step.to.ends_with("2026-10-30.md"))
        );
        assert!(
            plan.moves
                .iter()
                .any(|step| step.to.ends_with("data/journal/default/2026-10-29.md"))
        );
    }

    #[test]
    fn 日期换算对得上() {
        assert_eq!(civil_from_days(0), "1970-01-01");
        assert_eq!(civil_from_days(19_723), "2024-01-01");
    }
}
