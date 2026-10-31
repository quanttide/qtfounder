//! `qtfounder code`：给写代码的 AI 配的两道——反思与功能审查。
//!
//! INTENT:
//!   核心目标：给写代码的 AI 一个它自己判不了的事实面——代码功能还对不对得上
//!   边界：不改代码、不下结论、不排严重度；事实到锚点为止，判断归人与 AI
//!   约束：退出码四码有分界（有命中 1、缺席 3）；契约是显性文件，改口径不碰代码
//!   假设：读者是 AI，它逐条执行而不只是读，所以每道只递可核的那部分
//!
//! 两道：
//!
//! - `code reflect` 任务后反思——这轮干完，架构里多了什么以前没有的毛病；
//! - `code review` 功能审查——从代码的事实面（函数清单、复杂度、恒假条件）反推
//!   它在做什么、边界覆盖没，看功能还对不对得上。
//!
//! **代码的语言学特点（命名、句法、语篇、语用四层）归 `language`**，不在这儿——
//! 那四层判的是「下一个维护者能不能看懂」，与自然语言侧同一类活，见
//! `crate::language::code`。本模块只管功能，不管读起来顺不顺。
//!
//! 两道都出骨架与锚点，判断归 AI 与人。验收标准在 `src/code/default-contract.toml`。

pub mod extract;
pub mod ir;
pub mod reflect;
pub mod review;
pub mod rules;

#[cfg(test)]
mod tests;

use std::path::PathBuf;

use clap::{Args, Subcommand};

/// `code` 的两道：反思与功能审查。
#[derive(Args, Debug)]
pub struct CodeArgs {
    #[command(subcommand)]
    pub command: CodeCommand,
    /// 输出 JSON
    #[arg(long, global = true)]
    pub json: bool,
    /// 契约路径（缺省按 src/code/default-contract.toml 逐级往上找）
    #[arg(long, global = true)]
    pub rules: Option<PathBuf>,
}

/// `code` 的子命令；必须给一个——语用检查已在 `language`。
#[derive(Subcommand, Debug)]
pub enum CodeCommand {
    /// 完成任务后的架构反思清单：看什么、为什么算缺陷、长什么样
    Reflect(reflect::ReflectArgs),
    /// 功能审查：从事实面反推功能对不对得上，出报告骨架与交付纪律
    Review(review::ReviewArgs),
}

/// 入口：返回退出码（0 无命中 / 1 有命中 / 2 契约或目标读不了）。
pub fn run_code(args: &CodeArgs) -> Result<i32, String> {
    match &args.command {
        CodeCommand::Reflect(reflect_args) => {
            reflect::run(reflect_args, args.rules.as_deref(), args.json)
        }
        CodeCommand::Review(review_args) => {
            review::run(review_args, args.rules.as_deref(), args.json)
        }
    }
}
