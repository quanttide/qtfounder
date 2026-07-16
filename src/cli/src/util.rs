use std::path::PathBuf;

pub fn expand_home(p: &str) -> PathBuf {
    if p.starts_with('~') {
        PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(&p[2..])
    } else {
        PathBuf::from(p)
    }
}

pub fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        s[..max].to_string()
    } else {
        s.to_string()
    }
}

pub fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        0.0
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

pub fn stddev(v: &[f64], m: f64) -> f64 {
    if v.len() < 2 {
        0.0
    } else {
        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
    }
}
