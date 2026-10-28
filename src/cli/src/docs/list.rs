//! 列表层：三项以上成块。三段式占比过高、单项过长，都说明这段本该是段落。
//!
//! 三段式是 AI 生成列表最稳定的指纹：`- **要点**：解释` 这种结构，人类写作很少整段这么排。
//! 行级扫描足够，不引 Markdown AST——代码块外按标记行切块即可。

use std::sync::LazyLock;

use regex::Regex;

use super::common::{char_count, excerpt_of};
use super::contract::{Contract, List as ListConfig};
use super::model::{Finding, Kind, Severity};

static ITEM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:[-*+]|\d+[.)])\s+(.*)$")
        .expect("列表标记正则写坏了：改 ITEM，再跑 cargo test")
});

static LABEL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\*\*[^*]+\*\*\s*[：:]").expect("三段式正则写坏了：改 LABEL，再跑 cargo test")
});

struct Item {
    line: usize,
    text: String,
}

struct Block {
    start: usize,
    items: Vec<Item>,
}

pub(super) fn check_list(rel: &str, text: &str, contract: &Contract) -> Vec<Finding> {
    let config = &contract.list;
    if config.max_item_chars == 0 && config.label_ratio <= 0.0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut block: Option<Block> = None;
    let mut in_code = false;

    for (index, raw) in text.lines().enumerate() {
        let trimmed = raw.trim();
        if trimmed.starts_with("```") {
            flush(&mut block, config, rel, &mut out);
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        if let Some(caps) = ITEM.captures(raw) {
            let item = Item {
                line: index + 1,
                text: caps[1].trim().to_string(),
            };
            match &mut block {
                Some(open) => open.items.push(item),
                None => {
                    block = Some(Block {
                        start: index + 1,
                        items: vec![item],
                    })
                }
            }
            continue;
        }
        // 列表项内的续行：缩进着写，算进上一条。
        if let Some(open) = &mut block
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && !trimmed.starts_with('|')
            && raw.starts_with(' ')
        {
            if let Some(last) = open.items.last_mut() {
                last.text.push('\n');
                last.text.push_str(trimmed);
            }
            continue;
        }
        flush(&mut block, config, rel, &mut out);
    }
    flush(&mut block, config, rel, &mut out);
    out
}

/// 这一行是不是列表项。格式状态按同一处识别数项数。
pub(super) fn is_item(raw: &str) -> bool {
    ITEM.is_match(raw)
}

fn flush(block: &mut Option<Block>, config: &ListConfig, rel: &str, out: &mut Vec<Finding>) {
    let Some(block) = block.take() else {
        return;
    };
    if block.items.len() < config.min_items.max(1) {
        return;
    }
    let total = block.items.len();
    let labelled = block
        .items
        .iter()
        .filter(|item| LABEL.is_match(item.text.trim_start()))
        .count();
    let ratio = labelled as f64 / total as f64;
    if config.label_ratio > 0.0 && ratio > config.label_ratio {
        out.push(Finding::new(
            Kind::List,
            Severity::Error,
            rel,
            block.start,
            excerpt_of(&block.items[0].text),
            format!(
                "{labelled}/{total} 项是「加粗要点 + 解释」三段式（上限 {:.0}%）：整段重写为散文，只留真正并列的内容",
                config.label_ratio * 100.0
            ),
        ));
    }
    if config.max_item_chars > 0 {
        for item in &block.items {
            let count = char_count(&item.text);
            if count > config.max_item_chars {
                out.push(Finding::new(
                    Kind::List,
                    Severity::Error,
                    rel,
                    item.line,
                    excerpt_of(&item.text),
                    format!(
                        "单项 {count} 字（上限 {}）：拆成段落，列表项只放一句",
                        config.max_item_chars
                    ),
                ));
            }
        }
    }
}
