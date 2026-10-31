//! `code review` 与 `write review` 的集成测试：真的跑二进制，按退出码与输出判。
//!
//! 单元测试（`review::tests`）验证存储，这里验证命令面：骨架出不出得来、
//! 体会记不记得进、多个人各记一条、领域（code／write）分不分得开、
//! 缺 --user 时退不退出 2。

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_qtfounder");

/// 用仓里的契约，不用用户级那份（用户级可能还是旧的、没有 [[review]]）。
fn code_contract() -> String {
    format!(
        "{}/src/code/default-contract.toml",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn write_contract() -> String {
    format!(
        "{}/src/write/default-contract.toml",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// 一个用完就删的临时目录。
struct TempRoot {
    path: PathBuf,
}

impl TempRoot {
    fn new(name: &str) -> Self {
        // 并行跑时同名会互删，加一个进程内唯一序号。
        use std::sync::atomic::{AtomicU32, Ordering};
        static SEQ: AtomicU32 = AtomicU32::new(0);
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "qtfounder-review-{}-{name}-{seq}",
            std::process::id()
        ));
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

/// 跑一次，数据根固定到临时目录，避免污染真实的 XDG 数据。
fn run(data_root: &Path, cwd: &Path, args: &[&str]) -> Outcome {
    let output = Command::new(BIN)
        .args(args)
        .current_dir(cwd)
        .env("QTFOUNDER_DATA", data_root)
        .env("QTFOUNDER_CODE_CONTRACT", code_contract())
        .env("QTFOUNDER_WRITE_CONTRACT", write_contract())
        .output()
        .expect("二进制跑得起来");
    Outcome {
        code: output.status.code().expect("正常退出"),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

/// 记一条体会：正文走标准输入。
fn record(data_root: &Path, cwd: &Path, args: &[&str], body: &str) -> Outcome {
    let mut child = Command::new(BIN)
        .args(args)
        .current_dir(cwd)
        .env("QTFOUNDER_DATA", data_root)
        .env("QTFOUNDER_CODE_CONTRACT", code_contract())
        .env("QTFOUNDER_WRITE_CONTRACT", write_contract())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("二进制跑得起来");
    child
        .stdin
        .as_mut()
        .expect("有标准输入")
        .write_all(body.as_bytes())
        .expect("写标准输入");
    let output = child.wait_with_output().expect("跑完");
    Outcome {
        code: output.status.code().expect("正常退出"),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn write_doc(root: &Path, name: &str) {
    let text = "# 标题\n\n一段正文，用来当评审目标。\n";
    std::fs::write(root.join(name), text).expect("写文档");
}

#[test]
fn code_review出骨架带体会节() {
    let data = TempRoot::new("data");
    let work = TempRoot::new("work");
    write_doc(work.path(), "a.md");

    let out = run(data.path(), work.path(), &["code", "review", "a.md"]);
    assert_eq!(out.code, 0, "零命中退 0：{}", out.stderr);
    assert!(out.stdout.contains("# 功能审查"), "{}", out.stdout);
    assert!(
        out.stdout.contains("## 使用体会"),
        "骨架该有体会节：{}",
        out.stdout
    );
    assert!(
        out.stdout.contains("--record"),
        "空态该提示怎么记：{}",
        out.stdout
    );
}

#[test]
fn code_review体会缺user退2() {
    let data = TempRoot::new("data");
    let work = TempRoot::new("work");
    write_doc(work.path(), "a.md");

    let out = record(
        data.path(),
        work.path(),
        &["code", "review", "a.md", "--record"],
        "没有指明是谁的体会",
    );
    assert_eq!(out.code, 2, "缺 --user 是用法错：{}", out.stderr);
    assert!(out.stderr.contains("--user"), "{}", out.stderr);
}

#[test]
fn code_review体会空正文退2() {
    let data = TempRoot::new("data");
    let work = TempRoot::new("work");
    write_doc(work.path(), "a.md");

    let out = record(
        data.path(),
        work.path(),
        &["code", "review", "a.md", "--record", "--user", "alice"],
        "   \n",
    );
    assert_eq!(out.code, 2, "空正文是用法错：{}", out.stderr);
}

#[test]
fn code_review多人各记一条并列出() {
    let data = TempRoot::new("data");
    let work = TempRoot::new("work");
    write_doc(work.path(), "a.md");

    let first = record(
        data.path(),
        work.path(),
        &["code", "review", "a.md", "--record", "--user", "alice"],
        "alice 的判断",
    );
    assert_eq!(first.code, 0, "{}", first.stderr);
    let second = record(
        data.path(),
        work.path(),
        &["code", "review", "a.md", "--record", "--user", "bob"],
        "bob 的判断",
    );
    assert_eq!(second.code, 0, "{}", second.stderr);

    let out = run(
        data.path(),
        work.path(),
        &["code", "review", "a.md", "--experiences"],
    );
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(out.stdout.contains("alice"), "{}", out.stdout);
    assert!(out.stdout.contains("bob"), "{}", out.stdout);
    assert!(
        out.stdout.contains("2 条"),
        "两个人该是两条：{}",
        out.stdout
    );
}

#[test]
fn code_review体会出现在报告里() {
    let data = TempRoot::new("data");
    let work = TempRoot::new("work");
    write_doc(work.path(), "a.md");

    record(
        data.path(),
        work.path(),
        &["code", "review", "a.md", "--record", "--user", "alice"],
        "记在报告里的一条",
    );
    let out = run(data.path(), work.path(), &["code", "review", "a.md"]);
    assert!(out.stdout.contains("记在报告里的一条"), "{}", out.stdout);
    assert!(out.stdout.contains("[alice]"), "{}", out.stdout);
}

#[test]
fn write_review出读者骨架不从作者角度() {
    let data = TempRoot::new("data");
    let work = TempRoot::new("work");
    write_doc(work.path(), "a.md");

    let out = run(data.path(), work.path(), &["write", "review", "a.md"]);
    assert_eq!(out.code, 0, "零命中退 0：{}", out.stderr);
    assert!(out.stdout.contains("# 读者评审"), "{}", out.stdout);
    assert!(out.stdout.contains("## 读者视角"), "{}", out.stdout);
    assert!(out.stdout.contains("读者是谁"), "{}", out.stdout);
    // 立场：禁止滑回作者/编辑角度。
    assert!(
        out.stdout.contains("视角别滑回作者侧"),
        "骨架该写明立场：{}",
        out.stdout
    );
}

#[test]
fn write_review读者体会与代码体会分得开() {
    let data = TempRoot::new("data");
    let work = TempRoot::new("work");
    write_doc(work.path(), "a.md");

    record(
        data.path(),
        work.path(),
        &["code", "review", "a.md", "--record", "--user", "alice"],
        "代码审查的体会",
    );
    record(
        data.path(),
        work.path(),
        &["write", "review", "a.md", "--record", "--user", "carol"],
        "读者评审的体会",
    );

    let code = run(
        data.path(),
        work.path(),
        &["code", "review", "a.md", "--experiences"],
    );
    assert!(code.stdout.contains("代码审查的体会"), "{}", code.stdout);
    assert!(
        !code.stdout.contains("读者评审的体会"),
        "code 侧不该有 write 体会：{}",
        code.stdout
    );

    let write = run(
        data.path(),
        work.path(),
        &["write", "review", "a.md", "--experiences"],
    );
    assert!(write.stdout.contains("读者评审的体会"), "{}", write.stdout);
    assert!(
        !write.stdout.contains("代码审查的体会"),
        "write 侧不该有 code 体会：{}",
        write.stdout
    );
}

#[test]
fn write_review多人读者各记一条() {
    let data = TempRoot::new("data");
    let work = TempRoot::new("work");
    write_doc(work.path(), "a.md");

    record(
        data.path(),
        work.path(),
        &["write", "review", "a.md", "--record", "--user", "carol"],
        "carol 读到这停了",
    );
    record(
        data.path(),
        work.path(),
        &["write", "review", "a.md", "--record", "--user", "dave"],
        "dave 这节最解渴",
    );

    let out = run(
        data.path(),
        work.path(),
        &["write", "review", "a.md", "--experiences"],
    );
    assert!(out.stdout.contains("carol"), "{}", out.stdout);
    assert!(out.stdout.contains("dave"), "{}", out.stdout);
    assert!(
        out.stdout.contains("2 条"),
        "两个读者该是两条：{}",
        out.stdout
    );
}

#[test]
fn write_review体会缺user退2() {
    let data = TempRoot::new("data");
    let work = TempRoot::new("work");
    write_doc(work.path(), "a.md");

    let out = record(
        data.path(),
        work.path(),
        &["write", "review", "a.md", "--record"],
        "忘了说是谁",
    );
    assert_eq!(out.code, 2, "缺 --user 是用法错：{}", out.stderr);
    assert!(out.stderr.contains("--user"), "{}", out.stderr);
}

#[test]
fn write_review目标不在退3() {
    let data = TempRoot::new("data");
    let work = TempRoot::new("work");

    let out = run(data.path(), work.path(), &["write", "review", "nothing.md"]);
    assert_eq!(out.code, 3, "目标缺席退 3：{}", out.stderr);
}

#[test]
fn write_review_json_带读者视角与体会() {
    let data = TempRoot::new("data");
    let work = TempRoot::new("work");
    write_doc(work.path(), "a.md");

    record(
        data.path(),
        work.path(),
        &["write", "review", "a.md", "--record", "--user", "carol"],
        "JSON 里的一条",
    );
    let out = run(
        data.path(),
        work.path(),
        &["write", "review", "a.md", "--json"],
    );
    let parsed: serde_json::Value = serde_json::from_str(&out.stdout).expect("是合法 JSON");
    assert!(
        parsed["reader_layers"].as_array().is_some(),
        "读者视角是个数组"
    );
    assert!(
        !parsed["experiences"].as_array().unwrap().is_empty(),
        "体会进 JSON"
    );
    assert_eq!(parsed["experiences"][0]["user"], "carol");
}
