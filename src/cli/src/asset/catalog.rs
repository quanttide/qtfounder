//! 资产清单：十八类资产在仓库里各是什么状态。
//!
//! 只读文件系统，不改目录。路径相对 `--root` 算，缺省是当前目录。

use std::path::{Path, PathBuf};

use clap::Args;
use serde_json::json;

use crate::meta::contract::{Category, Contract};

#[derive(Args, Debug)]
pub struct CatalogArgs {
    /// 仓根；资产的相对路径按它算
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    /// 只列仓库里已存在的资产
    #[arg(long)]
    pub present: bool,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &CatalogArgs, rules: Option<&Path>) -> Result<i32, String> {
    let contract = Contract::load(rules)?;
    let rows: Vec<(&Category, bool)> = contract
        .category
        .iter()
        .map(|category| (category, args.root.join(&category.path).exists()))
        .filter(|(_, present)| !args.present || *present)
        .collect();

    if args.json {
        let assets: Vec<serde_json::Value> = rows
            .iter()
            .map(|(category, present)| {
                json!({
                    "name": category.name,
                    "label": category.label,
                    "path": category.path,
                    "family": category.family,
                    "present": present,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "root": args.root.display().to_string(),
                "assets": assets,
            }))
            .unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(0);
    }

    let total = contract.category.len();
    let found = contract
        .category
        .iter()
        .filter(|category| args.root.join(&category.path).exists())
        .count();
    println!("根：{}", args.root.display());
    println!(
        "{} 类资产，仓库里有 {found} 类，缺 {} 类",
        total,
        total - found
    );
    println!();
    for (category, present) in rows {
        println!(
            "{}  {}  {}  {}  {}",
            category.name,
            category.label,
            category.path,
            category.family,
            if present { "有" } else { "缺" }
        );
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(root: &Path) -> Vec<(String, bool)> {
        let contract = Contract::bundled();
        contract
            .category
            .iter()
            .map(|category| (category.name.clone(), root.join(&category.path).exists()))
            .collect()
    }

    #[test]
    fn 清单覆盖十八类资产() {
        let contract = Contract::bundled();
        assert_eq!(contract.category.len(), 18);
    }

    #[test]
    fn 目录在不在按根算() {
        let dir = std::env::temp_dir().join(format!("qtfounder-asset-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("data/report")).unwrap();
        std::fs::write(dir.join("data/report/x.md"), "正文").unwrap();

        let found = rows(&dir);
        assert!(
            found
                .iter()
                .any(|(name, present)| name == "report" && *present)
        );
        assert!(
            found
                .iter()
                .any(|(name, present)| name == "insight" && !present)
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
