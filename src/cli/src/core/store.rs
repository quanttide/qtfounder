//! 存储根：CLI 自己的数据放哪。纯 `std`，不碰 clap 与 serde——core 保持零框架依赖。

use std::path::PathBuf;

/// 应用目录名，拼在 XDG 数据根之下。
pub const APP_DIR: &str = "qtfounder";

/// 存储根：`QTFOUNDER_DATA` → `$XDG_DATA_HOME/qtfounder` → `~/.local/share/qtfounder`。
///
/// 三级都空时退成相对路径 `qtfounder`，不报错：读不到 HOME 的环境下仍能跑，
/// 只是文件落在当前目录——比直接退不出去有用。
pub fn data_home() -> PathBuf {
    data_home_from(
        std::env::var_os("QTFOUNDER_DATA").map(PathBuf::from),
        std::env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}

/// 纯函数形式的根解析，三个入参都显式传入，便于把每个分支都钉住。
pub fn data_home_from(
    override_dir: Option<PathBuf>,
    xdg_data_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> PathBuf {
    if let Some(overridden) = override_dir.filter(|path| !path.as_os_str().is_empty()) {
        return overridden;
    }
    if let Some(xdg) = xdg_data_home.filter(|path| !path.as_os_str().is_empty()) {
        return xdg.join(APP_DIR);
    }
    match home.filter(|path| !path.as_os_str().is_empty()) {
        Some(home) => home.join(".local").join("share").join(APP_DIR),
        None => PathBuf::from(APP_DIR),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 覆盖变量优先于一切() {
        let root = data_home_from(
            Some(PathBuf::from("/tmp/x")),
            Some(PathBuf::from("/xdg")),
            Some(PathBuf::from("/home/u")),
        );
        assert_eq!(root, PathBuf::from("/tmp/x"));
    }

    #[test]
    fn 有xdg就拼应用目录() {
        let root = data_home_from(None, Some(PathBuf::from("/xdg")), Some(PathBuf::from("/home/u")));
        assert_eq!(root, PathBuf::from("/xdg/qtfounder"));
    }

    #[test]
    fn 没xdg才用home下的点local_share() {
        let root = data_home_from(None, None, Some(PathBuf::from("/home/u")));
        assert_eq!(root, PathBuf::from("/home/u/.local/share/qtfounder"));
    }

    #[test]
    fn 三级全空退成相对路径() {
        assert_eq!(data_home_from(None, None, None), PathBuf::from("qtfounder"));
    }

    #[test]
    fn 空串当没有() {
        let root = data_home_from(
            Some(PathBuf::from("")),
            Some(PathBuf::from("/xdg")),
            None,
        );
        assert_eq!(root, PathBuf::from("/xdg/qtfounder"));
    }
}
