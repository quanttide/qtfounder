//! 一条命中长什么样。与 `code` 那套保持同一个形状：层 → 问题 → 依据 → 建议。

/// 六类检查，名字与契约里的段一一对应。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Placement,
    Banned,
    Required,
    Numbers,
    Syntax,
    Table,
    Voice,
}

impl Kind {
    pub fn key(self) -> &'static str {
        match self {
            Kind::Placement => "placement",
            Kind::Banned => "banned",
            Kind::Required => "required",
            Kind::Numbers => "numbers",
            Kind::Syntax => "syntax",
            Kind::Table => "table",
            Kind::Voice => "voice",
        }
    }

    pub fn parse(text: &str) -> Option<Kind> {
        [
            Kind::Placement,
            Kind::Banned,
            Kind::Required,
            Kind::Numbers,
            Kind::Syntax,
            Kind::Table,
            Kind::Voice,
        ]
        .into_iter()
        .find(|kind| kind.key() == text)
    }

    pub const ALL: [Kind; 7] = [
        Kind::Placement,
        Kind::Banned,
        Kind::Required,
        Kind::Numbers,
        Kind::Syntax,
        Kind::Table,
        Kind::Voice,
    ];
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
