//! `asset axiom`：看资产领域的前提——管这个领域具体怎么运转的那些。
//!
//! 公理按领域归，这一面打 `domain = "asset"` 的。元模型领域（`domain = "meta"`）
//! 走 `meta axiom`；两个领域怎么分与新增领域的规矩，见 user-guide/meta.md
//! 「两个领域的边界」。这里只读，不标层。

use std::path::Path;

use clap::Args;

use crate::meta::axiom::print_domain;
use crate::meta::contract::Contract;

#[derive(Args, Debug)]
pub struct AxiomArgs {
    /// 只看一条公理
    pub name: Option<String>,
    /// 输出 JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &AxiomArgs, rules: Option<&Path>) -> Result<i32, String> {
    let contract = Contract::load(rules)?;
    print_domain(&contract, false, args.name.as_deref(), args.json)
}
