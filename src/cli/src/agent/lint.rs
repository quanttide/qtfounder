//! `agent skill lint`：检查 Skill 的命名与写法是否合标准。
//!
//! 这套标准还在试：名字怎么起、说明写到什么程度，都还没定死。
//! 所以判断层只读 `default-contract.toml`，改口径改文件。
//!
//! 判的是「Agent 能不能靠名字和说明找对这份 Skill」，
//! 不是文章写得好不好——文字层面归 `language`。

use std::path::{Path, PathBuf};

use clap::Args;
use walkdir::WalkDir;

use super::contract::Contract;

#[derive(Args, Debug)]
pub struct LintArgs {
    /// 要查的目录；会往下找所有 SKILL.md
    #[arg(default_value = ".")]
    pub root: PathBuf,
    /// 契约路径（缺省按 src/agent/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 最多列几个文件
    #[arg(long, default_value_t = 15)]
    pub top: usize,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

/// 一处待改。错误的行号取 frontmatter 里的那一行；没有行的写在文件级。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub path: String,
    pub line: Option<usize>,
    pub severity: Severity,
    pub kind: &'static str,
    pub message: String,
    pub advice: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Suspect,
}

impl Severity {
    fn key(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Suspect => "suspect",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Error => "错误",
            Self::Suspect => "嫌疑",
        }
    }
}

pub fn run(args: &LintArgs) -> Result<i32, String> {
    let contract = Contract::load(args.rules.as_deref())?;
    if !args.root.exists() {
        return Err(format!("目录不存在：{}", args.root.display()));
    }

    let skills = scan(&args.root, &contract)?;
    let mut findings = Vec::new();
    for skill in &skills {
        findings.extend(inspect(skill, &contract));
    }
    findings.extend(duplicates(&skills));
    findings.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.line.unwrap_or(0).cmp(&b.line.unwrap_or(0)))
    });

    if args.json {
        print_json(&findings, skills.len());
    } else {
        print_report(&findings, skills.len(), args.top);
    }
    Ok(i32::from(!findings.is_empty()))
}

/// 一个 SKILL.md 的解析结果。字段留原始行号，报错时指得到。
#[derive(Debug, Clone)]
pub(super) struct SkillFile {
    pub(super) path: PathBuf,
    pub(super) dir_name: String,
    pub(super) name: Option<String>,
    pub(super) name_line: Option<usize>,
    pub(super) description: Option<String>,
    pub(super) description_line: Option<usize>,
    pub(super) body: String,
}

pub(super) fn scan(root: &Path, contract: &Contract) -> Result<Vec<SkillFile>, String> {
    let mut skills = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_entry(|entry| {
        !contract
            .scope
            .exclude
            .iter()
            .any(|name| entry.file_name() == name.as_str())
    }) {
        let entry = entry.map_err(|error| format!("扫描失败：{error}"))?;
        if !entry.file_type().is_file() || entry.file_name() != "SKILL.md" {
            continue;
        }
        let text = std::fs::read_to_string(entry.path())
            .map_err(|error| format!("读不了 {}：{error}", entry.path().display()))?;
        skills.push(parse(entry.path(), &text));
    }
    skills.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(skills)
}

pub(super) fn parse(path: &Path, text: &str) -> SkillFile {
    let dir_name = path
        .parent()
        .and_then(|dir| dir.file_name())
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut lines = text.lines().enumerate();
    let Some((_, first)) = lines.next() else {
        return SkillFile {
            path: path.to_path_buf(),
            dir_name,
            name: None,
            name_line: None,
            description: None,
            description_line: None,
            body: String::new(),
        };
    };
    if first.trim() != "---" {
        return SkillFile {
            path: path.to_path_buf(),
            dir_name,
            name: None,
            name_line: None,
            description: None,
            description_line: None,
            body: text.to_string(),
        };
    }

    let mut name = None;
    let mut name_line = None;
    let mut description = None;
    let mut description_line = None;
    let mut body_start = text.lines().count() + 1;
    let mut collecting_description = false;

    for (index, line) in lines {
        let line_number = index + 1;
        if line.trim() == "---" {
            body_start = line_number + 1;
            break;
        }
        let trimmed = line.trim_start();
        if collecting_description {
            let indented = line.starts_with(' ') || line.starts_with('\t');
            if indented {
                let text = trimmed.trim_matches('>').trim();
                if !text.is_empty() {
                    let entry = description.get_or_insert_with(String::new);
                    if !entry.is_empty() {
                        entry.push(' ');
                    }
                    entry.push_str(text);
                }
                continue;
            }
            collecting_description = false;
        }
        if let Some(value) = trimmed.strip_prefix("name:") {
            name = Some(value.trim().to_string());
            name_line = Some(line_number);
        } else if let Some(value) = trimmed.strip_prefix("description:") {
            let value = value.trim();
            if value == ">" || value == "|" {
                collecting_description = true;
                description_line = Some(line_number);
                description = Some(String::new());
            } else {
                description = Some(value.to_string());
                description_line = Some(line_number);
            }
        }
    }

    let body: String = text
        .lines()
        .skip(body_start.saturating_sub(1))
        .collect::<Vec<&str>>()
        .join("\n");

    SkillFile {
        path: path.to_path_buf(),
        dir_name,
        name,
        name_line,
        description,
        description_line,
        body,
    }
}

pub(super) fn inspect(skill: &SkillFile, contract: &Contract) -> Vec<Finding> {
    let mut findings = Vec::new();
    let at = |line: Option<usize>, severity, kind, message: String, advice: &str| Finding {
        path: skill.path.display().to_string(),
        line,
        severity,
        kind,
        message,
        advice: advice.to_string(),
    };

    let Some(name) = &skill.name else {
        findings.push(at(
            None,
            Severity::Error,
            "name",
            "frontmatter 里没有 name".to_string(),
            "加 `name: <范围>-<动词>`，与目录名一致",
        ));
        return findings;
    };

    if name != &skill.dir_name {
        findings.push(at(
            skill.name_line,
            Severity::Error,
            "name",
            format!(
                "名字与目录对不上：目录是「{}」，frontmatter 是「{}」",
                skill.dir_name, name
            ),
            "改目录名或改 frontmatter，两处必须一致",
        ));
    }

    if !is_kebab(name) {
        findings.push(at(
            skill.name_line,
            Severity::Error,
            "name",
            format!("名字不是小写连字符：「{name}」"),
            "只用小写字母、数字和连字符，例如 git-commit",
        ));
    }

    if name.chars().count() > contract.name.max_chars {
        findings.push(at(
            skill.name_line,
            Severity::Error,
            "name",
            format!(
                "名字太长：{} 字，上限 {}",
                name.chars().count(),
                contract.name.max_chars
            ),
            "名字是给人记的，缩到两个词",
        ));
    }

    let segments: Vec<&str> = name.split('-').collect();
    if segments.len() != contract.name.segments {
        findings.push(at(
            skill.name_line,
            Severity::Suspect,
            "name",
            format!(
                "{} 段：习惯是 {} 段（范围-动词）",
                segments.len(),
                contract.name.segments
            ),
            "范围是命令入口或业务域，第二段写动词",
        ));
    }

    let scope = segments.first().copied().unwrap_or_default();
    if !contract.name.scopes.iter().any(|known| known == scope) {
        findings.push(at(
            skill.name_line,
            Severity::Suspect,
            "scope",
            format!("范围「{scope}」没有登记"),
            "新范围先写进契约的 scopes，再统一命名",
        ));
    }

    if let Some(last) = segments.last()
        && let Some(suffix) = contract
            .name
            .noun_suffixes
            .iter()
            .find(|suffix| last.ends_with(suffix.as_str()))
    {
        findings.push(at(
            skill.name_line,
            Severity::Error,
            "verb",
            format!("第二段以名词后缀「-{suffix}」收尾：「{last}」"),
            "换成动词原形，读名字就知道做什么",
        ));
    }

    match &skill.description {
        None => findings.push(at(
            skill.description_line,
            Severity::Error,
            "description",
            "没有 description".to_string(),
            "写清做什么、什么时候用",
        )),
        Some(text) => {
            if text.chars().count() > contract.description.max_chars {
                findings.push(at(
                    skill.description_line,
                    Severity::Error,
                    "description",
                    format!(
                        "说明太长：{} 字，上限 {}",
                        text.chars().count(),
                        contract.description.max_chars
                    ),
                    "客户端会截断，超出部分读者看不到",
                ));
            }
            if !contract
                .description
                .trigger_words
                .iter()
                .any(|word| text.contains(word))
            {
                findings.push(at(
                    skill.description_line,
                    Severity::Suspect,
                    "description",
                    "说明里没有写什么时候用".to_string(),
                    "补一句「当……时使用」，让 Agent 判得准",
                ));
            }
        }
    }

    if contract.name.cli_scopes.iter().any(|known| known == scope) {
        let command = format!("qtfounder {scope}");
        if !skill.body.contains(&command) {
            findings.push(at(
                None,
                Severity::Suspect,
                "command",
                format!("名字指向 `{command}`，正文里却没有这条命令"),
                "补上命令示例，或把范围改成它真正对应的入口",
            ));
        }
    }

    if contract.body.require_heading
        && !skill
            .body
            .lines()
            .any(|line| line.trim_start().starts_with("# "))
    {
        findings.push(at(
            None,
            Severity::Suspect,
            "heading",
            "正文没有一级标题".to_string(),
            "加一行 `# 标题`，打开就知道这是什么",
        ));
    }

    findings
}

pub(super) fn duplicates(skills: &[SkillFile]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, skill) in skills.iter().enumerate() {
        let Some(name) = &skill.name else {
            continue;
        };
        let repeated = skills[..index]
            .iter()
            .any(|earlier| earlier.name.as_deref() == Some(name.as_str()));
        if repeated {
            findings.push(Finding {
                path: skill.path.display().to_string(),
                line: skill.name_line,
                severity: Severity::Error,
                kind: "unique",
                message: format!("名字「{name}」重复出现"),
                advice: "同一个名字只能有一份 Skill，改名或删掉多余的".to_string(),
            });
        }
    }
    findings
}

fn is_kebab(name: &str) -> bool {
    let valid = |ch: char| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-';
    !name.is_empty()
        && name.chars().all(valid)
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
}

// ---------- 输出 ----------

fn print_report(findings: &[Finding], scanned: usize, top: usize) {
    if findings.is_empty() {
        println!("审了 {scanned} 个 Skill，没有命中：命名、说明与正文都合标准");
        return;
    }
    println!("审了 {scanned} 个 Skill，{} 处待改\n", findings.len());
    for finding in findings.iter().take(top * 8) {
        let line = finding
            .line
            .map(|line| format!("L{line}  "))
            .unwrap_or_default();
        println!(
            "{}  {line}[{}] {}  {}",
            finding.path,
            finding.severity.label(),
            finding.kind,
            finding.message
        );
        println!("          → {}", finding.advice);
    }
    let errors = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count();
    println!(
        "\n合计 {} 处（错误 {errors}，嫌疑 {}）",
        findings.len(),
        findings.len() - errors
    );
    println!("标准还在试：报的是嫌疑不是判决，读上下文再改。");
}

fn print_json(findings: &[Finding], scanned: usize) {
    let items: Vec<serde_json::Value> = findings
        .iter()
        .map(|finding| {
            serde_json::json!({
                "path": finding.path,
                "line": finding.line,
                "severity": finding.severity.key(),
                "kind": finding.kind,
                "message": finding.message,
                "advice": finding.advice,
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::json!({"scanned": scanned, "findings": items})
    );
}
