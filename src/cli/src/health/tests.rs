//! 离线自测：历法换算、宽筛、判定解析、环节归一。不联网。

use super::corpus::Message;
use super::corpus::{civil_from_ts, expand_home};
use super::judge::{parse_judged, stage_title};
use super::screen::screen;

fn msg(id: &str, content: &str) -> Message {
    Message {
        id: id.to_string(),
        session_id: "s1".to_string(),
        ts: 0.0,
        date: "2026-10-22".to_string(),
        hour: 18,
        content: content.to_string(),
    }
}

#[test]
fn timestamp_to_beijing_date() {
    assert_eq!(civil_from_ts(1792664763.76), ("2026-10-22".to_string(), 18));
    assert_eq!(civil_from_ts(0.0), ("1970-01-01".to_string(), 8));
    assert_eq!(civil_from_ts(1767225600.0), ("2026-01-01".to_string(), 8));
}

#[test]
fn expand_home_prefix() {
    let path = expand_home("~/.hermes/state.db");
    assert!(!path.to_string_lossy().starts_with('~'));
    assert_eq!(expand_home("/tmp/x"), std::path::PathBuf::from("/tmp/x"));
}

#[test]
fn bare_ack_dropped() {
    let (cands, rest) = screen(vec![msg("1", "好的"), msg("2", "同意。")]);
    assert!(cands.is_empty());
    assert_eq!(rest, 2);
}

#[test]
fn signal_word_kept_even_if_short() {
    let (cands, _) = screen(vec![msg("1", "累")]);
    assert_eq!(cands.len(), 1);
}

#[test]
fn long_message_kept() {
    let (cands, _) = screen(vec![msg(
        "1",
        "这一段话足够长并且没有任何疲劳方面的词汇表达",
    )]);
    assert_eq!(cands.len(), 1);
}

#[test]
fn parse_plain_array() {
    let raw = r#"[{"id":"a1","signal":"fatigue","implicit":false,"intensity":4,"stage":"B","reason":"他说看不懂","sources":[{"label":"输出太长","quote":"这个输出还是太长了"}]}]"#;
    let items = parse_judged(raw);
    assert_eq!(items.len(), 1);
    let (id, judged) = &items[0];
    assert_eq!(id, "a1");
    assert_eq!(judged.signal, "fatigue");
    assert_eq!(judged.stage, "B");
    assert_eq!(judged.sources[0].quote, "这个输出还是太长了");
}

#[test]
fn no_quote_means_no_hit() {
    let raw = r#"[{"id":"a1","signal":"fatigue","sources":[{"label":"输出太长"}]}]"#;
    let items = parse_judged(raw);
    assert_eq!(items[0].1.signal, "");
}

#[test]
fn two_arrays_back_to_back() {
    let raw = r#"[{"id":"a1","signal":""}] 说明：[{"id":"a2","signal":"friction","stage":"A","sources":[{"label":"返工","quote":"又改错了"}]}]"#;
    let items = parse_judged(raw);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].1.signal, "");
    assert_eq!(items[1].1.stage, "A");
}

#[test]
fn chinese_keys_and_stage_fallback() {
    let raw = r#"[{"id":"a1","信号":"摩擦","强度":5,"环节":"没把选项收敛到能拍","来源":[{"来源":"被替做决定","原句":"谁让你手动触发的"}],"隐式":"是"}]"#;
    let items = parse_judged(raw);
    let judged = &items[0].1;
    assert_eq!(judged.signal, "friction");
    assert_eq!(judged.intensity, 5);
    // 环节只认 A–E 字母开头；给的是全名，认不出就落到 C
    assert_eq!(judged.stage, "C");
    assert!(judged.implicit);
}

#[test]
fn unknown_stage_is_not_user() {
    let raw =
        r#"[{"id":"a1","signal":"fatigue","stage":"用户","sources":[{"label":"x","quote":"y"}]}]"#;
    let items = parse_judged(raw);
    assert_eq!(items[0].1.stage, "C");
    assert_eq!(stage_title("C"), "没给判据、没法验证");
}

#[test]
fn intensity_clamped() {
    let raw = r#"[{"id":"a1","signal":"fatigue","intensity":99,"stage":"E","sources":[{"label":"x","quote":"y"}]}]"#;
    let items = parse_judged(raw);
    assert_eq!(items[0].1.intensity, 5);
}

#[test]
fn garbage_returns_empty() {
    assert!(parse_judged("模型说了一堆话没有 JSON").is_empty());
}
