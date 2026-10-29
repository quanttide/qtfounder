//! `qtfounder asset`：资产契约、清单、关系、公理与维护。
//!
//! `contract`、`catalog`、`relation`、`axiom` 是只读视图，数据源是归位契约——
//! 十八类长期记忆加两端，共二十种资产。`maintain` 整理 context 的「领域 / 资产」结构，并按关系搬到长期记忆仓。

pub mod axiom;
pub mod catalog;
pub mod contract;
pub mod maintain;
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
    /// 看资产领域的公理：管这个领域怎么运转的前提
    Axiom(axiom::AxiomArgs),
    /// 整理 context 并搬到长期记忆仓
    Maintain(maintain::MaintainArgs),
}

pub fn run(args: &AssetArgs) -> Result<i32, String> {
    match &args.command {
        AssetCommand::Contract(command) => contract::run(command, args.rules.as_deref()),
        AssetCommand::Catalog(command) => catalog::run(command, args.rules.as_deref()),
        AssetCommand::Relation(command) => relation::run(command, args.rules.as_deref()),
        AssetCommand::Axiom(command) => axiom::run(command, args.rules.as_deref()),
        AssetCommand::Maintain(command) => maintain::run(command, args.rules.as_deref()),
    }
}
