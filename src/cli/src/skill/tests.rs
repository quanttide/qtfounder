//! 嵌入清单的冒烟测试：能注册，才装得出去。
//!
//! 清单由 `build.rs` 生成：从主仓构建时有九条，单独 clone 这个仓时为空。
//! 空清单不算错，但来源存在就必须有条目。

use quanttide_founder::skills;

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
fn 主仓构建时带上归位与提交两个入口() {
    let ids: Vec<&str> = registry::all().iter().map(|entry| entry.id).collect();
    for expected in ["meta-classify", "git-commit"] {
        if ids.is_empty() {
            return;
        }
        assert!(ids.contains(&expected), "清单里缺 {expected}：{ids:?}");
    }
}
