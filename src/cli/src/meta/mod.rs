//! `qtfounder meta`：归位这一侧。
//!
//! 四部分：`model` 看本体（两族九宫格的轴），`relation` 看关系（迁移与分辨句），
//! `axiom` 看公理（归位时不重新论证的前提），`classify` 看范畴（把一条内容放进十八格）。
//! 十八格的目录、领域线索、降格规则与分辨句都在 `default-contract.toml` 里，判定层只读它。
//!
//! 只判定，不改写、不落盘、不搬文件：跨域搬迁按 `meta-classify` 执行。

pub mod axiom;
pub mod classify;
pub mod contract;
pub mod model;
pub mod relation;

#[cfg(test)]
mod tests;

use clap::Subcommand;

use axiom::AxiomArgs;
use classify::ClassifyArgs;
use model::ModelArgs;
use relation::RelationArgs;

/// `meta` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum MetaCommand {
    /// 模型：看两族九宫格的轴与格
    Model(ModelArgs),
    /// 关系：看一格的迁出、迁入与分辨句
    Relation(RelationArgs),
    /// 公理：看归位时不重新论证的前提
    Axiom(AxiomArgs),
    /// 范畴：给一段内容一张落位卡（域、格、档、写法）
    Classify(ClassifyArgs),
}

/// 归位侧入口。
#[derive(clap::Args, Debug)]
pub struct MetaArgs {
    #[command(subcommand)]
    pub command: MetaCommand,
}

pub fn run(args: &MetaArgs) -> Result<i32, String> {
    match &args.command {
        MetaCommand::Model(model) => model::run(model),
        MetaCommand::Relation(relation) => relation::run(relation),
        MetaCommand::Axiom(axiom) => axiom::run(axiom),
        MetaCommand::Classify(classify) => classify::run(classify),
    }
}
