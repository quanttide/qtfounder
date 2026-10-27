//! `qtfounder git`：工作区的 Git 环境。
//!
//! 只管钩子的装、查、卸，不改钩子内容；主仓与各层子模块一并覆盖。

mod hook;

#[cfg(test)]
mod tests;

use clap::Subcommand;

/// `git` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum GitCommand {
    /// Hook：装到主仓与子模块、查配置漂移、卸下
    Hook(hook::HookArgs),
}

/// `git` 入口。
#[derive(clap::Args, Debug)]
pub struct GitArgs {
    #[command(subcommand)]
    pub command: GitCommand,
}

pub fn run(args: &GitArgs) -> Result<i32, String> {
    match &args.command {
        GitCommand::Hook(hook) => hook::run(hook),
    }
}
