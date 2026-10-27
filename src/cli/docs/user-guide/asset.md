# `qtfounder asset` 用户指南

资产契约、清单与关系，三个都是只读视图。数据源是归位契约：十八格就是十八类资产。这里不另存一份资产表，改资产改 `src/meta/default-contract.toml`。

## 一、contract：每类资产写成什么样

```bash
qtfounder asset contract            # 十八类全看
qtfounder asset contract insight    # 只看一类
qtfounder asset contract --json
```

输出边界、分档与写法。分档里写「默认」的是兜底项。

## 二、catalog：仓库里有哪些

```bash
qtfounder asset catalog                  # 当前目录
qtfounder asset catalog --root ..        # 指定仓根
qtfounder asset catalog --present        # 只列已有的
```

清单看路径在不在，不扫文件内容。`--root` 缺省是当前目录；在主仓根跑才是完整清单。

## 三、relation：和谁一类、和谁同目录

```bash
qtfounder asset relation
qtfounder asset relation report
```

不带参数按家族与目录两条线分组；带资产名看它的家族、目录与同目录兄弟。只读契约，不推断依赖。

## 四、契约在哪

`src/meta/default-contract.toml`，查找顺序与 `meta` 一致；`--rules` 可换一份。
