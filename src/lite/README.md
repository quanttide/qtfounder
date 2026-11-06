# lite

量潮创始人的微信小程序，基于 Taro 4、React 与 TypeScript，构建产物输出到 `dist/`，由微信开发者工具导入运行。

## 命令

```bash
npm run dev:weapp    # 监听构建
npm run build:weapp  # 构建产物到 dist/
```

## 接入 provider

首页两份列表（改稿章节、memory 文档）来自 provider（`src/provider`，FC 部署，双层 JWT）：

```bash
cp env.local.example env.local                     # env.local 被 .gitignore 的 *.local 覆盖，不进 git
cd ../provider && go run ./cmd/mint -ttl 720h      # 签发 30 天令牌（私钥只在本机）
# 把令牌填进 src/lite/env.local 的 PROVIDER_TOKEN，BASE_URL 填触发器地址
cd .. && npm run build:weapp                       # 构建时注入为 process.env.PROVIDER_*
```

- 令牌过期或校验不过时首页会给出可读错误与重试，按提示重铸即可。
- **令牌会进入 `dist/` 构建产物**（客户端鉴权的固有代价，小程序 JS 可被反编译）：仓库与源码始终不含令牌，但上架分发前必须重新评估——届时改为短期令牌＋服务端中转，或按 README 下面的域名/鉴权方案升级。
- 真机调试需在微信后台把触发器域名加入 request 合法域名，或开发期在开发者工具勾选「不校验合法域名」。

## H5 端（浏览器）

```bash
npm run dev:h5     # 开发服务器，默认 http://localhost:10086
npm run build:h5   # 静态产物到 dist-h5/
```

- H5 产物独立在 `dist-h5/`（已 gitignore），与小程序 `dist/` 互不覆盖——`project.config.json` 的 `miniprogramRoot` 只认 `dist/`。
- 浏览器读线上数据靠跨域：provider 侧把 CORS 预检 OPTIONS 短路 204、FC 网关补 `Access-Control-*` 头，本地或任意静态站托管都能直连，无需代理。
- 上线＝把 `dist-h5/` 挂到任意 HTTPS 静态站；令牌同样进产物，分发评估与上节相同。

## 上架准备

1. 在微信公众平台注册小程序并取得 AppID，替换 `project.config.json` 中的 `appid`，当前占位值为 `touristappid`，替换后需重新执行构建。
2. 用微信开发者工具导入本目录，工具按 `project.config.json` 的 `miniprogramRoot` 指向 `dist/` 预览与调试。
3. 在小程序后台配置服务器域名，把 `dist/` 内请求所用域名加入 request 合法域名，否则真机调试与线上请求会被拦截。
4. 选定服务类目并填写《用户隐私保护指引》，涉及用户信息的小程序在提审前必须完成，否则无法通过审核。
5. 上传体验版供内部验收，再提交审核；审核通过后发布，后续版本沿用同一 AppID 迭代。

## 发布（CI）

`.github/workflows/deploy-lite.yml` 在推送 `lite/v1.0.0` 或 `v1.0.0` 形式的 tag 时自动执行 `npm ci`、`npm run build:weapp`，再由 `scripts/upload.js` 调用 miniprogram-ci 上传体验版；也可在 Actions 页面手动触发并指定版本号，版本号取 tag 最后一段并去掉 `v` 前缀。

需在仓库 Secrets 配置 `MINIPROGRAM_PRIVATE_KEY`（公众平台「开发设置」下载的密钥文件全文）、`MINIPROGRAM_APPID`、`MINIPROGRAM_ROBOT`（可选，1-30，默认 1）。若公众平台开启了 IP 白名单需关闭，因 GitHub Actions 出口 IP 不固定。
