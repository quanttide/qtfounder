import { Link } from 'react-router-dom'

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div className="layout">
      <header className="header">
        <Link to="/">Quanttide Founder</Link>
      </header>
      <main>{children}</main>
      <footer className="footer">
        <span>&copy; 2026</span>
      </footer>
    </div>
  )
}
