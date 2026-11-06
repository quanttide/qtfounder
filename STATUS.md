# qtfounder 状态报告

> 更新日期：2026-11-04
> 最新 commit：b197dbf（`cli/v0.1.0`）

## 项目定位

量潮创始人（qtfounder）仓：**创始人第二大脑的规则化**——把协作中形成的规矩落成可执行的规则：CLI 的审计与契约、提交钩子、随发 Skill。

## 迁移状态

整体目标：把本仓内容逐渐提交到各个 `qtcloud-*` 独立仓。

| 内容 | 去处／处置 | 状态 |
|------|-----------|------|
| `meta`（归位/元模型十一条入口与归位契约） | `qtcloud-meta` | ✅ 已迁出（`cli/v0.1.0` 起 `meta` 子命令淘汰） |
| `infra` | `qtcloud-infra` | 🔶 归宿已定，未迁 |
| fiction/memory 架构（`src/provider` 数据源） | 废弃删除 | ⚠️ 待删除 |
| 其余模块与命令组 | 各 `qtcloud-*`（site/studio/lite 去处待拍板） | 🔶 逐个拍板，不预设 |

## 模块状态

| 模块 | 说明 | 版本 | 状态 |
|------|------|------|------|
| `src/site/` | 创始人官网（React + Vite） | v0.1.0-beta.2 | ✅ 可用（beta.2 后 3 笔未发） |
| `src/studio/` | 创始人工作台（Flutter） | v0.0.2 | 🔶 重建中：`lib/asset`、`lib/data`、`lib/meta` 已入，其余按模块重建 |
| `src/provider/` | 创作数据 API（Go） | 0.1.0（仅 CHANGELOG，无 tag） | ⚠️ 待删除：fiction/memory 架构已废弃 |
| `src/lite/` | 微信小程序（Taro + React + TS） | — | 🔶 构建产物经微信开发者工具导入，无 tag |
| `src/cli/` | CLI（Rust） | cli/v0.1.0 | ✅ 14 个命令组、随发 11 条 Skill、356 测试全绿；meta 已迁出 |

## fiction/memory（已废弃，待删除）

fiction（小说仓库）与 memory（创始人记忆仓库）是已废弃的架构：`src/provider` 的数据源与旧数据流（fiction+memory → studio/provider → studio Web）随之废弃，待删除；环境变量 `QTFOUNDER_FICTION_PATH` / `QTFOUNDER_MEMORY_PATH` 只在删除前支撑 provider 运行。

## 验证状态

| 模块 | 验证 |
|------|------|
| src/studio | flutter analyze 零问题 · flutter test 通过 · flutter build web 成功 |
| src/provider | go build / go vet / go test 全通过（真实数据源） |
| src/cli | cargo build / cargo test 通过 |
| src/site | npm 构建正常 |

## 最近进展

- 2026-11-06：src/lite 出 H5 端（`build:h5`/`dev:h5`，产物独立 `dist-h5/`）；src/provider 把 CORS 预检 OPTIONS 短路 204 放行浏览器跨域
- 2026-11-06：src/lite 接入 provider——首页读取 `/chapters` 与 `/memory`（Bearer JWT 从 env.local 构建时注入，仓库不含令牌）
- 2026-11-06：src/provider 鉴权由静态密钥改为 RS256 JWT（FC 触发器与应用层双层校验），`/health` 更名 `/healthz`，新增 `cmd/mint` 签发工具，`QTFOUNDER_SECRET_KEY` 退役
- 2026-11-04：定位更正为「第二大脑的规则化」（原「创作现场」定位作废）；fiction/memory 架构废弃——provider、数据源与数据流标记待删除；立 qtcloud-* 迁移走向
- 2026-11-03：`cli/v0.1.0` 发布——meta 淘汰并入 `qtcloud-meta`，随发 Skill 十二→十一条；此后破坏性变更跳中间位
- 2026-10-28：src/studio 新增 `lib/asset/` 资产抽查模块（参考 recaller 思路，bloc 架构；扫描 quanttide-founder 文档与代码，卡片流保留/删除，删除移入 `.trash/`）
- 2026-10-28：src/studio 清空 review 代码（`packages/review_tool`、`lib/review` 等），只保留入口 `lib/main.dart`，等待重新设计
- 2026-10-17：src/studio 业务代码归档至 data/archive/platform/qtfounder/studio，仅保留入口与骨架
- 2026-09-27：src/cli 精简为脚手架（移除 health 功能，情绪画像归 memory 五轴档案）
- 2026-08-15：src/provider Go 初始化（创作数据 API）+ src/studio 数据源配置
- 2026-08-15：src/studio Flutter 初始化（量潮创始人工作台）
- 2026-08-14：site v0.1.0-alpha.4（CLI health 上线）
