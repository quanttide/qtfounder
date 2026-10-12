/**
 * 上传微信小程序体验版，供 CI 调用。
 *
 * 环境变量：
 *   MINIPROGRAM_PRIVATE_KEY  上传密钥文件内容（必填）
 *   MINIPROGRAM_APPID        小程序 AppID（必填）
 *   MINIPROGRAM_ROBOT        机器人编号 1-30（可选，默认 1）
 *   RELEASE_VERSION          版本号，如 v1.2.3 或 1.2.3（可选，缺省取 package.json）
 */
const ci = require('miniprogram-ci')
const fs = require('fs')
const os = require('os')
const path = require('path')

const DIST = path.resolve(__dirname, '../dist')
const CONFIG_PATH = path.join(DIST, 'project.config.json')
const KEY_PATH = path.join(os.tmpdir(), 'miniprogram-upload-' + process.pid + '.key')

// miniprogram-ci 的子进程会把同一错误重复抛一次，忽略以免掩盖脚本自身的退出码
process.on('unhandledRejection', (e) => {
  console.error('忽略重复错误：' + (e && e.message ? e.message : e))
})

function resolveVersion () {
  // 兼容 v1.2.3、lite/v1.2.3 两种 tag 写法
  const raw = (process.env.RELEASE_VERSION || '').split('/').pop().replace(/^v/, '')
  if (/^\d+\.\d+\.\d+$/.test(raw)) return raw
  // 手动触发且未指定版本时回退到 package.json
  return require('../package.json').version
}

;(async () => {
  const appid = process.env.MINIPROGRAM_APPID
  const privateKey = process.env.MINIPROGRAM_PRIVATE_KEY
  if (!appid || !privateKey) {
    throw new Error('缺少 MINIPROGRAM_APPID 或 MINIPROGRAM_PRIVATE_KEY，请在仓库 Secrets 中配置')
  }
  if (!fs.existsSync(CONFIG_PATH)) {
    throw new Error('产物目录缺少 project.config.json，请先执行 npm run build:weapp：' + DIST)
  }

  fs.writeFileSync(KEY_PATH, privateKey)
  try {
    // 产物里的 appid 是占位值，上传前与真实 AppID 对齐
    const config = JSON.parse(fs.readFileSync(CONFIG_PATH, 'utf8'))
    config.appid = appid
    fs.writeFileSync(CONFIG_PATH, JSON.stringify(config, null, 2) + '\n')

    const project = new ci.Project({
      appid,
      type: 'miniProgram',
      projectPath: DIST,
      privateKeyPath: KEY_PATH,
      ignores: ['node_modules/**/*']
    })

    const version = resolveVersion()
    const sha = process.env.GITHUB_SHA ? process.env.GITHUB_SHA.slice(0, 7) : ''
    const result = await ci.upload({
      project,
      version,
      desc: ('CI upload ' + sha).trim(),
      robot: Number(process.env.MINIPROGRAM_ROBOT || 1),
      setting: {
        es6: true,
        es7: true,
        minifyJS: true,
        minifyWXML: true,
        minifyWXSS: true,
        autoPrefixWXSS: true
      },
      onProgressUpdate: console.log
    })
    console.log('上传成功，版本 ' + version + '：' + JSON.stringify(result))
  } catch (e) {
    console.error('上传失败：' + (e && e.message ? e.message : e))
    process.exitCode = 1
  } finally {
    fs.rmSync(KEY_PATH, { force: true })
  }
})()
