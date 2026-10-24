//! `think load`：从对话语料里看消耗落在哪。
//!
//! 判的不是「情绪好不好」，而是**这股消耗来自哪里、落在助手哪个环节上**——
//! 每条命中都要挂一句原话，判不动的就明说不判，不用「双方都有」含糊过去。
//! 口径写在 `prompt.md`；契约（环节定义）在 `judge.rs::STAGES`。

pub mod corpus;
pub mod judge;
pub mod screen;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::PathBuf;

use clap::Args;

use judge::{Judged, STAGES};

/// 负载：从会话库看消耗信号落在助手的哪个环节。
#[derive(Args, Debug)]
pub struct LoadArgs {
    /// Hermes 会话库（只读）
    #[arg(long, default_value = "~/.hermes/state.db")]
    pub db: String,
    /// 只取该日期之后的语料（YYYY-MM-DD）
    #[arg(long)]
    pub since: Option<String>,
    /// 本轮最多判多少条（控成本）
    #[arg(long, default_value_t = 200)]
    pub limit: usize,
    /// 每批送几条
    #[arg(long, default_value_t = 20)]
    pub batch: usize,
    /// 并发几批
    #[arg(long, default_value_t = 4)]
    pub workers: usize,
    /// 来源清单列几条
    #[arg(long, default_value_t = 10)]
    pub top: usize,
    /// 命中数超过这个值就以退出码 1 结束；0 表示「有命中就报 1」
    #[arg(long, default_value_t = 0)]
    pub max_hits: usize,
    /// 把完整报告写成 markdown
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// 判定口径（prompt.md）路径；缺省按 QTFOUNDER_HEALTH_PROMPT、源码树、~/.config/qtfounder/health/ 的顺序找
    #[arg(long)]
    pub prompt: Option<PathBuf>,
    /// 只统计候选量，不调模型
    #[arg(long)]
    pub dry_run: bool,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

/// 一条命中的完整记录，报告与 JSON 都用它。
struct Hit {
    id: String,
    date: String,
    hour: u32,
    signal: String,
    implicit: bool,
    intensity: u8,
    stage: String,
    reason: String,
    sources: Vec<(String, String)>,
}

/// 报告长、分支多，是因为它同时做四件事：取语料、过筛、送判、按「消耗去向」汇总。
/// 不拆开是因为四步共用同一批计数，拆了就要互相传十来个临时量，读者反而要来回跳。
pub fn run_load(args: &LoadArgs) -> Result<i32, String> {
    let db = corpus::expand_home(&args.db);
    let messages = corpus::load(&db, args.since.as_deref())?;
    let total = messages.len();
    let (cands, rest) = screen::screen(messages);
    let cand_total = cands.len();

    println!("qtfounder think load");
    println!(
        "语料：{} 条用户消息（同会话同内容已去重，来自 {}）",
        total,
        db.display()
    );
    println!(
        "进入判定 {} 条；{} 条过短或纯确认，未进池",
        cand_total, rest
    );

    if args.dry_run {
        println!("（--dry-run：只统计候选量，没有调模型）");
        return Ok(0);
    }

    let picked: Vec<(String, Vec<String>, corpus::Message)> = cands
        .into_iter()
        .take(args.limit)
        .map(|(message, hits)| {
            let id = message.id.clone();
            (id, hits, message)
        })
        .collect();
    if picked.is_empty() {
        println!("没有候选，未调模型。");
        return Ok(0);
    }
    if picked.len() < cand_total {
        println!(
            "本轮只判前 {} 条（--limit），其余 {} 条下轮再看",
            picked.len(),
            cand_total - picked.len()
        );
    }

    let (model, base_url, api_key) = judge::credentials()?;
    let prompt = judge::load_prompt(args.prompt.as_deref())?;
    let inputs: Vec<(String, String)> = picked
        .iter()
        .map(|(id, _, message)| (id.clone(), message.content.clone()))
        .collect();
    let batches: Vec<Vec<(String, String)>> = inputs
        .chunks(args.batch.max(1))
        .map(|chunk| chunk.to_vec())
        .collect();
    let worker_count = args.workers.clamp(1, batches.len().max(1));
    let mut groups: Vec<Vec<Vec<(String, String)>>> = vec![Vec::new(); worker_count];
    for (idx, batch) in batches.into_iter().enumerate() {
        groups[idx % worker_count].push(batch);
    }

    let mut judged: HashMap<String, Judged> = HashMap::new();
    let mut failures: Vec<String> = Vec::new();
    std::thread::scope(|scope| {
        let handles: Vec<_> = groups
            .into_iter()
            .map(|group| {
                let (model, base_url, api_key) = (model.clone(), base_url.clone(), api_key.clone());
                let prompt = prompt.clone();
                scope.spawn(move || {
                    let mut out: Vec<(String, Judged)> = Vec::new();
                    let mut errs: Vec<String> = Vec::new();
                    for batch in group {
                        match judge::judge_batch(&batch, &prompt, &model, &base_url, &api_key) {
                            Ok(items) => out.extend(items),
                            Err(error) => errs.push(error),
                        }
                    }
                    (out, errs)
                })
            })
            .collect();
        for handle in handles {
            match handle.join() {
                Ok((out, errs)) => {
                    for (id, verdict) in out {
                        judged.insert(id, verdict);
                    }
                    failures.extend(errs);
                }
                Err(_) => failures.push("一个判定线程异常退出".to_string()),
            }
        }
    });

    for failure in failures.iter().take(3) {
        eprintln!("判定失败：{failure}");
    }

    let mut hits: Vec<Hit> = Vec::new();
    let mut lexical_hits = 0usize;
    for (id, words, message) in &picked {
        if !words.is_empty() {
            lexical_hits += 1;
        }
        let Some(verdict) = judged.get(id) else {
            continue;
        };
        if verdict.signal.is_empty() {
            continue;
        }
        hits.push(Hit {
            id: id.clone(),
            date: message.date.clone(),
            hour: message.hour,
            signal: verdict.signal.clone(),
            implicit: verdict.implicit,
            intensity: verdict.intensity,
            stage: verdict.stage.clone(),
            reason: verdict.reason.clone(),
            sources: verdict
                .sources
                .iter()
                .map(|s| (s.label.clone(), s.quote.clone()))
                .collect(),
        });
    }
    hits.sort_by(|a, b| a.date.cmp(&b.date).then(a.hour.cmp(&b.hour)));

    let fatigue: Vec<&Hit> = hits.iter().filter(|h| h.signal == "fatigue").collect();
    let friction: Vec<&Hit> = hits.iter().filter(|h| h.signal == "friction").collect();

    let mut stage_counts: HashMap<String, usize> = HashMap::new();
    for hit in &hits {
        *stage_counts.entry(hit.stage.clone()).or_insert(0) += 1;
    }
    let mut stage_rows: Vec<(String, usize)> = stage_counts.into_iter().collect();
    stage_rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let mut label_counts: HashMap<String, (usize, String)> = HashMap::new();
    for hit in &hits {
        for (label, quote) in &hit.sources {
            let entry = label_counts
                .entry(label.clone())
                .or_insert((0, quote.clone()));
            entry.0 += 1;
        }
    }
    let mut label_rows: Vec<(String, usize, String)> = label_counts
        .into_iter()
        .map(|(label, (count, quote))| (label, count, quote))
        .collect();
    label_rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let mut month_counts: HashMap<String, usize> = HashMap::new();
    for hit in &fatigue {
        let month = hit.date.chars().take(7).collect::<String>();
        *month_counts.entry(month).or_insert(0) += 1;
    }
    let mut month_rows: Vec<(String, usize)> = month_counts.into_iter().collect();
    month_rows.sort();

    let judged_total = judged.len();
    println!(
        "\n判了 {} 条，命中 {} 条：疲劳 {} 条、摩擦 {} 条（隐式 {} / {}）",
        judged_total,
        hits.len(),
        fatigue.len(),
        friction.len(),
        fatigue.iter().filter(|h| h.implicit).count(),
        friction.iter().filter(|h| h.implicit).count(),
    );

    if !stage_rows.is_empty() {
        println!("\n消耗落在助手这一侧的哪个环节（责任不往用户身上推，也不写「不明」）：");
        for (code, count) in &stage_rows {
            let share = *count as f64 / hits.len() as f64 * 100.0;
            println!(
                "  {code} {} —— {count} 条（{share:.0}%）",
                judge::stage_title(code)
            );
        }
    }

    if !label_rows.is_empty() {
        println!("\n消耗来源（原样列出，未做同义归并）：");
        for (label, count, quote) in label_rows.iter().take(args.top) {
            println!("  {count}×{label}　例：「{}」", truncate(quote, 40));
        }
    }

    if let Some(path) = &args.out {
        let report = render_markdown(
            args,
            total,
            cand_total,
            rest,
            judged_total,
            lexical_hits,
            &hits,
            &stage_rows,
            &label_rows,
            &month_rows,
        );
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("建目录 {} 失败：{error}", parent.display()))?;
        }
        std::fs::write(path, report)
            .map_err(|error| format!("写报告 {} 失败：{error}", path.display()))?;
        println!("\n报告已写入 {}", path.display());
    }

    if args.json {
        let rows: Vec<_> = hits
            .iter()
            .map(|hit| {
                serde_json::json!({
                    "id": hit.id,
                    "date": hit.date,
                    "hour": hit.hour,
                    "signal": hit.signal,
                    "implicit": hit.implicit,
                    "intensity": hit.intensity,
                    "stage": hit.stage,
                    "reason": hit.reason,
                    "sources": hit.sources.iter().map(|(label, quote)| serde_json::json!({"label": label, "quote": quote})).collect::<Vec<_>>(),
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::json!({
                "messages": total,
                "candidates": cand_total,
                "unpooled": rest,
                "judged": judged_total,
                "hits": rows,
            })
        );
    }

    println!("\n报的是嫌疑不是诊断：每条都在上面挂了原句，逐条读原文再改。");
    let over = if args.max_hits == 0 {
        !hits.is_empty()
    } else {
        hits.len() > args.max_hits
    };
    Ok(i32::from(over))
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max).collect();
    out.push('…');
    out
}

/// 参数多，是因为报告里每一节都要不同的计数；与其在函数里回头算，不如一次传齐。
#[allow(clippy::too_many_arguments)]
fn render_markdown(
    args: &LoadArgs,
    total: usize,
    cand_total: usize,
    rest: usize,
    judged_total: usize,
    lexical_hits: usize,
    hits: &[Hit],
    stage_rows: &[(String, usize)],
    label_rows: &[(String, usize, String)],
    month_rows: &[(String, usize)],
) -> String {
    let mut out = String::new();
    out.push_str("# 消耗检查报告\n\n");
    out.push_str(&format!(
        "- 样本：用户消息 {total} 条（去重后），进入判定 {cand_total} 条，未进池 {rest} 条，实际判了 {judged_total} 条\n"
    ));
    out.push_str(&format!("- 会话库：{}\n", args.db));
    out.push_str("- 本报告回答：这股消耗来自哪里、落在助手哪个环节上\n");
    out.push_str(
        "- 口径：只判说话人自己的状态；技术提问、命令、纠错不算；每条来源必须带原话，无原话不计命中\n\n",
    );

    let fatigue = hits.iter().filter(|h| h.signal == "fatigue").count();
    let friction = hits.iter().filter(|h| h.signal == "friction").count();
    out.push_str("## 一、命中概况\n\n");
    out.push_str(&format!(
        "- 命中 {} 条：疲劳 {fatigue} 条、摩擦 {friction} 条\n",
        hits.len()
    ));
    out.push_str(&format!(
        "- 隐式命中（无关键词、靠语气）：{} 条\n",
        hits.iter().filter(|h| h.implicit).count()
    ));
    out.push_str(&format!(
        "- 词面信号（含累/困/烦等词）覆盖 {lexical_hits} 条——有疲劳词不等于在说自己累\n\n"
    ));

    out.push_str("## 二、消耗落在哪个环节\n\n");
    if hits.is_empty() {
        out.push_str("（无命中。）\n\n");
    } else {
        out.push_str(&format!(
            "分母：{} 条命中。责任全部记在助手一侧，不往用户身上推，也不用「不明」含糊。\n\n",
            hits.len()
        ));
        out.push_str("| 环节 | 条数 | 占比 |\n|---|---|---|\n");
        for (code, count) in stage_rows {
            out.push_str(&format!(
                "| {} {} | {count} | {:.0}% |\n",
                code,
                judge::stage_title(code),
                *count as f64 / hits.len() as f64 * 100.0
            ));
        }
        out.push('\n');
        for (code, _) in stage_rows {
            out.push_str(&format!("### {code} {}\n\n", judge::stage_title(code)));
            for hit in hits.iter().filter(|h| &h.stage == code).take(3) {
                if let Some((label, quote)) = hit.sources.first() {
                    out.push_str(&format!(
                        "- {}　{label}：> 「{}」\n",
                        hit.date,
                        truncate(quote, 60)
                    ));
                }
            }
            out.push('\n');
        }
    }

    out.push_str("## 三、消耗来源（原样列出，未做同义归并）\n\n");
    if label_rows.is_empty() {
        out.push_str("（无。）\n\n");
    } else {
        for (label, count, quote) in label_rows.iter().take(args.top) {
            out.push_str(&format!(
                "- {count}×{label}　例：「{}」\n",
                truncate(quote, 50)
            ));
        }
        out.push('\n');
    }

    out.push_str("## 四、时间分布（疲劳命中）\n\n");
    if month_rows.is_empty() {
        out.push_str("（无。）\n\n");
    } else {
        for (month, count) in month_rows {
            out.push_str(&format!("- {month}：{count} 条\n"));
        }
        out.push('\n');
    }

    out.push_str("## 五、这份报告自带哪些偏差\n\n");
    out.push_str("- 语料只有与助手的对话：与人当面聊的、会议录音都不在这里面\n");
    out.push_str(
        "- 单模型单次判定，没有第二人复核；来源短语是概括，是「值得核对的假设」，不是因果\n",
    );
    out.push_str("- 命中要求原文片段，语气很轻的累会被漏掉；未进池的条数见上，未做穿漏抽样\n");
    out.push_str("- 责任口径由用户定死为「全部在助手一侧」，所以本报告只回答「落在哪个环节」，不回答「谁的错」\n");
    out.push('\n');
    out.push_str("## 六、结论\n\n");
    out.push_str("结论由人写。脚本只出数字与证据。\n");
    let _ = STAGES;
    out
}
