// 首页「想法」：**直接改这份文件**，推 `lite/*` tag 即发布。
// 记念头——想到什么写什么，一句话加一段说明，没有状态、不划掉。
// 任务、心情、进度快照不进这里（「不想动」「已安排下去」看过就没了，留不住）。
//
// 写法：group 是分组标签（一般写日期），text 是一行标题，
// detail 是这条的完整说明（可写多行，页面原样换行显示）。

export interface Idea {
  group: string
  text: string
  detail: string
}

export const IDEAS: Idea[] = []
