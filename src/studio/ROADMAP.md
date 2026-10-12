# ROADMAP

## config

- [ ] 移除默认环境变量，减少泄漏环境信息，使用.env管理
- [ ] 移除硬编码数据路径：桌面端默认数据路径 `$HOME/repos/quanttide-founder/assets/{fiction,memory}` 写死在 `lib/config.dart`、`lib/repositories/asset_catalog_engine.dart`、`lib/screens/create_screen.dart`、`lib/bloc/app_bloc_provider.dart` 与 6 处文档里，改为按配置项解析，不写死单机路径

## repo

- [ ] data文件夹整体改为仓储模式，文件夹也改名。
