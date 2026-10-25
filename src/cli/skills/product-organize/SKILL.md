---
name: product-organize
description: 把 `qtfounder product` 记录库里的需求差按主题整理进 `data/profile/product/requirement/`，用 CLI 读取聚类并在归档成功后清理缓存。当用户要汇总或归档澄清记录库、按主题整理需求差、或清缓存时使用。
---

# 产品需求归档

本地记录库是缓存：逐条记录、随归档清空。`data/profile/product/requirement/` 是持久档案：一主题一档、长期保留。归档就是把这个方向走完——读记录库，分类成主题档，校验落盘，再用 CLI 清缓存。

## 前置

```bash
command -v qtfounder || echo "先装 qtfounder"
```

记录库由 CLI 管，不直接读文件；要临时换数据目录，设 `QTFOUNDER_DATA`。目标仓是 `data/profile`（`quanttide-profile-of-founder`），档在 `product/requirement/`。先把 profile 仓 `git pull`，再看 `product/index.md` 里已有哪些主题。

## 分类验收标准

产品需求与协作差分开放：换个产品就不成立的（做什么、给谁、分几层交付）写成产品档放 `product/` 顶层，一产品一档；`requirement/` 只收与 AI 协作的差。

- 主题来自「实际做了什么」这一列，不是来自任务名、日期或工具。同一件事下的多条差，`actual` 归一化后同键的归一类。
- 主题名是协作维度上的短语（如「输出形态与篇幅」「边界与范围」「验收与收尾」），不是命令名、不是产品模块名。
- 已存在的主题档优先归入；确实新类型才新增档，并在 `product/index.md` 的表格里登记一行。
- 一个主题一档，档名用英文 slug（与 `think/`、`work/` 一致）。
- 每条记录都要进档案：单次出现进「观察」档，不留在缓存里。

## 主题档结构

```markdown
# 主题名

> 来源：澄清记录库（`clarify.json`）中 `actual` 归为同一类的记录，涉及 `clr_xxx`、`clr_yyy`；状态：观察 / 类型 / 规则；已归档。

**状态：观察。**（出现 1 次）/ **类型**（≥2 次）/ **规则**（已落 `rules.md`）

## 差

一句话说清这一类差是什么：原以为 X，实际 Y。

## 记录

- `clr_xxx`（req_xxx，日期）：原以为 …… / 实际 ……

## 该用的提示词

> 可复用的那句，取自记录里的 `prompt` 或由它归并，仍用第一人称。

## 规则

已落库的规则原话（`rules.md` 那一行）；未立就写「未立」。
```

状态三档照抄 `think/` 的分层习惯：出现 1 次只算观察，≥2 次才算类型，落了规则升为规则。

## 流程

1. 取聚类：`qtfounder product organize --json`。CLI 已按 `actual` 归一化键聚好，并给出每条记录的 id、所属需求、任务、时间、原以为、实际与提示词，以及 `rules.md` 里的规则原文。
2. 与 `product/requirement/` 既有档比对：能归入的归入，不能的才新增。
3. 逐档改写，不是追加：更新状态、差的一句话、记录明细、提示词、规则节。同一档里同一记录只出现一次。
4. 同步 `product/index.md` 的表格与 profile 仓 `CHANGELOG.md` 的 `[Unreleased]`。
5. 提交并推送 profile 仓，再更新 `data/profile` 子模块指针。缓存清理只在这一步成功之后做。
6. 校验：对记录库里每条 `clr_` id，在 `product/requirement/*.md` 里搜一次，必须都能搜到。搜不到的那条不删，在汇报里点名。
7. 清缓存：把校验通过的 id 交给 `qtfounder product clear --id <clr_id> --id <clr_id>`。
8. 汇报：归档了哪些主题、写了几条记录、删了几条缓存；有记录没能归类就单独说。

## 清缓存

先校验，再删除，顺序不能反。删除只走 CLI：

```bash
qtfounder product clear --id clr_xxx --id clr_yyy --json
```

`--json` 给 `removed`（删掉的 id）、`missing`（记录库里已不存在的 id）与 `left`（剩余记录数）。`left` 不为 0 说明还有未归档的记录，留在下一次。`rules.md` 一个字都不动——那是用户自己的规则文件，不是缓存。

已归档的记录全部清掉后，对应的空需求也一并移除，记录清空。

**后果要说清**：缓存删掉就没有第二份，档案成为唯一复制件；去重也不再依赖记录库，以后遇到同类差，比对档案里的主题与提示词。

## 边界

- 不改 `rules.md`：规则文本永远是用户原话，这里只读。
- 不生成规则：只呈现「该用的提示词」和已落库的规则。
- 不按日期切档、不按命令切档。
- 没有成功写进档案并推送，就绝不删缓存。

## 排错

记录库读不到或 `organize` 给空聚类，多半是已经清空过：直接看 `product/requirement/` 与 `product/index.md`，不要重建缓存。归类时拿不准的，先归入最接近的既有档并在记录里保留原话，下次归档再拆。
