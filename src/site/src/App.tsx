import { BrowserRouter, Routes, Route } from "react-router-dom";
import Layout from "./components/Layout";
import Home from "./pages/Home";
import Works from "./pages/Works";
import WorkDetail from "./pages/WorkDetail";
import Brain from "./pages/Brain";

export default function App() {
  return (
    <BrowserRouter basename="/qtfounder">
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
