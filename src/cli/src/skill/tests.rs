/// 嵌入清单的冒烟测试：能注册，才装得出去。
///
/// 清单来自本包 `skills/`：三条随 CLI 发布的 Skill。改这里就要同步 `skills/`。
use quanttide_founder::agent::skills;

use super::registry;

#[test]
fn 嵌进来的每条都能注册() {
    for entry in registry::all() {
        let summary = skills::describe(entry).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(summary.name, entry.id);
        assert!(
            !summary.description.is_empty(),
            "{} 的 frontmatter 缺 description",
            entry.id
        );
    }
}

#[test]
fn 有来源就得有条目() {
    if let Some(source) = registry::source() {
        assert!(!registry::all().is_empty(), "来源 {source} 下一条都没读到");
    }
}

#[test]
fn 嵌入清单只有三条() {
    let ids: Vec<&str> = registry::all().iter().map(|entry| entry.id).collect();
    if ids.is_empty() {
        return;
    }
    assert_eq!(
        ids,
        vec!["language-govern", "meta-classify", "think-reflect"],
        "随 CLI 发布的 Skill 应当只有这三条"
    );
}
