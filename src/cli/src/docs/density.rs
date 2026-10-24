//! 密度兜底：缺陷规则没命中时，看整篇的字数花在结构上多少、有没有散文过渡。
//!
//! 判的是整篇，不是某一行。两个指标都由块序列算出：
//! 结构块占比 = 表格与列表字数 / 正文字数；无散文率 = 没有过渡段落的连续结构块 / 结构块总数。
//! 阈值先留空，用 `--stats` 把数字摆出来，跑完语料再回填。

use super::common::char_count;
use super::contract::Density as DensityConfig;
use super::model::{Finding, Kind, Severity};

#[derive(Debug, Default, Clone, Copy)]
pub struct Measures {
    /// 正文总字数（段落 + 结构块，不含标题）
    pub chars: usize,
    /// 表格与列表的字数
    pub block_chars: usize,
    /// 结构块个数
    pub blocks: usize,
    /// 落在「连续结构块」里的结构块个数（不含孤立的单块）
    pub run_blocks: usize,
    pub longest_run_line: usize,
    pub longest_run_len: usize,
    pub first_block_line: usize,
}

impl Measures {
    pub fn block_ratio(&self) -> f64 {
        if self.chars == 0 {
            return 0.0;
        }
        self.block_chars as f64 / self.chars as f64
    }

    pub fn run_ratio(&self) -> f64 {
        if self.blocks == 0 {
            return 0.0;
        }
        self.run_blocks as f64 / self.blocks as f64
    }
}

enum BlockKind {
    Paragraph,
    Structure,
    /// 标题与代码块：不进字数，也把前后结构块隔开
    Separator,
}

struct Block {
    kind: BlockKind,
    line: usize,
    chars: usize,
}

/// 把正文切成块。连续列表行算一个块，连续表格行算一个块，其余非空行算段落；
/// 标题、代码块、frontmatter 作为分隔，不参与字数。
pub(super) fn measure(text: &str, min_paragraph_chars: usize) -> Measures {
    let blocks = blocks_of(text);
    let mut measures = Measures::default();
    let mut run_len = 0usize;
    let mut run_start = 0usize;

    let close_run = |run_len: &mut usize, run_start: usize, measures: &mut Measures| {
        if *run_len >= 2 {
            measures.run_blocks += *run_len;
            if *run_len > measures.longest_run_len {
                measures.longest_run_len = *run_len;
                measures.longest_run_line = run_start;
            }
        }
        *run_len = 0;
    };

    for block in &blocks {
        match block.kind {
            BlockKind::Separator => {
                close_run(&mut run_len, run_start, &mut measures);
            }
            BlockKind::Paragraph => {
                measures.chars += block.chars;
                // 短段落挡不住两串结构块，长段落才算过渡
                if block.chars >= min_paragraph_chars.max(1) {
                    close_run(&mut run_len, run_start, &mut measures);
                }
            }
            BlockKind::Structure => {
                measures.chars += block.chars;
                measures.block_chars += block.chars;
                measures.blocks += 1;
                if measures.first_block_line == 0 {
                    measures.first_block_line = block.line;
                }
                if run_len == 0 {
                    run_start = block.line;
                }
                run_len += 1;
            }
        }
    }
    close_run(&mut run_len, run_start, &mut measures);
    measures
}

fn blocks_of(text: &str) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    let mut in_code = false;
    let mut in_frontmatter = false;
    let mut seen_content = false;
    let mut force_new = false;

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
            in_code = !in_code;
            push_separator(&mut blocks, line);
            continue;
        }
        if in_code {
            continue;
        }
        if trimmed.is_empty() {
            // 空行结束当前块：列表之后接一张表，是两个块不是一个
            force_new = true;
            continue;
        }
        if !trimmed.is_empty() {
            seen_content = true;
        }

        let kind = if trimmed.starts_with('#') {
            BlockKind::Separator
        } else if trimmed.starts_with('|') || is_list_item(trimmed) {
            BlockKind::Structure
        } else {
            BlockKind::Paragraph
        };
        push_block(&mut blocks, kind, line, char_count(trimmed), force_new);
        force_new = false;
    }
    blocks
}

fn is_list_item(trimmed: &str) -> bool {
    let rest = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
        .or_else(|| trimmed.strip_prefix("+ "));
    if rest.is_some() {
        return true;
    }
    // 有序列表：1. / 1)
    let digits: String = trimmed
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        return false;
    }
    let rest = &trimmed[digits.len()..];
    rest.starts_with(". ") || rest.starts_with(") ")
}

/// 同类相邻块合并；换类、或中间有空行就开新块。
fn push_block(
    blocks: &mut Vec<Block>,
    kind: BlockKind,
    line: usize,
    chars: usize,
    force_new: bool,
) {
    if !force_new && let Some(last) = blocks.last_mut() {
        let same = matches!(
            (&last.kind, &kind),
            (BlockKind::Structure, BlockKind::Structure)
                | (BlockKind::Paragraph, BlockKind::Paragraph)
        );
        if same {
            last.chars += chars;
            return;
        }
    }
    blocks.push(Block { kind, line, chars });
}

fn push_separator(blocks: &mut Vec<Block>, line: usize) {
    // 连续分隔符不重复插
    if matches!(blocks.last().map(|b| &b.kind), Some(BlockKind::Separator)) {
        return;
    }
    blocks.push(Block {
        kind: BlockKind::Separator,
        line,
        chars: 0,
    });
}

/// 只出问题，不出数字：超阈值时选超出更多的那一项报一条。
pub(super) fn judge(rel: &str, measures: &Measures, config: &DensityConfig) -> Option<Finding> {
    if config.min_chars > 0 && measures.chars < config.min_chars {
        return None;
    }
    let block_ratio = measures.block_ratio();
    let run_ratio = measures.run_ratio();
    let mut worst: Option<(f64, usize, String)> = None;

    if config.max_block_ratio > 0.0 && block_ratio > config.max_block_ratio {
        let line = if measures.longest_run_line > 0 {
            measures.longest_run_line
        } else {
            measures.first_block_line
        };
        worst = Some((
            block_ratio - config.max_block_ratio,
            line,
            format!(
                "结构块占正文 {:.0}%（上限 {:.0}%）：把连续条目改写成段落，只留真正需要对照的部分",
                block_ratio * 100.0,
                config.max_block_ratio * 100.0
            ),
        ));
    }
    if config.max_run_ratio > 0.0 && run_ratio > config.max_run_ratio {
        let overshoot = run_ratio - config.max_run_ratio;
        if worst.as_ref().is_none_or(|(seen, _, _)| overshoot > *seen) {
            worst = Some((
                overshoot,
                measures.longest_run_line,
                format!(
                    "{} 个连续结构块中间没有段落（上限 {:.0}% 的结构块如此）：在块之间写过渡段",
                    measures.longest_run_len,
                    config.max_run_ratio * 100.0
                ),
            ));
        }
    }

    worst.map(|(_, line, advice)| {
        Finding::new(
            Kind::Density,
            Severity::Suspect,
            rel,
            line.max(1),
            format!(
                "块占比 {:.0}%，无散文率 {:.0}%，{} 字",
                block_ratio * 100.0,
                run_ratio * 100.0,
                measures.chars
            ),
            advice,
        )
    })
}
