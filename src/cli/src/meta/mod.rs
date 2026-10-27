//! `qtfounder meta`：归位这一侧。
//!
//! 名词读本体：`model` 看轴与族、`relation` 看边、`axiom` 看前提、`property` 看字段、
//! `category` 看格、`home` 看搬出去的去处、`domain` 看这个域里有什么。动词干三件事：`classify` 把一条内容放进某一格，
//! `validate` 查文档与契约是否还对得上，`compare` 比两个范式。
//! `paradigm` 把归位写成一个范式（对象、态射、合成律）：归位本身是其中之一，别的范式
//! 放在同一张单上，只为对比与吸收。十八格的目录、领域线索、降格规则与分辨句都在
//! `default-contract.toml` 里。
//!
//! 只判断，不改写、不落盘、不搬文件：跨域搬迁按 `meta-classify` 执行。

pub mod axiom;
pub mod category;
pub mod classify;
pub mod compare;
pub mod contract;
pub mod domain;
pub mod home;
pub mod model;
pub mod paradigm;
pub mod property;
pub mod relation;
pub mod validate;

#[cfg(test)]
mod tests;

use clap::Subcommand;

use axiom::AxiomArgs;
use category::CategoryArgs;
use classify::ClassifyArgs;
use compare::CompareArgs;
use domain::DomainArgs;
use home::HomeArgs;
use model::ModelArgs;
use paradigm::ParadigmArgs;
use property::PropertyArgs;
use relation::RelationArgs;
use validate::ValidateArgs;

/// `meta` 下的子命令。
#[derive(Subcommand, Debug)]
pub enum MetaCommand {
    /// 模型：看两族九宫格的轴与格
    Model(ModelArgs),
    /// 关系：看一格的迁出、迁入与分辨句
    Relation(RelationArgs),
    /// 公理：看归位时不重新论证的前提
    Axiom(AxiomArgs),
    /// 属性：看标准属性——字段名、所属组与定义
    Property(PropertyArgs),
    /// 范畴：看十八格各收什么
    Category(CategoryArgs),
    /// 去处：看内容搬出本仓能去哪里
    Home(HomeArgs),
    /// 域（本体论）：看这个域里有哪些范畴、属性与公理
    Domain(DomainArgs),
    /// 范式：看每个范式的对象、态射与合成律
    Paradigm(ParadigmArgs),
    /// 范畴（动词）：给一段内容一张落位卡
    Classify(ClassifyArgs),
    /// 对比（动词）：比两个范式的对象、态射与形状
    Compare(CompareArgs),
    /// 验证：查文档镜像与契约是否还对得上
    Validate(ValidateArgs),
}

/// 归位侧入口。
#[derive(clap::Args, Debug)]
pub struct MetaArgs {
    #[command(subcommand)]
    pub command: MetaCommand,
}

pub fn run(args: &MetaArgs) -> Result<i32, String> {
    match &args.command {
        MetaCommand::Model(model) => model::run(model),
        MetaCommand::Relation(relation) => relation::run(relation),
        MetaCommand::Axiom(axiom) => axiom::run(axiom),
        MetaCommand::Property(property) => property::run(property),
        MetaCommand::Category(category) => category::run(category),
        MetaCommand::Home(home) => home::run(home),
        MetaCommand::Domain(domain) => domain::run(domain),
        MetaCommand::Paradigm(paradigm) => paradigm::run(paradigm),
        MetaCommand::Classify(classify) => classify::run(classify),
        MetaCommand::Compare(compare) => compare::run(compare),
        MetaCommand::Validate(validate) => validate::run(validate),
    }
}
