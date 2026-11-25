package creative

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"
)

func TestGetChapterHTTP(t *testing.T) {
	fiction := t.TempDir()
	dir := filepath.Join(fiction, "职场言情", "4_改稿")
	if err := os.MkdirAll(dir, 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, "2_3_深夜长谈.md"), []byte("夜很长"), 0o644); err != nil {
		t.Fatal(err)
	}

	mux := http.NewServeMux()
	mux.HandleFunc("GET /chapters/{id}", NewHandler(NewRepository(fiction, t.TempDir())).GetChapter)

	t.Run("存在返回 200 与正文", func(t *testing.T) {
		rec := httptest.NewRecorder()
		mux.ServeHTTP(rec, httptest.NewRequest("GET", "/chapters/2_3", nil))
		if rec.Code != http.StatusOK {
			t.Fatalf("code = %d, body = %s", rec.Code, rec.Body.String())
		}
		var detail ChapterDetail
		if err := json.Unmarshal(rec.Body.Bytes(), &detail); err != nil {
			t.Fatalf("响应不是 JSON: %v", err)
		}
		if detail.Content != "夜很长" {
			t.Errorf("content = %q", detail.Content)
		}
	})

	t.Run("不存在返回 404 而非 500", func(t *testing.T) {
		rec := httptest.NewRecorder()
		mux.ServeHTTP(rec, httptest.NewRequest("GET", "/chapters/9_9", nil))
		if rec.Code != http.StatusNotFound {
			t.Errorf("code = %d, want 404", rec.Code)
		}
	})
}
