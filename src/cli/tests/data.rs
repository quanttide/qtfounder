//! `data` 集成测试：真的跑二进制，按退出码与输出判。
//!
//! 数据目录一律指向 tempfile（`QTFOUNDER_DATA`），不污染 XDG。

use std::fs;

use assert_cmd::Command;
use tempfile::TempDir;

/// 临时数据目录（整个数据目录，含 data/rules 等）。
fn lab() -> TempDir {
    TempDir::new().expect("临时目录")
}

/// 取一个绑定到该数据目录的命令。
fn cmd(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("qtfounder").expect("构建二进制");
    cmd.env("QTFOUNDER_DATA", dir.path());
    cmd
}

const RULE_JSON: &str = r#"{"name":"日志四分类","source":{"kind":"journal","split":"blank-line"},"labels":[{"name":"fact","label":"事实","define":"发生了什么"},{"name":"feeling","label":"感受"}],"batch":5}"#;

fn save_rule(dir: &TempDir) -> String {
    let output = cmd(dir)
        .args(["data", "rule", "save", "--stdin", "--json"])
        .write_stdin(RULE_JSON)
        .output()
        .expect("跑起来");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("是 JSON");
    value["id"].as_str().expect("有 id").to_string()
}

#[test]
fn 规则存取一来回() {
    let dir = lab();
    let id = save_rule(&dir);
    assert!(id.starts_with("rul_"), "{id}");

    // list 看得到
    let output = cmd(&dir)
        .args(["data", "rule", "list", "--json"])
        .output()
        .expect("跑起来");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("是 JSON");
    assert_eq!(value["rules"].as_array().expect("数组").len(), 1);
    assert_eq!(value["rules"][0]["id"], id);
    assert_eq!(value["rules"][0]["labels"], 2);

    // show 看全文
    let output = cmd(&dir)
        .args(["data", "rule", "show", &id, "--json"])
        .output()
        .expect("跑起来");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("是 JSON");
    assert_eq!(value["name"], "日志四分类");
    assert_eq!(value["source"]["kind"], "journal");
    assert_eq!(value["batch"], 5);
}

#[test]
fn 带已有id覆盖未知id报未找到() {
    let dir = lab();
    let id = save_rule(&dir);

    let overwrite = format!(
        r#"{{"id":"{id}","name":"改名了","source":{{"kind":"journal","split":"blank-line"}},"labels":[{{"name":"fact","label":"事实"}}]}}"#
    );
    let output = cmd(&dir)
        .args(["data", "rule", "save", "--stdin"])
        .write_stdin(overwrite)
        .output()
        .expect("跑起来");
    assert_eq!(output.status.code(), Some(0));

    let output = cmd(&dir)
        .args(["data", "rule", "list"])
        .output()
        .expect("跑起来");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("改名了"), "{stdout}");
    assert!(!stdout.contains("日志四分类"), "覆盖后旧名不在");

    let output = cmd(&dir)
        .args(["data", "rule", "show", "rul_NONE"])
        .output()
        .expect("跑起来");
    assert_eq!(output.status.code(), Some(3));
    let value: serde_json::Value = serde_json::from_slice(&output.stderr).expect("错误是 JSON");
    assert_eq!(value["error"]["code"], "NOT_FOUND");
}

#[test]
fn 校验不过退二() {
    let dir = lab();
    let output = cmd(&dir)
        .args(["data", "rule", "save", "--stdin"])
        .write_stdin(
            r#"{"name":"没标签","source":{"kind":"journal","split":"blank-line"},"labels":[]}"#,
        )
        .output()
        .expect("跑起来");
    assert_eq!(output.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&output.stderr).expect("错误是 JSON");
    assert_eq!(value["error"]["code"], "USAGE");
}

#[test]
fn 结果导入与统计() {
    let dir = lab();
    let export = dir.path().join("lite-export.jsonl");
    fs::write(
        &export,
        concat!(
            "{\"task\":\"tsk_A\",\"unit\":\"u1\",\"label\":\"fact\"}\n",
            "{\"task\":\"tsk_A\",\"unit\":\"u2\",\"label\":\"feeling\"}\n",
            "{\"task\":\"tsk_A\",\"unit\":\"u3\",\"label\":\"fact\"}\n",
        ),
    )
    .expect("写导出文件");

    let output = cmd(&dir)
        .args(["data", "result", "import"])
        .arg(&export)
        .arg("--json")
        .output()
        .expect("跑起来");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("是 JSON");
    assert_eq!(value["imported"], 3);

    let output = cmd(&dir)
        .args(["data", "result", "list", "--json"])
        .output()
        .expect("跑起来");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("是 JSON");
    assert_eq!(value["tasks"][0]["task"], "tsk_A");
    assert_eq!(value["tasks"][0]["total"], 3);
    assert_eq!(value["tasks"][0]["labels"]["fact"], 2);

    // 重复导入追加不覆盖
    let output = cmd(&dir)
        .args(["data", "result", "import"])
        .arg(&export)
        .output()
        .expect("跑起来");
    assert_eq!(output.status.code(), Some(0));
    let output = cmd(&dir)
        .args(["data", "result", "list"])
        .output()
        .expect("跑起来");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("共 6 条"), "{stdout}");
}

#[test]
fn 导入缺字段退二未知任务退三() {
    let dir = lab();
    let export = dir.path().join("no-task.jsonl");
    fs::write(&export, "{\"unit\":\"u1\",\"label\":\"fact\"}\n").expect("写导出文件");

    let output = cmd(&dir)
        .args(["data", "result", "import"])
        .arg(&export)
        .output()
        .expect("跑起来");
    assert_eq!(output.status.code(), Some(2));

    let output = cmd(&dir)
        .args(["data", "result", "list", "--task", "tsk_NONE"])
        .output()
        .expect("跑起来");
    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn 任务包列表空态与未知id() {
    let dir = lab();
    let output = cmd(&dir)
        .args(["data", "task", "list"])
        .output()
        .expect("跑起来");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("暂无任务包"), "{stdout}");

    let output = cmd(&dir)
        .args(["data", "task", "show", "tsk_NONE"])
        .output()
        .expect("跑起来");
    assert_eq!(output.status.code(), Some(3));
}
