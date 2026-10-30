//! 核心层：跨命令共享的常量与类型。域依赖 core，core 不依赖域。
//!
//! 只放「怎么走」不放「根是什么」：仓根有三种（git toplevel、estate、工具仓根），
//! 合成一个会把 `data/roadmap` 这种子模块情况藏起来，所以原语与语义分开。

pub mod contract;
pub mod exit;
pub mod path;
pub mod store;
