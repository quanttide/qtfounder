import { defineConfig, type UserConfigExport } from '@tarojs/cli'
import fs from 'fs'
import path from 'path'
import TsconfigPathsPlugin from 'tsconfig-paths-webpack-plugin'
import devConfig from './dev'
import prodConfig from './prod'

// env.local（gitignored，模板见 env.local.example）持有 provider 地址与令牌，
// 构建时注入为 process.env.PROVIDER_*：仓库与源码不含令牌，产物里会有（客户端鉴权的固有代价，见 README）。
const loadEnvLocal = (): Record<string, string> => {
  const file = path.join(__dirname, '..', 'env.local')
  if (!fs.existsSync(file)) return {}
  const values: Record<string, string> = {}
  for (const line of fs.readFileSync(file, 'utf8').split(/\r?\n/)) {
    const matched = /^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$/.exec(line)
    if (!matched) continue
    values[matched[1]] = matched[2].trim().replace(/^(['"])([\s\S]*)\1$/, '$2')
  }
  return values
}

const envLocal = loadEnvLocal()

// https://taro-docs.jd.com/docs/next/config#defineconfig-辅助函数
export default defineConfig<'webpack5'>(async (merge, { command, mode }) => {
  const baseConfig: UserConfigExport<'webpack5'> = {
    projectName: 'lite',
    date: '2026-10-12',
    designWidth: 750,
    deviceRatio: {
      640: 2.34 / 2,
      750: 1,
      375: 2,
      828: 1.81 / 2
    },
    sourceRoot: 'src',
    // h5 产物独立到 dist-h5，不覆盖小程序的 dist/（project.config.json 的 miniprogramRoot 指向 dist）
    outputRoot: process.env.TARO_ENV === 'h5' ? 'dist-h5' : 'dist',
    plugins: [
      "@tarojs/plugin-generator"
    ],
    defineConstants: {
      'process.env.PROVIDER_BASE_URL': JSON.stringify(envLocal.PROVIDER_BASE_URL ?? ''),
      'process.env.PROVIDER_TOKEN': JSON.stringify(envLocal.PROVIDER_TOKEN ?? '')
    },
    copy: {
      patterns: [
      ],
      options: {
      }
    },
    framework: 'react',
    compiler: 'webpack5',
    cache: {
      enable: false // Webpack 持久化缓存配置，建议开启。默认配置请参考：https://docs.taro.zone/docs/config-detail#cache
    },
    mini: {
      postcss: {
        pxtransform: {
          enable: true,
          config: {

          }
        },
        cssModules: {
          enable: false, // 默认为 false，如需使用 css modules 功能，则设为 true
          config: {
            namingPattern: 'module', // 转换模式，取值为 global/module
            generateScopedName: '[name]__[local]___[hash:base64:5]'
          }
        }
      },
      webpackChain(chain) {
        chain.resolve.plugin('tsconfig-paths').use(TsconfigPathsPlugin)
      }
    },
  }


  if (process.env.NODE_ENV === 'development') {
    // 本地开发构建配置（不混淆压缩）
    return merge({}, baseConfig, devConfig)
  }
  // 生产构建配置（默认开启压缩混淆等）
  return merge({}, baseConfig, prodConfig)
})
