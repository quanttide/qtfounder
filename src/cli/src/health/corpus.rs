//! 语料层：从 Hermes 会话库里取说话人自己说的话。
//!
//! 只读打开 `state.db`，只取 `role='user'`——分析的是他自己说了什么状态，不是助手的回应。
//! 同会话内内容完全相同的消息只算一条：会话表里留着被压缩/吸收的副本，不去重会把同一个
//! 「累」算两三次，命中数虚高（实测 3514 条里 1257 条是副本）。

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

/// 一条用户消息。
#[derive(Clone, Debug)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub ts: f64,
    /// 北京时间（+08:00）下的日期与小时。会话库存的是 unix 时间戳。
    pub date: String,
    pub hour: u32,
    pub content: String,
}

pub fn expand_home(raw: &str) -> PathBuf {
    if let Some(rest) = raw.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return Path::new(&home).join(rest);
    }
    PathBuf::from(raw)
}

/// 把 unix 秒换成北京时间（固定 +08:00，无夏令时）下的（年-月-日，小时）。
fn civil_from_ts(ts: f64) -> (String, u32) {
    // 用整数日做历法换算，避免引时区库：CST 恒为 UTC+8。
    let shifted = ts + 8.0 * 3600.0;
    let days = (shifted / 86400.0).floor() as i64;
    let secs_of_day = shifted - (days as f64) * 86400.0;
    let hour = (secs_of_day / 3600.0) as u32 % 24;
    // Howard Hinnant 的 civil_from_days
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (format!("{year:04}-{m:02}-{d:02}"), hour)
}

/// 读全部用户消息（去重、按时间排序）。`since` 给日期前缀（如 2026-09-01）时只取之后的。
pub fn load(db: &Path, since: Option<&str>) -> Result<Vec<Message>, String> {
    if !db.exists() {
        return Err(format!("会话库不存在：{}", db.display()));
    }
    let conn = Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|err| format!("打不开会话库 {}：{err}", db.display()))?;
    let mut stmt = conn
        .prepare(
            "select m.id, m.session_id, m.timestamp, m.content from messages m \
             where m.role = 'user' and m.content is not null and length(trim(m.content)) > 0",
        )
        .map_err(|err| format!("查询失败：{err}"))?;
    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let session_id: Option<String> = row.get(1)?;
            let ts: Option<f64> = row.get(2)?;
            let content: String = row.get(3)?;
            Ok((id, session_id.unwrap_or_default(), ts.unwrap_or(0.0), content))
        })
        .map_err(|err| format!("读取失败：{err}"))?;

    let mut out: Vec<Message> = Vec::new();
    for row in rows {
        let (id, session_id, ts, content) = row.map_err(|err| format!("行解析失败：{err}"))?;
        let (date, hour) = civil_from_ts(ts);
        if let Some(since) = since
            && date.as_str() < since
        {
            continue;
        }
        out.push(Message { id, session_id, ts, date, hour, content });
    }
    out.sort_by(|a, b| a.ts.partial_cmp(&b.ts).unwrap_or(std::cmp::Ordering::Equal));

    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut deduped = Vec::with_capacity(out.len());
    for msg in out {
        let key = (msg.session_id.clone(), msg.content.split_whitespace().collect::<Vec<_>>().join(" "));
        if seen.insert(key) {
            deduped.push(msg);
        }
    }
    Ok(deduped)
}
