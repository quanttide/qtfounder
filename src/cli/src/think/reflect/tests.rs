//! 反思记录的纯函数测试：解析、缺栏与下一问、到期筛选、重复扭曲。

use serde_json::json;

use super::{
    Experiment, Ledger, Reflection, is_date, list_outcome, missing_fields, next_question,
    normalize_distortion, parse_input, recurring,
};
use crate::think::model::Model;

fn model() -> Model {
    Model::bundled()
}

fn full_record() -> Reflection {
    parse_input(
        &json!({
        "situation": "周三亲自去谈供应商",
        "thought": "只有我去谈才不会谈砸",
        "emotion": "焦虑",
        "intensity": 70,
        "distortions": ["personalization", "catastrophizing"],
        "evidence_against": ["团队谈过三次，只差一个点"],
        "experiment": {"prediction": "不碰两周会出小错", "review_on": "2026-11-07"}
        }),
        &model(),
    )
    .expect("完整记录应能解析")
}

#[test]
fn english_keys_are_stored_as_chinese_labels() {
    let record = full_record();
    assert_eq!(record.distortions, vec!["个人化", "灾难化"]);
}

#[test]
fn unknown_distortion_lists_chinese_labels() {
    let error = normalize_distortion("瞎猜", &model()).expect_err("不该认识");
    assert!(error.contains("个人化"), "{error}");
}

#[test]
fn missing_fields_follow_filling_order() {
    let record = parse_input(&json!({"situation": "a", "thought": "b"}), &model()).expect("解析");
    assert_eq!(
        missing_fields(&record, &model()),
        vec![
            "emotion",
            "distortions",
            "evidence_against",
            "balanced",
            "experiment"
        ]
    );
    assert_eq!(
        next_question(&missing_fields(&record, &model()), &model()),
        "这件事让你感到什么？强度大概几分（0-100）？"
    );
}

#[test]
fn complete_record_has_no_next_question() {
    let record = full_record();
    assert_eq!(missing_fields(&record, &model()), vec!["balanced"]);
    let with_balanced = parse_input(
        &json!({
        "situation": "a", "thought": "b", "emotion": "焦虑", "intensity": 40,
        "distortions": ["读心"], "evidence_against": ["c"], "balanced": "d",
        "experiment": {"prediction": "e"}
        }),
        &model(),
    )
    .expect("解析");
    assert!(missing_fields(&with_balanced, &model()).is_empty());
    assert!(next_question(&[], &model()).contains("回填结果"));
}

#[test]
fn emotion_and_intensity_must_pair() {
    let error = parse_input(
        &json!({"situation": "a", "thought": "b", "emotion": "焦虑"}),
        &model(),
    )
    .expect_err("缺强度应报错");
    assert!(error.contains("intensity"), "{error}");

    let error = parse_input(
        &json!({"situation": "a", "thought": "b", "intensity": 30}),
        &model(),
    )
    .expect_err("缺情绪应报错");
    assert!(error.contains("emotion"), "{error}");
}

#[test]
fn intensity_above_100_is_rejected() {
    let error = parse_input(
        &json!({
            "situation": "a", "thought": "b", "emotion": "焦虑", "intensity": 120
        }),
        &model(),
    )
    .expect_err("越界应报错");
    assert!(error.contains("0-100"), "{error}");
}

#[test]
fn review_date_must_be_iso() {
    assert!(is_date("2026-11-07"));
    assert!(!is_date("11月7日"));
    assert!(!is_date("2026-1-7"));

    let error = parse_input(
        &json!({
            "situation": "a", "thought": "b",
            "experiment": {"prediction": "会出事", "review_on": "11月7日"}
        }),
        &model(),
    )
    .expect_err("日期不对应报错");
    assert!(error.contains("YYYY-MM-DD"), "{error}");
}

#[test]
fn domain_and_control_reject_unknown_values() {
    let error = parse_input(
        &json!({
            "situation": "a", "thought": "b", "domain": "me"
        }),
        &model(),
    )
    .expect_err("不该认识的取值");
    assert!(error.contains("company"), "{error}");
}

#[test]
fn ledger_marks_due_and_open_separately() {
    let mut due = full_record();
    due.id = "ref_DUE1".to_string();
    if let Some(experiment) = &mut due.experiment {
        experiment.review_on = Some("2020-01-01".to_string());
    }
    let mut later = full_record();
    later.id = "ref_LATE".to_string();
    if let Some(experiment) = &mut later.experiment {
        experiment.review_on = Some("2999-01-01".to_string());
    }
    let mut observed = full_record();
    observed.id = "ref_DONE".to_string();
    if let Some(experiment) = &mut observed.experiment {
        experiment.review_on = Some("2020-01-01".to_string());
        experiment.result = Some("没出事".to_string());
    }
    let ledger = Ledger {
        reflections: vec![due, later, observed],
    };

    let outcome = list_outcome(&ledger, "2026-10-24", false, &model());
    assert_eq!(outcome.due.len(), 1);
    assert_eq!(outcome.due[0].id, "ref_DUE1");
    assert!(
        outcome.open.iter().all(|item| item.id != "ref_DONE"),
        "回填过的实验不该留在未填完里"
    );

    let due_only = list_outcome(&ledger, "2026-10-24", true, &model());
    assert!(due_only.open.is_empty(), "--due 时不出未填完");
}

#[test]
fn recurring_needs_two_occurrences() {
    let mut first = full_record();
    first.distortions = vec!["个人化".to_string()];
    let mut second = full_record();
    second.distortions = vec!["个人化".to_string(), "灾难化".to_string()];
    let ledger = Ledger {
        reflections: vec![first, second],
    };
    let groups = recurring(&ledger);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].label, "个人化");
    assert_eq!(groups[0].count, 2);
}

#[test]
fn experiment_without_result_keeps_record_open() {
    let mut record = full_record();
    record.experiment = Some(Experiment {
        prediction: "会出事".to_string(),
        review_on: None,
        result: None,
        emotion_after: None,
    });
    let ledger = Ledger {
        reflections: vec![record],
    };
    let outcome = list_outcome(&ledger, "2026-10-24", false, &model());
    assert_eq!(outcome.open.len(), 1);
}
