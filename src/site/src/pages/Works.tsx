import { useState } from 'react'
import { Link } from 'react-router-dom'
import { fictionWorks } from '../data/works'

type Filter = '全部' | '改稿'
const filters: Filter[] = ['全部', '改稿']

export default function Works() {
  const [activeFilter, setActiveFilter] = useState<Filter>('全部')

  const filtered = activeFilter === '全部'
    ? fictionWorks
    : fictionWorks.filter(w => w.type === activeFilter)

  return (
    <div className="page works-page">
      <Link to="/" className="back-link">&larr; 首页</Link>
      <h1>作品</h1>

      <nav className="filter-nav">
        {filters.map(f => (
          <button
            key={f}
            className={f === activeFilter ? 'filter-active' : ''}
            onClick={() => setActiveFilter(f)}
          >
            {f}
          </button>
        ))}
      </nav>

      {filtered.length === 0 ? (
        <p className="empty">暂无匹配作品</p>
      ) : (
        <div className="works-list">
          {filtered.map(work => (
            <div className="work-item" key={work.slug}>
              <span className="work-type">{work.type}</span>
              <Link to={`/works/fiction/drafts/${work.slug}`} className="work-title">
                {work.title}
              </Link>
              <span className="work-date">{work.date}</span>
              <p className="work-desc">{work.description}</p>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
