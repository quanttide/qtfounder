//! `infra` 的存储：一份订阅表，落在 XDG 数据根下。
//!
//! 表里只存「什么事件、跑什么命令」，**不存事件怎么来、周期怎么算**。解释权在驱动方——
//! 今天是 git 钩子与 crontab，将来是云端事件总线。换驱动方不动这张表，这是它唯一的
//! 约束：CLI 自己不解释事件名与周期串，原样透传。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::store;

/// 触发方式。两种并列，不设第三种——将来总线接入也是走「事件」这一种。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Trigger {
    /// 具名事件发生时跑。事件名原样交给驱动方，CLI 不校验、不枚举。
    Event { name: String },
    /// 按周期跑。周期串（`1d`、`*/5 * * * *`…）原样交给驱动方。
    Every { spec: String },
}

/// 一条订阅：一个名字、一种触发、一条命令。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Subscription {
    /// 订阅名，表内唯一，命令按它点名。
    pub name: String,
    pub trigger: Trigger,
    /// 要跑的命令，经 `sh -c` 执行。
    pub run: String,
    /// 停用后 `fire` 与 `run` 都跳过它，但记录还在——停用不是删除。
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// 整张表。包成 struct 而不是裸数组，将来加总线端点之类的字段不动文件形状。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registry {
    #[serde(default)]
    pub subscriptions: Vec<Subscription>,
}

impl Registry {
    /// 按名取一条。取不到返回提示——`fire` 时逐条跑，不需要这个。
    pub fn get(&self, name: &str) -> Option<&Subscription> {
        self.subscriptions.iter().find(|item| item.name == name)
    }

    /// 事件名下的已启用订阅，保持表内顺序。
    pub fn listening(&self, event: &str) -> Vec<&Subscription> {
        self.subscriptions
            .iter()
            .filter(|item| item.enabled && item.trigger == (Trigger::Event { name: event.to_string() }))
            .collect()
    }
}

/// 表文件：`<数据根>/infra/subscriptions.json`。
pub fn registry_path() -> PathBuf {
    store::data_home().join("infra").join("subscriptions.json")
}

/// 读表。文件不在就是空表——第一次 `add` 之前没有表是正常的，不是错误。
pub fn load(path: &Path) -> Result<Registry, String> {
    if !path.exists() {
        return Ok(Registry::default());
    }
    let text = std::fs::read_to_string(path)
        .map_err(|err| format!("读不了 {}：{err}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|err| format!("{} 解析失败：{err}", path.display()))
}

/// 写表。建父目录，末尾补一个换行。
pub fn save(path: &Path, registry: &Registry) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("建不了目录 {}：{err}", parent.display()))?;
    }
    let mut text = serde_json::to_string_pretty(registry)
        .map_err(|err| format!("序列化失败：{err}"))?;
    text.push('\n');
    std::fs::write(path, text)
        .map_err(|err| format!("写不了 {}：{err}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(name: &str) -> Trigger {
        Trigger::Event {
            name: name.to_string(),
        }
    }

    fn sub(name: &str, trigger: Trigger, enabled: bool) -> Subscription {
        Subscription {
            name: name.to_string(),
            trigger,
            run: "true".to_string(),
            enabled,
        }
    }

    #[test]
    fn 表文件不在就是空表() {
        let path = std::env::temp_dir().join(format!("qtfounder-infra-none-{}", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let registry = load(&path).expect("文件不在应是空表，不是错误");
        assert!(registry.subscriptions.is_empty());
    }

    #[test]
    fn 存了再读回来一致() {
        let dir = std::env::temp_dir().join(format!("qtfounder-infra-round-{}", std::process::id()));
        let path = dir.join("subs.json");
        let mut registry = Registry::default();
        registry.subscriptions.push(sub("a", event("pre-commit"), true));
        registry.subscriptions.push(sub("b", Trigger::Every { spec: "0 3 * * *".into() }, true));
        save(&path, &registry).expect("写得进去");
        let back = load(&path).expect("读得回来");
        assert_eq!(back, registry);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 派发名单只收已启用且盯这个事件的() {
        let mut registry = Registry::default();
        registry.subscriptions.push(sub("on", event("pre-commit"), true));
        registry.subscriptions.push(sub("off", event("pre-commit"), false));
        registry.subscriptions.push(sub("other", event("post-release"), true));
        let names: Vec<&str> = registry
            .listening("pre-commit")
            .iter()
            .map(|item| item.name.as_str())
            .collect();
        assert_eq!(names, vec!["on"]);
    }

    #[test]
    fn 没写enabled就当启用() {
        let json = r#"{"name":"a","trigger":{"kind":"event","name":"x"},"run":"true"}"#;
        let item: Subscription = serde_json::from_str(json).expect("能解析");
        assert!(item.enabled, "enabled 缺省应为真");
    }

    #[test]
    fn 周期触发与事件触发存成不同的形() {
        let json = r#"{"kind":"every","spec":"1d"}"#;
        let trigger: Trigger = serde_json::from_str(json).expect("能解析");
        assert_eq!(trigger, Trigger::Every { spec: "1d".to_string() });
    }
}
