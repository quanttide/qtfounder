# `qtfounder language` 接口

## 一、命令面

```
qtfounder language [ROOT] [--scope SCOPE] [--kind KIND] [--rules PATH]
                   [--pool DIR] [--top N] [--json] [--sense] [--limit N]
```

**ROOT**：目录或单文件，缺省当前目录。递归找 `.md`。

**`--scope`**：`default` 审 root 下全部 `.md`，只跳过契约 `[scope] exclude` 里的流水、报告、归档、示例与第三方依赖；`all` 连 exclude 也不放过。缺省 `default`。审哪些文件由 ROOT 决定，工具不再内嵌目录白名单。

**`--kind`**：`placement`／`banned`／`required`／`numbers`／`syntax`／`voice`，只报一类。表格层已迁至 `docs`。

**`--rules PATH`**：指定契约。缺省按 `QTFOUNDER_LANGUAGE_CONTRACT`、再往上找 `src/language/default-contract.toml`。

**`--pool DIR`**：你自己的文风基准目录，缺省 `data/profile`。词表自查拿它比对：禁用词若在你自己的文字里也出现，末尾提示可出库。

**`--top N`**：最多列几个文件，缺省 15。`--top 0` 只留汇总行。

**`--sense`**：顺带跑语义层。**`--limit N`**：最多送判几段，缺省 10。

## 二、退出码

- **0**：无命中。
- **1**：有命中，或语义层判出「在记操作」的段。
- **2**：用不了——契约读不到、`--scope`／`--kind` 取值不认识、有文件读不了。

## 三、JSON 结构

```json
{
  "scanned_files": 121,
  "findings": [
    {
      "kind": "voice",
      "severity": "suspect",
      "verdict": "real",
      "path": "docs/gallery/x.md",
      "line": 42,
      "excerpt": "这一段有六个内部标识符却没有一处对读者说话",
      "advice": "要么逐个说明它是什么，要么这段该进日志"
    }
  ],
  "banned_in_own_prose": ["验收标准"]
}
```

字段含义：`scanned_files` 是这次真审了的文件数——它是 0 就说明范围没设对，别看 `findings` 空着就当通过。

`verdict` 是复核层给每条命中的结论。四个取值是 `real`（真问题）、`mention`（误报·提及，词被引号包着讨论，不是拿来用）、`lead-in`（误报·标题并入，加粗小标签被当成句子成分）、`quoted-output`（误报·引用输出，抄的是工具自己的「合计 N 处」）。只判这三种机判得了的形态。体裁错配和词的多义仍归人判。

- `kind` 是六层之一，稳定；`severity` 只有 `error` 与 `suspect` 两值，后者是嫌疑不是判决。
- `excerpt` 是命中处的截断原文，最多九十汉字，用来定位，不要拿来当验收标准。
- `banned_in_own_prose` 列出「你的文风里也在用的禁用词」，是自查提示，不参与退出码。

**`--json` 与 `--sense` 不同时输出**：语义层的结果只在人读模式下打印。理由是全量扫描的 JSON 要可复跑，混入模型结论就不再可复跑。

## 四、语义层的接口

直连 OpenAI 兼容的 `/chat/completions`，`temperature=0`，`max_tokens=300`，只接受 JSON 回复。

环境变量：`LLM_API_KEY`、`LLM_MODEL`（缺省 `deepseek-chat`）、`LLM_BASE_URL`（缺省 `https://api.deepseek.com`）。密钥也可放在 `~/.pi/agent/auth.json` 的 `deepseek.key`。**密钥不落日志、不打印。**

模型回复解析不出 JSON 时降级为 `unknown`，理由取原文前一百二十字。降级不是失败——判不了就归人。

## 五、稳定性承诺

`kind` 的六个取值与 `severity` 的两个取值不变；`excerpt`／`advice` 的措辞会变，别做断言。语义层的判断类只有 `reader`／`process`／`unknown`／`error` 四个，加类算大版本。
