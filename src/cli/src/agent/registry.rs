//! 编译期嵌入的 Skill 清单，由 `build.rs` 生成。
//!
//! 生成文件在 `OUT_DIR`：嵌的是本包 `skills/` 里的全部 Skill，随二进制一起发布。
//! 改来源用 `QTFOUNDER_SKILLS_DIR`。

use quanttide_founder::agent::skill::EmbeddedSkill;

include!(concat!(env!("OUT_DIR"), "/embedded_skills.rs"));

/// 全部嵌入的 Skill，按目录名排序。
pub fn all() -> &'static [EmbeddedSkill] {
    SKILLS
}

/// 嵌入来源；空清单时为 `None`。
pub fn source() -> Option<&'static str> {
    SOURCE
}

pub fn find(id: &str) -> Option<&'static EmbeddedSkill> {
    SKILLS.iter().find(|entry| entry.id == id)
}

#[cfg(test)]
mod tests {
    use quanttide_founder::agent::skill;

    #[test]
    fn 嵌进来的每条都能注册() {
        for entry in super::all() {
            let summary = skill::describe(entry).unwrap_or_else(|error| panic!("{error}"));
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
        if let Some(source) = super::source() {
            assert!(!super::all().is_empty(), "来源 {source} 下一条都没读到");
        }
    }

    #[test]
    fn 嵌入清单只有九条() {
        let ids: Vec<&str> = super::all().iter().map(|entry| entry.id).collect();
        if ids.is_empty() {
            return;
        }
        assert_eq!(
            ids,
            vec![
                "design-review",
                "docs-format",
                "execute-collect",
                "language-govern",
                "meta-classify",
                "product-collect",
                "product-organize",
                "think-reflect",
                "write-express",
            ],
            "随 CLI 发布的 Skill 应当只有这九条"
        );
    }
}
