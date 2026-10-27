//! `qtfounder agent`：给 Agent 用的资产。
//!
//! 一组命令两件事：`install` 把嵌入的 Skill 装到目标位置，
//! `lint` 查 Skill 的命名与写法。前者只搬运，后者只读。
//! 检查标准还在试，规则在 `default-contract.toml`。

pub mod contract;
mod install;
mod lint;
mod registry;
mod relation;
mod skill;

#[cfg(test)]
mod tests;

use clap::Subcommand;

/// `agent` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum AgentCommand {
    /// Skill：装 Skill、查安装状态、查写法
    Skill(skill::SkillArgs),
    /// 关系：看 Skill 引用了哪些命令、哪些命令组还没 Skill
    Relation(relation::RelationArgs),
}

/// `agent` 入口。
#[derive(clap::Args, Debug)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub command: AgentCommand,
}

pub fn run(args: &AgentArgs) -> Result<i32, String> {
    match &args.command {
        AgentCommand::Skill(skill) => skill::run(skill),
        AgentCommand::Relation(relation) => relation::run(relation),
    }
}
