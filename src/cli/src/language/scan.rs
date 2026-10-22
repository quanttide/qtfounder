//! 七层检查：落点、自造术语、体系词、数字、句法、表格、语用。
//!
//! 判据只有能机判的才进来。逻辑跳跃、这句话是谁写的、这篇该放哪层——判不了，不进这里；
//! 误报多了判据就被忽略。

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;

use super::contract::Contract;
use super::model::{Finding, Kind, Severity};

const COMMON_ACRONYMS: [&str; 34] = [
    "AI", "LLM", "JSON", "YAML", "URL", "URI", "HTTP", "HTTPS", "API", "SQL", "HTML", "CSS", "PDF", "CSV", "ID",
    "KB", "MB", "GB", "TB", "UI", "UX", "CPU", "RAM", "GPU", "SDK", "DNS", "TLS", "SSL", "UTC", "GMT", "OK", "IP",
    "OS", "PC",
];

fn hanzi_count(text: &str) -> usize {
    text.chars().filter(|ch| ('\u{4e00}'..='\u{9fa5}').contains(ch)).count()
}

fn rel_of(path: &Path, root: Option<&Path>) -> String {
    match root {
        Some(root) => path.strip_prefix(root).unwrap_or(path).display().to_string(),
        None => path.display().to_string(),
    }
}

fn in_dir(rel: &str, prefix: &str) -> bool {
    rel == prefix || rel.starts_with(&format!("{}/", prefix.trim_end_matches('/')))
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
    if config.denominator_patterns.is_empty() {
        return Vec::new();
    }
    let number = Regex::new(r"(\d+(?:\.\d+)?)\s*%").expect("百分比正则");
    let count = Regex::new(r"(\d[\d,]*)\s*(?:处|次|条|份|行)").expect("计数正则");
    // 阈值与序数不是统计量：`≥2 次`、`第 2 次浮出`、`N 次阈值` 都不需要分母
    let non_stat = Regex::new(r"(?:[≥≤<>＞＜]\s*\d+|第\s*\d+\s*(?:次|步|轮|阶段)|\d+\s*次(?:阈值|为限|以内|以上))")
        .expect("非统计正则");
    let denominators: Vec<Regex> = config
        .denominator_patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect();

    let mut out = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        let stripped = non_stat.replace_all(line, "");
        if !number.is_match(&stripped) && !count.is_match(&stripped) {
            continue;
        }
        if denominators.iter().any(|pattern| pattern.is_match(line)) {
            continue;
        }
        if config.source_words.iter().any(|word| line.contains(word.as_str())) {
            continue;
        }
        out.push(Finding::new(
            Kind::Numbers,
            Severity::Suspect,
            rel,
            lineno + 1,
            line.trim().chars().take(90).collect(),
            "数字没有分母也没有来源：补「共 N」「占 N/M」或实测出处，否则读者无从复核".to_string(),
        ));
    }
    out
}

/// 句法层：句子太长、长定语链、一逗到底、名词化堆叠、段落过长。0 = 关闭。
fn check_syntax(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let config = &contract.syntax;
    if config.max_sentence_chars == 0
        && config.max_de_chain == 0
        && config.max_commas == 0
        && config.max_nominal == 0
        && config.max_paragraph_chars == 0
    {
        return Vec::new();
    }
    let nominal = Regex::new(r"[\u{4e00}-\u{9fa5}]{2}(?:度|性|化|式)|[\u{4e00}-\u{9fa5}]{1,4}主义").expect("名词化正则");

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
        if in_code || in_front || trimmed.starts_with('|') || trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        let paragraph_hanzi = hanzi_count(trimmed);
        if config.max_paragraph_chars > 0 && paragraph_hanzi > config.max_paragraph_chars {
            out.push(Finding::new(
                Kind::Syntax,
                Severity::Suspect,
                rel,
                lineno + 1,
                trimmed.chars().take(70).collect(),
                format!("这一段 {paragraph_hanzi} 汉字（上限 {}）：拆成几段，每段一个意思", config.max_paragraph_chars),
            ));
        }
        for sentence in raw.split(['。', '！', '？']) {
            let body = sentence.trim();
            if body.is_empty() {
                continue;
            }
            let count = hanzi_count(body);
            let excerpt: String = body.chars().take(70).collect();
            if config.max_sentence_chars > 0 && count > config.max_sentence_chars {
                let severity = if config.max_sentence_error_chars > 0 && count > config.max_sentence_error_chars {
                    Severity::Error
                } else {
                    Severity::Suspect
                };
                out.push(Finding::new(
                    Kind::Syntax,
                    severity,
                    rel,
                    lineno + 1,
                    excerpt.clone(),
                    format!(
                        "一句 {count} 汉字（他在 {} 字以上只占 7%）：切成两句，或把并列项改成列表",
                        config.max_sentence_chars
                    ),
                ));
            }
            if config.max_de_chain > 0 && body.matches('的').count() > config.max_de_chain {
                out.push(Finding::new(
                    Kind::Syntax,
                    Severity::Suspect,
                    rel,
                    lineno + 1,
                    excerpt.clone(),
                    format!(
                        "一句里「的」{} 个（上限 {}）：定语链太长，拆成两句或改成短定语",
                        body.matches('的').count(),
                        config.max_de_chain
                    ),
                ));
            }
            if config.max_commas > 0 && body.matches('，').count() > config.max_commas {
                out.push(Finding::new(
                    Kind::Syntax,
                    Severity::Suspect,
                    rel,
                    lineno + 1,
                    excerpt.clone(),
                    format!(
                        "一句里逗号 {} 个（上限 {}）：一逗到底，断句或改列表",
                        body.matches('，').count(),
                        config.max_commas
                    ),
                ));
            }
            if config.max_nominal > 0 {
                let hits = nominal.find_iter(body).count();
                if hits > config.max_nominal {
                    out.push(Finding::new(
                        Kind::Syntax,
                        Severity::Suspect,
                        rel,
                        lineno + 1,
                        excerpt,
                        format!("一句里名词化词 {hits} 个（上限 {}）：把动作写成名词，读者要多绕一层", config.max_nominal),
                    ));
                }
            }
        }
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
            && cells[1]
                .iter()
                .all(|cell| !cell.is_empty() && cell.chars().all(|ch| ch == '-' || ch == ':' || ch == ' '));

        let first: String = rows[0].chars().take(70).collect();
        let mut push = |line: usize, excerpt: String, advice: String| {
            out.push(Finding::new(Kind::Table, Severity::Suspect, rel, line, excerpt, advice));
        };

        if config.require_header && !has_header {
            push(start, first.clone(), "表格第二行不是分隔行（|---|）：读者不知道每列是什么".to_string());
        }
        if config.max_columns > 0 && columns > config.max_columns {
            push(
                start,
                first.clone(),
                format!("这张表 {columns} 列（上限 {}）：手机上要横滚，删列或拆表", config.max_columns),
            );
        }
        let body_rows = if has_header { rows.len() - 2 } else { rows.len() };
        if config.max_rows > 0 && body_rows > config.max_rows {
            push(
                start,
                first.clone(),
                format!("这张表 {body_rows} 行（上限 {}）：拆表或改成列表", config.max_rows),
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
            && let Some(name) = headers.iter().find(|name| config.layout_headers.contains(name))
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
                            format!("单元格 {count} 汉字（上限 {}）：把段落从格子里拿出来写进正文", config.max_cell_chars),
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
            Regex::new(r"\b[0-9a-f]{7,40}\b").expect("hex"),
            Regex::new(r"[\w\-]+\.(?:md|py|rs|go|dart|toml|yaml|yml|json|sh|lock)\b").expect("文件名"),
            Regex::new(r"\b[A-Z]{2,}\d*\b").expect("缩写"),
            Regex::new(r"[\w.\-]+/[\w./\-]+").expect("路径"),
        ]
    });
    let backtick = Regex::new(r"`[^`\n]+`").expect("反引号");
    let chinese_only = Regex::new(r"^[\u{4e00}-\u{9fa5}，。：；、「」（）《》\s]+$").expect("纯中文");

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
        .filter(|item| !all.iter().any(|other| other != *item && other.contains(item.as_str())))
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
        let has_reader = config.reader_words.iter().any(|word| body.contains(word.as_str()));
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
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(_) => String::new(),
    };
    let mut findings = check_placement(&rel, contract);
    findings.extend(check_banned(&rel, &text, contract));
    findings.extend(check_required(&rel, &text, contract));
    findings.extend(check_numbers(&rel, &text, contract));
    findings.extend(check_syntax(&rel, &text, contract));
    findings.extend(check_table(&rel, &text, contract));
    findings.extend(check_voice(&rel, &text, contract));
    findings
}

/// 扫描范围：`default` 只审规范文档，`all` 连流水与报告一起。
pub fn markdown_files(root: &Path, contract: &Contract, scope: &str) -> Vec<PathBuf> {
    if root.is_file() {
        return vec![root.to_path_buf()];
    }
    let mut includes: Vec<String> = Vec::new();
    if scope == "all" {
        includes.push(String::new()); // 空串 = 全收
    } else {
        includes.extend(contract.scope.include_default.iter().cloned());
    }

    let mut out: Vec<PathBuf> = WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .filter(|path| path.extension().map(|ext| ext == "md").unwrap_or(false))
        .filter(|path| {
            let rel = rel_of(path, Some(root));
            // exclude 对所有 scope 都生效：all 只是放开 include 前缀，不是把第三方依赖也审一遍
            !contract.scope.exclude.iter().any(|skip| in_dir(&rel, skip))
        })
        .filter(|path| {
            if scope == "all" {
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
            includes.iter().any(|prefix| !prefix.is_empty() && in_dir(&rel, prefix))
        })
        .collect();
    out.sort();
    out
}

/// 全量扫描：返回（命中，读不了的文件）。
pub fn scan(root: &Path, contract: &Contract, scope: &str) -> (Vec<Finding>, Vec<String>) {
    let mut findings = Vec::new();
    let mut broken = Vec::new();
    let base = if root.is_dir() { Some(root) } else { None };
    for path in markdown_files(root, contract, scope) {
        if fs::read_to_string(&path).is_err() {
            broken.push(rel_of(&path, base));
            continue;
        }
        findings.extend(check_file(&path, contract, base));
    }
    (findings, broken)
}

/// 按类计数，给汇总行用。
pub fn summarize(findings: &[Finding]) -> HashMap<&'static str, usize> {
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    for finding in findings {
        *counts.entry(finding.kind.key()).or_insert(0) += 1;
    }
    counts
}
