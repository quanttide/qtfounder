//! 句法层：句太长、定语链太长、一逗到底、名词化堆叠、段落过长，加两条句式。

use super::common::{excerpt_of, hanzi_count};
use super::contract::{Contract, Syntax as SyntaxConfig};
use super::model::{Finding, Kind, Severity};
use regex::Regex;
pub(super) fn check_syntax(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let config = &contract.syntax;
    if syntax_is_off(config) {
        return Vec::new();
    }
    let patterns = SyntaxPatterns::new(config);
    let mut out = Vec::new();
    let mut in_code = false;
    let mut in_front = false;
    for (lineno, raw) in text.lines().enumerate() {
        let trimmed = raw.trim();
        if trimmed.starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if trimmed == "---" {
            in_front = !in_front;
            continue;
        }
        // 表格行、代码块、frontmatter、标题都不是「句子」：表格行按单元格拼接后必然超长
        if in_code
            || in_front
            || trimmed.starts_with('|')
            || trimmed.starts_with('#')
            || trimmed.is_empty()
        {
            continue;
        }
        if let Some(finding) = long_paragraph(config, rel, lineno, trimmed) {
            out.push(finding);
        }
        for sentence in raw.split(['。', '！', '？']) {
            out.extend(check_sentence(
                config,
                &patterns,
                rel,
                lineno,
                sentence.trim(),
            ));
        }
    }
    out
}

/// 每一项都可以用 0 或空表关掉；全关就不必往下走。
fn syntax_is_off(config: &SyntaxConfig) -> bool {
    config.max_sentence_chars == 0
        && config.max_de_chain == 0
        && config.max_commas == 0
        && config.max_nominal == 0
        && config.max_paragraph_chars == 0
        && config.push_to_reader_patterns.is_empty()
        && config.count_then_list_patterns.is_empty()
}

/// 三个正则集合：编译一次，逐句复用。
struct SyntaxPatterns {
    nominal: Regex,
    push_to_reader: Vec<Regex>,
    count_then_list: Vec<Regex>,
}

impl SyntaxPatterns {
    fn new(config: &SyntaxConfig) -> Self {
        let compile = |patterns: &Vec<String>| -> Vec<Regex> {
            patterns
                .iter()
                .filter_map(|pattern| Regex::new(pattern).ok())
                .collect()
        };
        Self {
            nominal: Regex::new(
                r"[\u{4e00}-\u{9fa5}]{2}(?:度|性|化|式)|[\u{4e00}-\u{9fa5}]{1,4}主义",
            )
            .expect("名词化正则写坏了：改 scan.rs 的 SyntaxPatterns::new，再跑 cargo test"),
            push_to_reader: compile(&config.push_to_reader_patterns),
            count_then_list: compile(&config.count_then_list_patterns),
        }
    }
}

fn long_paragraph(config: &SyntaxConfig, rel: &str, lineno: usize, line: &str) -> Option<Finding> {
    if config.max_paragraph_chars == 0 {
        return None;
    }
    let hanzi = hanzi_count(line);
    if hanzi <= config.max_paragraph_chars {
        return None;
    }
    Some(Finding::new(
        Kind::Syntax,
        Severity::Suspect,
        rel,
        lineno + 1,
        excerpt_of(line),
        format!(
            "这一段 {hanzi} 汉字（上限 {}）：拆成几段，每段一个意思",
            config.max_paragraph_chars
        ),
    ))
}

fn check_sentence(
    config: &SyntaxConfig,
    patterns: &SyntaxPatterns,
    rel: &str,
    lineno: usize,
    body: &str,
) -> Vec<Finding> {
    if body.is_empty() {
        return Vec::new();
    }
    let excerpt = excerpt_of(body);
    let metrics = Metrics {
        hanzi: hanzi_count(body),
        de: body.matches('的').count(),
        commas: body.matches('，').count(),
        nominal: patterns.nominal.find_iter(body).count(),
    };
    let mut out = threshold_findings(config, rel, lineno, &excerpt, &metrics);
    for advice in shape(patterns, body) {
        out.push(sentence_finding(
            rel,
            lineno,
            &excerpt,
            false,
            advice.to_string(),
        ));
    }
    out
}

/// 一句的四个计数：长短、定语链、逗号、名词化词。
struct Metrics {
    hanzi: usize,
    de: usize,
    commas: usize,
    nominal: usize,
}

/// 超过阈值的四项各报一条：句太长、定语链太长、一逗到底、名词化堆叠。
fn threshold_findings(
    config: &SyntaxConfig,
    rel: &str,
    lineno: usize,
    excerpt: &str,
    metrics: &Metrics,
) -> Vec<Finding> {
    let mut out = Vec::new();
    if config.max_sentence_chars > 0 && metrics.hanzi > config.max_sentence_chars {
        let over =
            config.max_sentence_error_chars > 0 && metrics.hanzi > config.max_sentence_error_chars;
        out.push(sentence_finding(
            rel,
            lineno,
            excerpt,
            over,
            format!(
                "一句 {} 汉字（他在 {} 字以上只占 7%）：切成两句，或把并列项改成列表",
                metrics.hanzi, config.max_sentence_chars
            ),
        ));
    }
    if config.max_de_chain > 0 && metrics.de > config.max_de_chain {
        out.push(sentence_finding(
            rel,
            lineno,
            excerpt,
            false,
            format!(
                "一句里「的」{} 个（上限 {}）：定语链太长，拆成两句或改成短定语",
                metrics.de, config.max_de_chain
            ),
        ));
    }
    if config.max_commas > 0 && metrics.commas > config.max_commas {
        out.push(sentence_finding(
            rel,
            lineno,
            excerpt,
            false,
            format!(
                "一句里逗号 {} 个（上限 {}）：一逗到底，断句或改列表",
                metrics.commas, config.max_commas
            ),
        ));
    }
    if config.max_nominal > 0 && metrics.nominal > config.max_nominal {
        out.push(sentence_finding(
            rel,
            lineno,
            excerpt,
            false,
            format!(
                "一句里名词化词 {} 个（上限 {}）：把动作写成名词，读者要多绕一层",
                metrics.nominal, config.max_nominal
            ),
        ));
    }
    out
}

fn sentence_finding(
    rel: &str,
    lineno: usize,
    excerpt: &str,
    over: bool,
    advice: String,
) -> Finding {
    let severity = if over {
        Severity::Error
    } else {
        Severity::Suspect
    };
    Finding::new(
        Kind::Syntax,
        severity,
        rel,
        lineno + 1,
        excerpt.to_string(),
        advice,
    )
}

/// 按句式报：把结论推回给读者、用件数代替内容。
fn shape(patterns: &SyntaxPatterns, body: &str) -> Vec<&'static str> {
    let mut out = Vec::new();
    if patterns
        .push_to_reader
        .iter()
        .any(|pattern| pattern.is_match(body))
    {
        out.push("「由你」把结论推回给读者：写出自己的判断和理由；要留选择就说清在选什么");
    }
    if patterns
        .count_then_list
        .iter()
        .any(|pattern| pattern.is_match(body))
    {
        out.push("用件数代替内容（「三件事」）：件数是空的，直接把条目写出来");
    }
    out
}
