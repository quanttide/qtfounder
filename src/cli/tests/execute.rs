//! `execute collect` / `execute draw` 集成测试：真的跑二进制，按退出码与输出判。
//!
//! 数据目录一律指向 tempfile（`QTFOUNDER_DATA`），不污染仓库 `data/`。

use std::fs;

use assert_cmd::Command;
use tempfile::TempDir;

/// 临时数据目录（整个数据目录，含 execute.json）。
fn lab() -> TempDir {
    TempDir::new().expect("临时目录")
}

/// 取一个绑定到该数据目录的命令（assert_cmd 的 Command 不可克隆，每次新建）。
fn cmd(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("qtfounder").expect("构建二进制");
    cmd.env("QTFOUNDER_DATA", dir.path());
    cmd
}

fn pool_json(dir: &TempDir) -> serde_json::Value {
    let text = fs::read_to_string(dir.path().join("execute.json")).expect("池子文件在");
    serde_json::from_str(&text).expect("池子是合法 JSON")
}

#[test]
fn 顶层多一个execute命令() {
    let dir = lab();
    let output = cmd(&dir).arg("--help").output().expect("跑起来");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("execute"), "{stdout}");
    assert!(
        stdout.contains("collect") && stdout.contains("draw"),
        "{stdout}"
    );
}

#[test]
fn 收一件再抽中它() {
    let dir = lab();
    let output = cmd(&dir)
        .args(["execute", "collect", "写周报"])
        .output()
        .expect("跑起来");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("已收"), "{stdout}");
    assert!(stdout.contains("写周报"), "{stdout}");

    let pool = pool_json(&dir);
    assert_eq!(pool["pending"].as_array().expect("数组").len(), 1);
    assert_eq!(pool["pending"][0]["text"], "写周报");
    assert_eq!(pool["pending"][0]["priority"], 1, "没给优先级就按 1");

    // 池里只有一件，抽签必中它
    let output = cmd(&dir)
        .args(["execute", "draw"])
        .output()
        .expect("跑起来");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("抽中"), "{stdout}");
    assert!(stdout.contains("写周报"), "{stdout}");

    let pool = pool_json(&dir);
    assert!(
        pool["pending"].as_array().expect("数组").is_empty(),
        "抽走了"
    );
    assert_eq!(pool["drawn"].as_array().expect("数组").len(), 1, "留痕");
    assert_eq!(pool["drawn"][0]["text"], "写周报");
}

#[test]
fn 批量收集按优先级落池() {
    let dir = lab();
    let input = r#"{"todos":[{"text":"写周报","priority":3},{"text":"订机票"}]}"#;
    let output = cmd(&dir)
        .args(["execute", "collect", "--stdin", "--json"])
        .write_stdin(input)
        .output()
        .expect("跑起来");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("是 JSON");
    assert_eq!(value["collected"].as_array().expect("数组").len(), 2);
    assert_eq!(value["deduplicated"], 0);
    assert_eq!(value["pending"], 2);

    let pool = pool_json(&dir);
    assert_eq!(pool["pending"][0]["priority"], 3, "显式优先级");
    assert_eq!(pool["pending"][1]["priority"], 1, "缺省 1");
}

#[test]
fn 同文本只收一次() {
    let dir = lab();
    for _ in 0..2 {
        let output = cmd(&dir)
            .args(["execute", "collect", "写周报"])
            .output()
            .expect("跑起来");
        assert_eq!(output.status.code(), Some(0));
    }
    let pool = pool_json(&dir);
    assert_eq!(pool["pending"].as_array().expect("数组").len(), 1, "去重");
}

#[test]
fn 高优先级的签面更大() {
    let dir = lab();
    // 权重 9:1 连抽 20 次，高优先级的必然被抽中更多次
    let input = r#"{"todos":[{"text":"要紧的事","priority":9},{"text":"不要紧的事"}]}"#;
    let output = cmd(&dir)
        .args(["execute", "collect", "--stdin"])
        .write_stdin(input)
        .output()
        .expect("跑起来");
    assert_eq!(output.status.code(), Some(0));

    let mut hit = 0;
    for _ in 0..20 {
        let output = cmd(&dir)
            .args(["execute", "draw", "--json"])
            .output()
            .expect("跑起来");
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("是 JSON");
        if value["text"] == "要紧的事" {
            hit += 1;
        }
        // 抽走一件就补一件，保持池里两件
        let text = value["text"].as_str().expect("text 是字符串");
        let priority = value["priority"].as_u64().expect("priority 是数");
        let input = format!(r#"{{"todos":[{{"text":"{text}","priority":{priority}}}]}}"#);
        let output = cmd(&dir)
            .args(["execute", "collect", "--stdin"])
            .write_stdin(input)
            .output()
            .expect("跑起来");
        assert_eq!(output.status.code(), Some(0));
    }
    assert!(
        hit >= 10,
        "9:1 的签面，20 次该抽中高优先级至少 10 次，实际 {hit}"
    );
}

#[test]
fn 空池抽签报未找到退三() {
    let dir = lab();
    let output = cmd(&dir)
        .args(["execute", "draw"])
        .output()
        .expect("跑起来");
    assert_eq!(output.status.code(), Some(3));
    let value: serde_json::Value = serde_json::from_slice(&output.stderr).expect("错误是 JSON");
    assert_eq!(value["error"]["code"], "NOT_FOUND");
    assert!(
        value["error"]["hint"]
            .as_str()
            .expect("有提示")
            .contains("collect")
    );
}

#[test]
fn 位置参数与stdin不兼得() {
    let dir = lab();
    let output = cmd(&dir)
        .args(["execute", "collect", "写周报", "--stdin"])
        .write_stdin(r#"{"todos":[{"text":"订机票"}]}"#)
        .output()
        .expect("跑起来");
    assert_eq!(output.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&output.stderr).expect("错误是 JSON");
    assert_eq!(value["error"]["code"], "USAGE");
}

#[test]
fn 零优先级拒收() {
    let dir = lab();
    let output = cmd(&dir)
        .args(["execute", "collect", "写周报", "--priority", "0"])
        .output()
        .expect("跑起来");
    assert_eq!(output.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&output.stderr).expect("错误是 JSON");
    assert!(
        value["error"]["message"]
            .as_str()
            .expect("有消息")
            .contains("优先级")
    );
}

#[test]
fn 空文本拒收() {
    let dir = lab();
    let output = cmd(&dir)
        .args(["execute", "collect", "  "])
        .output()
        .expect("跑起来");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn 空stdin拒收() {
    let dir = lab();
    let output = cmd(&dir)
        .args(["execute", "collect", "--stdin"])
        .write_stdin(r#"{"todos":[]}"#)
        .output()
        .expect("跑起来");
    assert_eq!(output.status.code(), Some(2));
}
