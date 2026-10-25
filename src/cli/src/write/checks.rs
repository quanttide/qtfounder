//! 四层检查：空话、口号、指路、结构。
//!
//! 空话看整篇密度，逐句报会淹没有用信息；口号与指路逐行定位；结构看标题层级与长文分节。

use std::collections::HashSet;

use regex::Regex;

use super::contract::Contract;
use super::model::{Finding, Kind, Severity};

/// 命令行给的画像；没给的按 front matter 与路径推断。
#[derive(Debug, Clone, Default)]
pub struct Selection {
    pub genre: Option<String>,
    pub audience: Option<String>,
    pub topic: Option<String>,
}

pub fn check_text(
    path: &str,
    text: &str,
    contract: &Contract,
    selection: &Selection,
) -> Vec<Finding> {
    let (genre, audience, topic) = resolve_profiles(path, text, contract, selection);
    let mut findings = Vec::new();
    findings.extend(check_genre(path, text, contract, &genre));
    findings.extend(check_audience(path, text, contract, &audience));
    findings.extend(check_topic(path, text, contract, &topic));
    findings.extend(check_empty(path, text, contract));
    findings.extend(check_slogan(path, text, contract));
    findings.extend(check_reference(path, text, contract));
    findings.extend(check_structure(path, text, contract));
    findings.sort_by(|a, b| a.line.unwrap_or(0).cmp(&b.line.unwrap_or(0)));
    findings
}

/// 空话按每千字密度判。低于门槛不报，避免把偶发用法当问题。
fn check_empty(path: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let rule = &contract.empty;
    if rule.max_per_1000 == 0 || rule.words.is_empty() {
        return Vec::new();
    }
    let prose = prose_text(text);
    let chars = char_count(&prose);
    if chars < rule.min_chars {
        return Vec::new();
    }
    let mut counts: Vec<(&str, usize)> = rule
        .words
        .iter()
        .map(|word| (word.as_str(), prose.matches(word.as_str()).count()))
        .filter(|(_, count)| *count > 0)
        .collect();
    let total: usize = counts.iter().map(|(_, count)| count).sum();
    let per_1000 = total * 1000 / chars.max(1);
    if per_1000 <= rule.max_per_1000 {
        return Vec::new();
    }
    counts.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    let top = counts
        .iter()
        .take(3)
        .map(|(word, count)| format!("{word} {count}"))
        .collect::<Vec<String>>()
        .join("、");
    vec![Finding::new(
        Kind::Empty,
        Severity::Suspect,
        path,
        None,
        format!(
            "空话 {per_1000} 处/千字（上限 {}）：{top}",
            rule.max_per_1000
        ),
        rule.advice.clone(),
    )]
}

fn check_slogan(path: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let mut patterns = Vec::new();
    for item in &contract.slogan.patterns {
        if let Ok(regex) = Regex::new(&item.pattern) {
            patterns.push((regex, item.advice.clone()));
        }
    }
    let mut findings = Vec::new();
    for (line_no, line) in prose_lines(text) {
        let bare = strip_mentions(line);
        for (regex, advice) in &patterns {
            if regex.is_match(&bare) {
                findings.push(Finding::new(
                    Kind::Slogan,
                    Severity::Suspect,
                    path,
                    Some(line_no),
                    excerpt(line),
                    advice.clone(),
                ));
            }
        }
    }
    findings
}

fn check_reference(path: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let patterns: Vec<Regex> = contract
        .reference
        .patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect();
    if patterns.is_empty() {
        return Vec::new();
    }
    let mut findings = Vec::new();
    for (line_no, line) in prose_lines(text) {
        let bare = strip_mentions(line);
        for regex in &patterns {
            if regex.is_match(&bare) {
                findings.push(Finding::new(
                    Kind::Reference,
                    Severity::Suspect,
                    path,
                    Some(line_no),
                    excerpt(line),
                    contract.reference.advice.clone(),
                ));
                break;
            }
        }
    }
    findings
}

fn check_structure(path: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let rule = &contract.structure;
    let mut findings = Vec::new();
    let chars: usize = prose_lines(text).map(|(_, line)| char_count(line)).sum();
    let mut in_code = false;
    let mut h1_lines = Vec::new();
    let mut has_h2 = false;
    let mut last_level = 0usize;

    for (index, line) in text.lines().enumerate() {
        let line_no = index + 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        let Some(level) = heading_level(trimmed) else {
            continue;
        };
        if level == 1 {
            h1_lines.push(line_no);
        }
        if level == 2 {
            has_h2 = true;
        }
        if rule.max_heading_jump > 0 && last_level > 0 && level > last_level + rule.max_heading_jump
        {
            findings.push(Finding::new(
                Kind::Structure,
                Severity::Suspect,
                path,
                Some(line_no),
                excerpt(line),
                format!("标题从 {last_level} 级跳到 {level} 级；中间那层没写出来"),
            ));
        }
        last_level = level;
    }

    if rule.require_heading && h1_lines.is_empty() {
        findings.push(Finding::new(
            Kind::Structure,
            Severity::Error,
            path,
            None,
            "全文没有一级标题".to_string(),
            "加一行 `# 标题`，打开就知道这是什么".to_string(),
        ));
    }

    if rule.max_h1 > 0 && h1_lines.len() > rule.max_h1 {
        for line_no in h1_lines.iter().skip(rule.max_h1) {
            findings.push(Finding::new(
                Kind::Structure,
                Severity::Suspect,
                path,
                Some(*line_no),
                "第二个一级标题".to_string(),
                "一篇只留一个一级标题；其余降级或拆篇".to_string(),
            ));
        }
    }

    if rule.min_chars_per_section > 0 && chars >= rule.min_chars_per_section && !has_h2 {
        findings.push(Finding::new(
            Kind::Structure,
            Severity::Suspect,
            path,
            None,
            format!("全文 {chars} 字，没有二级标题"),
            "按一节一件事分开；没有二级标题，读者只能通读".to_string(),
        ));
    }

    findings
}

fn check_genre(path: &str, text: &str, contract: &Contract, name: &str) -> Vec<Finding> {
    let Some(profile) = contract.genre.get(name) else {
        let names: Vec<&str> = contract.genre.keys().map(String::as_str).collect();
        return vec![unknown_profile(Kind::Genre, path, "体裁", name, &names)];
    };

    let mut findings = Vec::new();
    let headings = headings(text);
    for requirement in &profile.required {
        let found = headings.iter().any(|(_, _, title)| {
            requirement
                .any_of
                .iter()
                .any(|keyword| title.contains(keyword))
        }) || prose_lines(text).any(|(_, line)| {
            requirement
                .any_of
                .iter()
                .any(|keyword| line.contains(keyword))
        });
        if !found {
            findings.push(Finding::new(
                Kind::Genre,
                Severity::Suspect,
                path,
                None,
                format!("体裁「{name}」缺「{}」一节", requirement.name),
                format!("补一节标题含「{}」的内容", requirement.any_of.join("、")),
            ));
        }
    }

    let h2 = headings.iter().filter(|(_, level, _)| *level == 2).count();
    if profile.min_h2 > 0 && h2 < profile.min_h2 {
        findings.push(Finding::new(
            Kind::Genre,
            Severity::Suspect,
            path,
            None,
            format!(
                "体裁「{name}」只有 {h2} 个二级标题，下限 {}",
                profile.min_h2
            ),
            "按节分开，每节只讲一个意思".to_string(),
        ));
    }
    if profile.max_h2 > 0 && h2 > profile.max_h2 {
        findings.push(Finding::new(
            Kind::Genre,
            Severity::Suspect,
            path,
            None,
            format!("体裁「{name}」有 {h2} 个二级标题，上限 {}", profile.max_h2),
            "合并同类节，或拆成多篇".to_string(),
        ));
    }

    for word in &profile.forbid {
        for (line_no, line) in prose_lines(text) {
            if strip_mentions(line).contains(word) {
                findings.push(Finding::new(
                    Kind::Genre,
                    Severity::Suspect,
                    path,
                    Some(line_no),
                    excerpt(line),
                    format!("体裁「{name}」不写「{word}」"),
                ));
            }
        }
    }
    findings
}

fn check_audience(path: &str, text: &str, contract: &Contract, name: &str) -> Vec<Finding> {
    let Some(profile) = contract.audience.get(name) else {
        let names: Vec<&str> = contract.audience.keys().map(String::as_str).collect();
        return vec![unknown_profile(Kind::Audience, path, "读者", name, &names)];
    };

    let mut findings = Vec::new();
    let prose = prose_text(text);
    if profile.require_reader_words && !profile.reader_words.iter().any(|word| prose.contains(word))
    {
        findings.push(Finding::new(
            Kind::Audience,
            Severity::Suspect,
            path,
            None,
            format!("读者是「{name}」，正文没有对读者说话"),
            format!(
                "补一句「{}」；读者找不到自己，只能旁观",
                profile.reader_words.join("、")
            ),
        ));
    }

    if profile.max_internal_ids > 0 {
        for paragraph in paragraphs(text) {
            let count = distinct_ids(&paragraph.text);
            if count <= profile.max_internal_ids {
                continue;
            }
            let has_reader = profile
                .reader_words
                .iter()
                .any(|word| paragraph.text.contains(word));
            if has_reader {
                continue;
            }
            findings.push(Finding::new(
                Kind::Audience,
                Severity::Suspect,
                path,
                Some(paragraph.line),
                format!("一段里 {count} 个内部标识，读者是「{name}」"),
                format!(
                    "上限 {} 个；拆成表格，或换成读者能懂的说法",
                    profile.max_internal_ids
                ),
            ));
        }
    }
    findings
}

fn check_topic(path: &str, text: &str, contract: &Contract, name: &str) -> Vec<Finding> {
    let Some(profile) = contract.topic.get(name) else {
        let names: Vec<&str> = contract.topic.keys().map(String::as_str).collect();
        return vec![unknown_profile(Kind::Topic, path, "主题", name, &names)];
    };

    let mut findings = Vec::new();
    let prose = prose_text(text);
    let used: Vec<&str> = profile
        .terms
        .iter()
        .filter(|term| prose.contains(term.as_str()))
        .map(|term| term.as_str())
        .collect();

    if profile.require_definition && !used.is_empty() {
        let defined = headings(text).iter().any(|(_, _, title)| {
            ["定义", "术语", "概念"]
                .iter()
                .any(|keyword| title.contains(keyword))
        });
        if !defined {
            findings.push(Finding::new(
                Kind::Topic,
                Severity::Suspect,
                path,
                None,
                format!("主题「{name}」用到「{}」，但没有定义一节", used.join("、")),
                "补一节标题含「定义」或「术语」的内容；主题词第一次出现前先交代".to_string(),
            ));
        }
    }
    findings
}

fn unknown_profile(kind: Kind, path: &str, axis: &str, name: &str, available: &[&str]) -> Finding {
    Finding::new(
        kind,
        Severity::Error,
        path,
        None,
        format!("契约里没有{axis}「{name}」"),
        format!("可选：{}；或在契约里补一段", available.join("、")),
    )
}

fn resolve_profiles(
    path: &str,
    text: &str,
    contract: &Contract,
    selection: &Selection,
) -> (String, String, String) {
    let genre = selection
        .genre
        .clone()
        .or_else(|| front_matter_value(text, "genre"))
        .or_else(|| contract.genre_for_path(path))
        .unwrap_or_else(|| "default".to_string());
    let audience = selection
        .audience
        .clone()
        .or_else(|| front_matter_value(text, "audience"))
        .unwrap_or_else(|| "default".to_string());
    let topic = selection
        .topic
        .clone()
        .or_else(|| front_matter_value(text, "topic"))
        .unwrap_or_else(|| "default".to_string());
    (genre, audience, topic)
}

fn front_matter_value(text: &str, key: &str) -> Option<String> {
    let mut lines = text.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let prefix = format!("{key}:");
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            break;
        }
        if let Some(value) = trimmed.strip_prefix(&prefix) {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn headings(text: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut in_code = false;
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        let Some(level) = heading_level(trimmed) else {
            continue;
        };
        out.push((index + 1, level, trimmed[level..].trim().to_string()));
    }
    out
}

struct Paragraph {
    line: usize,
    text: String,
}

fn paragraphs(text: &str) -> Vec<Paragraph> {
    let mut out = Vec::new();
    let mut current: Option<(usize, String)> = None;
    for (line_no, line) in prose_lines(text) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            if let Some((start, body)) = current.take() {
                out.push(Paragraph {
                    line: start,
                    text: body,
                });
            }
            continue;
        }
        match &mut current {
            Some((_, body)) => {
                body.push('\n');
                body.push_str(trimmed);
            }
            None => current = Some((line_no, trimmed.to_string())),
        }
    }
    if let Some((start, body)) = current {
        out.push(Paragraph {
            line: start,
            text: body,
        });
    }
    out
}

fn distinct_ids(text: &str) -> usize {
    let mut ids = HashSet::new();
    let mut current = String::new();
    let mut inside = false;
    for ch in text.chars() {
        if ch == '`' {
            if inside && !current.is_empty() {
                ids.insert(current.clone());
                current.clear();
            }
            inside = !inside;
            continue;
        }
        if inside {
            current.push(ch);
        }
    }
    ids.len()
}

fn heading_level(trimmed: &str) -> Option<usize> {
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    if level == 0 {
        return None;
    }
    match trimmed.chars().nth(level) {
        Some(' ') | Some('\t') => Some(level),
        _ => None,
    }
}

fn char_count(text: &str) -> usize {
    text.chars().filter(|ch| !ch.is_whitespace()).count()
}

fn excerpt(line: &str) -> String {
    line.trim().chars().take(70).collect()
}

/// 把讨论中的提法去掉再匹配：反引号与「」里是「说这个词」，不是「用这个词」。
fn strip_mentions(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_code = false;
    let mut in_quote = false;
    for ch in line.chars() {
        match ch {
            '`' => {
                in_code = !in_code;
                out.push(ch);
            }
            '「' if !in_code => {
                in_quote = true;
                out.push(ch);
            }
            '」' if !in_code => {
                in_quote = false;
                out.push(ch);
            }
            _ if in_code || in_quote => out.push(' '),
            _ => out.push(ch),
        }
    }
    out
}

/// 只留正文行，跳过代码块。
fn prose_text(text: &str) -> String {
    prose_lines(text)
        .map(|(_, line)| line)
        .collect::<Vec<&str>>()
        .join("\n")
}

/// 跳过代码块：命令里的路径不是指路，示例里的口号不是正文。
fn prose_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut in_code = false;
    text.lines().enumerate().filter_map(move |(index, line)| {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code = !in_code;
            return None;
        }
        if in_code {
            None
        } else {
            Some((index + 1, line))
        }
    })
}
