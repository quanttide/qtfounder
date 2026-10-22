//! 宽筛：把明显不含自身状态的短消息剔出去，只留候选。
//!
//! 判据要宽——目标是少漏，不是省调用。命中弱信号词、或长度够长的都留下。
//! 未入选的消息不丢弃，计数后报出来，供「穿漏率」自查用。

use super::corpus::Message;

/// 弱信号词：出现即入选候选。只求宽，不求准——准由模型那一步给。
const SIGNAL_WORDS: [&str; 46] = [
    "累", "疲", "乏", "困", "撑不住", "扛不", "精力", "力气", "提不起", "烦", "崩", "焦虑",
    "压力", "耗", "不想", "懒得", "没劲", "心力", "难受", "低落", "情绪", "失眠", "睡不着",
    "头疼", "头痛", "慌", "忙", "赶", "来不及", "拖", "熬", "厌倦", "抵触", "逃避", "放弃",
    "做不到", "搞不", "弄不", "转不动", "卡住", "倦", "看不懂", "太复杂", "复杂", "乱", "重来",
];

/// 纯操作类短句：即使命中词也不值得单独看。
const BARE_ACK: [&str; 16] = [
    "同意", "好的", "好", "嗯", "ok", "继续", "行", "可以", "对", "是的", "收到", "谢谢",
    "辛苦", "是", "否", "算了",
];

const MIN_CHARS: usize = 15;

fn strip_punct(text: &str) -> String {
    text.chars()
        .filter(|ch| !matches!(ch, '。' | '，' | '!' | '.' | '~' | '！' | '、' | ' '))
        .collect()
}

/// 返回（候选及命中的词，未入选条数）。
pub fn screen(messages: Vec<Message>) -> (Vec<(Message, Vec<String>)>, usize) {
    let mut cands = Vec::new();
    let mut rest = 0usize;
    for msg in messages {
        let hits: Vec<String> = SIGNAL_WORDS
            .iter()
            .filter(|word| msg.content.contains(**word))
            .map(|word| (*word).to_string())
            .collect();
        let bare = strip_punct(&msg.content);
        let is_ack = hits.is_empty() && BARE_ACK.iter().any(|ack| bare == strip_punct(ack));
        if is_ack {
            rest += 1;
            continue;
        }
        if !hits.is_empty() || msg.content.chars().count() >= MIN_CHARS {
            cands.push((msg, hits));
        } else {
            rest += 1;
        }
    }
    (cands, rest)
}
