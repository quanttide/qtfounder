import { BrowserRouter, Routes, Route } from "react-router-dom";
import Layout from "./components/Layout";
import Home from "./pages/Home";
import Works from "./pages/Works";
import WorkDetail from "./pages/WorkDetail";
import Brain from "./pages/Brain";

// 部署环境自适应：GitHub Pages 挂在 /qtfounder/ 子路径，OSS/CDN（founder.quanttide.com）
// 根路径直接访问时为 /，固定 basename 会导致 URL 不匹配而白屏
export default function App() {
  const basename = window.location.pathname.startsWith("/qtfounder")
    ? "/qtfounder"
    : "/";

  return (
    <BrowserRouter basename={basename}>
      <Layout>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/brain" element={<Brain />} />
          <Route path="/works" element={<Works />} />
          <Route path="/works/fiction/drafts/:slug" element={<WorkDetail />} />
        </Routes>
      </Layout>
    </BrowserRouter>
  );
}
