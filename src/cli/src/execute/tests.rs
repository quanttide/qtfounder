//! `execute` 的单元测试：抽签的加权、去重与校验。
//!
//! 集成测试跑真二进制看命令面；这里钉判定函数——同一 roll 下选谁是确定的。

use super::*;

#[test]
fn 均匀池里每件权重相等() {
    // 四件都是 1：roll 0..4 依次落在 0..3
    let priorities = [1, 1, 1, 1];
    assert_eq!(pick_index(&priorities, 0), 0);
    assert_eq!(pick_index(&priorities, 1), 1);
    assert_eq!(pick_index(&priorities, 3), 3);
    // roll 超出权重和就取模绕回
    assert_eq!(pick_index(&priorities, 4), 0);
}

#[test]
fn 优先级高的占更大的抽签面() {
    // 权重 3:1：前三格归第一件，第四格归第二件
    let priorities = [3, 1];
    assert_eq!(pick_index(&priorities, 0), 0);
    assert_eq!(pick_index(&priorities, 2), 0);
    assert_eq!(pick_index(&priorities, 3), 1);
}

#[test]
fn 零权重永远抽不中() {
    // 0:5：第一件一次也选不中
    let priorities = [0, 5];
    for roll in 0..20 {
        assert_eq!(pick_index(&priorities, roll), 1);
    }
}

#[test]
fn 全零权重兜底取末位() {
    // 权重和为零是调用方的错；不 panic，取末位
    let priorities = [0, 0, 0];
    assert_eq!(pick_index(&priorities, 7), 2);
}

#[test]
fn 空列表返回零() {
    let priorities: [u32; 0] = [];
    assert_eq!(pick_index(&priorities, 3), 0);
}

#[test]
fn 抽走一件后池子少一件() {
    let mut pool = Pool {
        pending: vec![
            Todo {
                id: "tdo_a".to_string(),
                text: "甲".to_string(),
                priority: 1,
                at: "2026-10-29T00:00:00Z".to_string(),
            },
            Todo {
                id: "tdo_b".to_string(),
                text: "乙".to_string(),
                priority: 1,
                at: "2026-10-29T00:00:00Z".to_string(),
            },
        ],
        drawn: Vec::new(),
    };
    let drawn = pool.take(0);
    assert_eq!(drawn.todo.id, "tdo_a");
    assert!(!drawn.drawn_at.is_empty(), "抽走时该留下时间");
    assert_eq!(pool.pending.len(), 1);
    assert_eq!(pool.pending[0].id, "tdo_b");
    assert_eq!(pool.drawn.len(), 1);
}
