//! 一条命中长什么样。四层共用同一个结构，输出才整齐。

/// 四层检查：词汇、句法、语篇、语用。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layer {
    Lexical,
    Syntax,
    Structure,
    Pragmatic,
}

impl Layer {
    pub fn key(self) -> &'static str {
        match self {
            Layer::Lexical => "lexical",
            Layer::Syntax => "syntax",
            Layer::Structure => "structure",
            Layer::Pragmatic => "pragmatic",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Layer::Lexical => "词汇层",
            Layer::Syntax => "句法层",
            Layer::Structure => "语篇层",
            Layer::Pragmatic => "语用层",
        }
    }

    pub fn parse(text: &str) -> Option<Layer> {
        match text {
            "lexical" => Some(Layer::Lexical),
            "syntax" => Some(Layer::Syntax),
            "structure" => Some(Layer::Structure),
            "pragmatic" => Some(Layer::Pragmatic),
            _ => None,
        }
    }

    pub const ALL: [Layer; 4] = [
        Layer::Lexical,
        Layer::Syntax,
        Layer::Structure,
        Layer::Pragmatic,
    ];
}

/// Grice 的四条合作原则：每条命中都要说清违反了哪一条。
#[derive(Clone, Copy, Debug)]
pub enum Principle {
    Quantity,
    Quality,
    Relation,
    Manner,
}

impl Principle {
    pub fn label(self) -> &'static str {
        match self {
            Principle::Quantity => "量准则（信息量适中）",
            Principle::Quality => "质准则（不说没依据的话）",
            Principle::Relation => "关系准则（只说相关的）",
            Principle::Manner => "方式准则（清晰无歧义）",
        }
    }
}

/// 命中的一条。
#[derive(Clone, Debug)]
pub struct Finding {
    pub layer: Layer,
    pub kind: &'static str,
    pub path: String,
    pub line: usize,
    pub message: String,
    pub principle: Principle,
    pub advice: &'static str,
}

impl Finding {
    pub fn new(
        layer: Layer,
        kind: &'static str,
        path: &str,
        line: usize,
        message: String,
        principle: Principle,
        advice: &'static str,
    ) -> Self {
        Self {
            layer,
            kind,
            path: path.to_string(),
            line,
            message,
            principle,
            advice,
        }
    }
}
