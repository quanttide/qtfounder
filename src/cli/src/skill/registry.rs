//! 编译期嵌入的 Skill 清单，由 `build.rs` 生成。
//!
//! 生成文件在 `OUT_DIR`：嵌的是本包 `skills/` 里的全部 Skill，随二进制一起发布。
//! 改来源用 `QTFOUNDER_SKILLS_DIR`。

use quanttide_founder::agent::skills::EmbeddedSkill;

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
