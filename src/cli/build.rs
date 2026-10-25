//! 把本包的 `skills/` 嵌入二进制。
//!
//! 来源按这个顺序取：`QTFOUNDER_SKILLS_DIR` → 包内的 `skills/`。
//! 找不到就生成空清单，构建不因此失败。
//! 生成的是 `include_str!` 声明，正文在编译期进二进制，装的时候不读源码树。

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

struct Entry {
    id: String,
    skill_md: PathBuf,
    auxiliary: Vec<(String, PathBuf)>,
}

fn main() {
    println!("cargo:rerun-if-env-changed=QTFOUNDER_SKILLS_DIR");

    let manifest =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("cargo 会设 CARGO_MANIFEST_DIR"));
    let source = std::env::var_os("QTFOUNDER_SKILLS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest.join("skills"));
    let source = std::fs::canonicalize(&source).unwrap_or(source);

    let entries = collect(&source);
    let code = render(&entries, &source);
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo 会设 OUT_DIR"));
    std::fs::write(out.join("embedded_skills.rs"), code).expect("写得进 OUT_DIR");
}

/// 一个目录一条 Skill：有 `SKILL.md` 才算，其余文件都当附属文件。
fn collect(source: &Path) -> Vec<Entry> {
    if !source.is_dir() {
        return Vec::new();
    }
    println!("cargo:rerun-if-changed={}", source.display());

    let mut skills: Vec<PathBuf> = std::fs::read_dir(source)
        .into_iter()
        .flatten()
        .flatten()
        .map(|item| item.path())
        .filter(|path| path.join("SKILL.md").is_file())
        .collect();
    skills.sort();

    skills
        .into_iter()
        .filter_map(|dir| {
            let id = dir.file_name()?.to_string_lossy().to_string();
            let mut auxiliary = Vec::new();
            walk(&dir, &dir, &mut auxiliary);
            auxiliary.sort();
            println!("cargo:rerun-if-changed={}", dir.display());
            Some(Entry {
                id,
                skill_md: dir.join("SKILL.md"),
                auxiliary,
            })
        })
        .collect()
}

fn walk(root: &Path, dir: &Path, found: &mut Vec<(String, PathBuf)>) {
    let Ok(items) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = items.flatten().map(|item| item.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            walk(root, &path, found);
            continue;
        }
        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };
        if relative == Path::new("SKILL.md") {
            continue;
        }
        // include_str! 只吃 UTF-8；二进制附属文件先不进二进制。
        if std::fs::read_to_string(&path).is_err() {
            println!("cargo:warning=跳过非文本附属文件 {}", path.display());
            continue;
        }
        found.push((relative.to_string_lossy().replace('\\', "/"), path));
    }
}

fn render(entries: &[Entry], source: &Path) -> String {
    let mut code = String::new();
    if entries.is_empty() {
        code.push_str("pub const SOURCE: Option<&str> = None;\n");
        code.push_str("pub const SKILLS: &[EmbeddedSkill] = &[];\n");
        return code;
    }

    writeln!(
        code,
        "pub const SOURCE: Option<&str> = Some({:?});",
        source.to_string_lossy()
    )
    .expect("写 String 不会失败");
    code.push_str("pub const SKILLS: &[EmbeddedSkill] = &[\n");
    for entry in entries {
        writeln!(code, "    EmbeddedSkill {{").expect("写 String 不会失败");
        writeln!(code, "        id: {:?},", entry.id).expect("写 String 不会失败");
        writeln!(
            code,
            "        skill_md: include_str!({:?}),",
            entry.skill_md.to_string_lossy()
        )
        .expect("写 String 不会失败");
        if entry.auxiliary.is_empty() {
            code.push_str("        auxiliary: &[],\n");
        } else {
            code.push_str("        auxiliary: &[\n");
            for (relative, path) in &entry.auxiliary {
                writeln!(
                    code,
                    "            ({:?}, include_str!({:?})),",
                    relative,
                    path.to_string_lossy()
                )
                .expect("写 String 不会失败");
            }
            code.push_str("        ],\n");
        }
        code.push_str("    },\n");
    }
    code.push_str("];\n");
    code
}
