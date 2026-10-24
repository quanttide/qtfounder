//! 判定层：把候选消息分批送模型，拿回「消耗信号 + 来源 + 责任环节」。
//!
//! 三条纪律：
//! 1. **temperature=0**：同一批跑两次要给同一个结论，否则这条命令不可复跑。
//! 2. **没有原文片段就不算命中**：判定必须能被原话核对，窄处写死在解析里，不靠叮嘱模型。
//! 3. **认不出的环节一律记 C（没给判据）**：责任全部落在助手一侧，不回到用户或「不明」。

use std::path::{Path, PathBuf};

use serde_json::Value;

const DEFAULT_MODEL: &str = "deepseek-chat";
const DEFAULT_BASE_URL: &str = "https://api.deepseek.com";

/// 助手一侧的五个环节。责任全在这一侧，没有「用户」「外部」「不明」。
pub const STAGES: [(&str, &str); 5] = [
    ("A", "没照做、忘了、擅自改"),
    ("B", "输出太复杂、看不懂"),
    ("C", "没给判据、没法验证"),
    ("D", "没把选项收敛到能拍"),
    ("E", "没把外部条件的坑先探掉"),
];

pub fn stage_title(code: &str) -> &'static str {
    STAGES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, title)| *title)
        .unwrap_or("没给判据、没法验证")
}

#[derive(Clone, Debug)]
pub struct Source {
    pub label: String,
    pub quote: String,
}

#[derive(Clone, Debug)]
pub struct Judged {
    /// "fatigue" / "friction" / 空
    pub signal: String,
    pub implicit: bool,
    pub intensity: u8,
    pub stage: String,
    pub reason: String,
    pub sources: Vec<Source>,
}

impl Judged {
    pub fn none() -> Self {
        Judged {
            signal: String::new(),
            implicit: false,
            intensity: 0,
            stage: String::new(),
            reason: String::new(),
            sources: Vec::new(),
        }
    }
}

/// 提示词查找顺序：`--prompt` → `QTFOUNDER_THINK_PROMPT` → 从当前目录逐级往上找 →
/// `~/.config/qtfounder/think/`。四道都找不到就报错，**不内置兜底**——
/// 静默回退会让人以为改过的口径生效了。
pub fn prompt_path(custom: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(path) = custom {
        if path.is_file() {
            return Ok(path.to_path_buf());
        }
        return Err(format!("--prompt 指的文件不存在：{}", path.display()));
    }
    if let Ok(env_path) = std::env::var("QTFOUNDER_THINK_PROMPT")
        && !env_path.is_empty()
    {
        let path = PathBuf::from(&env_path);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!("QTFOUNDER_THINK_PROMPT 指的文件不存在：{env_path}"));
    }
    if let Ok(start) = std::env::current_dir() {
        let mut dir = start.as_path();
        loop {
            for candidate in [
                "src/think/load/prompt.md",
                "src/cli/src/think/load/prompt.md",
            ] {
                let path = dir.join(candidate);
                if path.is_file() {
                    return Ok(path);
                }
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let path = Path::new(&home).join(".config/qtfounder/think/prompt.md");
        if path.is_file() {
            return Ok(path);
        }
    }
    let bundled = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/think/load/prompt.md");
    if bundled.is_file() {
        return Ok(bundled);
    }
    Err(
        "找不到判定口径（prompt.md）：用 --prompt 指定，或把它装到 ~/.config/qtfounder/think/"
            .to_string(),
    )
}

pub fn load_prompt(custom: Option<&Path>) -> Result<String, String> {
    let path = prompt_path(custom)?;
    std::fs::read_to_string(&path).map_err(|error| format!("读 {} 失败：{error}", path.display()))
}

/// 凭据：LLM_API_KEY → ~/.pi/agent/auth.json → ~/.bashrc 里的 export。密钥只读不打印。
pub fn credentials() -> Result<(String, String, String), String> {
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
    // Hermes 的凭据池把这个 key 记为 env:DEEPSEEK_API_KEY，而它只 export 在交互式 shell
    // 的 .bashrc 里，非交互进程读不到环境变量，所以这里自己解析一次。
    let bashrc = Path::new(&home).join(".bashrc");
    if let Ok(text) = std::fs::read_to_string(&bashrc) {
        for line in text.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("export DEEPSEEK_API_KEY=") {
                let key = rest.trim_matches(|c| c == '"' || c == '\'').trim();
                if !key.is_empty() {
                    return Ok((model, base_url, key.to_string()));
                }
            }
        }
    }
    Err(format!(
        "没有可用密钥：设 LLM_API_KEY，或在 {} 里配 deepseek，或在 ~/.bashrc 里 export DEEPSEEK_API_KEY",
        auth.display()
    ))
}

fn clip(text: &str, max: usize) -> String {
    let squeezed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if squeezed.chars().count() <= max {
        return squeezed;
    }
    let mut out: String = squeezed.chars().take(max).collect();
    out.push('…');
    out
}

/// 一批消息拼成提示词。
pub fn build_batch(items: &[(String, String)]) -> String {
    let mut body = String::from("以下是待判定的消息，逐条判定：\n\n");
    for (id, content) in items {
        body.push_str(&format!(
            "<message id=\"{id}\">{}</message>\n",
            clip(content, 900)
        ));
    }
    body
}

/// 判一批。返回（id → 判定，解析失败说明）。
pub fn judge_batch(
    items: &[(String, String)],
    prompt: &str,
    model: &str,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<(String, Judged)>, String> {
    if items.is_empty() {
        return Ok(Vec::new());
    }
    let payload = serde_json::json!({
        "model": model,
        "temperature": 0.0,
        "max_tokens": 8000,
        "messages": [
            {"role": "system", "content": prompt},
            {"role": "user", "content": build_batch(items)}
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
    Ok(parse_judged(&raw))
}

/// 从模型输出里取 JSON 数组。模型常见偏差：前后带说明、两个数组连写、键名换语言。
/// 逐个 `[` 位置试解析，取第一个能解析成功的数组——不猜、不截断。
pub fn parse_judged(raw: &str) -> Vec<(String, Judged)> {
    let bytes = raw.as_bytes();
    let mut out = Vec::new();
    let mut start = 0usize;
    while let Some(open) = raw[start..].find('[') {
        let open = start + open;
        // 括号计数找配对的 ]
        let mut depth = 0i32;
        let mut close = None;
        let mut in_str = false;
        let mut escaped = false;
        for (idx, byte) in bytes.iter().enumerate().skip(open) {
            let ch = *byte as char;
            if in_str {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_str = false;
                }
                continue;
            }
            match ch {
                '"' => in_str = true,
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(idx);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(close) = close else {
            start = open + 1;
            continue;
        };
        if let Ok(Value::Array(items)) = serde_json::from_str::<Value>(&raw[open..=close]) {
            for item in items {
                if let Some((id, judged)) = parse_item(&item) {
                    out.push((id, judged));
                }
            }
            start = close + 1;
        } else {
            start = open + 1;
        }
    }
    out
}

/// 从模型回的对象里按候选键名取第一个值——不同模型给的外层键名不一样。
fn pick<'a>(parsed: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|key| parsed.get(*key))
}

fn parse_item(item: &Value) -> Option<(String, Judged)> {
    let id = pick(item, &["id", "msg_id", "编号"])?
        .as_str()?
        .trim()
        .to_string();
    let signal_raw = pick(item, &["signal", "信号", "kind", "类型"])
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_lowercase();
    let mut signal = String::new();
    if signal_raw.contains("fatigue") || signal_raw.contains("疲劳") || signal_raw == "累" {
        signal = "fatigue".to_string();
    } else if signal_raw.contains("friction")
        || signal_raw.contains("摩擦")
        || signal_raw.contains("烦")
    {
        signal = "friction".to_string();
    }
    let mut sources: Vec<Source> = Vec::new();
    if let Some(Value::Array(list)) = pick(item, &["sources", "来源"]) {
        for entry in list {
            let (label, quote) = match entry {
                Value::Object(_) => {
                    let label = pick(entry, &["label", "来源", "source", "name", "短语"])
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    let quote = pick(entry, &["quote", "原句", "evidence", "证据"])
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    (label, quote)
                }
                Value::String(text) => (text.trim().to_string(), String::new()),
                _ => (String::new(), String::new()),
            };
            if !label.is_empty() {
                sources.push(Source { label, quote });
            }
        }
    }
    // 硬口径：没有任何原文片段的判定不算命中。
    if signal.is_empty() || sources.iter().all(|s| s.quote.is_empty()) {
        return Some((id, Judged::none()));
    }
    sources.retain(|s| !s.quote.is_empty());
    let stage_raw = pick(item, &["stage", "环节"])
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_uppercase();
    let stage = match stage_raw.chars().next() {
        Some('A') => "A",
        Some('B') => "B",
        Some('D') => "D",
        Some('E') => "E",
        _ => "C",
    }
    .to_string();
    let intensity = pick(item, &["intensity", "强度"])
        .and_then(|v| v.as_u64())
        .unwrap_or(3)
        .clamp(1, 5) as u8;
    let implicit = pick(item, &["implicit", "隐式"])
        .map(|v| match v {
            Value::Bool(b) => *b,
            Value::String(s) => matches!(s.trim(), "true" | "是" | "1"),
            _ => false,
        })
        .unwrap_or(false);
    let reason = pick(item, &["reason", "判据", "依据", "理由"])
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    Some((
        id,
        Judged {
            signal,
            implicit,
            intensity,
            stage,
            reason,
            sources,
        },
    ))
}
