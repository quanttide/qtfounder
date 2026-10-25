//! 宽筛：把明显不含自身状态的短消息剔出去，只留候选。
//!
//! 判据要宽——目标是少漏，不是省调用。弱信号词与长度门槛在认知模型的 `[screen]` 里，
//! 改词表改模型文件，不改这里。未入选的消息不丢弃，计数后报出来，供「穿漏率」自查用。

use super::corpus::Message;
use crate::think::model::Screen;

fn strip_punct(text: &str) -> String {
    text.chars()
        .filter(|ch| !matches!(ch, '。' | '，' | '!' | '.' | '~' | '！' | '、' | ' '))
        .collect()
}

/// 返回（候选及命中的词，未入选条数）。
pub fn screen(messages: Vec<Message>, config: &Screen) -> (Vec<(Message, Vec<String>)>, usize) {
    let mut cands = Vec::new();
    let mut rest = 0usize;
    for message in messages {
        let hits: Vec<String> = config
            .signal_words
            .iter()
            .filter(|word| message.content.contains(word.as_str()))
            .cloned()
            .collect();
        let bare = strip_punct(&message.content);
        let is_ack = hits.is_empty() && config.bare_ack.iter().any(|ack| bare == strip_punct(ack));
        if is_ack {
            rest += 1;
            continue;
        }
        if !hits.is_empty() || message.content.chars().count() >= config.min_chars {
            cands.push((message, hits));
        } else {
            rest += 1;
        }
    }
    (cands, rest)
}
