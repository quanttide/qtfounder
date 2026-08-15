package creative

import (
	"encoding/json"
	"net/http"
)

// Handler 创作数据 HTTP 处理器
type Handler struct {
	repo *Repository
}

// NewHandler 创建处理器
func NewHandler(repo *Repository) *Handler {
	return &Handler{repo: repo}
}

// ListChapters 处理 GET /api/chapters
func (h *Handler) ListChapters(w http.ResponseWriter, r *http.Request) {
	chapters, err := h.repo.ListChapters()
	if err != nil {
		http.Error(w, "改稿章节加载失败："+err.Error(), http.StatusInternalServerError)
		return
	}
	writeJSON(w, http.StatusOK, map[string]any{"chapters": chapters})
}

// ListMemoryDocs 处理 GET /api/memory
func (h *Handler) ListMemoryDocs(w http.ResponseWriter, r *http.Request) {
	docs, err := h.repo.ListMemoryDocs()
	if err != nil {
		http.Error(w, "memory 文档加载失败："+err.Error(), http.StatusInternalServerError)
		return
	}
	writeJSON(w, http.StatusOK, map[string]any{"docs": docs})
}

func writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(v)
}
