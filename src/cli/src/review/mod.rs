//! `review`：两个评审命令（`code review`、`write review`）共用的体会记录。
//!
//! 机器只递证据（骨架、锚点、纪律），**判断与体会归人**。这份存储就是「归人」的落点：
//! 审完之后，人把这次的使用体会记下来，落在这里；下次审同一件事时摊开看。
//! 可以按需记多个不同的人——体会带 `user`，同一件事可以有 alice 与 bob 各自的一条。
//!
//! 落 `<数据根>/review/experiences.json`，与 `data`、`execute` 同一个 XDG 约定。

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::store::data_home;

/// 数据文件名。
const DATA_FILE: &str = "experiences.json";

/// 一条使用体会：谁、对哪件事、什么角度、说了什么。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Experience {
    /// `code` 或 `write`——两个评审命令各占一个角度。
    pub domain: String,
    /// 谁的体会；同一件事可以有多个人各记一条。
    pub user: String,
    /// 对哪件事（被审的目标，如 `src/meta`）。
    pub target: String,
    /// 体会正文，人自己的话。
    pub body: String,
    /// ISO 8601（UTC，秒级）。
    pub at: String,
}

/// 数据文件的全部内容。
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Store {
    #[serde(default)]
    pub experiences: Vec<Experience>,
}

/// 体会数据文件路径：`<数据根>/review/experiences.json`。
pub fn data_file() -> PathBuf {
    data_file_from(data_home())
}

/// 纯函数形式的路径解析，便于测试钉住。
pub fn data_file_from(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join("review").join(DATA_FILE)
}

fn io_error(action: &str, path: &Path, err: std::io::Error) -> String {
    format!("无法{action} {}：{err}", path.display())
}

impl Store {
    /// 读数据文件；文件不存在按空库处理。
    pub fn load() -> Result<Self, String> {
        load_from(&data_file())
    }

    /// 写数据文件（先建目录）。
    pub fn save(&self) -> Result<(), String> {
        save_to(self, &data_file())
    }

    /// 记一条体会，落在末尾。
    pub fn push(&mut self, experience: Experience) {
        self.experiences.push(experience);
    }

    /// 某个角度下、对某件事的体会（按记录顺序）。
    pub fn for_target(&self, domain: &str, target: &str) -> Vec<&Experience> {
        self.experiences
            .iter()
            .filter(|item| item.domain == domain && item.target == target)
            .collect()
    }
}

/// 从指定文件读，便于测试注入临时目录。
pub fn load_from(path: &Path) -> Result<Store, String> {
    match fs::read_to_string(path) {
        Ok(text) if text.trim().is_empty() => Ok(Store::default()),
        Ok(text) => serde_json::from_str(&text)
            .map_err(|err| format!("数据文件解析失败 {}：{err}", path.display())),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Store::default()),
        Err(err) => Err(io_error("读取", path, err)),
    }
}

/// 写到指定文件，便于测试注入临时目录。
pub fn save_to(store: &Store, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|err| io_error("创建目录", parent, err))?;
    }
    let mut text =
        serde_json::to_string_pretty(store).map_err(|err| format!("数据序列化失败：{err}"))?;
    text.push('\n');
    let mut file = fs::File::create(path).map_err(|err| io_error("写入", path, err))?;
    file.write_all(text.as_bytes())
        .map_err(|err| io_error("写入", path, err))
}

/// 生成 id 里的时间戳：ISO 8601（UTC，秒级）。
pub fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    iso8601(secs)
}

/// 把 UTC 秒数换算成 ISO 8601（Howard Hinnant 的 civil calendar 算法）。
fn iso8601(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (hour, minute, second) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (y + i64::from(m <= 2), m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn experience(user: &str) -> Experience {
        Experience {
            domain: "code".to_string(),
            user: user.to_string(),
            target: "src/meta".to_string(),
            body: format!("{user} 说的体会"),
            at: "2026-10-30T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn 缺文件按空库() {
        let dir = std::env::temp_dir().join(format!("review_store_{}", std::process::id()));
        let path = data_file_from(&dir);
        let store = load_from(&path).expect("缺文件不该报错");
        assert!(store.experiences.is_empty());
    }

    #[test]
    fn 记一条存一来回() {
        let dir = std::env::temp_dir().join(format!("review_round_{}", std::process::id()));
        let path = data_file_from(&dir);
        let mut store = Store::default();
        store.push(experience("alice"));
        save_to(&store, &path).expect("写得进");

        let loaded = load_from(&path).expect("读得回");
        assert_eq!(loaded.experiences.len(), 1);
        assert_eq!(loaded.experiences[0].user, "alice");
    }

    #[test]
    fn 多个人各记一条() {
        let dir = std::env::temp_dir().join(format!("review_multi_{}", std::process::id()));
        let path = data_file_from(&dir);
        let mut store = Store::default();
        store.push(experience("alice"));
        store.push(experience("bob"));
        save_to(&store, &path).expect("写得进");

        let loaded = load_from(&path).expect("读得回");
        let users: Vec<&str> = loaded.experiences.iter().map(|e| e.user.as_str()).collect();
        assert_eq!(users, vec!["alice", "bob"]);
    }

    #[test]
    fn 按角度与目标筛() {
        let mut store = Store::default();
        store.push(experience("alice"));
        let mut other = experience("bob");
        other.target = "src/other".to_string();
        store.push(other);
        let mut write = experience("carol");
        write.domain = "write".to_string();
        store.push(write);

        assert_eq!(store.for_target("code", "src/meta").len(), 1);
        assert_eq!(store.for_target("code", "src/other").len(), 1);
        assert_eq!(store.for_target("write", "src/meta").len(), 1);
        assert_eq!(store.for_target("code", "nothing").len(), 0);
    }

    #[test]
    fn 时间戳形状() {
        // 2026-10-30T00:00:00Z 对应的 UTC 秒数。
        assert_eq!(iso8601(1_793_318_400), "2026-10-30T00:00:00Z");
    }
}
