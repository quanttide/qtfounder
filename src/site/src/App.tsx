import { BrowserRouter, Routes, Route } from "react-router-dom";
import Layout from "./components/Layout";
import Home from "./pages/Home";
import Works from "./pages/Works";
import WorkDetail from "./pages/WorkDetail";
import Brain from "./pages/Brain";

// 域名根路径部署（founder.quanttide.com，无子路径前缀）
export default function App() {
  return (
    <BrowserRouter>
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
