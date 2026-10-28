//! `design` 的集成测试：真的跑二进制，按退出码与输出判。
//!
//! 单元测试验证判定函数，这里验证命令面本身：子命令在不在、
//! 数字与命中怎么打、原则全文能不能摊开、退出码怎么分。

use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_qtfounder");

/// 一个用完就删的临时目录。
struct TempRoot {
    path: PathBuf,
}

impl TempRoot {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("qtfounder-design-{name}"));
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
    let output = Command::new(BIN)
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("二进制跑得起来");
    Outcome {
        code: output.status.code().expect("正常退出，不是被信号打断"),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn write_html(root: &Path, name: &str, text: &str) {
    std::fs::write(root.join(name), text).expect("写文件");
}

#[test]
fn 顶层多一个design命令() {
    let temp = TempRoot::new("help");
    let output = run(temp.path(), &["--help"]);
    assert_eq!(output.code, 0, "{}", output.stderr);
    assert!(output.stdout.contains("design"), "{}", output.stdout);
    assert!(
        output.stdout.contains("review") && output.stdout.contains("principles"),
        "{}",
        output.stdout
    );
}

#[test]
fn 小字号报缺陷并退一() {
    let temp = TempRoot::new("small-font");
    write_html(
        temp.path(),
        "index.html",
        "<style>\n.t { font-size: 11px; }\n</style>",
    );
    let output = run(temp.path(), &["design", "review", "."]);
    assert_eq!(output.code, 1, "{}", output.stdout);
    assert!(output.stdout.contains("缺陷"), "{}", output.stdout);
    assert!(output.stdout.contains("11.0px"), "{}", output.stdout);
}

#[test]
fn 干净文件退零() {
    let temp = TempRoot::new("clean");
    write_html(
        temp.path(),
        "index.html",
        "<style>\n.t { font-size: 14px; }\n</style>\n<button class=\"primary\">开始</button>",
    );
    let output = run(temp.path(), &["design", "review", "."]);
    assert_eq!(output.code, 0, "{}", output.stdout);
    assert!(output.stdout.contains("没有命中"), "{}", output.stdout);
}

#[test]
fn 没有html文件退零() {
    let temp = TempRoot::new("no-html");
    let output = run(temp.path(), &["design", "review", "."]);
    assert_eq!(output.code, 0, "{}", output.stderr);
    assert!(
        output.stdout.contains("没有 HTML 文件"),
        "{}",
        output.stdout
    );
}

#[test]
fn 目录递归扫html并跳过排除目录() {
    let temp = TempRoot::new("recurse");
    write_html(temp.path(), "a.html", "<style>.t{font-size:10px}</style>");
    std::fs::create_dir_all(temp.path().join("node_modules")).expect("建目录");
    write_html(
        temp.path(),
        "node_modules/b.html",
        "<style>.t{font-size:1px}</style>",
    );
    let output = run(temp.path(), &["design", "review", "."]);
    assert_eq!(output.code, 1, "{}", output.stdout);
    assert!(output.stdout.contains("a.html"), "{}", output.stdout);
    assert!(
        !output.stdout.contains("b.html"),
        "node_modules 不该扫：{}",
        output.stdout
    );
}

#[test]
fn json能解析出文件与命中() {
    let temp = TempRoot::new("json");
    write_html(
        temp.path(),
        "index.html",
        "<style>\n.t { font-size: 10px; }\n</style>\n<button class=\"primary\">一</button>\n<button class=\"primary\">二</button>",
    );
    let output = run(temp.path(), &["design", "review", ".", "--json"]);
    assert_eq!(output.code, 1, "{}", output.stderr);
    let value: serde_json::Value = serde_json::from_str(&output.stdout)
        .unwrap_or_else(|error| panic!("不是 JSON：{error}\n{}", output.stdout));
    assert_eq!(value["files"][0]["min_px"], 10.0);
    assert_eq!(value["files"][0]["primaries"], 2);
    assert!(
        value["findings"]
            .as_array()
            .expect("findings 是数组")
            .iter()
            .any(|item| item["kind"] == "primary")
    );
}

#[test]
fn 原则全文能摊开() {
    let temp = TempRoot::new("principles");
    let output = run(temp.path(), &["design", "principles"]);
    assert_eq!(output.code, 0, "{}", output.stderr);
    for keyword in [
        "第一条",
        "第九条",
        "第十二条",
        "评分标准",
        "10 | 敢给真实用户用",
    ] {
        assert!(
            output.stdout.contains(keyword),
            "缺 {keyword}：{}",
            output.stdout
        );
    }
}
