import { View, Text } from '@tarojs/components'
import { useLoad } from '@tarojs/taro'

import { IDEAS } from '../../data/ideas'
import { GOALS } from '../../data/goals'
import './index.scss'

// 纯静态首页：想法、目标各一块，各来自自己的数据文件，不请求任何服务端。
// 改内容＝改对应文件，推 lite/* tag 发布。
export default function Index () {
  useLoad(() => {})

  return (
    <View className='index'>
      <View className='header'>
        <Text className='title'>量潮创始人 lite</Text>
        <Text className='subtitle'>想法与目标</Text>
      </View>

      <View className='section'>
        <Text className='section-title'>想法（{IDEAS.length}）</Text>
        {IDEAS.map((idea) => (
          <View className='row' key={`${idea.group}/${idea.text}`}>
            <View className='row-body'>
              <Text className='row-main'>
                {idea.group !== '' && <Text className='cat'>{idea.group}</Text>}
                <Text className='row-text'>{idea.text}</Text>
              </Text>
              {idea.detail !== '' && <Text className='row-detail'>{idea.detail}</Text>}
            </View>
          </View>
        ))}
        {IDEAS.length === 0 && <Text className='empty'>暂无想法</Text>}
      </View>

      <View className='section'>
        <Text className='section-title'>目标（{GOALS.length}）</Text>
        {GOALS.map((goal) => (
          <View className='row' key={`${goal.group}/${goal.text}`}>
            <View className='row-body'>
              <Text className='row-main'>
                {goal.group !== '' && <Text className='cat'>{goal.group}</Text>}
                <Text className={goal.done ? 'row-text done' : 'row-text'}>{goal.text}</Text>
              </Text>
              {goal.detail !== '' && <Text className='row-detail'>{goal.detail}</Text>}
            </View>
          </View>
        ))}
        {GOALS.length === 0 && <Text className='empty'>暂无目标</Text>}
      </View>
    </View>
  )
}
