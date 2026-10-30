//! 路径：跨命令共享的定位原语。
//!
//! 只放「怎么走」，不放「根是什么」——仓根有三种（git toplevel、estate、工具仓根），
//! 合成一个会把 `data/roadmap` 这种子模块情况藏起来，所以这里只给向上走这一步，
//! 各命令自带判据与报错。

use std::path::{Path, PathBuf};

/// 从 `start` 起沿祖先目录向上，先出起点自己，再逐级往上，直到根。
///
/// `max_depth` 限最多出几段；`None` 表示走到根为止。产出顺序即离起点的距离，
/// 调用方要判「就在起点下」还是「在祖先里」，用 `.enumerate()` 数下标。
///
/// 不做 `canonicalize`：调用方各自决定要不要归一，这里只负责走。
pub fn upwards(start: &Path, max_depth: Option<usize>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut dir = Some(start);
    while let Some(current) = dir {
        if max_depth.is_some_and(|limit| out.len() >= limit) {
            break;
        }
        out.push(current.to_path_buf());
        dir = current.parent();
    }
    out
}
