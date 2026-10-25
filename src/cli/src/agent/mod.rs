//! `qtfounder agent`：给 Agent 用的资产。
//!
//! 现在只查 Skill 的命名与写法。这套口径还在试，判断层只读
//! `default-contract.toml`；规则定下来之前不进工具箱。

pub mod contract;
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
