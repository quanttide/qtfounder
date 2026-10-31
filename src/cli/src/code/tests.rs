//! `code` 两道的测试：反思清单与功能审查。
//!
//! 代码的语言学四层（词汇、句法、语篇、语用）已并入 `language`，那份测试在
//! `src/language/code/tests.rs`。这里只测还留在 `code` 的两道。

use super::reflect;
use super::review::FUNCTIONAL_QUESTIONS;
use super::rules::{ReflectRule, Rules};

// ---------- 反思清单 ----------

#[test]
fn 反思清单四条齐备且名字不重复() {
    let rules = Rules::load(None).expect("仓里应能找到默认契约");
    assert_eq!(
        rules.reflect.len(),
        4,
        "契约里应有四条反思规则，改数量要同步 skill 正文"
    );
    let mut seen = std::collections::HashSet::new();
    for rule in &rules.reflect {
        assert!(!rule.name.is_empty(), "name 不能为空");
        assert!(!rule.description.is_empty(), "{} 缺 description", rule.name);
        assert!(!rule.why.is_empty(), "{} 缺 why", rule.name);
        assert!(
            !rule.example.is_empty(),
            "{} 缺 example，--example 会打空行",
            rule.name
        );
        assert!(seen.insert(rule.name.as_str()), "name 重复：{}", rule.name);
    }
    assert_eq!(
        rules.reflect[0].name, "unmarked-additions",
        "头一条是无主的行：顺序即优先级"
    );
}

#[test]
fn 反思输出例子按需才给() {
    let rules = Rules::load(None).expect("仓里应能找到默认契约");
    let picked: Vec<&ReflectRule> = rules.reflect.iter().collect();
    let total = picked.len();

    let plain = reflect::render(&picked, total, false, false).expect("渲染正文");
    assert!(
        plain.contains("为什么算缺陷"),
        "缺省必给 description 与 why"
    );
    assert!(!plain.contains("长什么样"), "缺省不打例子，例子按需");

    let with_example = reflect::render(&picked, total, true, false).expect("渲染正文");
    for rule in &rules.reflect {
        assert!(
            with_example.contains(&rule.example),
            "--example 必须连 {} 的例子一起打",
            rule.name
        );
    }
}

#[test]
fn 反思输出_json_带全四层() {
    let rules = Rules::load(None).expect("仓里应能找到默认契约");
    let picked: Vec<&ReflectRule> = rules.reflect.iter().collect();
    let body = reflect::render(&picked, picked.len(), true, true).expect("渲染 JSON");
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("是合法 JSON");
    let list = parsed.as_array().expect("是数组");
    assert_eq!(list.len(), 4);
    for item in list {
        for key in ["name", "description", "why", "example"] {
            assert!(
                item.get(key)
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| !s.is_empty()),
                "JSON 里 {key} 不能空"
            );
        }
    }
}

#[test]
fn 反思收到路径时说它不扫代码() {
    // 沿用 `code .` 习惯的人最该先知道这件事，而不是以为规则名写错了。
    for arg in [".", "..", "src/meta", "src\\meta"] {
        assert!(reflect::looks_like_path(arg), "「{arg}」该被判成路径");
    }
    for arg in ["unmarked-additions", "no-fallback", "boundary-creep"] {
        assert!(!reflect::looks_like_path(arg), "「{arg}」是规则名不是路径");
    }
}

// ---------- 功能审查 ----------

#[test]
fn 审查纪律六条齐备且名字不重复() {
    let rules = Rules::load(None).expect("仓里应能找到默认契约");
    assert_eq!(
        rules.review.len(),
        6,
        "契约里应有六条审查纪律，改数量要同步 skill 正文"
    );
    let mut seen = std::collections::HashSet::new();
    for rule in &rules.review {
        assert!(!rule.name.is_empty(), "name 不能为空");
        assert!(!rule.description.is_empty(), "{} 缺 description", rule.name);
        assert!(!rule.why.is_empty(), "{} 缺 why", rule.name);
        assert!(seen.insert(rule.name.as_str()), "name 重复：{}", rule.name);
    }
    assert_eq!(rules.review[0].name, "conclusion-first", "头一条是结论先行");
}

#[test]
fn 审查纪律含锚点与判定归人两条() {
    // 这两条是审查与反思、重构的分界：机械层只给锚点，判断归人。
    let rules = Rules::load(None).expect("仓里应能找到默认契约");
    let names: Vec<&str> = rules.review.iter().map(|rule| rule.name.as_str()).collect();
    assert!(names.contains(&"anchor-required"), "写不出锚点就不写");
    assert!(
        names.contains(&"candidates-not-verdicts"),
        "模型只给候选，判定归人"
    );
}

#[test]
fn 功能审查的四问是功能视角() {
    // 判的是「它该做什么」，不是「它怎么写」——四问里没有一条问读感或命名。
    assert_eq!(FUNCTIONAL_QUESTIONS.len(), 4);
    assert!(FUNCTIONAL_QUESTIONS[0].contains("要解决的问题"));
    assert!(FUNCTIONAL_QUESTIONS[1].contains("对外承诺的行为"));
    assert!(FUNCTIONAL_QUESTIONS[2].contains("边界与错误路径"));
    assert!(FUNCTIONAL_QUESTIONS[3].contains("还没验证"));
}
