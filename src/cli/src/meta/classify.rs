//! `meta classify`：给一段内容出一张落位卡。
//!
//! 判定分四步，顺序不能换：定域（留在这里还是去领域仓）、定格（十八格之一）、
//! 过准入（降格或退论文）、给写法。每张卡都要写清凭什么，
//! 并点出这次用到的公理；判不了就只问一句，不硬猜。

use std::io::Read;
use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use super::contract::{Axiom, Category, Contract, Domain};

#[derive(Args, Debug)]
pub struct ClassifyArgs {
    /// 要判的文件；不给就用 --stdin
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
    /// 从标准输入读
    #[arg(long)]
    pub stdin: bool,
    /// 契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long)]
    pub rules: Option<PathBuf>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

/// 一张卡的结论。收、搬、不收、判不了，四选一。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Place,
    Move,
    Reject,
    Undecided,
}

impl Verdict {
    fn key(self) -> &'static str {
        match self {
            Self::Place => "place",
            Self::Move => "move",
            Self::Reject => "reject",
            Self::Undecided => "undecided",
        }
    }

    fn exit_code(self) -> i32 {
        match self {
            Self::Place | Self::Move => 0,
            Self::Reject => 1,
            Self::Undecided => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub name: String,
    pub label: String,
    pub score: usize,
}

#[derive(Debug, Clone)]
pub struct Card {
    pub verdict: Verdict,
    pub domain: Option<Domain>,
    pub domain_form: Option<&'static str>,
    pub domain_reason: Option<String>,
    pub category: Option<Category>,
    pub grade: Option<String>,
    pub rule: Option<String>,
    pub question: Option<String>,
    pub candidates: Vec<Candidate>,
    pub reason: Option<String>,
    /// 这次判断用到的公理名字：只留出处，不参与算分。
    pub axioms: Vec<String>,
}

pub fn run(args: &ClassifyArgs) -> Result<i32, String> {
    if args.path.is_some() && args.stdin {
        return Err("二选一：给文件路径，或用 --stdin".to_string());
    }
    if args.path.is_none() && !args.stdin {
        return Err("缺输入：给一个文件路径，或用 --stdin".to_string());
    }
    let text = read_input(args)?;
    let contract = Contract::load(args.rules.as_deref())?;
    let card = judge(&text, &contract);
    if args.json {
        print_json(&card)?;
    } else {
        print_card(&card, &contract);
    }
    Ok(card.verdict.exit_code())
}

fn read_input(args: &ClassifyArgs) -> Result<String, String> {
    if let Some(path) = &args.path {
        if !path.is_file() {
            return Err(format!("文件不存在：{}", path.display()));
        }
        return std::fs::read_to_string(path)
            .map_err(|error| format!("读不了 {}：{error}", path.display()));
    }
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|error| format!("读取标准输入失败：{error}"))?;
    Ok(text)
}

/// 判定入口：给一段正文，给一张卡。
pub fn judge(text: &str, contract: &Contract) -> Card {
    // 给的是材料不是线索：一次判一篇，各条的线索会互相干扰。
    let paragraphs = paragraph_count(text);
    if paragraphs > contract.verdict.max_paragraphs {
        return Card {
            verdict: Verdict::Undecided,
            domain: None,
            domain_form: None,
            domain_reason: None,
            category: None,
            grade: None,
            rule: None,
            question: None,
            candidates: Vec::new(),
            reason: Some(format!(
                "这一篇有 {paragraphs} 段，是多条线索的材料，不是一条；一次判一篇会互相干扰。拆成一条再判。"
            )),
            axioms: cited(&contract, Verdict::Undecided, false),
        };
    }

    let mut scored: Vec<(usize, usize, &Category)> = contract
        .category
        .iter()
        .enumerate()
        .map(|(index, category)| (hits(text, &category.cues), index, category))
        .filter(|(score, _, _)| *score > 0)
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let candidates: Vec<Candidate> = scored
        .iter()
        .map(|(score, _, category)| Candidate {
            name: category.name.clone(),
            label: category.label.clone(),
            score: *score,
        })
        .collect();

    let domain = call_domain(text, contract);
    let (domain, domain_form, domain_reason) = match domain {
        Some((domain, matched)) => {
            let form = if hits(text, &contract.verdict.archive_cues) > 0 {
                "归档"
            } else {
                "移交"
            };
            let reason = format!(
                "正文在讲{}；判域只看正文，不看它现在住在哪。",
                matched.join("、")
            );
            (Some(domain), Some(form), Some(reason))
        }
        None => (None, None, None),
    };
    let has_domain = domain.is_some();

    let Some((top_score, _, top)) = scored.first() else {
        return Card {
            verdict: Verdict::Reject,
            domain,
            domain_form,
            domain_reason,
            category: None,
            grade: None,
            rule: None,
            question: None,
            candidates,
            reason: Some("一格都没命中——一句都套不上，留在日志里等它重复。".to_string()),
            axioms: cited(&contract, Verdict::Reject, has_domain),
        };
    };
    if *top_score < contract.verdict.min_hits {
        return Card {
            verdict: Verdict::Reject,
            domain,
            domain_form,
            domain_reason,
            category: None,
            grade: None,
            rule: None,
            question: None,
            candidates,
            reason: Some("命中太弱——一句都套不上，留在日志里等它重复。".to_string()),
            axioms: cited(&contract, Verdict::Reject, has_domain),
        };
    }

    if let Some((second_score, _, second)) = scored.get(1)
        && top_score - second_score < contract.verdict.min_margin
    {
        return Card {
            verdict: Verdict::Undecided,
            domain,
            domain_form,
            domain_reason,
            category: None,
            grade: None,
            rule: None,
            question: Some(contract.question(&top.name, &second.name).to_string()),
            candidates,
            reason: Some(format!(
                "{} 与 {} 只差 {} 分，不硬猜。",
                top.label,
                second.label,
                top_score - second_score
            )),
            axioms: cited(&contract, Verdict::Undecided, has_domain),
        };
    }

    // 准入：名义上进对了格、但过不了的，往下一格走。
    let top = *top;
    let (category, rule) = match contract.transition.iter().find(|transition| {
        transition.from == top.name
            && hits(text, &transition.require_any) > 0
            && hits(text, &transition.lack_any) == 0
    }) {
        Some(transition) => {
            let rule = transition.reason.clone();
            let target = contract
                .category(&transition.to)
                .cloned()
                .unwrap_or_else(|| top.clone());
            (target, Some(rule))
        }
        None => (top.clone(), None),
    };
    let grade = pick_grade(text, &category);
    let rule = rule.unwrap_or_else(|| category.description.clone());
    let verdict = if has_domain {
        Verdict::Move
    } else {
        Verdict::Place
    };
    Card {
        verdict,
        domain,
        domain_form,
        domain_reason,
        category: Some(category),
        grade,
        rule: Some(rule),
        question: None,
        candidates,
        reason: None,
        axioms: cited(&contract, verdict, has_domain),
    }
}

/// 落位卡点名的公理：只列这次判断真用到的，不摊全表；契约里没有的也不点。
///
/// 确认过的排前面，同组内保持契约顺序。公理不参与算分，只留一行出处，事后能对账：
/// 前提被改了还是规则被改了。
fn cited(contract: &Contract, verdict: Verdict, has_domain: bool) -> Vec<String> {
    let mut names = vec!["domain-before-category"];
    if has_domain {
        names.push("text-only");
        names.push("archive-vs-transfer");
    }
    match verdict {
        Verdict::Reject => names.push("no-fit"),
        Verdict::Place | Verdict::Move => {
            names.push("one-place");
            names.push("template-scope");
        }
        Verdict::Undecided => {}
    }
    let mut cited: Vec<&Axiom> = contract
        .axiom
        .iter()
        .filter(|axiom| names.contains(&axiom.name.as_str()))
        .collect();
    cited.sort_by_key(|axiom| axiom.status_key() != "confirmed");
    cited.into_iter().map(|axiom| axiom.name.clone()).collect()
}

/// 段数：空行分隔的正文段；标题、分割线不算，它们不承载线索。
fn paragraph_count(text: &str) -> usize {
    text.split("\n\n")
        .filter(|block| {
            block.chars().any(|ch| {
                !ch.is_whitespace() && !matches!(ch, '-' | '*' | '=' | '_' | '#' | '>' | '|')
            })
        })
        .count()
}

/// 命中数：同一个词重复出现只按两次算，免得一个口头禅把整段判成某一格。
fn hits(text: &str, cues: &[String]) -> usize {
    cues.iter()
        .filter(|cue| !cue.is_empty())
        .map(|cue| text.matches(cue.as_str()).count().min(2))
        .sum()
}

fn call_domain(text: &str, contract: &Contract) -> Option<(Domain, Vec<String>)> {
    let threshold = contract.verdict.min_domain_hits.max(1);
    let mut scored: Vec<(usize, &Domain, Vec<String>)> = contract
        .domain
        .iter()
        .map(|domain| {
            let matched: Vec<String> = domain
                .cues
                .iter()
                .filter(|cue| !cue.is_empty() && text.contains(cue.as_str()))
                .cloned()
                .collect();
            let score = hits(text, &domain.cues);
            (score, domain, matched)
        })
        .filter(|(score, _, _)| *score >= threshold)
        .collect();
    scored.sort_by_key(|item| std::cmp::Reverse(item.0));
    let (score, domain, matched) = scored.first()?;
    // 两个领域打平就不指路：猜错一个域要搬两趟。
    if let Some((next, _, _)) = scored.get(1)
        && next == score
    {
        return None;
    }
    Some(((*domain).clone(), matched.clone()))
}

fn pick_grade(text: &str, category: &Category) -> Option<String> {
    let matched = category
        .grade
        .iter()
        .find(|grade| !grade.fallback && hits(text, &grade.cues) > 0);
    let chosen = matched.or_else(|| category.grade.iter().find(|grade| grade.fallback));
    chosen.map(|grade| grade.label.clone())
}

// ---------- 输出 ----------

fn print_card(card: &Card, contract: &Contract) {
    println!("落位卡");
    println!();
    match (&card.domain, card.domain_form) {
        (Some(domain), Some(form)) => {
            println!("域：去 {}（{}，{}）", domain.label(), domain.name, form);
            if let Some(reason) = &card.domain_reason {
                println!("理由：{reason}");
            }
        }
        _ => println!("域：{}", contract.verdict.stay),
    }

    match card.verdict {
        Verdict::Place | Verdict::Move => {
            if let Some(category) = &card.category {
                println!("格：{}  {}", category.label, category.path);
                println!("档：{}", card.grade.as_deref().unwrap_or("不分档"));
                if let Some(rule) = &card.rule {
                    println!("凭什么：{rule}");
                }
                print_axioms(card, contract);
                println!();
                println!("写法：");
                print!("{}", category.skeleton);
                if !category.skeleton.ends_with('\n') {
                    println!();
                }
            }
        }
        Verdict::Undecided => {
            let listed: Vec<String> = card
                .candidates
                .iter()
                .take(3)
                .map(|item| format!("{} {}", item.label, item.score))
                .collect();
            println!("待定");
            if !listed.is_empty() {
                println!("候选：{}", listed.join("、"));
            }
            if let Some(reason) = &card.reason {
                println!("原因：{reason}");
            }
            if let Some(question) = &card.question {
                println!("问一句：{question}");
            }
            print_axioms(card, contract);
            println!("先别落。");
        }
        Verdict::Reject => {
            println!("不收");
            if let Some(reason) = &card.reason {
                println!("理由：{reason}");
            }
            if card.domain.is_some() {
                println!("本仓十八格都套不上，按 meta-classify 搬走。");
            }
            print_axioms(card, contract);
        }
    }

    println!();
    println!("判定只给去处，不改写、不落盘、不搬文件。");
}

/// 公理只给名字与状态：要看原话走 `meta axiom`，卡片上不重复展开。
fn print_axioms(card: &Card, contract: &Contract) {
    if card.axioms.is_empty() {
        return;
    }
    let listed: Vec<String> = card
        .axioms
        .iter()
        .map(
            |name| match contract.axiom.iter().find(|axiom| &axiom.name == name) {
                Some(axiom) => format!("{name}（{}）", axiom.status_label()),
                None => name.clone(),
            },
        )
        .collect();
    println!("公理：{}", listed.join("、"));
}

#[derive(Serialize)]
struct JsonCard<'a> {
    verdict: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<JsonDomain<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<JsonCategory<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    grade: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rule: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    question: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'a str>,
    candidates: Vec<JsonCandidate<'a>>,
    axioms: Vec<&'a str>,
}

#[derive(Serialize)]
struct JsonDomain<'a> {
    name: &'a str,
    label: &'a str,
    form: &'a str,
    reason: &'a str,
}

#[derive(Serialize)]
struct JsonCategory<'a> {
    name: &'a str,
    label: &'a str,
    path: &'a str,
    family: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    skeleton: Option<&'a str>,
}

#[derive(Serialize)]
struct JsonCandidate<'a> {
    name: &'a str,
    label: &'a str,
    score: usize,
}

fn print_json(card: &Card) -> Result<(), String> {
    let domain = card.domain.as_ref().map(|domain| JsonDomain {
        name: &domain.name,
        label: domain.label(),
        form: card.domain_form.unwrap_or("移交"),
        reason: card.domain_reason.as_deref().unwrap_or(""),
    });
    let category = card.category.as_ref().map(|category| JsonCategory {
        name: &category.name,
        label: &category.label,
        path: &category.path,
        family: &category.family,
        skeleton: Some(&category.skeleton),
    });
    let view = JsonCard {
        verdict: card.verdict.key(),
        domain,
        category,
        grade: card.grade.as_deref(),
        rule: card.rule.as_deref(),
        question: card.question.as_deref(),
        reason: card.reason.as_deref(),
        candidates: card
            .candidates
            .iter()
            .map(|candidate| JsonCandidate {
                name: &candidate.name,
                label: &candidate.label,
                score: candidate.score,
            })
            .collect(),
        axioms: card.axioms.iter().map(String::as_str).collect(),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&view).map_err(|error| format!("JSON 序列化失败：{error}"))?
    );
    Ok(())
}
