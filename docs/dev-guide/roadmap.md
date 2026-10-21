# ROADMAP

量潮创始人平台（qtfounder）路线图。状态报告见 [STATUS.md](../../STATUS.md)，模块级路线图见 `src/*/ROADMAP.md`。

## 迁移

把当前的 studio 实现移动到 qtcloud 并且清空现有实现，以方便重新定义一个更自由的版本。

### 资产

- [ ] 先封装到 toolkit 仓库，然后迁移到 qtcloud-asset
- [ ] 评估是否以及如何基于 OSS 的资产管理功能合并

### 写作

- [ ] 封装到 toolkit 仓库，然后对比写作云

## 新增

使用 quanttide-founder 的结构迁移到私有第二大脑 iGuo，统一管理公私两个结构。探索未来如何管理组织层面的公开和私有结构。

### 资产结构

- [ ] 公开和私有第二大脑都有 fiction 和 memory，且前者是成品，后者是素材；搭建一个稳定且显式的结构有利于同构维护

### 创意写作

- [ ] fiction 都来源于 memory，且私有的结构还没有完全建立起来；需要一个精细的结构来加工它
