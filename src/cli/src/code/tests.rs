//! 两件事一起验：每层验收标准能证伪，两个语言提取器给出一致结论。

use std::path::PathBuf;

use super::extract::{python, rust_lang, specs, tree_sitter_extractor};
use super::ir::Facts;
use super::model::Layer;
use super::{
    lexical, module, pragmatic, reflect,
    rules::{ReflectRule, Rules},
    syntax,
};

fn facts_rs(source: &str) -> Facts {
    rust_lang::extract(source).expect("Rust 样本必须是合法源码")
}

fn facts_py(source: &str) -> Facts {
    python::extract(source).expect("Python 样本必须是合法源码")
}

fn kinds(facts: &Facts, layer: Layer) -> Vec<&'static str> {
    let rules = Rules::load(None).expect("仓里应能找到默认契约：确认 src/code/default-contract.toml 在，或设 QTFOUNDER_CODE_CONTRACT");
    let findings = match layer {
        Layer::Lexical => lexical::check(facts, "t", &rules),
        Layer::Syntax => syntax::check(facts, "t", &rules),
        Layer::Structure => module::check(facts, "t", &rules),
        Layer::Pragmatic => pragmatic::check(facts, "t", &rules),
    };
    findings.iter().map(|finding| finding.kind).collect()
}

fn kinds_with(facts: &Facts, layer: Layer, rules: &Rules) -> Vec<&'static str> {
    let findings = match layer {
        Layer::Lexical => lexical::check(facts, "t", rules),
        Layer::Syntax => syntax::check(facts, "t", rules),
        Layer::Structure => module::check(facts, "t", rules),
        Layer::Pragmatic => pragmatic::check(facts, "t", rules),
    };
    findings.iter().map(|finding| finding.kind).collect()
}

fn all_kinds(facts: &Facts) -> Vec<&'static str> {
    let mut all = Vec::new();
    for layer in Layer::ALL {
        all.extend(kinds(facts, layer));
    }
    all.sort_unstable();
    all
}

#[test]
fn clean_function_reports_nothing() {
    let source = "/// 把时分合成分钟，供计费使用。\npub fn total_minutes(hours: u32, minutes: u32) -> u32 {\n    hours * 60 + minutes\n}\n";
    assert!(all_kinds(&facts_rs(source)).is_empty());
}

#[test]
fn clean_python_function_reports_nothing() {
    let source = "def total_minutes(hours, minutes):\n    \"\"\"把时分合成分钟，供计费使用。\"\"\"\n    return hours * 60 + minutes\n";
    assert!(all_kinds(&facts_py(source)).is_empty());
}

#[test]
fn vague_name_is_reported() {
    assert!(
        kinds(
            &facts_rs("fn f() {\n    let data = 1;\n    let _ = data;\n}\n"),
            Layer::Lexical
        )
        .contains(&"vague_name")
    );
    assert!(
        kinds(
            &facts_py("def f():\n    data = 1\n    print(data)\n"),
            Layer::Lexical
        )
        .contains(&"vague_name")
    );
}

#[test]
fn short_name_in_loop_is_exempt() {
    let rust = "fn f(items: Vec<u32>) -> u32 {\n    let mut total = 0;\n    for i in items {\n        total += i;\n    }\n    total\n}\n";
    let python =
        "def f(items):\n    total = 0\n    for i in items:\n        total += i\n    return total\n";
    assert!(!kinds(&facts_rs(rust), Layer::Lexical).contains(&"short_name"));
    assert!(!kinds(&facts_py(python), Layer::Lexical).contains(&"short_name"));
}

#[test]
fn abbreviation_is_reported() {
    assert!(
        kinds(
            &facts_rs("fn f() {\n    let ctx = 1;\n    let _ = ctx;\n}\n"),
            Layer::Lexical
        )
        .contains(&"abbreviation")
    );
    assert!(
        kinds(
            &facts_py("def f():\n    ctx = 1\n    print(ctx)\n"),
            Layer::Lexical
        )
        .contains(&"abbreviation")
    );
}

#[test]
fn deep_nesting_is_reported_in_both_languages() {
    let rust = "fn f(a: bool, b: bool, c: bool, d: bool) -> u32 {\n    if a {\n        if b {\n            if c {\n                if d {\n                    return 1;\n                }\n            }\n        }\n    }\n    0\n}\n";
    let python = "def f(a, b, c, d):\n    if a:\n        if b:\n            if c:\n                if d:\n                    return 1\n    return 0\n";
    assert!(kinds(&facts_rs(rust), Layer::Syntax).contains(&"deep_nesting"));
    assert!(kinds(&facts_py(python), Layer::Syntax).contains(&"deep_nesting"));
}

#[test]
fn too_many_params_is_reported_in_both_languages() {
    let rust = "fn f(a: u32, b: u32, c: u32, d: u32, e: u32, g: u32) -> u32 {\n    a + b + c + d + e + g\n}\n";
    let python = "def f(a, b, c, d, e, g):\n    return a\n";
    assert!(kinds(&facts_rs(rust), Layer::Syntax).contains(&"too_many_params"));
    assert!(kinds(&facts_py(python), Layer::Syntax).contains(&"too_many_params"));
}

#[test]
fn long_chain_is_reported_in_both_languages() {
    let rust = "fn f(text: &str) -> String {\n    text.trim().to_lowercase().replace('a', \"b\").split(' ').next().unwrap_or(\"\").to_string()\n}\n";
    let python =
        "def f(text):\n    return text.strip().lower().replace('a', 'b').split(' ')[0].upper()\n";
    assert!(kinds(&facts_rs(rust), Layer::Syntax).contains(&"long_chain"));
    assert!(kinds(&facts_py(python), Layer::Syntax).contains(&"long_chain"));
}

#[test]
fn dead_branch_is_reported_in_both_languages() {
    assert!(
        kinds(
            &facts_rs("fn f() {\n    if false {\n        println!(\"never\");\n    }\n}\n"),
            Layer::Structure
        )
        .contains(&"dead_branch")
    );
    assert!(
        kinds(
            &facts_py("def f():\n    if False:\n        print('never')\n"),
            Layer::Structure
        )
        .contains(&"dead_branch")
    );
}

#[test]
fn uninformative_name_is_reported_in_both_languages() {
    assert!(
        kinds(
            &facts_rs("fn handle(a: u32) -> u32 {\n    a\n}\n"),
            Layer::Pragmatic
        )
        .contains(&"uninformative_name")
    );
    assert!(
        kinds(
            &facts_py("def handle(a):\n    return a\n"),
            Layer::Pragmatic
        )
        .contains(&"uninformative_name")
    );
}

#[test]
fn docstring_echo_is_reported_in_both_languages() {
    let rust = "/// retry budget.\nfn retry_budget() -> u32 {\n    3\n}\n";
    let python = "def retry_budget():\n    \"\"\"retry budget.\"\"\"\n    return 3\n";
    assert!(kinds(&facts_rs(rust), Layer::Pragmatic).contains(&"docstring_echo"));
    assert!(kinds(&facts_py(python), Layer::Pragmatic).contains(&"docstring_echo"));
}

#[test]
fn vague_error_message_is_reported_in_both_languages() {
    assert!(
        kinds(
            &facts_rs("fn f(x: bool) {\n    if x {\n        panic!(\"bad\");\n    }\n}\n"),
            Layer::Pragmatic
        )
        .contains(&"vague_error")
    );
    assert!(
        kinds(
            &facts_py("def f(x):\n    if x:\n        raise ValueError('bad')\n"),
            Layer::Pragmatic
        )
        .contains(&"vague_error")
    );
}

#[test]
fn actionable_error_message_passes() {
    let rust = "fn f(x: bool) {\n    if x {\n        panic!(\"配置文件缺 model 字段，请在 llm 段下补上\");\n    }\n}\n";
    assert!(!kinds(&facts_rs(rust), Layer::Pragmatic).contains(&"vague_error"));
}

#[test]
fn name_type_mismatch_is_reported_in_both_languages() {
    let rust = "fn f() {\n    let user_map = vec![1, 2];\n    let _ = user_map;\n}\n";
    let python = "def f():\n    user_list = {'a': 1}\n    print(user_list)\n";
    assert!(kinds(&facts_rs(rust), Layer::Lexical).contains(&"name_type_mismatch"));
    assert!(kinds(&facts_py(python), Layer::Lexical).contains(&"name_type_mismatch"));
}

#[test]
fn too_many_defs_is_reported() {
    let mut rust = String::new();
    for index in 0..20 {
        rust.push_str(&format!("pub fn function_{index}() -> u32 {{ 0 }}\n"));
    }
    assert!(kinds(&facts_rs(&rust), Layer::Structure).contains(&"too_many_defs"));
}

/// 这条是整层设计成立与否的总验收：同一个缺陷两种语言各写一份，结论必须一样。
#[test]
fn both_languages_agree_on_every_sample() {
    let pairs: [(&str, &str, &str); 8] = [
        (
            "vague",
            "def f():\n    data = 1\n    print(data)\n",
            "fn f() {\n    let data = 1;\n    println!(\"{}\", data);\n}\n",
        ),
        (
            "abbrev",
            "def f():\n    ctx = 1\n    print(ctx)\n",
            "fn f() {\n    let ctx = 1;\n    println!(\"{}\", ctx);\n}\n",
        ),
        (
            "params",
            "def f(a, b, c, d, e, g):\n    return a\n",
            "fn f(a: u32, b: u32, c: u32, d: u32, e: u32, g: u32) -> u32 {\n    a\n}\n",
        ),
        (
            "nesting",
            "def f(a, b, c, d):\n    if a:\n        if b:\n            if c:\n                if d:\n                    return 1\n    return 0\n",
            "fn f(a: bool, b: bool, c: bool, d: bool) -> u32 {\n    if a {\n        if b {\n            if c {\n                if d {\n                    return 1;\n                }\n            }\n        }\n    }\n    0\n}\n",
        ),
        (
            "chain",
            "def f(text):\n    return text.strip().lower().replace('a', 'b').split(' ')[0].upper()\n",
            "fn f(text: &str) -> String {\n    text.trim().to_lowercase().replace('a', \"b\").split(' ').next().unwrap_or(\"\").to_uppercase()\n}\n",
        ),
        (
            "dead",
            "def f():\n    if False:\n        print('never')\n",
            "fn f() {\n    if false {\n        println!(\"never\");\n    }\n}\n",
        ),
        (
            "badname",
            "def handle(a):\n    return a\n",
            "fn handle(a: u32) -> u32 {\n    a\n}\n",
        ),
        (
            "vagueerr",
            "def f(x):\n    if x:\n        raise ValueError('bad')\n",
            "fn f(x: bool) {\n    if x {\n        panic!(\"bad\");\n    }\n}\n",
        ),
    ];
    let mut mismatched = Vec::new();
    for (name, python, rust) in pairs {
        let left = all_kinds(&facts_py(python));
        let right = all_kinds(&facts_rs(rust));
        if left != right {
            mismatched.push(format!("{name}: py={left:?} rs={right:?}"));
        }
    }
    assert!(
        mismatched.is_empty(),
        "两个提取器结论不一致：{mismatched:#?}"
    );
}

fn facts_go(source: &str) -> Facts {
    tree_sitter_extractor::extract(source, &specs::go()).expect("Go 样本必须合法")
}

fn facts_ts(source: &str) -> Facts {
    tree_sitter_extractor::extract(source, &specs::typescript()).expect("TS 样本必须合法")
}

fn facts_dart(source: &str) -> Facts {
    tree_sitter_extractor::extract(source, &specs::dart()).expect("Dart 样本必须合法")
}

#[test]
fn go_reports_params_and_nesting() {
    let source = "package main\n\nfunc handle(data int, b int, c int, d int, e int, g int) int {\n\tif data > 0 {\n\t\tif b > 0 {\n\t\t\tif c > 0 {\n\t\t\t\tif d > 0 {\n\t\t\t\t\treturn 1\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n\treturn 0\n}\n";
    let facts = facts_go(source);
    let found = kinds(&facts, Layer::Syntax);
    assert!(found.contains(&"too_many_params"), "{found:?}");
    assert!(found.contains(&"deep_nesting"), "{found:?}");
    assert!(kinds(&facts, Layer::Pragmatic).contains(&"uninformative_name"));
    assert!(kinds(&facts, Layer::Lexical).contains(&"vague_name"));
}

#[test]
fn typescript_reports_params_and_nesting() {
    let source = "function handle(data: number, b: number, c: number, d: number, e: number, g: number): number {\n  if (data > 0) {\n    if (b > 0) {\n      if (c > 0) {\n        if (d > 0) {\n          return 1;\n        }\n      }\n    }\n  }\n  return 0;\n}\n";
    let facts = facts_ts(source);
    assert!(kinds(&facts, Layer::Syntax).contains(&"too_many_params"));
    assert!(kinds(&facts, Layer::Syntax).contains(&"deep_nesting"));
    assert!(kinds(&facts, Layer::Lexical).contains(&"vague_name"));
}

#[test]
fn dart_reports_params_and_nesting() {
    let source = "int handle(int data, int b, int c, int d, int e, int g) {\n  if (data > 0) {\n    if (b > 0) {\n      if (c > 0) {\n        if (d > 0) {\n          return 1;\n        }\n      }\n    }\n  }\n  return 0;\n}\n";
    let facts = facts_dart(source);
    assert!(kinds(&facts, Layer::Syntax).contains(&"too_many_params"));
    assert!(kinds(&facts, Layer::Syntax).contains(&"deep_nesting"));
    assert!(kinds(&facts, Layer::Pragmatic).contains(&"uninformative_name"));
}

#[test]
fn five_languages_agree_on_the_same_defect() {
    let go_source = "package main\n\nfunc handle(a int, b int, c int, d int) int {\n\tif a > 0 {\n\t\tif b > 0 {\n\t\t\tif c > 0 {\n\t\t\t\tif d > 0 {\n\t\t\t\t\treturn 1\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n\treturn 0\n}\n";
    let ts_source = "function handle(a: number, b: number, c: number, d: number): number {\n  if (a > 0) {\n    if (b > 0) {\n      if (c > 0) {\n        if (d > 0) {\n          return 1;\n        }\n      }\n    }\n  }\n  return 0;\n}\n";
    let dart_source = "int handle(int a, int b, int c, int d) {\n  if (a > 0) {\n    if (b > 0) {\n      if (c > 0) {\n        if (d > 0) {\n          return 1;\n        }\n      }\n    }\n  }\n  return 0;\n}\n";
    for (name, facts) in [
        ("go", facts_go(go_source)),
        ("ts", facts_ts(ts_source)),
        ("dart", facts_dart(dart_source)),
    ] {
        let found = all_kinds(&facts);
        assert!(
            found.contains(&"deep_nesting"),
            "{name} 应报深嵌套：{found:?}"
        );
        assert!(
            found.contains(&"uninformative_name"),
            "{name} 应报不诚实函数名：{found:?}"
        );
    }
}

#[test]
fn contract_is_a_real_file_on_disk() {
    let rules = Rules::load(None).expect("仓里应能找到默认契约：确认 src/code/default-contract.toml 在，或设 QTFOUNDER_CODE_CONTRACT");
    assert_eq!(rules.syntax.max_depth, 3);
    assert!(rules.lexical.abbreviations.contains_key("ctx"));
    assert!(rules.lexical.whitelist.iter().any(|item| item == "tz"));
}

#[test]
fn uninformative_names_come_from_contract() {
    let facts = facts_rs("fn handle() {}\n");
    let mut rules = Rules::load(None)
        .expect("仓里应能找到默认契约：确认 src/code/default-contract.toml 在，或设 QTFOUNDER_CODE_CONTRACT");
    assert!(
        kinds_with(&facts, Layer::Pragmatic, &rules).contains(&"uninformative_name"),
        "默认契约里的词表应先命中"
    );
    rules.pragmatic.uninformative_names.clear();
    assert!(
        !kinds_with(&facts, Layer::Pragmatic, &rules).contains(&"uninformative_name"),
        "清空契约词表后不应再报"
    );
}

#[test]
fn a_changed_contract_changes_the_verdict() {
    // 把嵌套阈值放到 10，同一份代码就不该再报深嵌套——证明「只改配置就能改行为」
    let source = "fn f(a: bool, b: bool, c: bool, d: bool) -> u32 {\n    if a {\n        if b {\n            if c {\n                if d {\n                    return 1;\n                }\n            }\n        }\n    }\n    0\n}\n";
    let facts = facts_rs(source);
    let mut rules = Rules::load(None).expect("仓里应能找到默认契约：确认 src/code/default-contract.toml 在，或设 QTFOUNDER_CODE_CONTRACT");
    assert!(kinds_with(&facts, Layer::Syntax, &rules).contains(&"deep_nesting"));
    rules.syntax.max_depth = 10;
    assert!(!kinds_with(&facts, Layer::Syntax, &rules).contains(&"deep_nesting"));
}

#[test]
fn missing_contract_is_an_explicit_error() {
    let missing = std::path::Path::new("/nonexistent/contract.toml");
    let error = Rules::load(Some(missing)).expect_err("契约不存在时必须报错，不能偷偷用内置阈值");
    assert!(
        error.contains("契约读不了"),
        "报错要说清是契约读不了：{error}"
    );
}

#[test]
fn unknown_layer_name_is_rejected() {
    assert!(Layer::parse("nope").is_none());
    assert_eq!(Layer::parse("pragmatic"), Some(Layer::Pragmatic));
}

// ---------- 反思清单 ----------

#[test]
fn 反思清单四条齐备且名字不重复() {
    let rules = Rules::load(None).expect("仓里应能找到默认契约");
    assert_eq!(
        rules.reflect.len(),
        4,
        "契约里应有四条反思规则，改数量要同步 skill 正文"
    );
    let mut seen = std::collections::HashSet::new();
    for rule in &rules.reflect {
        assert!(!rule.name.is_empty(), "name 不能为空");
        assert!(!rule.description.is_empty(), "{} 缺 description", rule.name);
        assert!(!rule.why.is_empty(), "{} 缺 why", rule.name);
        assert!(
            !rule.example.is_empty(),
            "{} 缺 example，--example 会打空行",
            rule.name
        );
        assert!(seen.insert(rule.name.as_str()), "name 重复：{}", rule.name);
    }
    assert_eq!(
        rules.reflect[0].name, "unmarked-additions",
        "头一条是无主的行：顺序即优先级"
    );
}

#[test]
fn 反思输出例子按需才给() {
    let rules = Rules::load(None).expect("仓里应能找到默认契约");
    let picked: Vec<&ReflectRule> = rules.reflect.iter().collect();
    let total = picked.len();

    let plain = reflect::render(&picked, total, false, false).expect("渲染正文");
    assert!(
        plain.contains("为什么算缺陷"),
        "缺省必给 description 与 why"
    );
    assert!(!plain.contains("长什么样"), "缺省不打例子，例子按需");

    let with_example = reflect::render(&picked, total, true, false).expect("渲染正文");
    for rule in &rules.reflect {
        assert!(
            with_example.contains(&rule.example),
            "--example 必须连 {} 的例子一起打",
            rule.name
        );
    }
}

#[test]
fn 反思输出JSON带全四层() {
    let rules = Rules::load(None).expect("仓里应能找到默认契约");
    let picked: Vec<&ReflectRule> = rules.reflect.iter().collect();
    let body = reflect::render(&picked, picked.len(), true, true).expect("渲染 JSON");
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("是合法 JSON");
    let list = parsed.as_array().expect("是数组");
    assert_eq!(list.len(), 4);
    for item in list {
        for key in ["name", "description", "why", "example"] {
            assert!(
                item.get(key)
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| !s.is_empty()),
                "JSON 里 {key} 不能空"
            );
        }
    }
}

// ---------- 审查纪律 ----------

#[test]
fn 审查纪律六条齐备且名字不重复() {
    let rules = Rules::load(None).expect("仓里应能找到默认契约");
    assert_eq!(rules.review.len(), 6, "契约里应有六条审查纪律，改数量要同步 skill 正文");
    let mut seen = std::collections::HashSet::new();
    for rule in &rules.review {
        assert!(!rule.name.is_empty(), "name 不能为空");
        assert!(!rule.description.is_empty(), "{} 缺 description", rule.name);
        assert!(!rule.why.is_empty(), "{} 缺 why", rule.name);
        assert!(seen.insert(rule.name.as_str()), "name 重复：{}", rule.name);
    }
    assert_eq!(rules.review[0].name, "conclusion-first", "头一条是结论先行");
}

#[test]
fn 审查纪律含锚点与判定归人两条() {
    // 这两条是审查与反思、重构的分界：机械层只给锚点，判断归人。
    let rules = Rules::load(None).expect("仓里应能找到默认契约");
    let names: Vec<&str> = rules.review.iter().map(|rule| rule.name.as_str()).collect();
    assert!(names.contains(&"anchor-required"), "写不出锚点就不写");
    assert!(names.contains(&"candidates-not-verdicts"), "模型只给候选，判定归人");
}

#[test]
fn 审查骨架的意图五层与重构同一套() {
    // 反向流程用的五层，必须与 Skill code-refactor 的正向流程一字不差。
    assert_eq!(
        super::review::INTENT_LAYERS.map(|line| line.split('：').next().unwrap()),
        ["核心目标", "边界", "约束", "已知假设", "不确定性"]
    );
}

#[test]
fn 审查JSON的锚点带全八键() {
    // 与 code 自己的 JSON 同形：少了 kind 就没法分组，少了 principle 就不知道违反哪条。
    let rules = Rules::load(None).expect("仓里应能找到默认契约");
    let (findings, _) = super::collect(
        &PathBuf::from("src"),
        &rules,
        &[Layer::Pragmatic],
    );
    if findings.is_empty() {
        return;
    }
    let body = super::review::render_json_text(&rules.review, &findings, &PathBuf::from("src"));
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("是合法 JSON");
    let first = &parsed["findings"][0];
    for key in ["layer", "layer_name", "kind", "path", "line", "message", "principle", "advice"] {
        assert!(!first.get(key).is_none(), "review 的 JSON 少了 {key}");
    }
}
