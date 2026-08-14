import { useState } from 'react'
import { useParams, Link } from 'react-router-dom'
import { getWork } from '../data/works'

// 收录的版本正文：<slug>/<version>.md；v0/v1 待数据自动化从 git 历史生成
const fictionContent = import.meta.glob('/src/content/works/fiction/*/*.md', {
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
  const [version, setVersion] = useState<string | null>(null)

  if (!work) {
    return (
      <div className="page work-detail">
        <Link to="/works" className="back-link">&larr; 作品</Link>
        <p className="empty">作品不存在</p>
      </div>
    )
  }

  const revisions = work.revisions ?? []
  // 默认查看最新版本（轨迹最后一个）
  const activeVersion = version ?? revisions[revisions.length - 1]?.version ?? 'v2'

  const filePath = `/src/content/works/fiction/${work.slug}/${activeVersion}.md`
  const raw = fictionContent[filePath]

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

        {revisions.length > 0 && (
          <section className="revision-trail">
            <h2>改稿轨迹</h2>
            <nav className="revision-switch">
              {revisions.map(rev => (
                <button
                  key={rev.version}
                  className={rev.version === activeVersion ? 'filter-active' : ''}
                  onClick={() => setVersion(rev.version)}
                >
                  {rev.label}
                </button>
              ))}
            </nav>
            <div className="revision-list">
              {revisions.map(rev => (
                <div
                  className={`revision-item${rev.version === activeVersion ? ' revision-current' : ''}`}
                  key={rev.version}
                >
                  <span className="revision-version">{rev.version} {rev.label}</span>
                  <span className="revision-date">{rev.date}</span>
                  <span className="revision-note">{rev.note}</span>
                </div>
              ))}
            </div>
          </section>
        )}

        {raw ? (
          <div className="detail-content">
            {renderContent(raw)}
          </div>
        ) : (
          <p className="empty">该版本正文未收录，仅保留版本记录</p>
        )}
      </article>
    </div>
  )
}
