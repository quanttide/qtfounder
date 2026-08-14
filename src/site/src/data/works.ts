export type WorkType = '改稿' | '文章' | '游戏' | '工具'

export interface Revision {
  version: string // 'v0' | 'v1' | 'v2'
  label: string // 初稿 / 叙事调整 / 语言润色
  date: string
  note: string
}

export interface Work {
  slug: string
  title: string
  type: WorkType
  date: string
  description: string
  /** 文章/游戏/工具：外部原文或仓库链接 */
  link?: string
  /** 仅改稿：初稿 → 改稿轨迹 */
  revisions?: Revision[]
}

export const fictionWorks: Work[] = [
  {
    slug: 'coffee-reunion', title: '咖啡厅重逢', type: '改稿', date: '2026-05',
    description: '十年后偶然在一家咖啡厅重遇，两个人都没想到会以这样的方式再见。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿，沙发咖啡场景。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '调整时间线，补全重逢动机。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简对白，统一视角。' },
    ],
  },
  {
    slug: 'sleepless-night', title: '深夜失眠', type: '改稿', date: '2026-05',
    description: '深夜翻来覆去，各自想着同一个人。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '补全两人各自的心理线。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '压缩内心独白，收紧节奏。' },
    ],
  },
  {
    slug: 'exhibition-meeting', title: '展会再遇', type: '改稿', date: '2026-05',
    description: '工作场合的第二次碰面，比第一次多了点刻意。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '增加第二次碰面的刻意感。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简场景描写，突出对话。' },
    ],
  },
  {
    slug: 'evening-crayfish', title: '傍晚小龙虾', type: '改稿', date: '2026-05',
    description: '一起吃小龙虾的傍晚，辣出来的话题比平时多。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '调整辣味话题的推进顺序。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简对白，去掉冗余细节。' },
    ],
  },
  {
    slug: 'convenience-store-talk', title: '便利店谈心', type: '改稿', date: '2026-05',
    description: '深夜便利店的透明玻璃前，聊了一些白天说不出口的话。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '补全白天说不出口的语境。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简旁白，统一人称。' },
    ],
  },
  {
    slug: 'night-market-date', title: '夜市约会', type: '改稿', date: '2026-05',
    description: '在夜市麻辣烫摊前，两个人心照不宣地确认了彼此的心意。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '调整心意确认前的铺垫。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简动作描写，突出氛围。' },
    ],
  },
  {
    slug: 'morning-greetings', title: '互相问早', type: '改稿', date: '2026-05',
    description: '从第一条早安开始，每一天都有了期待。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '补全第一条早安的由来。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '压缩时间线，统一视角。' },
    ],
  },
  {
    slug: 'hotpot-at-home', title: '家里吃火锅', type: '改稿', date: '2026-05',
    description: '在家煮火锅是最放松的时刻，锅里的热气让话也多了起来。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '调整话题密度，补全放松感。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简描写，突出对话。' },
    ],
  },
  {
    slug: 'beach-walk', title: '海边散步', type: '改稿', date: '2026-05',
    description: '海风、沙滩、并肩走的人，这个晚上一切都刚好。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '补全海风与心情的呼应。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简抒情，收敛结尾。' },
    ],
  },
  {
    slug: 'bar-confession', title: '酒吧表白', type: '改稿', date: '2026-05',
    description: '一杯酒壮了胆，十年的喜欢终于说出了口。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '调整表白前的情感铺垫。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简对白，统一视角。' },
    ],
  },
  {
    slug: 'park-hug', title: '公园拥抱', type: '改稿', date: '2026-05',
    description: '公园长椅上的一个拥抱，比任何语言都更有力量。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '补全拥抱前的犹豫。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简旁白，突出动作。' },
    ],
  },
  {
    slug: 'study-companionship', title: '书房陪伴', type: '改稿', date: '2026-05',
    description: '他在书房工作，她在一旁看书，安静地待在一起就很好了。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '调整安静相处的节奏。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简细节，统一视角。' },
    ],
  },
  {
    slug: 'living-room-drama', title: '客厅看剧', type: '改稿', date: '2026-05',
    description: '窝在沙发上看了一整晚的剧，剧情没记住多少，旁边的呼吸声倒是记得很清楚。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '补全呼吸声的观察角度。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简剧情描写，突出氛围。' },
    ],
  },
  {
    slug: 'balcony-stargazing', title: '阳台看星星', type: '改稿', date: '2026-05',
    description: '阳台上数星星，数到后来谁也记不清数到哪了。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '调整数星星的节奏。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简对话，收敛结尾。' },
    ],
  },
]

export const works: Work[] = [
  ...fictionWorks,
  // 文章 / 游戏 / 工具：无详情页，链接到外部原文或仓库
  {
    slug: 'cognitive-engineering', title: '认知工程', type: '文章', date: '2026-04',
    description: '认知外化、重构与减负——系统性地降低 AI 原生时代的认知负担。',
    link: 'https://github.com/quanttide/quanttide-archive-of-founder',
  },
  {
    slug: 'qtgame-weiqi', title: '围棋 · v0.1.2', type: '游戏', date: '2026-06',
    description: '连 Pass 终局与胜负判定完成，AI 可以完整下完一盘棋。',
    link: 'https://github.com/quanttide/qtgame-weiqi',
  },
  {
    slug: 'qtgame-war', title: '战争游戏 · 实验期', type: '游戏', date: '2026-08',
    description: '意图到行动的示例实验，推进玩法机制定型。',
    link: 'https://github.com/quanttide/qtgame-war',
  },
  {
    slug: 'qtgame-tycoon', title: '经营游戏 · v0.0.1', type: '游戏', date: '2026-04',
    description: '初始项目结构与游戏入口，建立基础迭代闭环。',
    link: 'https://github.com/quanttide/qtgame-tycoon',
  },
  {
    slug: 'voice-input', title: '语音输入工作流', type: '工具', date: '2026-08',
    description: '以语音降低记录成本，把口头想法直接转成可整理的文本。',
    link: 'https://github.com/quanttide/quanttide-memory-of-founder',
  },
  {
    slug: 'qtfounder-cli', title: 'qtfounder-cli', type: '工具', date: '2026-08',
    description: '把健康检查、提交、审查等 DevOps 流程固化为命令行工具。',
    link: 'https://github.com/quanttide/qtfounder',
  },
  {
    slug: 'second-brain', title: '第二大脑', type: '工具', date: '2026-06',
    description: '以可版本化、可追溯、可复用的方式管理认知资产。',
    link: 'https://github.com/quanttide/quanttide-memory-of-founder',
  },
]

export function getWork(slug: string): Work | undefined {
  return works.find(w => w.slug === slug)
}
