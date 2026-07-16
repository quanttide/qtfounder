import { useParams, Link } from 'react-router-dom'
import { getWork } from '../data/works'

const fictionContent = import.meta.glob('/src/content/works/fiction/*.md', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>

function renderContent(raw: string) {
  const body = raw.replace(/^# .+\n/, '').trim()
  return body.split('\n\n').map((para, i) => (
    <p key={i}>
      {para.split('\n').flatMap((line, j) =>
        j === 0 ? [line] : [<br key={`br-${i}-${j}`} />, line]
      )}
    </p>
  ))
}

export default function WorkDetail() {
  const { slug } = useParams<{ slug: string }>()
  const work = slug ? getWork(slug) : undefined

  if (!work) {
    return (
      <div className="page work-detail">
        <Link to="/works" className="back-link">&larr; 作品</Link>
        <p className="empty">作品不存在</p>
      </div>
    )
  }

  const filePath = `/src/content/works/fiction/${work.slug}.md`
  const raw = fictionContent[filePath]

  if (!raw) {
    return (
      <div className="page work-detail">
        <Link to="/works" className="back-link">&larr; 作品</Link>
        <p className="empty">内容加载失败</p>
      </div>
    )
  }

  return (
    <div className="page work-detail">
      <Link to="/works" className="back-link">&larr; 作品</Link>
      <article>
        <header className="detail-header">
          <h1>{work.title}</h1>
          <div className="detail-meta">
            <span className="work-type">{work.type}</span>
            <span className="work-date">{work.date}</span>
          </div>
        </header>
        <div className="detail-content">
          {renderContent(raw)}
        </div>
      </article>
    </div>
  )
}
