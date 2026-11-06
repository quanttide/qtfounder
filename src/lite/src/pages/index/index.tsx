import { View, Text } from '@tarojs/components'
import { useLoad } from '@tarojs/taro'
import { useCallback, useState } from 'react'

import { Chapter, MemoryDoc, fetchChapters, fetchMemoryDocs, isProviderConfigured } from '../../api/provider'
import './index.scss'

interface PageState {
  loading: boolean
  error: string
  chapters: Chapter[]
  docs: MemoryDoc[]
}

const INITIAL: PageState = { loading: true, error: '', chapters: [], docs: [] }

export default function Index () {
  const [state, setState] = useState<PageState>(INITIAL)

  const load = useCallback(async () => {
    if (!isProviderConfigured()) {
      setState({
        ...INITIAL,
        loading: false,
        error: '未配置 provider：在 src/lite/env.local 写入 PROVIDER_BASE_URL 与 PROVIDER_TOKEN 后重新构建'
      })
      return
    }
    setState({ ...INITIAL })
    try {
      const [chapters, docs] = await Promise.all([fetchChapters(), fetchMemoryDocs()])
      setState({ loading: false, error: '', chapters, docs })
    } catch (e) {
      setState({ ...INITIAL, loading: false, error: e instanceof Error ? e.message : String(e) })
    }
  }, [])

  useLoad(() => {
    load()
  })

  const { loading, error, chapters, docs } = state

  return (
    <View className='index'>
      <View className='header'>
        <Text className='title'>量潮创始人 lite</Text>
        <Text className='subtitle'>创作数据来自 provider</Text>
      </View>

      {loading && (
        <View className='state'>
          <Text>加载中…</Text>
        </View>
      )}

      {!loading && error !== '' && (
        <View className='state' onClick={load}>
          <Text className='error'>{error}</Text>
          <Text className='retry'>点此重试</Text>
        </View>
      )}

      {!loading && error === '' && (
        <View>
          <View className='section'>
            <Text className='section-title'>改稿章节（{chapters.length}）</Text>
            {chapters.map((chapter) => (
              <View className='row' key={chapter.id}>
                <Text className='row-main'>{chapter.title}</Text>
                <Text className='row-sub'>{chapter.id}</Text>
              </View>
            ))}
            {chapters.length === 0 && <Text className='empty'>暂无章节</Text>}
          </View>

          <View className='section'>
            <Text className='section-title'>memory 文档（{docs.length}）</Text>
            {docs.map((doc) => (
              <View className='row' key={`${doc.category}/${doc.name}`}>
                <Text className='row-main'>
                  <Text className='cat'>{doc.category}</Text>
                  {doc.name}
                </Text>
              </View>
            ))}
            {docs.length === 0 && <Text className='empty'>暂无文档</Text>}
          </View>
        </View>
      )}
    </View>
  )
}
