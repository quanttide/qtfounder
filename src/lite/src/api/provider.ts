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

/** 章节详情（provider `GET /chapters/{id}`） */
export interface ChapterDetail {
  id: string
  title: string
  path: string
  content: string
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
  if (res.statusCode === 404) {
    throw new Error('章节不存在或已改名')
  }
  if (res.statusCode !== 200) {
    throw new Error(`provider 返回 ${res.statusCode}`)
  }
  return res.data as T
}

export function fetchChapters(): Promise<Chapter[]> {
  // 上游偶发空表（历史曾返回 null）统一回退成空数组，不让脏数据击穿渲染
  return request<{ chapters: Chapter[] }>('/chapters').then((data) => data.chapters ?? [])
}

export function fetchMemoryDocs(): Promise<MemoryDoc[]> {
  return request<{ docs: MemoryDoc[] }>('/memory').then((data) => data.docs ?? [])
}

export function fetchChapter(id: string): Promise<ChapterDetail> {
  // Taro 各端路由参数解码行为不一致（h5/weapp 常给未解码原样）：先还原成原文再统一编码，
  // 否则双重编码会让服务端拿 "22_%E4%B9%A6..." 找不到章。
  let raw = id
  if (raw.includes('%')) {
    try {
      raw = decodeURIComponent(raw)
    } catch {
      // 保持原样，交给服务端 404
    }
  }
  return request<ChapterDetail>(`/chapters/${encodeURIComponent(raw)}`)
}
