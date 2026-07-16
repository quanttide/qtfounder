use chrono::{DateTime, Datelike, TimeDelta, Utc};
use clap::Subcommand;

use crate::{git, util};

const EXTRACT_PROMPT: &str = "从以下文本中提取情绪状态。输出JSON：{\"dominant_mood\":\"\",\"valence\":0,\"arousal\":0,\"warning_signs\":[],\"emotional_needs\":[]} 纯JSON。";

#[derive(Clone, Subcommand)]
pub enum Commands {
    Check {
        #[arg(long, default_value = "~/docs/memory")]
        memory: String,
        #[arg(long, default_value = "~/docs/fiction")]
        fiction: String,
        #[arg(long, default_value_t = 7)]
        days: u32,
    },
    Track {
        #[arg(long, default_value = "~/docs/memory")]
        memory: String,
        #[arg(long, default_value = "~/docs/fiction")]
        fiction: String,
        #[arg(long)]
        csv: Option<String>,
    },
    History {
        #[arg(long)]
        csv: Option<String>,
    },
    Profile {
        #[arg(long, default_value = "~/docs/memory")]
        memory: String,
        #[arg(long, default_value = "~/docs/fiction")]
        fiction: String,
        #[arg(long, default_value = "output")]
        output: String,
    },
}

pub fn dispatch(cmd: Commands) {
    match cmd {
        Commands::Check {
            memory,
            fiction,
            days,
        } => cmd_check(&memory, &fiction, days),
        Commands::Track {
            memory,
            fiction,
            csv,
        } => cmd_track(&memory, &fiction, csv),
        Commands::History { csv } => cmd_history(csv),
        Commands::Profile {
            memory,
            fiction,
            output,
        } => cmd_profile(&memory, &fiction, &output),
    }
}

fn git_latest(repo: &str, days: u32, max: usize) -> String {
    let cutoff = Utc::now() - TimeDelta::days(days as i64);
    let path = util::expand_home(repo);
    git::commit_blobs(&path, max)
        .iter()
        .filter(|(ts, _)| DateTime::from_timestamp(*ts, 0).is_some_and(|dt| dt >= cutoff))
        .map(|(ts, content)| {
            let date = DateTime::from_timestamp(*ts, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            let c = util::truncate(content, 2000);
            format!("[{date}] {c}")
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn daily_content(repo: &str, max: usize) -> Vec<(String, String)> {
    use std::collections::HashMap;
    let path = util::expand_home(repo);
    let mut days: HashMap<String, Vec<String>> = HashMap::new();
    for (ts, content) in git::commit_blobs(&path, max) {
        let date = DateTime::from_timestamp(ts, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
        days.entry(date)
            .or_default()
            .push(util::truncate(&content, 2000));
    }
    let mut result: Vec<_> = days.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
        .into_iter()
        .map(|(d, texts)| (d, texts.join("\n\n")))
        .collect()
}

fn extract(text: &str) -> serde_json::Value {
    if text.is_empty() {
        return serde_json::json!({});
    }
    let llm = quanttide_agent::llm::LLM::default();
    let msg = &text[..text.len().min(4000)];
    match llm.complete(
        &[
            quanttide_agent::Message::new("system", EXTRACT_PROMPT),
            quanttide_agent::Message::new("user", msg),
        ],
        quanttide_agent::llm::CompleteOptions::default(),
    ) {
        Ok(r) => serde_json::from_str(r.content.trim()).unwrap_or_default(),
        Err(_) => serde_json::json!({}),
    }
}

fn cmd_check(memory: &str, fiction: &str, days: u32) {
    let (mem, fic) = (
        git_latest(memory, days, 100),
        git_latest(fiction, days, 100),
    );
    println!("日记: {}字  小说: {}字", mem.len(), fic.len());
    let (d, f) = (extract(&mem), extract(&fic));
    let (dv, fv) = (
        d["valence"].as_f64().unwrap_or(0.0),
        f["valence"].as_f64().unwrap_or(0.0),
    );
    println!(
        "现实: {} ({})  创作: {} ({})  差距: {:.1}",
        d["dominant_mood"].as_str().unwrap_or("?"),
        dv,
        f["dominant_mood"].as_str().unwrap_or("?"),
        fv,
        fv - dv
    );
}

fn cmd_track(memory: &str, fiction: &str, csv_path: Option<String>) {
    let csv_path = csv_path.unwrap_or_else(|| "HEALTH.csv".to_string());
    let (mem, fic) = (git_latest(memory, 7, 100), git_latest(fiction, 7, 100));
    let (d, f) = (extract(&mem), extract(&fic));
    let (dv, fv) = (d["valence"].as_f64(), f["valence"].as_f64());
    let gap = match (dv, fv) {
        (Some(d), Some(f)) => format!("{:.1}", f - d),
        _ => String::new(),
    };
    let now = Utc::now();
    let week = format!("{}-W{:02}", now.format("%Y"), now.iso_week().week());
    let date = now.format("%Y-%m-%d").to_string();
    let mut wtr = csv::WriterBuilder::new().from_path(&csv_path).unwrap();
    wtr.write_record(&[
        "week",
        "date",
        "diary_present",
        "diary_mood",
        "diary_valence",
        "fiction_present",
        "fiction_mood",
        "fiction_valence",
        "gap",
        "signals",
    ])
    .ok();
    wtr.write_record(&[
        week.as_str(),
        &date,
        if mem.is_empty() { "0" } else { "1" },
        d["dominant_mood"].as_str().unwrap_or(""),
        &dv.map(|v| format!("{v}")).unwrap_or_default(),
        if fic.is_empty() { "0" } else { "1" },
        f["dominant_mood"].as_str().unwrap_or(""),
        &fv.map(|v| format!("{v}")).unwrap_or_default(),
        &gap,
        &f["warning_signs"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|v| v.as_str().unwrap_or(""))
                    .collect::<Vec<_>>()
                    .join("; ")
            })
            .unwrap_or_default(),
    ])
    .ok();
    wtr.flush().ok();
    println!("已记录 {week} → {csv_path}");
}

fn cmd_history(csv_path: Option<String>) {
    let csv_path = csv_path.unwrap_or_else(|| "HEALTH.csv".to_string());
    let mut rdr = csv::Reader::from_path(&csv_path).unwrap_or_else(|_| {
        println!("暂无数据");
        std::process::exit(0);
    });
    let rows: Vec<_> = rdr.records().filter_map(|r| r.ok()).collect();
    println!("共 {} 周\n", rows.len());
    for r in rows.iter().rev().take(10).rev() {
        println!(
            "{:<10} {:<12} 日记:{:<6} 小说:{:<6} 差距:{:<6}",
            r.get(0).unwrap_or(""),
            r.get(1).unwrap_or(""),
            r.get(4).unwrap_or(""),
            r.get(7).unwrap_or(""),
            r.get(8).unwrap_or("")
        );
    }
    let vals: Vec<f64> = rows
        .iter()
        .filter_map(|r| r.get(4).and_then(|v| v.parse().ok()))
        .collect();
    let fvs: Vec<f64> = rows
        .iter()
        .filter_map(|r| r.get(7).and_then(|v| v.parse().ok()))
        .collect();
    if !vals.is_empty() {
        println!("\n日记均值: {:.2}", util::mean(&vals));
    }
    if !fvs.is_empty() {
        println!("小说均值: {:.2}", util::mean(&fvs));
    }
}

fn cmd_profile(memory: &str, fiction: &str, output: &str) {
    let (mem, fic) = (daily_content(memory, 500), daily_content(fiction, 500));
    let (mem_sample, fic_sample): (Vec<_>, Vec<_>) =
        (mem.iter().take(20).collect(), fic.iter().take(20).collect());
    println!(
        "分析 {} 天日记 + {} 天小说...",
        mem_sample.len(),
        fic_sample.len()
    );
    let (mut d_vals, mut f_vals) = (Vec::new(), Vec::new());
    for (_, text) in &mem_sample {
        if let Some(v) = extract(text)["valence"].as_f64() {
            d_vals.push(v);
        }
    }
    for (_, text) in &fic_sample {
        if let Some(v) = extract(text)["valence"].as_f64() {
            f_vals.push(v);
        }
    }
    let (da, ds, fa) = (
        util::mean(&d_vals),
        util::stddev(&d_vals, util::mean(&d_vals)),
        util::mean(&f_vals),
    );
    let profile = serde_json::json!({"baseline":{"diary_avg_valence":(da*100.0).round()/100.0,"diary_volatility":(ds*100.0).round()/100.0,"fiction_avg_valence":(fa*100.0).round()/100.0,"typical_gap":((fa-da)*100.0).round()/100.0},"thresholds":{"warning_if_below":((da-ds)*100.0).round()/100.0}});
    let out = util::expand_home(output).join("profile.yaml");
    std::fs::create_dir_all(out.parent().unwrap()).ok();
    std::fs::write(&out, serde_yaml::to_string(&profile).unwrap()).ok();
    println!("基线 → {}", out.display());
    println!(
        "日记 {da:.1}±{ds:.1}  小说 {fa:.1}  差距 {:.1}  预警 <{:.1}",
        fa - da,
        da - ds
    );
}
