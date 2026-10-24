//! `qtfounder think`：认知这一侧。
//!
//! 与另外四个子命令不同，这里不是检查器，没有命中清单与阈值：`think` 拿的是
//! 一个人的判断模型（认知扭曲、角色归属、行为实验），`reflect` 按这个模型
//! 记一条、到点回收。检查管产出物，这里管怎么想、怎么应对。

pub mod reflect;

use clap::Subcommand;

use crate::think::reflect::ReflectArgs;

/// `think` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum ThinkCommand {
    /// 记一条反思：情境、自动思维、认知扭曲、证据、行为实验
    Reflect(ReflectArgs),
}

/// 认知侧入口。
#[derive(clap::Args, Debug)]
pub struct ThinkArgs {
    #[command(subcommand)]
    pub command: ThinkCommand,
}

pub fn run(args: &ThinkArgs) -> Result<i32, String> {
    match &args.command {
        ThinkCommand::Reflect(reflect) => reflect::run(reflect),
    }
}
