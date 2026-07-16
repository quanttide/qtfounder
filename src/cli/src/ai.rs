use quanttide_agent::Message;
use quanttide_agent::llm::{CompleteOptions, LLM};
use serde::de::DeserializeOwned;

/// 调用 LLM 提取结构化 JSON 数据。
pub fn extract_json<T: DeserializeOwned>(system_prompt: &str, text: &str) -> Option<T> {
    if text.is_empty() {
        return None;
    }
    let llm = LLM::default();
    let msg = &text[..text.len().min(4000)];
    match llm.complete(
        &[
            Message::new("system", system_prompt),
            Message::new("user", msg),
        ],
        CompleteOptions::default(),
    ) {
        Ok(r) => serde_json::from_str(r.content.trim()).ok(),
        Err(_) => None,
    }
}
