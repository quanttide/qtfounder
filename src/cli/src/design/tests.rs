//! `design` 的单元测试：判函数级，命令面见 tests/design_review.rs。

use super::*;

#[test]
fn px低于十二报缺陷() {
    let (status, findings) = check("a.html", "<style>.t { font-size: 11px; }</style>");
    assert_eq!(status.fonts, 1);
    assert_eq!(status.min_px, Some(11.0));
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, Severity::Error);
    assert_eq!(findings[0].kind, "font");
}

#[test]
fn rem按十六像素根换算() {
    let (status, findings) = check("a.html", "<style>.t { font-size: 0.68rem; }</style>");
    // 0.68 × 16 = 10.88px
    assert_eq!(status.rem, 1);
    assert_eq!(status.min_px, Some(10.88));
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, Severity::Error);
}

#[test]
fn 十二到十四只报嫌疑() {
    let (_, findings) = check("a.html", "<style>.t { font-size: 13px; }</style>");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, Severity::Suspect);
}

#[test]
fn 十四以上不报() {
    let (_, findings) = check(
        "a.html",
        "<style>.t { font-size: 14px; font-size: 1rem; }</style>",
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn em只计数不换算() {
    let (status, findings) = check("a.html", "<style>.t { font-size: 2em; }</style>");
    assert_eq!(status.em, 1);
    assert_eq!(status.min_px, None);
    assert!(
        findings.is_empty(),
        "em 相对父级判不了，不该报：{findings:?}"
    );
}

#[test]
fn 大小写与空格都认() {
    let (status, _) = check("a.html", "<style>.t { FONT-SIZE : 14px; }</style>");
    assert_eq!(status.px, 1);
}

#[test]
fn 行号指向声明所在行() {
    let text = "<style>\n.a { color: red; }\n.b { font-size: 10px; }\n</style>";
    let (_, findings) = check("a.html", text);
    assert_eq!(findings[0].line, 3);
}

#[test]
fn 单个主按钮不报() {
    let (status, findings) = check(
        "a.html",
        r#"<button class="primary" onclick="go()">开始</button>"#,
    );
    assert_eq!(status.primaries, 1);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn 多个主按钮报一次嫌疑() {
    let text = r#"<button class="primary">派活</button>
<button class="primary wide">确认</button>
<button class="btn primary">再来一个</button>"#;
    let (status, findings) = check("a.html", text);
    assert_eq!(status.primaries, 3);
    assert_eq!(findings.len(), 1, "多于一个只报一次：{findings:?}");
    assert_eq!(findings[0].severity, Severity::Suspect);
    assert_eq!(findings[0].kind, "primary");
}

#[test]
fn 单引号与非连续类名也认() {
    let (status, _) = check(
        "a.html",
        r#"<button class='btn primary-action'>Go</button>"#,
    );
    // primary-action 里 primary 是词边界内的片段
    assert_eq!(status.primaries, 1);
}

#[test]
fn 普通按钮不算主按钮() {
    let (status, findings) = check(
        "a.html",
        r#"<button class="link">取消</button>
<button>提交</button>"#,
    );
    assert_eq!(status.primaries, 0);
    assert!(findings.is_empty(), "{findings:?}");
}
