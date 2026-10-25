//! 一条表达命中：层 → 位置 → 问题 → 改法。

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Empty,
    Slogan,
    Reference,
    Structure,
    Genre,
    Audience,
    Topic,
}

impl Kind {
    pub fn key(self) -> &'static str {
        match self {
            Kind::Empty => "empty",
            Kind::Slogan => "slogan",
            Kind::Reference => "reference",
            Kind::Structure => "structure",
            Kind::Genre => "genre",
            Kind::Audience => "audience",
            Kind::Topic => "topic",
        }
    }

    pub fn parse(text: &str) -> Option<Kind> {
        [
            Kind::Empty,
            Kind::Slogan,
            Kind::Reference,
            Kind::Structure,
            Kind::Genre,
            Kind::Audience,
            Kind::Topic,
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
    pub fn key(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Suspect => "suspect",
        }
    }

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
    pub line: Option<usize>,
    pub excerpt: String,
    pub advice: String,
}

impl Finding {
    pub fn new(
        kind: Kind,
        severity: Severity,
        path: &str,
        line: Option<usize>,
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
