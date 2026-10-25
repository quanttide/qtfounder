//! qtfounder-cli：量潮创始人 CLI。
//!
//! 添加新功能三步：
//! 1. 在 `src/` 下新建模块（如 `feature.rs`），用 `mod` 声明
//! 2. 在 `CliCommand` 枚举里注册一个子命令分支
//! 3. 把该功能的依赖加回 `Cargo.toml`

mod agent;
mod audit;
mod code;
mod docs;
mod language;
mod meta;
mod product;
mod think;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "qtfounder", version, about = "量潮创始人 CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,
}

/// 子命令注册表：新功能在这里加一个枚举分支。
#[derive(Subcommand)]
enum CliCommand {
    /// 代码语用检查：词汇、句法、语篇、语用四层
    Code(code::CodeArgs),
    /// 语言检查：落点、术语、数字、句法、语用六层
    Language(language::LanguageArgs),
    /// 文档审计：表格、列表与密度三层
    Docs(docs::DocsArgs),
    /// 认知：负载看消耗落点、reflect 记一条、model 看模型
    Think(think::ThinkArgs),
    /// 归位：判一段内容该进哪一格、要不要跨域
    Meta(meta::MetaArgs),
    /// 产品：需求与澄清，名词读、动词写
    Product(product::ProductArgs),
    /// Agent：装 Skill、查写法
    Agent(agent::AgentArgs),
    /// 文字体检：合并文字与文档审计，每期只说一处
    Audit(audit::AuditArgs),
}

fn main() {
    match Cli::parse().command {
        Some(CliCommand::Code(args)) => match code::run_code(&args) {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(2);
            }
        },
        Some(CliCommand::Language(args)) => match language::run_language(&args) {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(2);
            }
        },
        Some(CliCommand::Docs(args)) => match docs::run_docs(&args) {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(2);
            }
        },
        Some(CliCommand::Think(args)) => match think::run(&args) {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(2);
            }
        },
        Some(CliCommand::Meta(args)) => match meta::run(&args) {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(2);
            }
        },
        Some(CliCommand::Product(args)) => {
            if let Err(error) = product::run(args.command) {
                product::print_error(&error);
                std::process::exit(error.exit_code());
            }
        }
        Some(CliCommand::Agent(args)) => match agent::run(&args) {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(2);
            }
        },
        Some(CliCommand::Audit(args)) => match audit::run_audit(&args) {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(2);
            }
        },
        None => {
            println!("qtfounder：暂无子命令。");
            println!("新功能在 src/ 下新增模块，并在 CliCommand 注册。");
        }
    }
}
