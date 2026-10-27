//! 资产清单：20 种资产在仓库里各是什么状态。
//!
//! 只读文件系统，不改目录。路径相对 `--root` 算，缺省是当前目录。
//! 二十种 = context（入口）+ 十八类长期记忆 + archive（出口）。

use std::path::{Path, PathBuf};

use clap::Args;
use serde_json::json;

use crate::meta::contract::{Asset, Contract};

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
    let rows: Vec<(&Asset, bool)> = contract
        .asset
        .iter()
        .map(|asset| (asset, args.root.join(&asset.path).exists()))
        .filter(|(_, present)| !args.present || *present)
        .collect();

    if args.json {
        let assets: Vec<serde_json::Value> = rows
            .iter()
            .map(|(asset, present)| {
                json!({
                    "name": asset.name,
                    "label": asset.label(),
                    "path": asset.path,
                    "role": asset.role,
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

    let total = contract.asset.len();
    let found = contract
        .asset
        .iter()
        .filter(|asset| args.root.join(&asset.path).exists())
        .count();
    println!("根：{}", args.root.display());
    println!(
        "{total} 种资产，仓库里有 {found} 种，缺 {} 种",
        total - found
    );
    println!();
    for (asset, present) in rows {
        println!(
            "{}  {}  {}  {}  {}",
            asset.name,
            asset.label(),
            asset.path,
            asset.role,
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
            .asset
            .iter()
            .map(|asset| (asset.name.clone(), root.join(&asset.path).exists()))
            .collect()
    }

    #[test]
    fn 清单覆盖二十种资产() {
        let contract = Contract::bundled();
        assert_eq!(contract.asset.len(), 20);
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

    #[test]
    fn 工具链路径到具体仓库() {
        let contract = Contract::bundled();
        let toolkit = contract
            .asset
            .iter()
            .find(|asset| asset.name == "toolkit")
            .unwrap();
        assert_eq!(toolkit.path, "packages/quanttide-founder-toolkit/");
    }
}
