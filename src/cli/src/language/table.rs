//! 表格层：列数、单元格长度、行数、有无表头，以及「这段内容根本不需要表格」。

use super::common::hanzi_count;
use super::contract::{Contract, Table as TableConfig};
use super::model::{Finding, Kind, Severity};
pub(super) fn check_table(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
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

    // 一次看一整张表：列数、表头、行数、稀疏度、单元格长度、两列表、排版表头。
    // 分开做是因为「这张表该不该存在」和「这张表某个格子太长」是两类问题，混在一个循环里
    // 读者没法知道先改哪一条。
    let flush = |block: &mut Vec<(usize, String)>, out: &mut Vec<Finding>| {
        if block.len() < 2 {
            block.clear();
            return;
        }
        let table = Table::parse(block);
        shape_findings(&table, config, rel, out);
        cell_findings(&table, config, rel, out);
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

/// 一张表：起始行号、原始行、拆好的单元格。
struct Table {
    start: usize,
    rows: Vec<String>,
    cells: Vec<Vec<String>>,
}

impl Table {
    fn parse(block: &[(usize, String)]) -> Self {
        let rows: Vec<String> = block.iter().map(|(_, line)| line.clone()).collect();
        let cells = rows
            .iter()
            .map(|row| {
                row.trim()
                    .trim_matches('|')
                    .split('|')
                    .map(|cell| cell.trim().to_string())
                    .collect()
            })
            .collect();
        Self {
            start: block[0].0,
            rows,
            cells,
        }
    }

    fn columns(&self) -> usize {
        self.cells.iter().map(|row| row.len()).max().unwrap_or(0)
    }

    /// 第二行是 `|---|:--:|` 这种就算有表头。
    fn has_header(&self) -> bool {
        self.cells.len() > 1
            && self.cells[1].iter().all(|cell| {
                !cell.is_empty() && cell.chars().all(|ch| ch == '-' || ch == ':' || ch == ' ')
            })
    }

    fn excerpt(&self) -> String {
        self.rows[0].chars().take(70).collect()
    }
}

fn push(out: &mut Vec<Finding>, rel: &str, line: usize, excerpt: String, advice: String) {
    out.push(Finding::new(
        Kind::Table,
        Severity::Suspect,
        rel,
        line,
        excerpt,
        advice,
    ));
}

/// 这张表整体该不该是这个形状：列太多、没表头、行太少/太多、太稀疏、两列表、排版表头。
fn shape_findings(table: &Table, config: &TableConfig, rel: &str, out: &mut Vec<Finding>) {
    let columns = table.columns();
    let excerpt = table.excerpt();
    if config.max_columns > 0 && columns > config.max_columns {
        push(
            out,
            rel,
            table.start,
            excerpt.clone(),
            format!(
                "{columns} 列（上限 {}）：手机上要横滚，拆成两张表或改成列表",
                config.max_columns
            ),
        );
    }
    if config.require_header && !table.has_header() {
        push(
            out,
            rel,
            table.start,
            excerpt.clone(),
            "第二行不是 |---| 分隔行：读者不知道每列是什么".to_string(),
        );
    }
    let data_rows = if table.has_header() {
        table.rows.len().saturating_sub(2)
    } else {
        table.rows.len()
    };
    if config.min_rows > 0 && data_rows < config.min_rows {
        push(
            out,
            rel,
            table.start,
            excerpt.clone(),
            "数据行太少：一句话能说清的事不必用表".to_string(),
        );
    }
    if config.max_rows > 0 && data_rows > config.max_rows {
        push(
            out,
            rel,
            table.start,
            excerpt.clone(),
            format!("{data_rows} 行（上限 {}）：拆表或换成列表", config.max_rows),
        );
    }
    let empty = table
        .cells
        .iter()
        .flatten()
        .filter(|cell| cell.is_empty())
        .count();
    let total = table
        .cells
        .iter()
        .map(|row| row.len())
        .sum::<usize>()
        .max(1);
    if config.sparse_ratio > 0.0 && empty as f64 / total as f64 > config.sparse_ratio {
        push(
            out,
            rel,
            table.start,
            excerpt.clone(),
            "空单元格太多：内容撑不起这个形状".to_string(),
        );
    }
    if config.flag_two_column && columns == 2 {
        push_two_column(out, rel, table, &excerpt);
    }
    if let Some(word) = layout_header(table, config) {
        push(
            out,
            rel,
            table.start,
            excerpt,
            format!("表头「{word}」是排版词不是属性名：这张表在做排版，不是在对照"),
        );
    }
}

/// 单个格子的问题：装了一段话。
fn cell_findings(table: &Table, config: &TableConfig, rel: &str, out: &mut Vec<Finding>) {
    if config.max_cell_chars == 0 {
        return;
    }
    for (index, row) in table.cells.iter().enumerate() {
        for cell in row {
            let count = hanzi_count(cell);
            if count > config.max_cell_chars {
                push(
                    out,
                    rel,
                    table.start + index,
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

/// 两列表本质是「名词 + 一段解释」，列表比表自然。
fn push_two_column(out: &mut Vec<Finding>, rel: &str, table: &Table, excerpt: &str) {
    push(
        out,
        rel,
        table.start,
        excerpt.to_string(),
        "两列表：本质是「名词 + 一段解释」，改成列表更自然（- **名词**：解释）".to_string(),
    );
}

/// 表头用了「说明」「备注」这类排版词，说明这张表在排版不是在对照。
fn layout_header(table: &Table, config: &TableConfig) -> Option<String> {
    let header = table.cells.first()?;
    header
        .iter()
        .find(|cell| config.layout_headers.contains(cell))
        .cloned()
}
