//! `git hook` 的集成测试：在临时仓里真的装、查、卸。
//!
//! 只验命令面与配置结果，不验钩子脚本本身；每条用例用自己的临时目录。

#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_qtfounder");

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
        code: output.status.code().expect("正常退出"),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn git(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git 跑得起来")
}

fn init_repo(dir: &Path, executable: bool) {
    std::fs::create_dir_all(dir.join(".githooks")).expect("建钩子目录");
    let hook = dir.join(".githooks/pre-commit");
    std::fs::write(&hook, "#!/bin/sh\nexit 0\n").expect("写钩子");
    let mode = if executable { 0o755 } else { 0o644 };
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(mode)).expect("设权限");
    assert!(git(dir, &["init", "-q"]).status.success());
}

fn hooks_path(dir: &Path) -> Option<String> {
    let output = git(dir, &["config", "--get", "core.hooksPath"]);
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn managed(dir: &Path) -> String {
    std::fs::canonicalize(dir)
        .expect("临时目录存在")
        .join(".githooks")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn 装查卸一条线() {
    let temp = TempDir::new().expect("建临时目录");
    let root = temp.path();
    init_repo(root, true);

    let status = run(root, &["git", "hook", "status"]);
    assert_eq!(status.code, 1, "{}", status.stderr);
    assert!(status.stdout.contains("未安装"), "{}", status.stdout);

    let install = run(root, &["git", "hook", "install"]);
    assert_eq!(install.code, 0, "{}", install.stderr);
    assert_eq!(hooks_path(root).as_deref(), Some(managed(root).as_str()));

    let status = run(root, &["git", "hook", "status", "--json"]);
    assert_eq!(status.code, 0, "{}", status.stdout);
    let value: serde_json::Value = serde_json::from_str(&status.stdout).expect("是 JSON");
    assert_eq!(value["ok"], true);
    assert_eq!(value["repos"][0]["state"], "ok");

    let uninstall = run(root, &["git", "hook", "uninstall"]);
    assert_eq!(uninstall.code, 0, "{}", uninstall.stderr);
    assert!(hooks_path(root).is_none());
}

#[test]
fn 指向别处默认不覆盖() {
    let temp = TempDir::new().expect("建临时目录");
    let root = temp.path();
    init_repo(root, true);
    assert!(
        git(root, &["config", "core.hooksPath", "/tmp/other-hooks"])
            .status
            .success()
    );

    let install = run(root, &["git", "hook", "install"]);
    assert_eq!(install.code, 1, "{}", install.stderr);
    assert_eq!(hooks_path(root).as_deref(), Some("/tmp/other-hooks"));

    let forced = run(root, &["git", "hook", "install", "--force"]);
    assert_eq!(forced.code, 0, "{}", forced.stderr);
    assert_eq!(hooks_path(root).as_deref(), Some(managed(root).as_str()));
}

#[test]
fn 不可执行时拦住安装() {
    let temp = TempDir::new().expect("建临时目录");
    let root = temp.path();
    init_repo(root, false);
    assert!(
        git(root, &["config", "core.hooksPath", &managed(root)])
            .status
            .success()
    );

    let status = run(root, &["git", "hook", "status"]);
    assert_eq!(status.code, 1, "{}", status.stderr);
    assert!(status.stdout.contains("不可执行"), "{}", status.stdout);

    let install = run(root, &["git", "hook", "install"]);
    assert_eq!(install.code, 2, "{}", install.stderr);
    assert!(install.stderr.contains("不可执行"), "{}", install.stderr);
    assert_eq!(hooks_path(root).as_deref(), Some(managed(root).as_str()));
}

#[test]
fn 从别的目录用root指定仓库() {
    let temp = TempDir::new().expect("建临时目录");
    let root = temp.path().join("repo");
    std::fs::create_dir_all(&root).expect("建仓目录");
    init_repo(&root, true);

    let status = run(
        temp.path(),
        &[
            "git",
            "hook",
            "status",
            "--root",
            root.to_str().expect("路径可转"),
        ],
    );
    assert_eq!(status.code, 1, "{}", status.stderr);
    assert!(status.stdout.contains("未安装"), "{}", status.stdout);
}
