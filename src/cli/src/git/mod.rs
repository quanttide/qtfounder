//! `qtfounder git`：工作区的 Git 环境。
//!
//! 只管钩子的装、查、卸与执行，以及子模块指针的检查；不改内容，不改工作区。

mod common;
mod hook;
mod run;
mod submodule;

#[cfg(test)]
mod tests;

use clap::Subcommand;

/// `git` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum GitCommand {
    /// Hook：装到主仓与子模块、查配置漂移、执行钩子
    Hook(hook::HookArgs),
    /// Submodule：查指针、改动与推送状态
    Submodule(submodule::SubmoduleArgs),
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
        GitCommand::Submodule(submodule) => submodule::run(submodule),
    }
}
