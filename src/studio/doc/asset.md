# 资产抽查

## 目的

从 quanttide-founder 仓库的文档与代码里随机抽一批，人工决定保留还是删除。删除的文件移入仓库根的 `.trash/`，保留目录结构，不直接删除。

## 流程

1. `AssetRepository.resolve()` 解析仓库根：优先 `--dart-define=QTFOUNDER_ROOT`，否则从当前目录向上找同时有 `.gitmodules`、`apps/`、`data/` 的目录，最多八层。
2. `scan()` 递归扫描，只收文档（`.md`）与代码（dart / js / ts / go / rs / py 等），跳过 `.git`、`build`、`node_modules`、`target` 等目录。
3. `AssetCubit.load()` 打乱后取前 20 个，读取每个文件开头作为卡片预览。
4. 卡片流逐个决策：左滑删除、右滑保留，或点按钮。决策写入 `decisions`，栈空进确认页。
5. 确认页列出标记删除的清单，逐个移入 `.trash/`，进度实时显示。
6. 结果页列出成功与失败，失败的逐条给出原因。

## 数据模型

```dart
enum AssetKind { document, code }
enum Decision { keep, delete }

class AssetFile {
  final String id;       // 相对仓库根的路径，批次内唯一
  final String path;     // 绝对路径
  final AssetKind kind;  // 文档还是代码
  final int size;        // 字节数
  final DateTime modified;
}
```

批次状态用 sealed class 收敛为七种：`AssetLoading`、`AssetReady`、`AssetConfirm`、`AssetSubmitting`、`AssetDone`、`AssetEmpty`、`AssetError`。

## 文件操作

`moveToTrash()` 保证四件事：

- 回收目录不存在时先创建，并按相对路径建子目录；
- 目标重名时加 `yyyyMMdd-HHmmss-SSS-` 前缀，不覆盖已有文件；
- 优先 `File.rename`，跨设备失败时回退到复制、校验字节数后再删源；
- 不调用 `File.delete()`，删除路径只有「移到 `.trash/`」一条。

`.trash/` 已加入根 `.gitignore`，回收目录不进版本控制。

## 边界

- 只扫文档与代码后缀，配置文件（yaml / json / toml / lock）不在范围内。
- 不进 `.git`、`build`、`node_modules`、`target`、`.trash` 等目录。
- 预览只读文件开头（最多 40 行 / 4000 字节），不加载全文。
- 单个文件移动失败不影响同批其他文件，结果页逐条列出。
- 不接入 Provider，所有操作在本地文件系统完成。

## 接入

```dart
Navigator.of(context).push(
  MaterialPageRoute<void>(builder: (_) => const AssetAuditPage()),
);
```

`AssetAuditPage` 内部用 `AssetRepository.resolve()`；测试可注入自定义仓库。
