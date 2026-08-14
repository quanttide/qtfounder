export type ActivityType = '日记' | '小说' | '游戏' | '文章' | '工具'

export interface Activity {
  id: string
  type: ActivityType
  date: string
  title: string
  /** 一句话摘录或说明 */
  note: string
  link?: string
}

export const activities: Activity[] = [
  {
    id: 'journal-20260814',
    type: '日记',
    date: '2026-08-14',
    title: '影视规则显性化',
    note: '把创作规则显性化是个人平台的基础环节，这一步做好再做其他的。',
  },
  {
    id: 'qtfounder-cli-v010',
    type: '工具',
    date: '2026-08-14',
    title: 'qtfounder-cli v0.1.0',
    note: '健康检查子命令发布，基于 git 历史与 LLM 抽取情绪维度。',
    link: 'https://github.com/quanttide/qtfounder',
  },
  {
    id: 'war-intent-action',
    type: '游戏',
    date: '2026-08',
    title: '战争游戏 · 意图到行动',
    note: '新增 intent-to-action 示例，推进玩法机制实验。',
    link: 'https://github.com/quanttide/qtgame-war',
  },
  {
    id: 'weiqi-v012',
    type: '游戏',
    date: '2026-06-27',
    title: '围棋 · v0.1.2',
    note: '连 Pass 终局与胜负判定完成，AI 能完整下完一盘棋。',
    link: 'https://github.com/quanttide/qtgame-weiqi',
  },
  {
    id: 'second-brain',
    type: '工具',
    date: '2026-06',
    title: '第二大脑成型',
    note: '记忆仓库承载日志、文库、路线图，认知资产可版本化。',
  },
  {
    id: 'fiction-drafts-202605',
    type: '小说',
    date: '2026-05',
    title: '职场言情改稿 14 篇',
    note: '咖啡厅重逢等场景完成叙事调整与语言润色。',
  },
  {
    id: 'tycoon-v001',
    type: '游戏',
    date: '2026-04-30',
    title: '经营游戏 · v0.0.1',
    note: '初始项目结构与游戏入口，迭代闭环建立。',
    link: 'https://github.com/quanttide/qtgame-tycoon',
  },
  {
    id: 'essay-cognitive-engineering',
    type: '文章',
    date: '2026-04',
    title: '《认知工程》',
    note: '认知外化、重构与减负——AI 原生时代的认知负担如何系统性降低。',
  },
]
