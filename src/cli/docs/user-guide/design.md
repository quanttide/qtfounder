# design：交互设计走查

按十二条交互设计评判原则审界面交付物。原则全文随 CLI 发布在 `design-review` Skill 里，`design review` 只做其中机器判得了的两条的数字面。

## 为什么只有两条

十二条原则里，能从 HTML 里读出数字的只有两条：

- **第九条 中文微排版**：正文 ≥ 14px，辅助文字 ≥ 12px。更小的中文文字一律视为缺陷；
- **第五条 主行动唯一**：全屏只有一个主按钮层级。

其余十条——工作物可见、状态即邀请、动作有回应、不演戏、验收有对象、脑内渲染走查、自评分——机器判不了，归 `design-review` Skill 里的走查与自评分。数字先看，结论再读。

## 怎么数

字号只认 CSS 里的 `font-size` 声明：`px` 直接用，`rem` 按 16px 根字号换算（没解析 `<html>` 的自定义根字号），`em` 相对父级判不了，只计数不换算。

低于 12px 报缺陷；12–14px 报嫌疑——分不出这段是正文还是辅助文字，是正文就得抬到 14px。

主按钮数 class 含 `primary` 的 `<button>`。一个文件多于一个报一次嫌疑：多屏合订本一屏一个主按钮是合规的，逐屏自查归人，每个按钮都报一遍只是噪音。

## 报告长什么样

对 qtcloud 仓 `examples/studio/index.html` 实测（2026-10-28）的输出长这样：

```text
设计走查：examples/studio/index.html
  font-size 声明 27 处（px 0，rem 27，em 0），最小 10.9px（rem 按 16px 根换算）
  class 含 primary 的按钮 5 个
命中 18 处：
  [缺陷] font L74  font-size: 0.72rem
          → 第九条：更小的中文文字一律视为缺陷；辅助文字 ≥ 12px，正文 ≥ 14px
  [嫌疑] primary L482  <button class="primary
          → 第五条：主行动唯一；本文件 class 含 primary 的按钮共 5 个——多屏合订本可豁免，逐屏自查
报的是嫌疑不是判决：逐处读上下文再改。
```

目录模式一行一个文件，按命中数从多到少排。

## 看原则全文

`qtfounder design principles` 摊开十二条全文与评分标准。正文与 `design-review` Skill 同一份——装在系统位置的那份也来自这里。

## 与其他命令的关系

`design` 不接文档审计的契约：字号与主按钮的线写在原则第九条与第五条里，改线改原则。文档格式归 `docs`，文字归 `language`。
