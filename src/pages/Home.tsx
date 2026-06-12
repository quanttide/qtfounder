import { Link } from 'react-router-dom'
import { fictionWorks } from '../data/works'

export default function Home() {
  const recentWorks = fictionWorks.slice(0, 3)

  return (
    <div className="page home">
      <section className="hero">
        <h1 className="hero-tagline">
          创业者，写作者。<br />
          已交付的产品与文字，定义了我的认知。
        </h1>
      </section>

      <section className="about">
        <h2>关于</h2>
        <p>
          写代码，也写字。在做知识工程相关的事情，业余时间写小说。
        </p>
        <p>
          相信好的故事和好的产品一样，需要对细节的反复打磨。
        </p>
      </section>

      <section className="section-works">
        <h2>作品</h2>
        {recentWorks.map(work => (
          <div className="work-item" key={work.slug}>
            <span className="work-type">{work.type}</span>
            <Link to={`/works/fiction/drafts/${work.slug}`} className="work-title">
              {work.title}
            </Link>
            <span className="work-date">{work.date}</span>
            <p className="work-desc">{work.description}</p>
          </div>
        ))}
        <Link to="/works" className="view-all">查看全部 &rarr;</Link>
      </section>
    </div>
  )
}
