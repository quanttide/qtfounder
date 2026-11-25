// 首页待办：**直接改这份文件**，推 `lite/*` tag 即发布。
// 纯静态，不接服务端——没有加载、没有令牌、没有 500。
//
// 写法：group 是分组标签（一般写日期），text 是一行标题，
// detail 是这条的完整说明（可写多行，页面原样换行显示），done 用 true 划掉。

export interface Todo {
  group: string
  text: string
  detail: string
  done: boolean
}

export const TODOS: Todo[] = [
  {
    group: '2026-11-25',
    text: '先做漫画',
    detail: '小说大概有个思路改第三四章，先做漫画，但是不想动手。',
    done: false
  },
  {
    group: '2026-11-25',
    text: '小说：改第三四章',
    detail: '小说大概有个思路改第三四章；今天先做漫画，这条排在它之后。',
    done: false
  },
  {
    group: '2026-11-25',
    text: '课程：已安排下去',
    detail: '课程的事情已经安排下去了。',
    done: true
  },
  {
    group: '2026-11-25',
    text: '课程：联系更多人看',
    detail: '其实可以联系更多人看，但是也不太想做，觉得没意义，想等方向清晰些。',
    done: false
  },
  {
    group: '2026-11-25',
    text: '平台：把这些状态管理住',
    detail: '就很希望自己的平台可以帮着自己把这些状态都管理住。',
    done: false
  }
]
