//! 语义层：规则判不了的「这段到底是不是在跟读者说话」，交给模型判。
//!
//! 两条纪律：
//! 1. **只送规则挑出来的段**——全篇送模型既贵又慢，规则已经能滤掉大部分正常段；
//! 2. **temperature=0 + 只收 JSON**——同一段跑两次要给同一个结论，否则验收标准不可复跑。

use std::path::{Path, PathBuf};

use serde_json::Value;

use super::model::{Finding, Kind};

const DEFAULT_MODEL: &str = "deepseek-chat";
const DEFAULT_BASE_URL: &str = "https://api.deepseek.com";

pub struct Verdict {
    pub kind: String,
    pub why: String,
    pub fix: String,
}

pub struct Segment {
    pub path: String,
    pub line: usize,
    pub text: String,
}

fn prompt_path() -> PathBuf {
    // 与契约同一个查找口径：从当前目录往上找；找不到再退回编译期目录（开发时用）
    if let Ok(start) = std::env::current_dir() {
        let mut dir = start.as_path();
        loop {
            let candidate = dir.join("src/language/sense-prompt.md");
            if candidate.is_file() {
                return candidate;
            }
            let candidate = dir.join("src/cli/src/language/sense-prompt.md");
            if candidate.is_file() {
                return candidate;
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/language/sense-prompt.md")
}

pub fn load_prompt() -> String {
    std::fs::read_to_string(prompt_path()).unwrap_or_default()
}

/// 凭据：环境变量优先，其次本机凭据文件。密钥只读不打印。
fn credentials() -> Result<(String, String, String), String> {
    let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    let base_url = std::env::var("LLM_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
    if let Ok(key) = std::env::var("LLM_API_KEY")
        && !key.is_empty()
    {
        return Ok((model, base_url, key));
    }
    let home = std::env::var("HOME").map_err(|_| "读不到 HOME，取不到凭据".to_string())?;
    let auth = Path::new(&home).join(".pi/agent/auth.json");
    if let Ok(text) = std::fs::read_to_string(&auth)
        && let Ok(parsed) = serde_json::from_str::<Value>(&text)
        && let Some(key) = parsed
            .get("deepseek")
            .and_then(|node| node.get("key"))
            .and_then(|v| v.as_str())
    {
        return Ok((model, base_url, key.to_string()));
    }
    Err(format!(
        "没有可用密钥：设 LLM_API_KEY，或在 {} 里配 deepseek",
        auth.display()
    ))
}

/// 从规则命中里挑要送判的段：voice 与 syntax 命中，去重、限量。
pub fn suspect_segments(findings: &[Finding], limit: usize) -> Vec<Segment> {
    let mut seen: Vec<&str> = Vec::new();
    let mut picked = Vec::new();
    for finding in findings {
        if finding.kind != Kind::Voice && finding.kind != Kind::Syntax {
            continue;
        }
        let text = finding.excerpt.trim();
        if text.is_empty() || seen.contains(&text) {
            continue;
        }
        seen.push(text);
        picked.push(Segment {
            path: finding.path.clone(),
            line: finding.line,
            text: text.to_string(),
        });
        if picked.len() >= limit {
            break;
        }
    }
    picked
}

/// 判一段：reader（在对读者说话）还是 process（在记操作）。
pub fn classify(text: &str, model: &str, base_url: &str, api_key: &str) -> Result<Verdict, String> {
    let body = text.trim();
    if body.is_empty() {
        return Ok(Verdict {
            kind: "empty".into(),
            why: "空段".into(),
            fix: String::new(),
        });
    }
    let prompt = load_prompt();
    let payload = serde_json::json!({
        "model": model,
        "temperature": 0.0,
        "max_tokens": 300,
        "messages": [
            {"role": "system", "content": prompt},
            {"role": "user", "content": format!("文字：\n{body}")}
        ]
    });
    let mut response = ureq::post(&format!(
        "{}/chat/completions",
        base_url.trim_end_matches('/')
    ))
    .header("Authorization", &format!("Bearer {api_key}"))
    .header("Content-Type", "application/json")
    .send_json(payload)
    .map_err(|error| format!("模型调用失败：{error}"))?;
    let parsed: Value = response
        .body_mut()
        .read_json()
        .map_err(|error| format!("模型返回不是 JSON：{error}"))?;
    let raw = parsed["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();
    Ok(parse_verdict(&raw))
}

/// 从模型输出里取 JSON；取不到就降级成 unknown，不猜。
pub fn parse_verdict(raw: &str) -> Verdict {
    let start = raw.find('{');
    let end = raw.rfind('}');
    if let (Some(start), Some(end)) = (start, end)
        && end > start
        && let Ok(parsed) = serde_json::from_str::<Value>(&raw[start..=end])
    {
        return Verdict {
            kind: parsed["kind"].as_str().unwrap_or("unknown").to_string(),
            why: parsed["why"].as_str().unwrap_or("").to_string(),
            fix: parsed["fix"].as_str().unwrap_or("").to_string(),
        };
    }
    Verdict {
        kind: "unknown".into(),
        why: raw.chars().take(120).collect(),
        fix: String::new(),
    }
}

/// 跑整层：挑段、逐段判。返回（送判段数，判为操作记录的段数，逐条结果）。
pub fn judge_segments(
    findings: &[Finding],
    limit: usize,
) -> Result<Vec<(Segment, Verdict)>, String> {
    let segments = suspect_segments(findings, limit);
    if segments.is_empty() {
        return Ok(Vec::new());
    }
    let (model, base_url, api_key) = credentials()?;
    let mut out = Vec::new();
    for segment in segments {
        let verdict =
            classify(&segment.text, &model, &base_url, &api_key).unwrap_or_else(|error| Verdict {
                kind: "error".into(),
                why: error.chars().take(120).collect(),
                fix: String::new(),
            });
        out.push((segment, verdict));
    }
    Ok(out)
}
