//! 七层检查：落点、自造术语、体系词、数字、句法、表格、语用。
//!
//! 验收标准只有能机判的才进来。逻辑跳跃、这句话是谁写的、这篇该放哪层——判不了，不进这里；
//! 误报多了验收标准就被忽略。

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;

use super::contract::{Contract, Syntax as SyntaxConfig};
use super::model::{Finding, Kind, Severity, Verdict};

static QUOTED_OUTPUT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"合计\s*\d+\s*处").expect("引用输出正则写坏了：改 QUOTED_OUTPUT，再跑 cargo test")
});

const COMMON_ACRONYMS: [&str; 34] = [
    "AI", "LLM", "JSON", "YAML", "URL", "URI", "HTTP", "HTTPS", "API", "SQL", "HTML", "CSS", "PDF",
    "CSV", "ID", "KB", "MB", "GB", "TB", "UI", "UX", "CPU", "RAM", "GPU", "SDK", "DNS", "TLS",
    "SSL", "UTC", "GMT", "OK", "IP", "OS", "PC",
];

fn hanzi_count(text: &str) -> usize {
    text.chars()
        .filter(|ch| ('\u{4e00}'..='\u{9fa5}').contains(ch))
        .count()
}

fn rel_of(path: &Path, root: Option<&Path>) -> String {
    match root {
        Some(root) => path
            .strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string(),
        None => path.display().to_string(),
    }
}

fn in_dir(rel: &str, prefix: &str) -> bool {
    rel == prefix || rel.starts_with(&format!("{}/", prefix.trim_end_matches('/')))
}

/// exclude 条目：带 `/` 的按路径前缀算，不带的按同名文件或目录算（任意层级）。
/// 这样 `CHANGELOG.md`、`node_modules` 这类通用约定写一条就够，不会漏掉嵌套的那几份。
fn is_excluded(rel: &str, contract: &Contract) -> bool {
    contract.scope.exclude.iter().any(|skip| {
        if skip.contains('/') {
            in_dir(rel, skip)
        } else {
            rel.split('/').any(|part| part == skip)
        }
    })
}

/// 落点：文件放错了层。
fn check_placement(rel: &str, contract: &Contract) -> Vec<Finding> {
    let mut out = Vec::new();
    for (bad, why) in &contract.placement.banned_paths {
        if in_dir(rel, bad) {
            out.push(Finding::new(
                Kind::Placement,
                Severity::Error,
                rel,
                0,
                bad.clone(),
                why.clone(),
            ));
        }
    }
    out
}

/// 词汇层：自造术语。每次命中都带替代写法，否则读者不知道换成什么。
fn check_banned(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let mut out = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        for (word, entry) in &contract.banned {
            if line.contains(word.as_str()) {
                out.push(Finding::new(
                    Kind::Banned,
                    Severity::Error,
                    rel,
                    lineno + 1,
                    line.trim().chars().take(90).collect(),
                    format!("「{word}」→ {}", entry.advice),
                ));
            }
        }
    }
    out
}

/// 体系词：这类目录的文档里必须出现用户的说法，一次都不出现就是写歪了。
fn check_required(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let mut out = Vec::new();
    for (prefix, words) in &contract.required {
        if !in_dir(rel, prefix) {
            continue;
        }
        if words.iter().any(|word| text.contains(word.as_str())) {
            continue;
        }
        out.push(Finding::new(
            Kind::Required,
            Severity::Error,
            rel,
            0,
            format!("{rel} 属 {prefix}"),
            format!(
                "这类目录的文档里没有出现：{}。要么补一段判例结构（现象／错法／正法），要么说明它是做法型内容、该迁 handbook",
                words.join("／")
            ),
        ));
    }
    out
}

/// 数字：嫌疑不是判决——分母与来源都缺才报。
fn check_numbers(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let config = &contract.numbers;
    if config.denominator_patterns.is_empty() && config.source_words.is_empty() {
        return Vec::new();
    }
    // 只收「论据数字」：百分比、总数计数。描述用的数字（三四个人、最多十条、第 10 行、
    // 出现 1 次才算类型）不在此列——读者要的是量感，不问出处，要求分母反而啰嗦。
    let evidence_number = Regex::new(r"\d+(?:\.\d+)?\s*%|\d[\d,]*\s*(?:处|次|条|份|行)")
        .expect("论据数字正则写坏了：改 scan.rs 的 evidence_number，再跑 cargo test");
    let denominators: Vec<Regex> = config
        .denominator_patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect();

    let mut out = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        let hits: Vec<&str> = evidence_number
            .find_iter(line)
            .filter(|found| !is_descriptive_number(line, found.start(), found.end()))
            .map(|found| found.as_str())
            .collect();
        if hits.is_empty() {
            continue;
        }
        let sourced = denominators.iter().any(|pattern| pattern.is_match(line))
            || config
                .source_words
                .iter()
                .any(|word| line.contains(word.as_str()));
        if sourced {
            continue;
        }
        out.push(Finding::new(
            Kind::Numbers,
            Severity::Suspect,
            rel,
            lineno + 1,
            line.trim().chars().take(90).collect(),
            format!(
                "「{}」是论据数字，却没有分母也没有出处：补「共 N」「占 N/M」、文件:行、实测日期，否则读者无从复核",
                hits.join("」「")
            ),
        ));
    }
    out
}

/// 描述用的数字：阈值、上限、序数、估算、区间。这些不是论据，不该要求分母。
fn is_descriptive_number(line: &str, start: usize, end: usize) -> bool {
    let before: String = line[..start].chars().rev().take(8).collect();
    let before: String = before.chars().rev().collect();
    // 「第 10 行」中间常有空格，去掉尾空格再比对
    let before = before.trim_end();
    let after: String = line[end..].chars().take(3).collect();
    // 前面的限定词：多字的按包含算（「最多展示 10 条」），单字的按紧邻算（「约 20 处」）
    const LEAD_WORDS: [&str; 8] = ["最多", "至少", "至多", "不超过", "大概", "不到", "只", "才"];
    const LEAD_CHARS: [&str; 7] = ["约", "近", "第", "≥", "≤", "<", ">"];
    // 后面的模糊词与区间：多、来、左右、上下、以上/以下/以内、区间号，以及阈值句式的「只算／才算」
    const TAILS: [&str; 15] = [
        "多", "来", "左右", "上下", "以上", "以下", "以内", "–", "-", "~", "只算", "才算", "就算",
        "即算", "封顶",
    ];
    LEAD_WORDS.iter().any(|lead| before.contains(lead))
        || LEAD_CHARS.iter().any(|lead| before.ends_with(lead))
        || TAILS.iter().any(|tail| after.starts_with(tail))
}

/// 句法层：句子太长、长定语链、一逗到底、名词化堆叠、段落过长。0 = 关闭。
fn check_syntax(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
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

fn excerpt_of(body: &str) -> String {
    body.chars().take(70).collect()
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

/// 表格层：列数、单元格长度、行数、有无表头，以及「这段内容根本不需要表格」。
fn check_table(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let config = &contract.table;
    let active = config.max_columns > 0
        || config.max_cell_chars > 0
        || config.max_rows > 0
        || config.require_header
        || config.flag_two_column
        || config.min_rows > 0
        || config.sparse_ratio > 0.0
        || !config.layout_headers.is_empty();
    if !active {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut block: Vec<(usize, String)> = Vec::new();
    let mut in_code = false;

    let flush = |block: &mut Vec<(usize, String)>, out: &mut Vec<Finding>| {
        if block.len() < 2 {
            block.clear();
            return;
        }
        let start = block[0].0;
        let rows: Vec<&str> = block.iter().map(|(_, line)| line.as_str()).collect();
        let cells: Vec<Vec<String>> = rows
            .iter()
            .map(|row| {
                row.trim()
                    .trim_matches('|')
                    .split('|')
                    .map(|cell| cell.trim().to_string())
                    .collect()
            })
            .collect();
        let columns = cells.iter().map(|row| row.len()).max().unwrap_or(0);
        let has_header = cells.len() > 1
            && cells[1].iter().all(|cell| {
                !cell.is_empty() && cell.chars().all(|ch| ch == '-' || ch == ':' || ch == ' ')
            });

        let first: String = rows[0].chars().take(70).collect();
        let mut push = |line: usize, excerpt: String, advice: String| {
            out.push(Finding::new(
                Kind::Table,
                Severity::Suspect,
                rel,
                line,
                excerpt,
                advice,
            ));
        };

        if config.require_header && !has_header {
            push(
                start,
                first.clone(),
                "表格第二行不是分隔行（|---|）：读者不知道每列是什么".to_string(),
            );
        }
        if config.max_columns > 0 && columns > config.max_columns {
            push(
                start,
                first.clone(),
                format!(
                    "这张表 {columns} 列（上限 {}）：手机上要横滚，删列或拆表",
                    config.max_columns
                ),
            );
        }
        let body_rows = if has_header {
            rows.len() - 2
        } else {
            rows.len()
        };
        if config.max_rows > 0 && body_rows > config.max_rows {
            push(
                start,
                first.clone(),
                format!(
                    "这张表 {body_rows} 行（上限 {}）：拆表或改成列表",
                    config.max_rows
                ),
            );
        }
        if config.flag_two_column && columns == 2 {
            push(
                start,
                first.clone(),
                "两列表：本质是「名词 + 一段解释」，改成列表更自然（- **名词**：解释）".to_string(),
            );
        }
        if config.min_rows > 0 && body_rows < config.min_rows {
            push(
                start,
                first.clone(),
                format!("这张表只有 {body_rows} 行数据：等于一句「A 是 B」，不必用表"),
            );
        }
        let body: &[Vec<String>] = if has_header { &cells[2..] } else { &cells[..] };
        let flat: Vec<&String> = body.iter().flatten().collect();
        if config.sparse_ratio > 0.0 && !flat.is_empty() {
            let empty = flat.iter().filter(|cell| cell.is_empty()).count();
            let ratio = empty as f64 / flat.len() as f64;
            if ratio > config.sparse_ratio {
                push(
                    start,
                    first.clone(),
                    format!(
                        "空单元格 {empty}/{}（{:.0}%）：内容撑不起这个形状，改用列表",
                        flat.len(),
                        ratio * 100.0
                    ),
                );
            }
        }
        if !config.layout_headers.is_empty()
            && let Some(headers) = cells.first()
            && let Some(name) = headers
                .iter()
                .find(|name| config.layout_headers.contains(name))
        {
            push(
                start,
                first.clone(),
                format!("表头「{name}」是排版词不是属性名：这张表在做排版，不是在对照"),
            );
        }
        if config.max_cell_chars > 0 {
            for (index, row) in cells.iter().enumerate() {
                for cell in row {
                    let count = hanzi_count(cell);
                    if count > config.max_cell_chars {
                        push(
                            start + index,
                            cell.chars().take(70).collect(),
                            format!(
                                "单元格 {count} 汉字（上限 {}）：把段落从格子里拿出来写进正文",
                                config.max_cell_chars
                            ),
                        );
                    }
                }
            }
        }
        block.clear();
    };

    for (lineno, raw) in text.lines().enumerate() {
        let trimmed = raw.trim();
        if trimmed.starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if !in_code && trimmed.starts_with('|') {
            block.push((lineno + 1, trimmed.to_string()));
            continue;
        }
        flush(&mut block, &mut out);
    }
    flush(&mut block, &mut out);
    out
}

/// 抽一段里的内部标识符，同一个东西只算一次。
///
/// 踩过：`a/b.md` 会被三条正则各命中一次，计出三个假 ID 把短段落也报了。
/// 做法是先摘出反引号内容、其余从剥掉反引号的文本里找，最后去掉互为子串的重复。
fn internal_ids(body: &str) -> HashSet<String> {
    static PATTERNS: std::sync::OnceLock<Vec<Regex>> = std::sync::OnceLock::new();
    // 注意：Rust 的 regex 不支持前瞻，所以「提交号必须含字母」「路径要带 ASCII」这两条
    // 从正则里挪到下面的代码判——语义等价，别想着换回 look-ahead。
    let patterns = PATTERNS.get_or_init(|| {
        vec![
            Regex::new(r"\b[0-9a-f]{7,40}\b")
                .expect("十六进制正则写坏了：改 internal_ids，再跑 cargo test"),
            Regex::new(r"[\w\-]+\.(?:md|py|rs|go|dart|toml|yaml|yml|json|sh|lock)\b")
                .expect("文件名正则写坏了：改 internal_ids，再跑 cargo test"),
            Regex::new(r"\b[A-Z]{2,}\d*\b")
                .expect("缩写正则写坏了：改 internal_ids，再跑 cargo test"),
            Regex::new(r"[\w.\-]+/[\w./\-]+")
                .expect("路径正则写坏了：改 internal_ids，再跑 cargo test"),
        ]
    });
    let backtick =
        Regex::new(r"`[^`\n]+`").expect("反引号配对正则写坏了：改 internal_ids，再跑 cargo test");
    let chinese_only = Regex::new(r"^[\u{4e00}-\u{9fa5}，。：；、「」（）《》\s]+$")
        .expect("纯中文判定正则写坏了：改 internal_ids，再跑 cargo test");

    // 反引号里如果全是中文，那是术语强调（`正本`、`验收标准`），不是内部代号
    let mut ids: HashSet<String> = HashSet::new();
    let mut kept: Vec<String> = Vec::new();
    for found in backtick.find_iter(body) {
        let raw = found.as_str().trim_matches('`').trim();
        if chinese_only.is_match(raw) {
            kept.push(found.as_str().to_string());
        } else {
            ids.insert(raw.to_string());
            kept.push(" ".to_string());
        }
    }
    let rest = if kept.is_empty() {
        body.to_string()
    } else {
        let mut result = String::new();
        let mut cursor = 0;
        let matches: Vec<_> = backtick.find_iter(body).collect();
        for (index, found) in matches.iter().enumerate() {
            result.push_str(&body[cursor..found.start()]);
            result.push_str(&kept[index]);
            cursor = found.end();
        }
        result.push_str(&body[cursor..]);
        result
    };

    for (index, pattern) in patterns.iter().enumerate() {
        for found in pattern.find_iter(&rest) {
            let item = found.as_str().to_string();
            if COMMON_ACRONYMS.contains(&item.as_str()) {
                continue;
            }
            // 提交号必须含字母 a-f：纯数字串是秒数或数据库 id，不是 sha
            if index == 0 && !item.chars().any(|ch| ch.is_ascii_alphabetic()) {
                continue;
            }
            // 斜杠两侧至少一侧带 ASCII：「一/二」「A/B」不是路径
            if index == 3 && !item.chars().any(|ch| ch.is_ascii_alphanumeric()) {
                continue;
            }
            ids.insert(item);
        }
    }
    // 去掉互为子串的重复：长的留着，短的扔掉，否则同一个东西被算成两三个 ID
    let all: Vec<String> = ids.into_iter().collect();
    all.iter()
        .filter(|item| {
            !all.iter()
                .any(|other| other != *item && other.contains(item.as_str()))
        })
        .cloned()
        .collect()
}

/// 语用层：这段是不是在跟读者说话——内部标识符密度高且一句读者指代都没有。
fn check_voice(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let config = &contract.voice;
    if config.max_internal_ids == 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut paragraph: Vec<String> = Vec::new();
    let mut start = 0usize;
    let mut in_code = false;

    let flush = |paragraph: &mut Vec<String>, start: usize, out: &mut Vec<Finding>| {
        if paragraph.is_empty() {
            return;
        }
        let body = paragraph.join(" ");
        let ids = internal_ids(&body);
        let has_reader = config
            .reader_words
            .iter()
            .any(|word| body.contains(word.as_str()));
        if ids.len() > config.max_internal_ids && !has_reader {
            let mut sample: Vec<String> = ids.iter().take(6).cloned().collect();
            sample.sort();
            out.push(Finding::new(
                Kind::Voice,
                Severity::Suspect,
                rel,
                start,
                body.chars().take(70).collect(),
                format!(
                    "这一段有 {} 个内部标识符（{}…）却没有一处对读者说话：要么逐个说明它是什么，要么这段该进日志",
                    ids.len(),
                    sample.join("、")
                ),
            ));
        }
        paragraph.clear();
    };

    for (lineno, raw) in text.lines().enumerate() {
        let trimmed = raw.trim();
        if trimmed.starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code || trimmed.starts_with('|') || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.is_empty() {
            flush(&mut paragraph, start, &mut out);
            continue;
        }
        if paragraph.is_empty() {
            start = lineno + 1;
        }
        paragraph.push(trimmed.to_string());
    }
    flush(&mut paragraph, start, &mut out);
    out
}

/// 一个文件过七层。
pub fn check_file(path: &Path, contract: &Contract, root: Option<&Path>) -> Vec<Finding> {
    let rel = rel_of(path, root);
    let text = fs::read_to_string(path).unwrap_or_default();
    let mut findings = check_placement(&rel, contract);
    findings.extend(check_banned(&rel, &text, contract));
    findings.extend(check_required(&rel, &text, contract));
    findings.extend(check_numbers(&rel, &text, contract));
    findings.extend(check_syntax(&rel, &text, contract));
    findings.extend(check_table(&rel, &text, contract));
    findings.extend(check_voice(&rel, &text, contract));
    let lines: Vec<&str> = text.lines().collect();
    for finding in &mut findings {
        let line = lines
            .get(finding.line.saturating_sub(1))
            .copied()
            .unwrap_or("");
        judge(finding, line);
    }
    findings
}

/// 复核层：命中的这一条，是规则本身导致的误报，还是要改的真问题。
/// 只判机判得了的三种形态；判不了的不在这里假装判过。
fn judge(finding: &mut Finding, line: &str) {
    let verdict = match finding.kind {
        // 提及不是使用：词被引号包着讨论，比如「判据」→ 验收标准 这种段落
        Kind::Banned if quoted_word(&finding.advice).is_some_and(|word| is_quoted(line, word)) => {
            Verdict::Mention
        }
        // 引用工具自己的输出：合计 64 处（词汇层 9…）
        Kind::Numbers if QUOTED_OUTPUT.is_match(line) => Verdict::QuotedOutput,
        // 切分错：加粗小标签被并进了句子，句首的「**…**：」不是正文
        Kind::Syntax if line.trim_start().starts_with("**") && line.contains("**：") => {
            Verdict::LeadIn
        }
        _ => Verdict::Real,
    };
    finding.mark(verdict);
}

/// advice 形如「「判据」→ 验收标准」，取出被引的那个词。
fn quoted_word(advice: &str) -> Option<&str> {
    let rest = advice.strip_prefix('「')?;
    let end = rest.find('」')?;
    Some(&rest[..end])
}

/// 词在行里只当「被引的东西」出现：包在「」或反引号里，或作为反引号内术语的一部分
/// （`判定层` 这种组件名，词本身没被单独拿来用）。
fn is_quoted(line: &str, word: &str) -> bool {
    if line.contains(&format!("「{word}」")) || line.contains(&format!("`{word}`")) {
        return true;
    }
    let mut rest = line;
    while let Some(open) = rest.find('`') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('`') else { break };
        if after[..close].contains(word) {
            return true;
        }
        rest = &after[close + 1..];
    }
    false
}

/// 扫描范围：`default` 审全量但跳过 `[scope] exclude`（流水、第三方依赖）；`all` 连 exclude 也不放过。
/// `include_default` 为空 = 不按目录挑；仓自己的契约可以填前缀把范围收窄。
pub fn markdown_files(root: &Path, contract: &Contract, scope: &str) -> Vec<PathBuf> {
    if root.is_file() {
        return vec![root.to_path_buf()];
    }
    let includes = &contract.scope.include_default;
    let narrow = !includes.is_empty();

    let mut out: Vec<PathBuf> = WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .filter(|path| path.extension().map(|ext| ext == "md").unwrap_or(false))
        .filter(|path| {
            if scope == "all" {
                return true;
            }
            !is_excluded(&rel_of(path, Some(root)), contract)
        })
        .filter(|path| {
            // 空 include 就是不挑目录：审什么由 root 决定，不再由工具内嵌的目录名单决定
            if scope == "all" || !narrow {
                return true;
            }
            let rel = rel_of(path, Some(root));
            // root 本身可能在深层（如 docs/handbook）：祖先目录里出现过 include 前缀就算在范围内
            let ancestor_hit = root
                .components()
                .filter_map(|part| part.as_os_str().to_str())
                .any(|part| {
                    includes
                        .iter()
                        .any(|prefix| !prefix.is_empty() && part == prefix.trim_end_matches('/'))
                });
            if ancestor_hit {
                return true;
            }
            includes
                .iter()
                .any(|prefix| !prefix.is_empty() && in_dir(&rel, prefix))
        })
        .collect();
    out.sort();
    out
}

/// 全量扫描：返回（命中，读不了的文件，审了几个文件）。
pub fn scan(root: &Path, contract: &Contract, scope: &str) -> (Vec<Finding>, Vec<String>, usize) {
    let mut findings = Vec::new();
    let mut broken = Vec::new();
    let base = if root.is_dir() { Some(root) } else { None };
    let files = markdown_files(root, contract, scope);
    let scanned = files.len();
    for path in files {
        if fs::read_to_string(&path).is_err() {
            broken.push(rel_of(&path, base));
            continue;
        }
        findings.extend(check_file(&path, contract, base));
    }
    (findings, broken, scanned)
}

/// 按类计数，给汇总行用。
pub fn summarize(findings: &[Finding]) -> HashMap<&'static str, usize> {
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    for finding in findings {
        *counts.entry(finding.kind.key()).or_insert(0) += 1;
    }
    counts
}
