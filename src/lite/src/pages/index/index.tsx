import { View, Text } from '@tarojs/components'
import { useLoad } from '@tarojs/taro'

import { TODOS } from '../../data/todos'
import './index.scss'

// 纯静态首页：待办直接来自 data/todos.ts，不请求任何服务端。
// 改内容＝改那份文件，推 lite/* tag 发布。
export default function Index () {
  useLoad(() => {})

  return (
    <View className='index'>
      <View className='header'>
        <Text className='title'>量潮创始人 lite</Text>
        <Text className='subtitle'>待办清单</Text>
      </View>

      <View className='section'>
        <Text className='section-title'>待办（{TODOS.length}）</Text>
        {TODOS.map((todo) => (
          <View className='row' key={`${todo.group}/${todo.text}`}>
            <View className='todo-body'>
              <Text className='row-main'>
                {todo.group !== '' && <Text className='cat'>{todo.group}</Text>}
                <Text className={todo.done ? 'todo-done' : 'todo-item'}>{todo.text}</Text>
              </Text>
              {todo.detail !== '' && <Text className='todo-detail'>{todo.detail}</Text>}
            </View>
          </View>
        ))}
        {TODOS.length === 0 && <Text className='empty'>暂无待办</Text>}
      </View>
    </View>
  )
}
