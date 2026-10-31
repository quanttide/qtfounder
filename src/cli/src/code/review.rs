//! `qtfounder code review`：功能审查的骨架与纪律。
//!
//! 审查看的是**功能还对不对得上**，不是代码读起来顺不顺（那是 `language` 的四层）。
//! 正本是 Skill `code-review`，这份命令只做两件事：
//!
//! 1. 出报告骨架，四节固定，让人与 AI 填同一份形状；
//! 2. 从事实面（[`Facts`]）取**功能锚点**贴进「清单」——函数清单、复杂度过高、
//!    恒假条件、错误处理，每条自带 `file:line`，正是报告要求的证据格式。
//!
//! **锚点不排序、不标严重度**：严重度得对着功能预期判，那是判断，归写报告的人。
//! 命令只递事实。功能假设四节命令一个字不填——推预期是判断，不机械。
//! 审完把人的使用体会记下来，落在共用的体会存储里。

use std::io::Read as _;
use std::path::{Path, PathBuf};

use clap::Args;

use crate::code::extract;
use crate::code::ir::Facts;
use crate::code::rules::{ReviewRule, Rules};
use crate::core::exit;
use crate::review::{Experience, Store, now_iso8601};

/// 功能审查：出报告骨架，把事实面的功能锚点贴进去；审完把人的体会记下来。
#[derive(Args, Debug)]
pub struct ReviewArgs {
    /// 要审的目录或文件
    #[arg(default_value = ".")]
    pub root: PathBuf,
    /// 连纪律的 example 一起打——要不要例子按需，缺省省掉
    #[arg(long)]
    pub example: bool,
    /// 记一条使用体会：正文从标准输入读，配合 --user 指定是谁的体会
    #[arg(long)]
    pub record: bool,
    /// 体会属于谁；记一条时必给，多个用户各记各的
    #[arg(long)]
    pub user: Option<String>,
    /// 列出这个目标已记的使用体会，不出报告
    #[arg(long)]
    pub experiences: bool,
}

/// 功能审查的四问。推功能预期就按这四问——判的是「它该做什么」，不是「它怎么写」。
pub(super) const FUNCTIONAL_QUESTIONS: [&str; 4] = [
    "这段代码要解决的问题是什么，一句话说清",
    "对外承诺的行为是什么：输入什么、吐出什么、失败时怎样",
    "边界与错误路径覆盖了没：空、错、并发、规模上去之后",
    "哪些地方还没验证：没有测试、没有断言、没有真实调用走过",
];

/// 一处功能锚点：从事实面取来的、带位置的事实。
struct Anchor {
    path: String,
    line: usize,
    kind: &'static str,
    message: String,
}

pub fn run(
    args: &ReviewArgs,
    rules_path: Option<&std::path::Path>,
    json: bool,
) -> Result<i32, String> {
    if args.record {
        return record_experience(args);
    }
    if args.experiences {
        return list_experiences(args, json);
    }

    let rules = Rules::load(rules_path)?;
    if rules.review.is_empty() {
        return Err(
            "契约里没有 [[review]]：在 default-contract.toml 末尾补一段审查规则再跑".to_string(),
        );
    }
    if !args.root.exists() {
        eprintln!("目标不在：{}", args.root.display());
        return Ok(exit::NOT_FOUND);
    }

    let (anchors, broken) = collect_anchors(&args.root, &rules);
    for (path, why) in &broken {
        eprintln!("解析失败：{path} —— {why}");
    }

    let experiences = load_experiences(&args.root);
    if json {
        render_json(&rules.review, &anchors, &args.root, &experiences);
        return Ok(if anchors.is_empty() {
            exit::OK
        } else {
            exit::FINDINGS
        });
    }

    render_text(
        &rules.review,
        &anchors,
        &args.root,
        args.example,
        &experiences,
    );
    Ok(if anchors.is_empty() {
        exit::OK
    } else {
        exit::FINDINGS
    })
}

/// 走一遍目标，从事实面取功能锚点。返回（锚点，解析失败的文件）。
///
/// 锚点只取功能相关的四种：函数清单、复杂度过高、恒假条件、错误信息。命名与读感
/// 那些语言学的活归 `language`，不在这儿。
fn collect_anchors(root: &Path, rules: &Rules) -> (Vec<Anchor>, Vec<(String, String)>) {
    let mut anchors = Vec::new();
    let mut broken = Vec::new();

    for path in source_files(root, &rules.scope.exclude) {
        let label = display_path(root, &path);
        let source = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                broken.push((label, error.to_string()));
                continue;
            }
        };
        let facts = match extract::extract(&source, &path) {
            Some(Ok(facts)) => facts,
            Some(Err(error)) => {
                broken.push((label, error));
                continue;
            }
            None => continue,
        };
        anchors.extend(functional_anchors(&facts, &label, rules));
    }
    (anchors, broken)
}

/// 从一个文件的事实面取功能锚点。
fn functional_anchors(facts: &Facts, path: &str, rules: &Rules) -> Vec<Anchor> {
    let mut out = Vec::new();

    // 函数清单：每个函数一行，带参数与行数——审查要对着清单问「这事归它管吗」。
    for function in &facts.functions {
        out.push(Anchor {
            path: path.to_string(),
            line: function.line,
            kind: "function",
            message: format!(
                "{}（{} 参数，{} 行）",
                function.name,
                function.params,
                function.lines()
            ),
        });
    }

    // 恒假条件：这段永远不会执行——功能审查最硬的信号，先看是不是漏了分支。
    for line in &facts.false_conditions {
        out.push(Anchor {
            path: path.to_string(),
            line: *line,
            kind: "dead_branch",
            message: "条件是常量假：这段永远不会执行，确认是漏了分支还是死了的代码".to_string(),
        });
    }

    // 复杂度过高：嵌套深、认知负荷大、参数多——功能对不对最难在这种地方核。
    let syn = &rules.syntax;
    for function in &facts.functions {
        if function.cognitive > syn.max_cognitive {
            out.push(Anchor {
                path: path.to_string(),
                line: function.line,
                kind: "complex",
                message: format!(
                    "{} 的认知负荷约 {}（阈 {}）：分支叠得多，边界容易漏",
                    function.name, function.cognitive, syn.max_cognitive
                ),
            });
        }
        if function.params > syn.max_params {
            out.push(Anchor {
                path: path.to_string(),
                line: function.line,
                kind: "interface",
                message: format!(
                    "{} 有 {} 个参数（阈 {}）：对外承诺的输入面宽，错配代价大",
                    function.name, function.params, syn.max_params
                ),
            });
        }
    }

    // 错误信息：报错是功能承诺的一部分——读者（用户）看到它才知道失败时会怎样。
    let min_chars = rules.pragmatic.error_msg_min_chars;
    for function in &facts.functions {
        for (line, text) in &function.error_messages {
            let trimmed = text.trim();
            if trimmed.is_empty() || trimmed.chars().count() >= min_chars {
                continue;
            }
            out.push(Anchor {
                path: path.to_string(),
                line: *line,
                kind: "thin_error",
                message: format!(
                    "错误信息「{trimmed}」太短：失败时用户只知道错了，不知道哪里错、怎么办"
                ),
            });
        }
    }

    out
}

/// 要检查的代码文件：根是文件就直接用，是目录就递归挑认得的扩展名。
fn source_files(root: &Path, excludes: &[String]) -> Vec<PathBuf> {
    if root.is_file() {
        return vec![root.to_path_buf()];
    }
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .filter(|path| extract::supports(path))
        .filter(|path| {
            !path.components().any(|part| {
                part.as_os_str()
                    .to_str()
                    .map(|text| excludes.iter().any(|excluded| excluded == text))
                    .unwrap_or(false)
            })
        })
        .collect()
}

/// 报告里的路径：根是目录就给相对路径，是文件就给文件名。
fn display_path(root: &Path, path: &Path) -> String {
    if root.is_dir() {
        path.strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string()
    } else {
        path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default()
    }
}

/// 记一条使用体会：正文从标准输入读，谁的体会看 --user。多个用户各记各的。
fn record_experience(args: &ReviewArgs) -> Result<i32, String> {
    let Some(user) = args.user.as_deref().filter(|u| !u.trim().is_empty()) else {
        return Err("记一条体会要指明是谁的：--user <名字>".to_string());
    };
    let mut body = String::new();
    std::io::stdin()
        .read_to_string(&mut body)
        .map_err(|err| format!("读取标准输入失败：{err}"))?;
    let body = body.trim();
    if body.is_empty() {
        return Err("体会正文不能为空：从标准输入读一段你的判断".to_string());
    }

    let target = args.root.display().to_string();
    let mut store = Store::load()?;
    store.push(Experience {
        domain: "code".to_string(),
        user: user.to_string(),
        target: target.clone(),
        body: body.to_string(),
        at: now_iso8601(),
    });
    store.save()?;
    println!("已记 {user} 对 {target} 的一条体会");
    Ok(exit::OK)
}

/// 列出这个目标已记的使用体会（多个用户），不出报告。
fn list_experiences(args: &ReviewArgs, json: bool) -> Result<i32, String> {
    let experiences = load_experiences(&args.root);
    if json {
        let items: Vec<serde_json::Value> = experiences
            .iter()
            .map(|exp| {
                serde_json::json!({
                    "user": exp.user,
                    "body": exp.body,
                    "at": exp.at,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::json!({"target": args.root.display().to_string(), "experiences": items})
        );
        return Ok(exit::OK);
    }
    if experiences.is_empty() {
        println!("{} 还没有记过使用体会", args.root.display());
        return Ok(exit::OK);
    }
    println!(
        "{} 的使用体会（{} 条）：\n",
        args.root.display(),
        experiences.len()
    );
    for exp in experiences {
        println!("[{}] {}  {}", exp.user, exp.at, exp.body);
    }
    Ok(exit::OK)
}

/// 读这个目标已记的体会，按记录顺序。
fn load_experiences(root: &Path) -> Vec<Experience> {
    match Store::load() {
        Ok(store) => store
            .for_target("code", &root.display().to_string())
            .into_iter()
            .cloned()
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn render_text(
    rules: &[ReviewRule],
    anchors: &[Anchor],
    root: &Path,
    example: bool,
    experiences: &[Experience],
) {
    println!("# 功能审查：{}\n", root.display());

    println!("## 结论");
    println!("<一到三句：这段代码该解决什么、功能上最该先核的一件事>\n");

    println!("## 功能预期");
    for question in FUNCTIONAL_QUESTIONS {
        println!("- {question}");
    }
    println!();

    println!("## 清单与锚点");
    if anchors.is_empty() {
        println!(
            "事实面没取到锚点（可能没有认得的代码文件）；清单只能靠真读一遍，写不出位置的不写。"
        );
    } else {
        println!(
            "事实面取到 {} 处，每处都带 file:line，可直接当锚点：\n",
            anchors.len()
        );
        for anchor in anchors {
            println!(
                "  [{}] {}:{} — {}",
                anchor.kind, anchor.path, anchor.line, anchor.message
            );
        }
        println!("\n锚点不排序、不标严重度：功能对不对要对着预期判，那是判断。");
    }
    println!("对着清单逐个函数问：它做的事对得上名字吗、边界覆盖没、失败时会怎样。\n");

    println!("## 建议");
    println!("<可删项（死代码）、需与作者确认的行为、缺的测试与断言>\n");

    print_experiences(experiences);

    println!("## 交付纪律（{} 条）", rules.len());
    for rule in rules {
        println!("{}  {}", rule.name, rule.description);
        println!("  为什么：{}", rule.why);
        if example && !rule.example.is_empty() {
            println!("  长什么样：{}", rule.example);
        }
        println!();
    }
}

/// 把已记的使用体会摊开：多个用户各一条，是人的判断不是机器的。
fn print_experiences(experiences: &[Experience]) {
    if experiences.is_empty() {
        println!("## 使用体会\n");
        println!("<还没有记过；审完用 --record --user <名字> 记一条你的真实判断>\n");
        return;
    }
    println!("## 使用体会（{} 条）\n", experiences.len());
    for exp in experiences {
        println!("[{}] {}  {}\n", exp.user, exp.at, exp.body);
    }
}

fn render_json(rules: &[ReviewRule], anchors: &[Anchor], root: &Path, experiences: &[Experience]) {
    let rules_json: Vec<serde_json::Value> = rules
        .iter()
        .map(|rule| {
            serde_json::json!({
                "name": rule.name,
                "description": rule.description,
                "why": rule.why,
                "example": rule.example,
            })
        })
        .collect();
    let anchors_json: Vec<serde_json::Value> = anchors
        .iter()
        .map(|anchor| {
            serde_json::json!({
                "kind": anchor.kind,
                "path": anchor.path,
                "line": anchor.line,
                "message": anchor.message,
            })
        })
        .collect();
    let experiences_json: Vec<serde_json::Value> = experiences
        .iter()
        .map(|exp| {
            serde_json::json!({
                "user": exp.user,
                "at": exp.at,
                "body": exp.body,
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::json!({
            "root": root.display().to_string(),
            "functional_questions": FUNCTIONAL_QUESTIONS,
            "rules": rules_json,
            "anchors": anchors_json,
            "experiences": experiences_json,
        })
    );
}
