//! `qtfounder asset`：资产契约、清单与关系。
//!
//! 三个子命令都是只读视图，数据源是归位契约——十八格就是十八类资产。
//! `contract` 看每类资产该写成什么样，`catalog` 看仓库里有哪些，`relation` 看它们怎么分组。

pub mod catalog;
pub mod contract;
pub mod relation;

use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct AssetArgs {
    #[command(subcommand)]
    pub command: AssetCommand,
    /// 归位契约路径（缺省按 src/meta/default-contract.toml 逐级往上找）
    #[arg(long, global = true)]
    pub rules: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum AssetCommand {
    /// 看资产契约：每类资产的边界、分档与写法
    Contract(contract::ContractArgs),
    /// 看资产清单：路径、家族与仓库里在不在
    Catalog(catalog::CatalogArgs),
    /// 看资产关系：家族与目录分组
    Relation(relation::RelationArgs),
}

pub fn run(args: &AssetArgs) -> Result<i32, String> {
    match &args.command {
        AssetCommand::Contract(command) => contract::run(command, args.rules.as_deref()),
        AssetCommand::Catalog(command) => catalog::run(command, args.rules.as_deref()),
        AssetCommand::Relation(command) => relation::run(command, args.rules.as_deref()),
    }
}
