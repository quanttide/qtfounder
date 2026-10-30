//! `docs check`：技能索引与实际目录对不对得上。
//!
//! 索引指 `AGENTS.md` 与 `CONTRIBUTING.md` 里写的 `.agents/skills/<name>`。
//! 只比对名字：缺索引、失效链接两类，正文怎么写不管。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub fn run_check(root: &Path) -> Result<i32, String> {
    if !root.is_dir() {
        return Err(format!(
            "docs check 要给一个目录：{} 不是目录",
            root.display()
        ));
    }
    let skills = find_skills_dir(root);
    let mut on_disk = BTreeSet::new();
    if let Some((dir, _)) = &skills {
        for entry in
            std::fs::read_dir(dir).map_err(|error| format!("读不了 {}：{error}", dir.display()))?
        {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            if path.is_dir()
                && path.join("SKILL.md").is_file()
                && let Some(name) = path.file_name().and_then(|name| name.to_str())
            {
                on_disk.insert(name.to_string());
            }
        }
    }

    let mut referenced = BTreeSet::new();
    let mut read = Vec::new();
    for name in ["AGENTS.md", "CONTRIBUTING.md"] {
        let path = root.join(name);
        if !path.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("读不了 {}：{error}", path.display()))?;
        referenced.extend(names_in(&text));
        read.push(name);
    }

    if read.is_empty() && skills.is_none() {
        return Err(format!(
            "{} 里既没有 AGENTS.md / CONTRIBUTING.md，也没有 .agents/skills",
            root.display()
        ));
    }

    let missing_index: Vec<&String> = if skills.as_ref().is_some_and(|(_, under_root)| *under_root)
    {
        on_disk.difference(&referenced).collect()
    } else {
        // 技能目录在祖先仓：索引归祖先仓管，本仓只负责自己的引用能对上
        Vec::new()
    };
    let dangling: Vec<&String> = referenced.difference(&on_disk).collect();

    println!(
        "文档索引：读 {}，技能目录 {} 个（{}），正文引用 {} 个",
        if read.is_empty() {
            "（无 AGENTS.md / CONTRIBUTING.md）".to_string()
        } else {
            read.join("、")
        },
        on_disk.len(),
        match &skills {
            Some((dir, true)) => format!("{}", dir.display()),
            Some((dir, false)) => format!("{}，在祖先仓", dir.display()),
            None => "未找到".to_string(),
        },
        referenced.len()
    );
    if missing_index.is_empty() && dangling.is_empty() {
        match &skills {
            Some((_, false)) => println!("本仓引用都能对上（索引归祖先仓）"),
            _ => println!("索引与目录一致"),
        }
        return Ok(0);
    }
    if !missing_index.is_empty() {
        println!("\n目录里有、索引里没写 {} 个：", missing_index.len());
        for name in &missing_index {
            println!("  {name}：补进 AGENTS.md 或 CONTRIBUTING.md");
        }
    }
    if !dangling.is_empty() {
        println!("\n索引里有、目录里没有 {} 个：", dangling.len());
        for name in &dangling {
            println!("  {name}：删掉引用，或把技能补回 .agents/skills/");
        }
    }
    Ok(1)
}

/// 找 `.agents/skills`：先看给定目录，再最多上溯三层（从子目录跑也能找到）。
/// 第二个返回值表示它是否就在给定目录之下——在祖先仓时不查索引缺漏。
fn find_skills_dir(root: &Path) -> Option<(PathBuf, bool)> {
    for (depth, dir) in crate::core::path::upwards(root, Some(4)).into_iter().enumerate() {
        let candidate = dir.join(".agents/skills");
        if candidate.is_dir() {
            return Some((candidate, depth == 0));
        }
    }
    None
}

/// 正文里出现的技能名：`.agents/skills/<name>` 后面的第一段名字。
fn names_in(text: &str) -> BTreeSet<String> {
    const PREFIX: &str = ".agents/skills/";
    let mut out = BTreeSet::new();
    let mut rest = text;
    while let Some(at) = rest.find(PREFIX) {
        rest = &rest[at + PREFIX.len()..];
        let name: String = rest
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
            .collect();
        if !name.is_empty() {
            out.insert(name);
        }
    }
    out
}
