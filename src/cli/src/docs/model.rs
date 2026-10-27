//! 一条命中长什么样：层 → 位置 → 问题 → 替代动作。
//!
//! 三要素缺一不可——只有行号没有替代动作，报告退化成数字，读者拿到仍然不知道改哪。

/// 六类检查，名字与契约里的段一一对应。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Table,
    List,
    Density,
    Bold,
    Rule,
    Frontmatter,
    Journal,
}

impl Kind {
    pub fn key(self) -> &'static str {
        match self {
            Kind::Table => "table",
            Kind::List => "list",
            Kind::Density => "density",
            Kind::Bold => "bold",
            Kind::Rule => "rule",
            Kind::Frontmatter => "frontmatter",
            Kind::Journal => "journal",
        }
    }

    pub fn parse(text: &str) -> Option<Kind> {
        [
            Kind::Table,
            Kind::List,
            Kind::Density,
            Kind::Bold,
            Kind::Rule,
            Kind::Frontmatter,
            Kind::Journal,
        ]
        .into_iter()
        .find(|kind| kind.key() == text)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Severity {
    Error,
    Suspect,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "ERROR",
            Severity::Suspect => "嫌疑",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Finding {
    pub kind: Kind,
    pub severity: Severity,
    pub path: String,
    pub line: usize,
    pub excerpt: String,
    pub advice: String,
}

impl Finding {
    pub fn new(
        kind: Kind,
        severity: Severity,
        path: &str,
        line: usize,
        excerpt: String,
        advice: String,
    ) -> Self {
        Self {
            kind,
            severity,
            path: path.to_string(),
            line,
            excerpt,
            advice,
        }
    }
}
