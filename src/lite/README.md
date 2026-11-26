# lite

量潮创始人的微信小程序 + H5 同构页，基于 Taro 4、React 与 TypeScript。**纯静态：不接服务端**——想法与目标直接写在仓库里，改文件、推 tag 就是发布。

## 改想法与目标（日常就做这一件事）

两份文件各管一块，改哪块动哪份：

- `src/data/ideas.ts` —— 想法：记念头，一句话加一段说明，没有状态
- `src/data/goals.ts` —— 目标：记要达成的状态，`done: true` 划掉

一条长这样：

```ts
{
  group: '2026-11-25',   // 分组标签，一般写日期
  text: '平台：把这些状态管理住',  // 一行标题
  detail: '完整说明，可写多行，页面原样换行显示',
  done: false            // 目标才有：true 表示已达成，划掉
}
```

改完按下面的发布流程推 tag，线上就是新的。没有接口、没有令牌、没有加载失败态。

## 命令

```bash
npm install
npm run dev:weapp    # 小程序监听构建，产物到 dist/
npm run build:weapp  # 构建产物到 dist/，由微信开发者工具导入运行
npm run dev:h5       # H5 开发服务器，默认 http://localhost:10086
npm run build:h5     # 静态产物到 dist-h5/
```

- H5 产物独立在 `dist-h5/`（已 gitignore），与小程序 `dist/` 互不覆盖——`project.config.json` 的 `miniprogramRoot` 只认 `dist/`。

## 发布 H5（lite.founder.quanttide.com）

照 qtclass 的 Deploy Lite 惯例，推 tag 即发：

```bash
git tag lite/vX.Y.Z && git push origin lite/vX.Y.Z
```

链路：`build:h5` → 上传 OSS 桶 `qtfounder-lite`（静态资源长缓存、`index.html` no-cache）→ 刷新 CDN → `https://lite.founder.quanttide.com/`。同一次 push 会顺带跑小程序上传任务，未配 `MINIPROGRAM_*` 密钥时自动跳过、不产生失败。

仓库配置依赖：OSS/CDN 走 `ALIYUN_*` 密钥。CDN 证书是 acme.sh 签的 `lite.founder.quanttide.com`（LE，2027-02 到期）；续期后需用 `SetCdnDomainSSLCertificate` 重推，做法同 qtclass 的 `configure-class-cdn.sh`。

## 上架准备

1. 在微信公众平台注册小程序并取得 AppID，替换 `project.config.json` 中的 `appid`，当前占位值为 `touristappid`，替换后需重新执行构建。
2. 用微信开发者工具导入本目录，工具按 `project.config.json` 的 `miniprogramRoot` 指向 `dist/` 预览与调试。
3. 选定服务类目并填写《用户隐私保护指引》，涉及用户信息的小程序在提审前必须完成，否则无法通过审核。
4. 上传体验版供内部验收，再提交审核；审核通过后发布，后续版本沿用同一 AppID 迭代。

## 发布（CI）

`.github/workflows/deploy-lite.yml` 在推送 `lite/v*` 形式的 tag（如 `lite/v0.1.0-alpha.4`）时自动执行 `npm ci`、`npm run build:weapp`，再由 `scripts/upload.js` 调用 miniprogram-ci 上传体验版；也可在 Actions 页面手动触发并指定版本号，版本号取 tag 最后一段并去掉 `v` 前缀。

需在仓库 Secrets 配置 `MINIPROGRAM_PRIVATE_KEY`（公众平台「开发设置」下载的密钥文件全文）、`MINIPROGRAM_APPID`、`MINIPROGRAM_ROBOT`（可选，1-30，默认 1）。若公众平台开启了 IP 白名单需关闭，因 GitHub Actions 出口 IP 不固定。
