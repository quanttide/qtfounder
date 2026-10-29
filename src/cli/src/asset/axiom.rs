//! `asset axiom`：看资产层的前提——离不开 estate 物理形态的那些。
//!
//! 公理分两层，这一层打 `domain = "asset"` 的。本体论层（`domain = "meta"`）
//! 走 `meta axiom`；分层的判据与已知别扭点见 user-guide/meta.md
//! 「分层的边界是判断题」。这里只读，不判定。

use std::path::Path;

use clap::Args;

use crate::meta::axiom::print_layer;
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
    print_layer(&contract, false, args.name.as_deref(), args.json)
}
