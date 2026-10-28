//! 格式状态：一篇文档用了多少格式元素，只数不下结论。
//!
//! 与审计分工：审计说哪一处不该这么写，状态给整篇的数字——有没有滥用格式，
//! 先看数再读结论。计数与判定共用同一处识别（分割线走 `rule::rule_lines`、
//! 列表走 `list::is_item`、加粗走 `bold::bold_spans`），不另写一套，
//! 免得状态说三处、审计说四处。

use std::collections::HashSet;

use super::bold::bold_spans;
use super::common::char_count;
use super::list::is_item;
use super::rule::rule_lines;

/// 一篇文档的格式元素计数。全零表示一篇没有格式元素的纯散文。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Status {
    /// 正文字数：frontmatter 与代码块之外的非空行，含标题与表格行
    pub chars: usize,
    /// ATX 标题个数（`#` 到 `######`）
    pub headings: usize,
    /// 其中一级标题个数：章程要求全文唯一
    pub h1: usize,
    /// 最深标题层级；0 表示没有标题
    pub deepest: usize,
    /// 表格张数（连续两行以上 `|` 才算一张）
    pub tables: usize,
    /// 这些表的总行数
    pub table_rows: usize,
    /// 列表块数：中间只隔空行的松散列表算同一块
    pub lists: usize,
    pub list_items: usize,
    /// 加粗处数；表头加粗不算正文标重点
    pub bold: usize,
    /// 分割线条数；frontmatter、代码块与 setext 下划线不算
    pub rules: usize,
    /// 块引用段数：连续 `>` 行算一段
    pub quotes: usize,
    /// 代码块个数
    pub code_blocks: usize,
}

impl Status {
    /// 格式元素总数，这七类怎么算照章程第四条：标题、表格、列表、加粗、分割线、块引用、代码块各算一处。
    pub fn elements(&self) -> usize {
        self.headings
            + self.tables
            + self.lists
            + self.bold
            + self.rules
            + self.quotes
            + self.code_blocks
    }
}

pub fn count(text: &str) -> Status {
    let dividers: HashSet<usize> = rule_lines(text).into_iter().collect();
    let mut status = Status::default();
    let mut in_code = false;
    let mut in_frontmatter = false;
    let mut seen_content = false;
    let mut table_run = 0usize;
    let mut list_open = false;
    let mut quote_open = false;

    // 表格与审计一样按连续行切块：不足两行的散行不配叫表
    fn close_table(status: &mut Status, table_run: &mut usize) {
        if *table_run >= 2 {
            status.tables += 1;
            status.table_rows += *table_run;
        }
        *table_run = 0;
    }

    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim();

        if !seen_content && trimmed == "---" {
            in_frontmatter = true;
            seen_content = true;
            continue;
        }
        if in_frontmatter {
            if trimmed == "---" {
                in_frontmatter = false;
            }
            continue;
        }
        if trimmed.starts_with("```") {
            close_table(&mut status, &mut table_run);
            list_open = false;
            quote_open = false;
            if !in_code {
                status.code_blocks += 1;
            }
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        if !trimmed.is_empty() {
            seen_content = true;
        }
        if dividers.contains(&line) {
            close_table(&mut status, &mut table_run);
            list_open = false;
            quote_open = false;
            status.rules += 1;
            continue;
        }
        if trimmed.is_empty() {
            // 空行断表与引用，但不断列表：松散列表还是同一块
            close_table(&mut status, &mut table_run);
            quote_open = false;
            continue;
        }

        status.chars += char_count(trimmed);

        if trimmed.starts_with('#') {
            close_table(&mut status, &mut table_run);
            list_open = false;
            quote_open = false;
            status.headings += 1;
            let level = trimmed.chars().take_while(|ch| *ch == '#').count();
            if level == 1 {
                status.h1 += 1;
            }
            status.deepest = status.deepest.max(level);
            continue;
        }
        if trimmed.starts_with('|') {
            table_run += 1;
            list_open = false;
            quote_open = false;
            continue;
        }
        // 表头加粗不算：表头本来就该加粗，那是表格的写法
        status.bold += bold_spans(raw).len();

        if is_item(raw) {
            close_table(&mut status, &mut table_run);
            quote_open = false;
            if !list_open {
                status.lists += 1;
                list_open = true;
            }
            status.list_items += 1;
            continue;
        }
        if trimmed.starts_with('>') {
            close_table(&mut status, &mut table_run);
            list_open = false;
            if !quote_open {
                status.quotes += 1;
                quote_open = true;
            }
            continue;
        }
        // 列表项的续行把列表接住；其余行（段落、图片）都断开
        let continuation = list_open && (raw.starts_with(' ') || raw.starts_with('\t'));
        if !continuation {
            close_table(&mut status, &mut table_run);
            list_open = false;
            quote_open = false;
        }
    }
    close_table(&mut status, &mut table_run);
    status
}
