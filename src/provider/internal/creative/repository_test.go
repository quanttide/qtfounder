package creative

import (
	"path/filepath"
	"testing"
)

func TestIDAndTitleOf(t *testing.T) {
	id, title := idOf("1_1_咖啡厅重逢"), titleOf("1_1_咖啡厅重逢")
	if id != "1_1" {
		t.Errorf("idOf = %q, want 1_1", id)
	}
	if title != "咖啡厅重逢" {
		t.Errorf("titleOf = %q, want 咖啡厅重逢", title)
	}
}

func TestListChapters(t *testing.T) {
	// 真实数据源（测试环境路径：src/provider/internal/creative → quanttide-founder 根 = 6 级）
	repo := NewRepository(
		filepath.Join("..", "..", "..", "..", "..", "..", "assets", "fiction"),
		filepath.Join("..", "..", "..", "..", "..", "..", "assets", "memory"),
	)
	chapters, err := repo.ListChapters()
	if err != nil {
		t.Skipf("数据源不可用（跳过）：%v", err)
	}
	if len(chapters) == 0 {
		t.Error("应读到改稿章节")
	}
	// 第一个章节应为 1_1
	if chapters[0].ID != "1_1" {
		t.Errorf("首章节 ID = %q, want 1_1", chapters[0].ID)
	}
}

func TestListMemoryDocs(t *testing.T) {
	repo := NewRepository(
		filepath.Join("..", "..", "..", "..", "..", "..", "assets", "fiction"),
		filepath.Join("..", "..", "..", "..", "..", "..", "assets", "memory"),
	)
	docs, err := repo.ListMemoryDocs()
	if err != nil {
		t.Skipf("数据源不可用（跳过）：%v", err)
	}
	if len(docs) == 0 {
		t.Error("应读到 memory 文档")
	}
}
