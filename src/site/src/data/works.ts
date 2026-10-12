export type WorkType = '小说' | '文章' | '游戏' | '工具'

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
  /** 小说：数据源编号前缀（如 1.1、10.1） */
  number?: string
  /** 文章/游戏/工具：外部原文或仓库链接 */
  link?: string
  /** 仅小说：初稿 → 改稿轨迹 */
  revisions?: Revision[]
}

export const fictionWorks: Work[] = [
  {
    slug: 'preface', title: '前言', type: '小说', number: '0', date: '2026-08',
    description: '初稿前言，记录便利店谈心场景的写作起点。',
    revisions: [
      { version: 'v1', label: '初稿', date: '2026-08', note: '初稿版本。' },
    ],
  },
{
    slug: 'coffee-reunion', title: '咖啡厅重逢', type: '小说', number: '1.1', date: '2026-05',
    description: '十年后偶然在一家咖啡厅重遇，两个人都没想到会以这样的方式再见。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿，沙发咖啡场景。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '调整时间线，补全重逢动机。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简对白，统一视角。' },
    ],
  },
{
    slug: 'sleepless-night', title: '深夜失眠', type: '小说', number: '1.2', date: '2026-05',
    description: '深夜翻来覆去，各自想着同一个人。',
    revisions: [
      { version: 'v1', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '压缩内心独白，收紧节奏。' },
    ],
  },
{
    slug: 'exhibition-meeting', title: '展会再遇', type: '小说', number: '2.1', date: '2026-05',
    description: '工作场合的第二次碰面，比第一次多了点刻意。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '增加第二次碰面的刻意感。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简场景描写，突出对话。' },
    ],
  },
  {
    slug: 'male-lead-speech', title: '男主演讲', type: '小说', number: '2.2', date: '2026-08',
    description: '男主在行业论坛上的演讲，关于软件、工程与艺术创作。',
    revisions: [
      { version: 'v1', label: '初稿', date: '2026-08', note: '初稿版本。' },
    ],
  },
{
    slug: 'evening-crayfish', title: '傍晚小龙虾', type: '小说', number: '2.3', date: '2026-05',
    description: '一起吃小龙虾的傍晚，辣出来的话题比平时多。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '调整辣味话题的推进顺序。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简对白，去掉冗余细节。' },
    ],
  },
  {
    slug: 'work-daily', title: '工作日常', type: '小说', number: '2.3', date: '2026-08',
    description: '男主展示自己的命令行工具，日常工作中的创作片段。',
    revisions: [
      { version: 'v1', label: '初稿', date: '2026-08', note: '初稿版本。' },
    ],
  },
  {
    slug: 'work-help', title: '女主工作求助', type: '小说', number: '3.1', date: '2026-08',
    description: '女主被邮件淹没，向男主求助的开场。',
    revisions: [
      { version: 'v1', label: '初稿', date: '2026-08', note: '初稿版本。' },
    ],
  },
  {
    slug: 'chat', title: '男女主聊天', type: '小说', number: '3.2', date: '2026-08',
    description: '心里空洞的女主，想找他陪自己说说话。',
    revisions: [
      { version: 'v1', label: '初稿', date: '2026-08', note: '初稿版本。' },
    ],
  },
{
    slug: 'convenience-store-talk', title: '便利店谈心', type: '小说', number: '4.1', date: '2026-05',
    description: '深夜便利店的透明玻璃前，聊了一些白天说不出口的话。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '补全白天说不出口的语境。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简旁白，统一人称。' },
    ],
  },
{
    slug: 'night-market-date', title: '夜市约会', type: '小说', number: '4.2', date: '2026-05',
    description: '在夜市麻辣烫摊前，两个人心照不宣地确认了彼此的心意。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '调整心意确认前的铺垫。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简动作描写，突出氛围。' },
    ],
  },
{
    slug: 'morning-greetings', title: '互相问早', type: '小说', number: '4.3', date: '2026-05',
    description: '从第一条早安开始，每一天都有了期待。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '补全第一条早安的由来。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '压缩时间线，统一视角。' },
    ],
  },
{
    slug: 'hotpot-at-home', title: '家里吃火锅', type: '小说', number: '5.1', date: '2026-05',
    description: '在家煮火锅是最放松的时刻，锅里的热气让话也多了起来。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '调整话题密度，补全放松感。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简描写，突出对话。' },
    ],
  },
{
    slug: 'campus-revisit', title: '同游母校', type: '小说', number: '6.1', date: '2026-08',
    description: '回到母校，想起十年前唯一一次和她一起上课的时光。',
    revisions: [
      { version: 'v2', label: '语言润色', date: '2026-08', note: '当前改稿版本。' },
    ],
  },
{
    slug: 'beach-walk', title: '海边散步', type: '小说', number: '6.2', date: '2026-05',
    description: '海风、沙滩、并肩走的人，这个晚上一切都刚好。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '补全海风与心情的呼应。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简抒情，收敛结尾。' },
    ],
  },
{
    slug: 'bar-confession', title: '酒吧表白', type: '小说', number: '7.1', date: '2026-05',
    description: '一杯酒壮了胆，十年的喜欢终于说出了口。',
    revisions: [
      { version: 'v1', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简对白，统一视角。' },
    ],
  },
  {
    slug: 'evening-walk', title: '男女主散步', type: '小说', number: '7.1', date: '2026-08',
    description: '并肩走回家的路上，两人之间那堵绕不过去的墙。',
    revisions: [
      { version: 'v1', label: '初稿', date: '2026-08', note: '初稿版本。' },
    ],
  },
{
    slug: 'park-hug', title: '公园拥抱', type: '小说', number: '7.2', date: '2026-05',
    description: '公园长椅上的一个拥抱，比任何语言都更有力量。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '补全拥抱前的犹豫。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简旁白，突出动作。' },
    ],
  },
{
    slug: 'beach-walk-again', title: '海边再散步', type: '小说', number: '8.1', date: '2026-08',
    description: '再次在海边散步，终于解开“你在我身后”的心结。',
    revisions: [
      { version: 'v1', label: '初稿', date: '2026-08', note: '初稿版本。' },
      { version: 'v2', label: '语言润色', date: '2026-08', note: '当前改稿版本。' },
    ],
  },
{
    slug: 'friends-dinner', title: '好友聚餐', type: '小说', number: '8.2', date: '2026-08',
    description: '四人聚餐，牵线的真相在笑声中揭晓。',
    revisions: [
      { version: 'v2', label: '语言润色', date: '2026-08', note: '当前改稿版本。' },
    ],
  },
{
    slug: 'study-companionship', title: '书房陪伴', type: '小说', number: '10.1', date: '2026-05',
    description: '他在书房工作，她在一旁看书，安静地待在一起就很好了。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '调整安静相处的节奏。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简细节，统一视角。' },
    ],
  },
{
    slug: 'snow-talk', title: '赏雪谈心', type: '小说', number: '11.1', date: '2026-08',
    description: '雪天谈心，她第一次说起心里那个空旷的仓库。',
    revisions: [
      { version: 'v2', label: '语言润色', date: '2026-08', note: '当前改稿版本。' },
    ],
  },
{
    slug: 'career-dilemma', title: '女主事业困境', type: '小说', number: '11.2', date: '2026-08',
    description: '事业困境中的深夜对话，关于“如果重来一次”的假设。',
    revisions: [
      { version: 'v2', label: '语言润色', date: '2026-08', note: '当前改稿版本。' },
    ],
  },
{
    slug: 'caring-for-him', title: '女主照顾男主', type: '小说', number: '12.1', date: '2026-08',
    description: '失眠的深夜，她陪着他，也试着照顾那个不肯好好休息的人。',
    revisions: [
      { version: 'v2', label: '语言润色', date: '2026-08', note: '当前改稿版本。' },
    ],
  },
{
    slug: 'living-room-drama', title: '客厅看剧', type: '小说', number: '12.2', date: '2026-05',
    description: '窝在沙发上看了一整晚的剧，剧情没记住多少，旁边的呼吸声倒是记得很清楚。',
    revisions: [
      { version: 'v0', label: '初稿', date: '2026-04', note: '完整初稿。' },
      { version: 'v1', label: '叙事调整', date: '2026-05', note: '补全呼吸声的观察角度。' },
      { version: 'v2', label: '语言润色', date: '2026-05', note: '精简剧情描写，突出氛围。' },
    ],
  },
{
    slug: 'balcony-stargazing', title: '阳台看星星', type: '小说', number: '12.3', date: '2026-05',
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
    link: 'https://github.com/quanttide/quanttide-context-of-founder',
  },
  {
    slug: 'qtfounder-cli', title: 'qtfounder-cli', type: '工具', date: '2026-08',
    description: '把健康检查、提交、审查等 DevOps 流程固化为命令行工具。',
    link: 'https://github.com/quanttide/qtfounder',
  },
  {
    slug: 'second-brain', title: '第二大脑', type: '工具', date: '2026-06',
    description: '以可版本化、可追溯、可复用的方式管理认知资产。',
    link: 'https://github.com/quanttide/quanttide-context-of-founder',
  },
]

export function getWork(slug: string): Work | undefined {
  return works.find(w => w.slug === slug)
}
