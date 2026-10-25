//! `meta classify` 的判定测试。
//!
//! 每组一对：真样本判对格，反样本不掉进相邻的格。

use super::classify::{Verdict, judge};
use super::contract::Contract;

fn contract() -> Contract {
    Contract::bundled()
}

fn place(text: &str) -> super::classify::Card {
    let card = judge(text, &contract());
    assert_eq!(
        card.verdict,
        Verdict::Place,
        "{text} 应判落位，实际 {:?}，候选 {:?}",
        card.verdict,
        card.candidates
    );
    card
}

fn key_of(card: &super::classify::Card) -> &str {
    card.cell
        .as_ref()
        .map(|cell| cell.key.as_str())
        .unwrap_or("")
}

#[test]
fn 机制句进洞察且默认假说() {
    let card = place("初稿必冗余，因为遗漏可见、冗余不可见。");
    assert_eq!(key_of(&card), "insight");
    assert_eq!(card.grade.as_deref(), Some("假说"));
    assert!(card.rule.unwrap().contains("抽掉日期"));
}

#[test]
fn 有行动证据的机制句升已确认() {
    let card = place("其实砍一轮已经落地，改了三次都一样。");
    assert_eq!(key_of(&card), "insight");
    assert_eq!(card.grade.as_deref(), Some("已确认"));
}

#[test]
fn 排了先后的是路线图不是洞察() {
    let card = place("先做存量巡检，用数出来的量决定闸门要不要装。");
    assert_eq!(key_of(&card), "roadmap");
    assert_eq!(card.grade.as_deref(), Some("已决策"));

    let pending = place("要不要装闸门还待定。");
    assert_eq!(key_of(&pending), "roadmap");
    assert_eq!(pending.grade.as_deref(), Some("待决策候选"));
}

#[test]
fn 原话进愿景且不分档() {
    let card = place("我要清晰的依据，不要含糊的规则。");
    assert_eq!(key_of(&card), "intention");
    assert_eq!(card.grade, None);
}

#[test]
fn 带日期与当事人的留日志() {
    let card = place("今天上午见了客户，下午改完报价。");
    assert_eq!(key_of(&card), "journal");
    assert_eq!(card.grade, None);
}

#[test]
fn 权责划分进章程() {
    let card = place("章程规定，签字的权责归创始人。");
    assert_eq!(key_of(&card), "bylaw");
}

#[test]
fn 阈值没出处退论文() {
    let card = place("阈值定在七成，凭感觉估的。");
    assert_eq!(key_of(&card), "essay");
    assert!(card.rule.unwrap().contains("阈值没有出处"));
}

#[test]
fn 只对那次成立的进案例() {
    let card = place("那次写入花了三小时，参与人只有我。");
    assert_eq!(key_of(&card), "gallery");
}

#[test]
fn 套不上的一律不收() {
    let card = judge("天气不错。", &contract());
    assert_eq!(card.verdict, Verdict::Reject);
    assert!(card.reason.unwrap().contains("套不上"));
}

#[test]
fn 两格打平只问一句() {
    let card = judge("其实先做哪一步还没定。", &contract());
    assert_eq!(card.verdict, Verdict::Undecided);
    let question = card.question.expect("打平时要给出分辨句");
    assert!(question.contains("先做"), "分辨句应能分开两格：{question}");
    assert!(card.reason.unwrap().contains("不硬猜"));
}

#[test]
fn 正文讲什么领域就指哪个域() {
    let card = judge("小说第三章的文风还要再改。", &contract());
    let domain = card.domain.expect("应指到 write 领域");
    assert_eq!(domain.name, "quanttide-write");
    assert_eq!(card.domain_form, Some("移交"));
    assert_eq!(card.verdict, Verdict::Reject);
    assert!(card.reason.unwrap().contains("套不上"));
}

#[test]
fn 跨域且能落格时判搬走() {
    let card = judge("小说第三章的文风要改，这是我要的效果。", &contract());
    assert_eq!(card.verdict, Verdict::Move);
    assert_eq!(key_of(&card), "intention");
    assert_eq!(card.domain_form, Some("移交"));
}

#[test]
fn 旧东西搬去归档只移动不改写() {
    let card = judge("旧版小说的文风作废。", &contract());
    assert_eq!(card.domain_form, Some("归档"));
}

#[test]
fn 整篇材料先拆条再判() {
    let text = "第一段说今天见了客户。\n\n第二段说其实有规律。\n\n第三段说先做这件事。\n\n第四段说章程要改。\n\n第五段说阈值待标定。\n\n第六段说那次改稿花了三小时。";
    let card = judge(text, &contract());
    assert_eq!(card.verdict, Verdict::Undecided);
    assert!(card.cell.is_none(), "整篇材料不该直接落格");
    assert!(card.reason.unwrap().contains("拆成一条"));
}

#[test]
fn 退出码分四档() {
    assert_eq!(place("今天上午见了客户。").verdict, Verdict::Place);
    assert_eq!(judge("天气不错。", &contract()).verdict, Verdict::Reject);
    assert_eq!(
        judge("其实先做哪一步还没定。", &contract()).verdict,
        Verdict::Undecided
    );
}

#[test]
fn 每张落位卡都带写法() {
    for text in [
        "初稿必冗余，因为遗漏可见、冗余不可见。",
        "先做存量巡检，用数出来的量决定闸门要不要装。",
        "今天上午见了客户。",
    ] {
        let card = place(text);
        let skeleton = card.cell.expect("收了的要有格").skeleton;
        assert!(!skeleton.trim().is_empty(), "{text} 缺写法");
    }
}

#[test]
fn 一个口头禅只按两次算() {
    // 「其实」出现十次也只按两次算，剩下的分要靠别的线索。
    let text = "其实其实其实其实其实其实其实其实其实其实，先做这一步。";
    let card = judge(text, &contract());
    let insight = card
        .candidates
        .iter()
        .find(|candidate| candidate.key == "insight")
        .expect("应命中洞察");
    assert_eq!(insight.score, 2);
}
