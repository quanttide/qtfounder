# 数据源与契约

## 数据源

Studio 规划消费两类外部仓库：fiction（小说）和 memory（创始人记忆）。父仓当前把记忆放在 `data/context`；studio 契约仍沿用 `memory` 命名，重建数据层时需要统一。

路径通过环境变量注入，沿用 `QTFOUNDER_FICTION_PATH` 和 `QTFOUNDER_MEMORY_PATH`。当前骨架不读取数据源；环境变量和默认路径在恢复业务层时生效，不把单机绝对路径写进代码。

## 契约

`assets/contracts/` 有两份契约：

- [fiction.yaml](../assets/contracts/fiction.yaml)：一级是小说，二级是阶段，文件编号决定排序，无编号排在最后。
- [memory.yaml](../assets/contracts/memory.yaml)：一级是记忆类型，二级仅 journal 可选，文件自由命名。

契约只描述 studio 如何解读目录，不进入内容仓库。新增资产时优先新增契约，而不是在页面里写死目录语义。

## 运行时状态

契约目前是静态资产，没有解析引擎。重建资产页时再实现目录遍历、命名解析和排序；在实现之前，文档不把 `CatalogTree` 之类的类型当作现有 API。

## 结构对照

| 维度 | fiction | memory |
| --- | --- | --- |
| 一级 | 一本小说 | 一类记忆 |
| 二级 | 阶段 | 仅 journal 的子目录 |
| 文件命名 | 编号决定排序 | 自由命名 |
| 排序 | 编号升序，无编号在后 | 名称自然序，日期日志按日期 |

## 边界

契约是消费方视角，放 studio 端。重建数据层时先定路径配置，再读契约；不要在两个内容仓库里各放一份语义规则。父仓目录改名后，先更新契约的 `root` 和路径映射，再启用资产页。
