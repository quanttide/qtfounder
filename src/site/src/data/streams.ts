export interface Stream {
  id: string
  name: string
  status: string
  latest: string
  next: string
}

export const streams: Stream[] = [
  {
    id: 'fiction',
    name: '小说',
    status: '职场言情 14 篇改稿定稿',
    latest: '咖啡厅重逢等场景完成语言润色（2026-05）',
    next: '开启校园言情改稿，统一视角与对白节奏',
  },
  {
    id: 'game',
    name: '游戏',
    status: '围棋 v0.1.2 终局可用',
    latest: '连 Pass 终局与胜负判定完成（2026-06）',
    next: '战争游戏"意图到行动"实验定型',
  },
  {
    id: 'tool',
    name: '工具',
    status: '语音输入工作流与 qtfounder-cli 并行迭代',
    latest: 'qtfounder-cli v0.1.0 发布（2026-08）',
    next: '把提交与发布流程完整固化进 CLI',
  },
  {
    id: 'think',
    name: '思考',
    status: '认知工程文章已发布',
    latest: '影视规则显性化讨论（2026-08）',
    next: '把创作规则沉淀为可复用方法论',
  },
]
