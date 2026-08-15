package creative

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// Repository 创作数据存储：从 fiction/memory 仓库读取（文件系统）
type Repository struct {
	FictionPath string // QTFOUNDER_FICTION_PATH
	MemoryPath  string // QTFOUNDER_MEMORY_PATH
}

// NewRepository 创建仓库
func NewRepository(fictionPath, memoryPath string) *Repository {
	return &Repository{FictionPath: fictionPath, MemoryPath: memoryPath}
}

// ListChapters 改稿章节列表（fiction/职场言情/4_改稿/*.md）
func (r *Repository) ListChapters() ([]Chapter, error) {
	dir := filepath.Join(r.FictionPath, "职场言情", "4_改稿")
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, err
	}
	var chapters []Chapter
	for _, e := range entries {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".md") {
			continue
		}
		name := strings.TrimSuffix(e.Name(), ".md")
		chapters = append(chapters, Chapter{
			ID:    idOf(name),
			Title: titleOf(name),
			Path:  filepath.Join(dir, e.Name()),
		})
	}
	sort.Slice(chapters, func(i, j int) bool { return lessChapterID(chapters[i].ID, chapters[j].ID) })
	return chapters, nil
}

// lessChapterID 按数值比较章节编号："10_1" 应排在 "1_1" 之后
func lessChapterID(a, b string) bool {
	ai, aj := chapterNumbers(a)
	bi, bj := chapterNumbers(b)
	if ai != bi {
		return ai < bi
	}
	return aj < bj
}

// chapterNumbers 解析章节编号 "10_1" → (10, 1)
func chapterNumbers(id string) (int, int) {
	var a, b int
	_, err := fmt.Sscanf(id, "%d_%d", &a, &b)
	if err != nil {
		return 0, 0
	}
	return a, b
}

// ListMemoryDocs memory 文档列表（roadmap/*.md + context/*.md）
func (r *Repository) ListMemoryDocs() ([]MemoryDoc, error) {
	var docs []MemoryDoc
	for _, category := range []string{"roadmap", "context"} {
		dir := filepath.Join(r.MemoryPath, category)
		entries, err := os.ReadDir(dir)
		if err != nil {
			continue // 单目录缺失不阻断
		}
		for _, e := range entries {
			if e.IsDir() || !strings.HasSuffix(e.Name(), ".md") {
				continue
			}
			docs = append(docs, MemoryDoc{
				Name:     strings.TrimSuffix(e.Name(), ".md"),
				Category: category,
				Path:     filepath.Join(dir, e.Name()),
			})
		}
	}
	sort.Slice(docs, func(i, j int) bool {
		if docs[i].Category != docs[j].Category {
			return docs[i].Category < docs[j].Category
		}
		return docs[i].Name < docs[j].Name
	})
	return docs, nil
}

// idOf 章节编号：文件名 "1_1_咖啡厅重逢" → "1_1"
func idOf(name string) string {
	parts := strings.SplitN(name, "_", 3)
	if len(parts) >= 2 {
		return parts[0] + "_" + parts[1]
	}
	return name
}

// titleOf 章节标题："1_1_咖啡厅重逢" → "咖啡厅重逢"
func titleOf(name string) string {
	parts := strings.SplitN(name, "_", 3)
	if len(parts) >= 3 {
		return parts[2]
	}
	return name
}
