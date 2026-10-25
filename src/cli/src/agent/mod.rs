//! `qtfounder agent`：给 Agent 用的资产。
//!
//! 两层：`install` 把嵌入的 Skill 装到目标仓（命令名是 `qtfounder skill`），
//! `skill` 查 Skill 的命名与写法（命令名是 `qtfounder agent skill`）。
//! 后者标准还在试，规则在 `default-contract.toml`。

pub mod contract;
pub mod install;
mod registry;
mod skill;

#[cfg(test)]
mod tests;

use clap::Subcommand;

/// `agent` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum AgentCommand {
    /// Skill：查命名、说明与正文的写法
    Skill(skill::SkillArgs),
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
    }
}
