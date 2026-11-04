# 量潮创始人(`qtfounder`)

创始人第二大脑的规则化：把协作中形成的规矩落成可执行的规则——CLI 的审计与契约、提交钩子、随发 Skill。

> **走向**：本仓内容逐渐提交到各个 `qtcloud-*` 独立仓（先例：`meta` → `qtcloud-meta`；`infra` → `qtcloud-infra` 已定未迁）。

## 结构

```
├── src/
│   ├── site/      → 创始人官网（React + Vite）
│   ├── studio/    → 创始人工作台（Flutter）
│   ├── provider/  → 服务端（待删除：fiction/memory 架构已废弃）
│   ├── lite/      → 微信小程序（Taro）
│   └── cli/       → CLI（Rust）
├── STATUS.md      → 项目状态
└── README.md
```

## 许可

Apache 2.0
