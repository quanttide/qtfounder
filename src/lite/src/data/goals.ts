// 首页「目标」：**直接改这份文件**，推 `lite/*` tag 即发布。
// 记要达成的状态——写「要成什么样」，不写「下一步做什么」（那是任务）。
// 达成后把 done 改成 true，页面划掉留痕。
//
// 写法：group 是分组标签（一般写日期），text 是一行标题，
// detail 是这条的完整说明（可写多行，页面原样换行显示）。

export interface Goal {
  group: string
  text: string
  detail: string
  done: boolean
}

export const GOALS: Goal[] = [
  {
    group: '2026-11-25',
    text: '平台：把这些状态管理住',
    detail: '就很希望自己的平台可以帮着自己把这些状态都管理住。',
    done: false
  }
]
