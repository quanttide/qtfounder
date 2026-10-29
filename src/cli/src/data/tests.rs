//! `data` 的单元测试：根解析、规则校验与结果聚合。

use super::*;

#[test]
fn 根解析三档() {
    let override_dir = data_home_from(
        Some(PathBuf::from("/tmp/x")),
        Some(PathBuf::from("/xdg")),
        Some(PathBuf::from("/home/u")),
    );
    assert_eq!(override_dir, PathBuf::from("/tmp/x"), "覆盖优先");

    let xdg = data_home_from(
        None,
        Some(PathBuf::from("/xdg")),
        Some(PathBuf::from("/home/u")),
    );
    assert_eq!(xdg, PathBuf::from("/xdg/qtfounder"), "其次 XDG");

    let home = data_home_from(None, None, Some(PathBuf::from("/home/u")));
    assert_eq!(home, PathBuf::from("/home/u/.local/share/qtfounder"));

    // 空覆盖视为未设
    let empty = data_home_from(Some(PathBuf::from("")), Some(PathBuf::from("/xdg")), None);
    assert_eq!(empty, PathBuf::from("/xdg/qtfounder"));
}

fn rule(labels: Vec<RuleLabel>) -> Rule {
    Rule {
        id: "rul_TEST".to_string(),
        name: "日志四分类".to_string(),
        source: RuleSource {
            kind: "journal".to_string(),
            split: "blank-line".to_string(),
        },
        labels,
        batch: 20,
        at: "2026-10-29T08:00:00Z".to_string(),
    }
}

fn label(name: &str) -> RuleLabel {
    RuleLabel {
        name: name.to_string(),
        label: format!("{name}显示"),
        define: None,
        yes: None,
        no: None,
    }
}

#[test]
fn 规则校验五把尺子() {
    assert!(validate_rule(&rule(vec![label("fact")])).is_ok());

    let mut empty_name = rule(vec![label("fact")]);
    empty_name.name = "  ".to_string();
    assert!(validate_rule(&empty_name).is_err(), "空规则名");

    let mut zero_batch = rule(vec![label("fact")]);
    zero_batch.batch = 0;
    assert!(validate_rule(&zero_batch).is_err(), "批次为零");

    assert!(validate_rule(&rule(vec![])).is_err(), "没有标签");

    assert!(
        validate_rule(&rule(vec![RuleLabel {
            name: "fact".to_string(),
            label: String::new(),
            define: None,
            yes: None,
            no: None,
        }]))
        .is_err(),
        "显示名为空"
    );

    assert!(
        validate_rule(&rule(vec![label("fact"), label("fact")])).is_err(),
        "机器名重复"
    );
}

#[test]
fn 聚合按任务分组标签按数排序() {
    let results = vec![
        ResultLine {
            task: "tsk_A".to_string(),
            unit: "u1".to_string(),
            label: "fact".to_string(),
            at: None,
        },
        ResultLine {
            task: "tsk_A".to_string(),
            unit: "u2".to_string(),
            label: "feeling".to_string(),
            at: None,
        },
        ResultLine {
            task: "tsk_A".to_string(),
            unit: "u3".to_string(),
            label: "fact".to_string(),
            at: None,
        },
        ResultLine {
            task: "tsk_B".to_string(),
            unit: "u4".to_string(),
            label: "need".to_string(),
            at: None,
        },
    ];

    let tallies = tally(&results);
    assert_eq!(tallies.len(), 2);
    assert_eq!(tallies[0].task, "tsk_A", "按任务名升序");
    assert_eq!(tallies[0].total, 3);
    assert_eq!(
        tallies[0].labels,
        vec![("fact".to_string(), 2), ("feeling".to_string(), 1)],
        "条数降序"
    );
    assert_eq!(tallies[1].total, 1);
}

#[test]
fn 任务包快照字段齐全() {
    // lite 只认任务包，规则快照缺失就断供——钉住序列化形状。
    let task = Task {
        id: "tsk_TEST".to_string(),
        rule_id: "rul_TEST".to_string(),
        rule: rule(vec![label("fact")]),
        units: vec![Unit {
            id: "data/journal/default/2026-07/2026-07-20.md#1".to_string(),
            domain: "default".to_string(),
            text: "默认域的一段。".to_string(),
        }],
        at: "2026-10-29T08:00:00Z".to_string(),
    };
    let json = serde_json::to_value(&task).expect("可序列化");
    assert_eq!(json["rule"]["labels"][0]["name"], "fact");
    assert_eq!(json["units"][0]["domain"], "default");
}
