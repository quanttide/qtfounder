//! 四层各一条真样本：判据要能证伪，不能只证明「它跑得起来」。

use super::lexical;
use super::model::Layer;
use super::module;
use super::pragmatic;
use super::rules::Rules;
use super::syntax;

fn parsed(source: &str) -> syn::File {
    syn::parse_file(source).expect("测试样本必须是合法 Rust")
}

fn kinds(source: &str, layer: Layer) -> Vec<&'static str> {
    let rules = Rules::default();
    let tree = parsed(source);
    let findings = match layer {
        Layer::Lexical => lexical::check(&tree, "t.rs", &rules),
        Layer::Syntax => syntax::check(&tree, "t.rs", &rules),
        Layer::Structure => module::check(&tree, "t.rs", &rules),
        Layer::Pragmatic => pragmatic::check(&tree, "t.rs", source, &rules),
    };
    findings.iter().map(|finding| finding.kind).collect()
}

#[test]
fn clean_function_reports_nothing() {
    let source = "/// 把时分合成分钟，供计费使用。\npub fn total_minutes(hours: u32, minutes: u32) -> u32 {\n    hours * 60 + minutes\n}\n";
    let rules = Rules::default();
    let tree = parsed(source);
    let mut all = lexical::check(&tree, "t.rs", &rules);
    all.extend(syntax::check(&tree, "t.rs", &rules));
    all.extend(module::check(&tree, "t.rs", &rules));
    all.extend(pragmatic::check(&tree, "t.rs", source, &rules));
    assert!(all.is_empty(), "干净函数不该有命中：{:?}", all.iter().map(|f| f.kind).collect::<Vec<_>>());
}

#[test]
fn vague_name_is_reported() {
    assert!(kinds("fn f() {\n    let data = 1;\n    let _ = data;\n}\n", Layer::Lexical).contains(&"vague_name"));
}

#[test]
fn short_name_in_loop_is_exempt() {
    let source = "fn f(items: Vec<u32>) -> u32 {\n    let mut total = 0;\n    for i in items {\n        total += i;\n    }\n    total\n}\n";
    let found = kinds(source, Layer::Lexical);
    assert!(!found.contains(&"short_name"), "循环变量不该报短名：{found:?}");
}

#[test]
fn abbreviation_is_reported() {
    assert!(kinds("fn f() {\n    let ctx = 1;\n    let _ = ctx;\n}\n", Layer::Lexical).contains(&"abbreviation"));
}

#[test]
fn deep_nesting_is_reported() {
    let source = "fn f(a: bool, b: bool, c: bool, d: bool) -> u32 {\n    if a {\n        if b {\n            if c {\n                if d {\n                    return 1;\n                }\n            }\n        }\n    }\n    0\n}\n";
    assert!(kinds(source, Layer::Syntax).contains(&"deep_nesting"));
}

#[test]
fn too_many_params_is_reported() {
    let source = "fn f(a: u32, b: u32, c: u32, d: u32, e: u32, g: u32) -> u32 {\n    a + b + c + d + e + g\n}\n";
    assert!(kinds(source, Layer::Syntax).contains(&"too_many_params"));
}

#[test]
fn long_chain_is_reported() {
    let source = "fn f(text: &str) -> String {\n    text.trim().to_lowercase().replace('a', \"b\").split(' ').next().unwrap_or(\"\").to_string()\n}\n";
    assert!(kinds(source, Layer::Syntax).contains(&"long_chain"));
}

#[test]
fn dead_branch_is_reported() {
    assert!(kinds("fn f() {\n    if false {\n        println!(\"never\");\n    }\n}\n", Layer::Structure).contains(&"dead_branch"));
}

#[test]
fn uninformative_name_is_reported() {
    assert!(kinds("fn handle(value: u32) -> u32 {\n    value\n}\n", Layer::Pragmatic).contains(&"uninformative_name"));
}

#[test]
fn docstring_echo_is_reported() {
    let source = "/// retry budget.\nfn retry_budget() -> u32 {\n    3\n}\n";
    assert!(kinds(source, Layer::Pragmatic).contains(&"docstring_echo"));
}

#[test]
fn vague_error_message_is_reported() {
    let source = "fn f(flag: bool) {\n    if flag {\n        panic!(\"bad\");\n    }\n}\n";
    assert!(kinds(source, Layer::Pragmatic).contains(&"vague_error"));
}

#[test]
fn actionable_error_message_passes() {
    let source = "fn f(flag: bool) {\n    if flag {\n        panic!(\"配置文件缺 model 字段，请在 llm 段下补上\");\n    }\n}\n";
    assert!(!kinds(source, Layer::Pragmatic).contains(&"vague_error"));
}

#[test]
fn too_many_defs_is_reported() {
    let mut source = String::new();
    for index in 0..20 {
        source.push_str(&format!("pub fn function_{index}() -> u32 {{ 0 }}\n"));
    }
    assert!(kinds(&source, Layer::Structure).contains(&"too_many_defs"));
}

#[test]
fn rules_come_from_the_embedded_table() {
    let rules = Rules::default();
    assert_eq!(rules.syntax.max_depth, 3);
    assert!(rules.lexical.abbreviations.contains_key("ctx"));
    assert!(rules.lexical.whitelist.iter().any(|item| item == "tz"));
}

#[test]
fn unknown_layer_name_is_rejected() {
    assert!(Layer::parse("nope").is_none());
    assert_eq!(Layer::parse("pragmatic"), Some(Layer::Pragmatic));
}
