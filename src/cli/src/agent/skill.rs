//! `qtfounder agent skill`：装 Skill，查写法。
//!
//! 四个入口：`list` 与 `check` 看装到哪一步，`install` 装，`lint` 查命名与写法。
//! 安装只搬运，不动内容；检查只看名字、说明与正文，不改文字。

use clap::{Args, Subcommand};

use super::install::{self, InstallArgs, ListArgs};
use super::lint::{self, LintArgs};

/// `agent skill` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum SkillCommand {
    /// 看嵌了哪些 Skill，装到哪一步
    List(ListArgs),
    /// 装一个或全部
    Install(InstallArgs),
    /// 只查未装与不一致；有就退 1
    Check(ListArgs),
    /// 查命名、说明与正文的写法
    Lint(LintArgs),
}

/// `agent skill` 入口。
#[derive(Args, Debug)]
pub struct SkillArgs {
    #[command(subcommand)]
    pub command: SkillCommand,
}

pub fn run(args: &SkillArgs) -> Result<i32, String> {
    match &args.command {
        SkillCommand::List(list) => install::report(list, false),
        SkillCommand::Check(list) => install::report(list, true),
        SkillCommand::Install(command) => install::install(command),
        SkillCommand::Lint(lint) => lint::run(lint),
    }
}
