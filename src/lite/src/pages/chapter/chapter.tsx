import { View, Text } from '@tarojs/components'
import { useLoad, useRouter } from '@tarojs/taro'
import { useCallback, useState } from 'react'

import { ChapterDetail, fetchChapter } from '../../api/provider'
import './chapter.scss'

interface PageState {
  loading: boolean
  error: string
  detail: ChapterDetail | null
}

const INITIAL: PageState = { loading: true, error: '', detail: null }

export default function Chapter () {
  const router = useRouter()
  const id = router.params.id ?? ''
  const [state, setState] = useState<PageState>(INITIAL)

  const load = useCallback(async () => {
    if (!id) {
      setState({ loading: false, error: '缺少章节 id', detail: null })
      return
    }
    setState({ ...INITIAL })
    try {
      const detail = await fetchChapter(id)
      setState({ loading: false, error: '', detail })
    } catch (e) {
      setState({ loading: false, error: e instanceof Error ? e.message : String(e), detail: null })
    }
  }, [id])

  useLoad(() => {
    load()
  })

  const { loading, error, detail } = state

  return (
    <View className='chapter'>
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

      {!loading && detail && (
        <View>
          <Text className='title'>{detail.title}</Text>
          <Text className='body'>{detail.content}</Text>
        </View>
      )}
    </View>
  )
}
