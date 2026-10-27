//! `qtfounder meta`：归位这一侧。
//!
//! 检查器问「写得好不好」，这里问「该放在哪」。十八格的目录、每个领域的线索、
//! 降格规则与分辨句都在 `default-contract.toml` 里，判定层只读它。
//!
//! 只判定，不改写、不落盘、不搬文件：跨域搬迁按 `meta-classify` 执行。

pub mod classify;
pub mod contract;
pub mod relation;

#[cfg(test)]
mod tests;

use clap::Subcommand;

use classify::ClassifyArgs;
use relation::RelationArgs;

/// `meta` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum MetaCommand {
    /// 归位：给一段内容一张落位卡（域、格、档、写法）
    Classify(ClassifyArgs),
    /// 关系：看一格的迁出、迁入与分辨句
    Relation(RelationArgs),
}

/// 归位侧入口。
#[derive(clap::Args, Debug)]
pub struct MetaArgs {
    #[command(subcommand)]
    pub command: MetaCommand,
}

pub fn run(args: &MetaArgs) -> Result<i32, String> {
    match &args.command {
        MetaCommand::Classify(classify) => classify::run(classify),
        MetaCommand::Relation(relation) => relation::run(relation),
    }
}
