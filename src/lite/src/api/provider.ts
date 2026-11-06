import Taro from '@tarojs/taro'

/** 改稿章节（provider `GET /chapters`） */
export interface Chapter {
  id: string
  title: string
  path: string
}

/** memory 文档（provider `GET /memory`） */
export interface MemoryDoc {
  name: string
  category: string
  path: string
}

const BASE_URL = process.env.PROVIDER_BASE_URL || ''
const TOKEN = process.env.PROVIDER_TOKEN || ''

/** 构建时是否注入了 provider 地址与令牌（env.local） */
export function isProviderConfigured(): boolean {
  return Boolean(BASE_URL && TOKEN)
}

async function request<T>(path: string): Promise<T> {
  if (!isProviderConfigured()) {
    throw new Error('未配置 provider：在 env.local 写入 PROVIDER_BASE_URL 与 PROVIDER_TOKEN 后重新构建')
  }
  const res = await Taro.request({
    url: `${BASE_URL}${path}`,
    header: { Authorization: `Bearer ${TOKEN}` },
    timeout: 10000
  })
  // FC 触发器边缘鉴权：无令牌 400（JWTTokenIsMissing），签名不过或过期 401（JWTTokenIsInvalid）
  if (res.statusCode === 401) {
    throw new Error('令牌已过期或校验不过：go run ../provider/cmd/mint -ttl 720h 重铸后更新 env.local 并重新构建')
  }
  if (res.statusCode === 400) {
    throw new Error('令牌没随请求带上：检查 env.local 的 PROVIDER_TOKEN 是否填了值并重新构建')
  }
  if (res.statusCode !== 200) {
    throw new Error(`provider 返回 ${res.statusCode}`)
  }
  return res.data as T
}

export function fetchChapters(): Promise<Chapter[]> {
  return request<{ chapters: Chapter[] }>('/chapters').then((data) => data.chapters)
}

export function fetchMemoryDocs(): Promise<MemoryDoc[]> {
  return request<{ docs: MemoryDoc[] }>('/memory').then((data) => data.docs)
}
