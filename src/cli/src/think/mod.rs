//! `qtfounder think`：认知这一侧。
//!
//! 与另外四个子命令不同，这里不是检查器，没有命中清单与阈值：
//! `load` 从会话语料看消耗落在哪，`reflect` 按模型记一条、到点回收，
//! `model` 把当前生效的模型摊开。检查管产出物，这里管怎么想、怎么应对。

pub mod load;
pub mod model;
pub mod reflect;

use clap::Subcommand;

use reflect::ReflectArgs;

/// `think` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum ThinkCommand {
    /// 负载：从会话语料看消耗落在助手哪个环节
    Load(load::LoadArgs),
    /// 记一条反思：情境、自动思维、认知扭曲、证据、行为实验
    Reflect(ReflectArgs),
    /// 看当前生效的认知模型（或指定一份）
    Model(model::ModelArgs),
}

/// 认知侧入口。
#[derive(clap::Args, Debug)]
pub struct ThinkArgs {
    #[command(subcommand)]
    pub command: ThinkCommand,
}

pub fn run(args: &ThinkArgs) -> Result<i32, String> {
    match &args.command {
        ThinkCommand::Load(load) => load::run_load(load),
        ThinkCommand::Reflect(reflect) => reflect::run(reflect),
        ThinkCommand::Model(model) => model::run(model),
    }
}
