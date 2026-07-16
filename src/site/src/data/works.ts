export interface Work {
  slug: string
  title: string
  type: '改稿'
  date: string
  description: string
}

export const fictionWorks: Work[] = [
  { slug: 'coffee-reunion', title: '咖啡厅重逢', type: '改稿', date: '2026-05', description: '十年后偶然在一家咖啡厅重遇，两个人都没想到会以这样的方式再见。' },
  { slug: 'sleepless-night', title: '深夜失眠', type: '改稿', date: '2026-05', description: '深夜翻来覆去，各自想着同一个人。' },
  { slug: 'exhibition-meeting', title: '展会再遇', type: '改稿', date: '2026-05', description: '工作场合的第二次碰面，比第一次多了点刻意。' },
  { slug: 'evening-crayfish', title: '傍晚小龙虾', type: '改稿', date: '2026-05', description: '一起吃小龙虾的傍晚，辣出来的话题比平时多。' },
  { slug: 'convenience-store-talk', title: '便利店谈心', type: '改稿', date: '2026-05', description: '深夜便利店的透明玻璃前，聊了一些白天说不出口的话。' },
  { slug: 'night-market-date', title: '夜市约会', type: '改稿', date: '2026-05', description: '在夜市麻辣烫摊前，两个人心照不宣地确认了彼此的心意。' },
  { slug: 'morning-greetings', title: '互相问早', type: '改稿', date: '2026-05', description: '从第一条早安开始，每一天都有了期待。' },
  { slug: 'hotpot-at-home', title: '家里吃火锅', type: '改稿', date: '2026-05', description: '在家煮火锅是最放松的时刻，锅里的热气让话也多了起来。' },
  { slug: 'beach-walk', title: '海边散步', type: '改稿', date: '2026-05', description: '海风、沙滩、并肩走的人，这个晚上一切都刚好。' },
  { slug: 'bar-confession', title: '酒吧表白', type: '改稿', date: '2026-05', description: '一杯酒壮了胆，十年的喜欢终于说出了口。' },
  { slug: 'park-hug', title: '公园拥抱', type: '改稿', date: '2026-05', description: '公园长椅上的一个拥抱，比任何语言都更有力量。' },
  { slug: 'study-companionship', title: '书房陪伴', type: '改稿', date: '2026-05', description: '他在书房工作，她在一旁看书，安静地待在一起就很好了。' },
  { slug: 'living-room-drama', title: '客厅看剧', type: '改稿', date: '2026-05', description: '窝在沙发上看了一整晚的剧，剧情没记住多少，旁边的呼吸声倒是记得很清楚。' },
  { slug: 'balcony-stargazing', title: '阳台看星星', type: '改稿', date: '2026-05', description: '阳台上数星星，数到后来谁也记不清数到哪了。' },
]

export function getWork(slug: string): Work | undefined {
  return fictionWorks.find(w => w.slug === slug)
}
