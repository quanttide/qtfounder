import { Link } from 'react-router-dom'

export default function Brain() {
  const systems = [
    {
      name: 'Memory',
      path: 'https://github.com/quanttide/quanttide-memory-of-founder',
      desc: '工作记忆。日志、文库、路线图——记录思考过程和决策依据。',
    },
    {
      name: 'Fiction',
      path: 'https://github.com/quanttide/quanttide-fiction-of-founder',
      desc: '小说创作资产。灵感、脚本、初稿、成稿——从想法到作品的完整链条。',
    },
    {
      name: 'Archive',
      path: 'https://github.com/quanttide/quanttide-archive-of-founder',
      desc: '已完成项目的归档。不再活跃但值得保留的资产。',
    },
  ]

  return (
    <div className="page brain">
      <section className="hero">
        <h1 className="hero-tagline">
          第二大脑
        </h1>
        <p>
          以可版本化、可追溯、可复用的方式，管理认知资产。
        </p>
      </section>

      <section className="about">
        <h2>原则</h2>
        <div className="principle-list">
          <div className="principle-item">
            <h3>写下来</h3>
            <p>大脑擅长的不是存储，是处理。把想法写下来，才能腾出空间思考下一步。</p>
          </div>
          <div className="principle-item">
            <h3>可追溯</h3>
            <p>每个决定都有上下文。Git 记录的不只是代码，还有想法的演变过程。</p>
          </div>
          <div className="principle-item">
            <h3>可复用</h3>
            <p>素材、方法论、模式——从具体项目中提炼，在下一次工作中复用。</p>
          </div>
        </div>
      </section>

      <section className="section-works">
        <h2>资产仓库</h2>
        {systems.map(s => (
          <div className="work-item" key={s.name}>
            <span className="work-type">repo</span>
            <a href={s.path} className="work-title" target="_blank" rel="noopener noreferrer">
              {s.name}
            </a>
            <p className="work-desc">{s.desc}</p>
          </div>
        ))}
      </section>

      <section className="about">
        <h2>工作流</h2>
        <p>
          以日志记录日常，以文库沉淀方法论，以路线图指引方向。三者构成一个循环：
          记录 → 提炼 → 规划 → 执行 → 记录。
        </p>
        <p>
          <Link to="/works">查看作品 &rarr;</Link>
        </p>
      </section>
    </div>
  )
}
