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

/// 复核结论：这一条是规则本身导致的误报，还是要改的真问题。
/// 只收机判得了的三种误报形态；体裁错配、词的多义这类判不了，仍归人判。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    /// 真问题
    Real,
    /// 提及，不是使用：词被引号包着讨论（「判据」→ 验收标准 这种段落）
    Mention,
    /// 规则切分错：加粗小标题被并进了句子
    LeadIn,
    /// 引用工具自己的输出（「合计 64 处（词汇层 9…）」）
    QuotedOutput,
}

impl Verdict {
    pub fn key(self) -> &'static str {
        match self {
            Verdict::Real => "real",
            Verdict::Mention => "mention",
            Verdict::LeadIn => "lead-in",
            Verdict::QuotedOutput => "quoted-output",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Verdict::Real => "真问题",
            Verdict::Mention => "误报·提及",
            Verdict::LeadIn => "误报·标题并入",
            Verdict::QuotedOutput => "误报·引用输出",
        }
    }

    pub fn is_false_positive(self) -> bool {
        self != Verdict::Real
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
    pub verdict: Verdict,
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
            verdict: Verdict::Real,
        }
    }

    /// 复核层标一次：这条是规则导致的误报，还是真问题。
    pub fn mark(&mut self, verdict: Verdict) {
        self.verdict = verdict;
    }
}
