# lite

量潮创始人的微信小程序，基于 Taro 4、React 与 TypeScript，构建产物输出到 `dist/`，由微信开发者工具导入运行。

## 命令

```bash
npm run dev:weapp    # 监听构建
npm run build:weapp  # 构建产物到 dist/
```

## 上架准备

1. 在微信公众平台注册小程序并取得 AppID，替换 `project.config.json` 中的 `appid`，当前占位值为 `touristappid`，替换后需重新执行构建。
2. 用微信开发者工具导入本目录，工具按 `project.config.json` 的 `miniprogramRoot` 指向 `dist/` 预览与调试。
3. 在小程序后台配置服务器域名，把 `dist/` 内请求所用域名加入 request 合法域名，否则真机调试与线上请求会被拦截。
4. 选定服务类目并填写《用户隐私保护指引》，涉及用户信息的小程序在提审前必须完成，否则无法通过审核。
5. 上传体验版供内部验收，再提交审核；审核通过后发布，后续版本沿用同一 AppID 迭代。

## 发布（CI）

`.github/workflows/deploy-lite.yml` 在推送 `lite/v1.0.0` 或 `v1.0.0` 形式的 tag 时自动执行 `npm ci`、`npm run build:weapp`，再由 `scripts/upload.js` 调用 miniprogram-ci 上传体验版；也可在 Actions 页面手动触发并指定版本号，版本号取 tag 最后一段并去掉 `v` 前缀。

需在仓库 Secrets 配置 `MINIPROGRAM_PRIVATE_KEY`（公众平台「开发设置」下载的密钥文件全文）、`MINIPROGRAM_APPID`、`MINIPROGRAM_ROBOT`（可选，1-30，默认 1）。若公众平台开启了 IP 白名单需关闭，因 GitHub Actions 出口 IP 不固定。
