//! `agent skill` 的集成测试：真的跑二进制，按退出码、输出与落盘结果判。
//!
//! 单元测试验证判定函数，这里验证命令面本身：子命令在不在、装完落到哪、
//! 退出码怎么分、契约按什么顺序找。每条用例用自己的临时目录，跑完就删。

use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_qtfounder");
const CONTRACT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/agent/default-contract.toml"
);

/// 随二进制发布的十一条，与 `registry` 的单元测试同一份名单。
const PUBLISHED: [&str; 11] = [
    "code-reflect",
    "code-review",
    "design-review",
    "docs-format",
    "execute-collect",
    "language-govern",
    "meta-classify",
    "product-collect",
    "product-organize",
    "think-reflect",
    "write-express",
];

const GOOD_SKILL: &str = "---\nname: git-commit\ndescription: 规范提交。当用户要提交时使用。\n---\n\n# git-commit\n\n按约定提交：`qtfounder git commit`。\n";

/// 一个用完就删的临时目录。
struct TempRoot {
    path: PathBuf,
}

impl TempRoot {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("qtfounder-it-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("建临时目录");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

struct Outcome {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(cwd: &Path, args: &[&str]) -> Outcome {
    run_with_env(cwd, args, &[])
}

/// 跑一次并清掉可能影响判定的环境变量；需要时再单独设上。
fn run_with_env(cwd: &Path, args: &[&str], envs: &[(&str, &str)]) -> Outcome {
    let mut command = Command::new(BIN);
    command
        .args(args)
        .current_dir(cwd)
        .env_remove("QTFOUNDER_AGENT_CONTRACT");
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command.output().expect("二进制跑得起来");
    Outcome {
        code: output.status.code().expect("正常退出，不是被信号打断"),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn write_skill(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative).join("SKILL.md");
    std::fs::create_dir_all(path.parent().expect("SKILL.md 有父目录")).expect("建目录");
    std::fs::write(path, text).expect("写文件");
}

fn json(output: &Outcome) -> serde_json::Value {
    serde_json::from_str(&output.stdout)
        .unwrap_or_else(|error| panic!("不是 JSON：{error}\nstdout：{}", output.stdout))
}

fn ids(value: &serde_json::Value) -> Vec<String> {
    value["skills"]
        .as_array()
        .expect("skills 是数组")
        .iter()
        .map(|item| item["id"].as_str().expect("id 是字符串").to_string())
        .collect()
}

fn status_of<'a>(value: &'a serde_json::Value, id: &str) -> &'a str {
    value["skills"]
        .as_array()
        .expect("skills 是数组")
        .iter()
        .find(|item| item["id"] == id)
        .unwrap_or_else(|| panic!("没有 {id}"))["status"]
        .as_str()
        .expect("status 是字符串")
}

// ---------- 命令面 ----------

#[test]
fn 顶层有十一个命令且装skill并入agent() {
    let temp = TempRoot::new("top-level");
    let output = run(temp.path(), &["--help"]);
    assert_eq!(output.code, 0, "{}", output.stderr);
    for name in [
        "code", "language", "docs", "write", "think", "meta", "product", "agent", "git", "audit",
        "design",
    ] {
        assert!(
            output.stdout.contains(name),
            "顶层少了 {name}：\n{}",
            output.stdout
        );
    }
    assert!(
        !output
            .stdout
            .lines()
            .any(|line| line.trim_start().starts_with("skill ")),
        "顶层不该再有一个独立的 skill 命令：\n{}",
        output.stdout
    );
}

#[test]
fn 风格检查有四个子命令() {
    let temp = TempRoot::new("skill-help");
    let output = run(temp.path(), &["agent", "skill", "--help"]);
    assert_eq!(output.code, 0, "{}", output.stderr);
    for name in ["list", "install", "check", "lint"] {
        assert!(
            output.stdout.contains(name),
            "`agent skill` 少了 {name}：\n{}",
            output.stdout
        );
    }
}

#[test]
fn 版本号与包一致() {
    let temp = TempRoot::new("version");
    let output = run(temp.path(), &["--version"]);
    assert_eq!(output.code, 0, "{}", output.stderr);
    assert!(
        output.stdout.contains(env!("CARGO_PKG_VERSION")),
        "版本号对不上：{}",
        output.stdout
    );
}

// ---------- list / check ----------

#[test]
fn 列出的十一条都未装且不建目录() {
    let temp = TempRoot::new("list-missing");
    let root = temp.path().join("repo");
    let output = run(
        temp.path(),
        &[
            "agent",
            "skill",
            "list",
            "--root",
            root.to_str().expect("路径是 UTF-8"),
            "--json",
        ],
    );
    assert_eq!(output.code, 0, "{}", output.stderr);

    let value = json(&output);
    assert_eq!(ids(&value), PUBLISHED.to_vec());
    for id in PUBLISHED {
        assert_eq!(status_of(&value, id), "missing");
        assert!(
            !value["skills"]
                .as_array()
                .expect("数组")
                .iter()
                .any(|item| item["description"].as_str().unwrap_or("").is_empty()),
            "{id} 的说明不该为空"
        );
    }
    assert!(!root.exists(), "只列不装，不该顺手建出目录");
}

#[test]
fn 未装时检查退一() {
    let temp = TempRoot::new("check-missing");
    let root = temp.path().join("repo");
    let output = run(
        temp.path(),
        &[
            "agent",
            "skill",
            "check",
            "--root",
            root.to_str().expect("路径是 UTF-8"),
        ],
    );
    assert_eq!(output.code, 1, "{}", output.stdout);
    assert!(output.stdout.contains("未装"), "{}", output.stdout);
}

#[test]
fn 装完之后检查通过() {
    let temp = TempRoot::new("install-all");
    let root = temp.path().join("repo");
    let root_arg = root.to_str().expect("路径是 UTF-8");

    let install = run(
        temp.path(),
        &["agent", "skill", "install", "--root", root_arg],
    );
    assert_eq!(install.code, 0, "{}", install.stderr);
    assert!(install.stdout.contains("装了 11 个"), "{}", install.stdout);
    for id in PUBLISHED {
        assert!(
            root.join(".agents/skills")
                .join(id)
                .join("SKILL.md")
                .is_file(),
            "{id} 没落到 .agents/skills"
        );
    }

    let check = run(
        temp.path(),
        &["agent", "skill", "check", "--root", root_arg],
    );
    assert_eq!(check.code, 0, "{}", check.stdout);
    assert!(
        check.stdout.contains("都与嵌入版本一致"),
        "{}",
        check.stdout
    );

    let list = run(
        temp.path(),
        &["agent", "skill", "list", "--root", root_arg, "--json"],
    );
    let value = json(&list);
    for id in PUBLISHED {
        assert_eq!(status_of(&value, id), "current", "{id} 装完却不是最新");
    }
}

#[test]
fn 只装一条时其余仍未装() {
    let temp = TempRoot::new("install-one");
    let root = temp.path().join("repo");
    let root_arg = root.to_str().expect("路径是 UTF-8");

    let install = run(
        temp.path(),
        &[
            "agent",
            "skill",
            "install",
            "language-govern",
            "--root",
            root_arg,
        ],
    );
    assert_eq!(install.code, 0, "{}", install.stderr);
    assert!(install.stdout.contains("装了 1 个"), "{}", install.stdout);

    let list = run(
        temp.path(),
        &["agent", "skill", "list", "--root", root_arg, "--json"],
    );
    let value = json(&list);
    assert_eq!(status_of(&value, "language-govern"), "current");
    for id in [
        "design-review",
        "meta-classify",
        "product-collect",
        "product-organize",
        "think-reflect",
    ] {
        assert_eq!(status_of(&value, id), "missing", "{id} 不该被顺手装上");
    }
}

#[test]
fn 未知名字安装退二并列出可用的() {
    let temp = TempRoot::new("install-unknown");
    let root = temp.path().join("repo");
    let output = run(
        temp.path(),
        &[
            "agent",
            "skill",
            "install",
            "no-such-skill",
            "--root",
            root.to_str().expect("路径是 UTF-8"),
        ],
    );
    assert_eq!(output.code, 2, "{}", output.stdout);
    assert!(
        output.stderr.contains("没有嵌入「no-such-skill」"),
        "{}",
        output.stderr
    );
    assert!(
        output.stderr.contains("language-govern"),
        "报错要给出可用的名字：{}",
        output.stderr
    );
}

#[test]
fn 强制安装会覆盖改动() {
    let temp = TempRoot::new("install-force");
    let root = temp.path().join("repo");
    let root_arg = root.to_str().expect("路径是 UTF-8");

    let first = run(
        temp.path(),
        &["agent", "skill", "install", "--root", root_arg],
    );
    assert_eq!(first.code, 0, "{}", first.stderr);

    let installed = root.join(".agents/skills/think-reflect/SKILL.md");
    std::fs::write(&installed, "手改过\n").expect("改得动");

    let stale = run(
        temp.path(),
        &["agent", "skill", "check", "--root", root_arg],
    );
    assert_eq!(stale.code, 1, "{}", stale.stdout);
    assert!(stale.stdout.contains("think-reflect"), "{}", stale.stdout);
    assert!(stale.stdout.contains("不一致"), "{}", stale.stdout);

    let forced = run(
        temp.path(),
        &["agent", "skill", "install", "--force", "--root", root_arg],
    );
    assert_eq!(forced.code, 0, "{}", forced.stderr);
    assert!(
        std::fs::read_to_string(&installed)
            .expect("读得到")
            .contains("think-reflect"),
        "强制安装之后应当恢复原文"
    );

    let clean = run(
        temp.path(),
        &["agent", "skill", "check", "--root", root_arg],
    );
    assert_eq!(clean.code, 0, "{}", clean.stdout);
}

#[test]
fn 复制模式会同时投递两份() {
    let temp = TempRoot::new("install-copy");
    let root = temp.path().join("repo");
    let root_arg = root.to_str().expect("路径是 UTF-8");

    let output = run(
        temp.path(),
        &[
            "agent", "skill", "install", "--copy", "--claude", "--root", root_arg,
        ],
    );
    assert_eq!(output.code, 0, "{}", output.stderr);
    for id in PUBLISHED {
        assert!(
            root.join(".agents/skills")
                .join(id)
                .join("SKILL.md")
                .is_file(),
            "{id} 的 canonical 没写出来"
        );
        assert!(
            root.join(".claude/skills")
                .join(id)
                .join("SKILL.md")
                .is_file(),
            "{id} 没投到 .claude/skills"
        );
    }
}

#[test]
fn 安装的json带数量与失败清单() {
    let temp = TempRoot::new("install-json");
    let root = temp.path().join("repo");
    let output = run(
        temp.path(),
        &[
            "agent",
            "skill",
            "install",
            "--copy",
            "--root",
            root.to_str().expect("路径是 UTF-8"),
            "--json",
        ],
    );
    assert_eq!(output.code, 0, "{}", output.stderr);

    let value = json(&output);
    let installed = value["installed"].as_array().expect("installed 是数组");
    let failed = value["failed"].as_array().expect("failed 是数组");
    assert_eq!(installed.len(), PUBLISHED.len());
    assert!(failed.is_empty(), "不该有失败：{failed:?}");
    for item in installed {
        let path = PathBuf::from(item["path"].as_str().expect("path 是字符串"));
        assert!(path.join("SKILL.md").is_file(), "{} 没落盘", path.display());
    }
}

// ---------- lint ----------

#[test]
fn 合规的写法审查退零() {
    let temp = TempRoot::new("lint-good");
    write_skill(temp.path(), "skills/git-commit", GOOD_SKILL);

    let output = run(
        temp.path(),
        &["agent", "skill", "lint", "--rules", CONTRACT],
    );
    assert_eq!(output.code, 0, "{}", output.stdout);
    assert!(output.stdout.contains("没有命中"), "{}", output.stdout);
}

#[test]
fn 命中时退一并给出替代动作() {
    let temp = TempRoot::new("lint-bad");
    write_skill(
        temp.path(),
        "skills/product-management",
        "---\nname: product-management\ndescription: 整理需求。\n---\n\n# 整理需求\n",
    );

    let output = run(
        temp.path(),
        &["agent", "skill", "lint", "--rules", CONTRACT],
    );
    assert_eq!(output.code, 1, "{}", output.stdout);
    assert!(output.stdout.contains("名词后缀"), "{}", output.stdout);
    assert!(output.stdout.contains("错误"), "{}", output.stdout);
    assert!(
        output.stdout.contains("→"),
        "报告要给出怎么改：{}",
        output.stdout
    );
}

#[test]
fn 审查结果能解析成json() {
    let temp = TempRoot::new("lint-json");
    write_skill(
        temp.path(),
        "skills/product-management",
        "---\nname: product-management\ndescription: 整理需求。\n---\n\n# 整理需求\n",
    );

    let output = run(
        temp.path(),
        &["agent", "skill", "lint", "--rules", CONTRACT, "--json"],
    );
    assert_eq!(output.code, 1, "{}", output.stderr);

    let value = json(&output);
    assert_eq!(value["scanned"], 1);
    let findings = value["findings"].as_array().expect("findings 是数组");
    assert!(!findings.is_empty(), "命中时 findings 不该为空");
    for finding in findings {
        for field in ["path", "severity", "kind", "message", "advice"] {
            assert!(
                finding[field].as_str().is_some_and(|text| !text.is_empty()),
                "{field} 不该缺：{finding}"
            );
        }
        let severity = finding["severity"].as_str().expect("severity 是字符串");
        assert!(["error", "suspect"].contains(&severity), "{severity}");
    }
}

#[test]
fn 重复名字审查退一() {
    let temp = TempRoot::new("lint-duplicate");
    write_skill(temp.path(), "a/git-commit", GOOD_SKILL);
    write_skill(temp.path(), "b/git-commit", GOOD_SKILL);

    let output = run(
        temp.path(),
        &["agent", "skill", "lint", "--rules", CONTRACT],
    );
    assert_eq!(output.code, 1, "{}", output.stdout);
    assert!(output.stdout.contains("重复出现"), "{}", output.stdout);
}

#[test]
fn 默认扫描跳过排除目录() {
    let temp = TempRoot::new("lint-exclude");
    write_skill(temp.path(), "skills/git-commit", GOOD_SKILL);
    write_skill(
        temp.path(),
        "target/skills/broken",
        "---\nname: somewhere-else\ndescription: 做点什么。当需要时使用。\n---\n\n# 坏样本\n",
    );

    let output = run(
        temp.path(),
        &["agent", "skill", "lint", "--rules", CONTRACT],
    );
    assert_eq!(output.code, 0, "{}", output.stdout);
    assert!(
        output.stdout.contains("审了 1 个 Skill"),
        "target 下不该被扫到：{}",
        output.stdout
    );
}

#[test]
fn 契约路径不对时退二() {
    let temp = TempRoot::new("lint-no-rules");
    write_skill(temp.path(), "skills/git-commit", GOOD_SKILL);

    let output = run(
        temp.path(),
        &[
            "agent",
            "skill",
            "lint",
            "--rules",
            "/nonexistent/default-contract.toml",
        ],
    );
    assert_eq!(output.code, 2, "{}", output.stdout);
    assert!(output.stderr.contains("--rules"), "{}", output.stderr);
}

#[test]
fn 目录不存在时退二() {
    let temp = TempRoot::new("lint-no-dir");
    let output = run(
        temp.path(),
        &["agent", "skill", "lint", "no-such-dir", "--rules", CONTRACT],
    );
    assert_eq!(output.code, 2, "{}", output.stdout);
    assert!(output.stderr.contains("目录不存在"), "{}", output.stderr);
}

#[test]
fn 环境变量契约优先于源码树() {
    let temp = TempRoot::new("lint-env-rules");
    write_skill(temp.path(), "skills/git-commit", GOOD_SKILL);

    let custom = temp.path().join("custom.toml");
    std::fs::write(
        &custom,
        "[name]\nsegments = 3\nmax_chars = 32\nscopes = [\"git\"]\ncli_scopes = []\nnoun_suffixes = []\n\n[description]\nmax_chars = 1024\ntrigger_words = [\"当\"]\n\n[body]\nrequire_heading = true\n",
    )
    .expect("写契约");

    let from_env = run_with_env(
        temp.path(),
        &["agent", "skill", "lint"],
        &[(
            "QTFOUNDER_AGENT_CONTRACT",
            custom.to_str().expect("路径是 UTF-8"),
        )],
    );
    assert_eq!(from_env.code, 1, "{}", from_env.stdout);
    assert!(
        from_env.stdout.contains("3 段"),
        "应当按环境变量给的契约判：{}",
        from_env.stdout
    );

    let from_flag = run_with_env(
        temp.path(),
        &["agent", "skill", "lint", "--rules", CONTRACT],
        &[(
            "QTFOUNDER_AGENT_CONTRACT",
            custom.to_str().expect("路径是 UTF-8"),
        )],
    );
    assert_eq!(
        from_flag.code, 0,
        "--rules 应当压过环境变量：{}",
        from_flag.stdout
    );
}

#[test]
fn 环境变量指向不存在的契约时退二() {
    let temp = TempRoot::new("lint-env-broken");
    write_skill(temp.path(), "skills/git-commit", GOOD_SKILL);

    let output = run_with_env(
        temp.path(),
        &["agent", "skill", "lint"],
        &[("QTFOUNDER_AGENT_CONTRACT", "/nonexistent/x.toml")],
    );
    assert_eq!(output.code, 2, "{}", output.stdout);
    assert!(
        output.stderr.contains("指向的契约不存在"),
        "{}",
        output.stderr
    );
}
