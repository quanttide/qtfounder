package creative

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// ErrNotFound 请求的条目在数据源里不存在（转 404，不 500）
var ErrNotFound = errors.New("条目不存在")

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
	chapters := make([]Chapter, 0)
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

// GetChapter 章节详情（正文）：只在改稿目录的文件名清单里比对 id，
// 用户输入从不拼进路径——越界形状的 id 只会「查无此章」，天然无路径穿越。
func (r *Repository) GetChapter(id string) (ChapterDetail, error) {
	dir := filepath.Join(r.FictionPath, "职场言情", "4_改稿")
	entries, err := os.ReadDir(dir)
	if err != nil {
		return ChapterDetail{}, err
	}
	for _, e := range entries {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".md") {
			continue
		}
		name := strings.TrimSuffix(e.Name(), ".md")
		if idOf(name) != id {
			continue
		}
		path := filepath.Join(dir, e.Name())
		body, err := os.ReadFile(path)
		if err != nil {
			return ChapterDetail{}, err
		}
		return ChapterDetail{ID: id, Title: titleOf(name), Path: path, Content: string(body)}, nil
	}
	return ChapterDetail{}, fmt.Errorf("%w: %s", ErrNotFound, id)
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

// ListMemoryDocs memory 文档列表（journal/*.md + handbook/*.md）
func (r *Repository) ListMemoryDocs() ([]MemoryDoc, error) {
	// 空表也要是非 nil 切片：nil 经 json 编码成 null，会击穿客户端的数组假设
	docs := make([]MemoryDoc, 0)
	for _, category := range []string{"journal", "handbook"} {
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

// ListTodos 待办列表：递归找两个数据源里的 TODO.md，解析 `- [ ]` / `- [x]`
// 与所在 `## 小节`；单个文件读不到不阻断（同 ListMemoryDocs 的容忍度），
// 找不到任何待办时返回非 nil 空切片，json 编成 [] 而不是 null
func (r *Repository) ListTodos() ([]Todo, error) {
	todos := make([]Todo, 0)
	for _, root := range []string{r.FictionPath, r.MemoryPath} {
		if root == "" {
			continue
		}
		_ = filepath.WalkDir(root, func(path string, d os.DirEntry, err error) error {
			if err != nil {
				return nil // 子树不可读（权限/断链）跳过，不整表失败
			}
			if d.IsDir() {
				if d.Name() == ".git" {
					return fs.SkipDir
				}
				return nil
			}
			if d.Name() != "TODO.md" {
				return nil
			}
			items, perr := parseTodos(path)
			if perr != nil {
				return nil
			}
			todos = append(todos, items...)
			return nil
		})
	}
	return todos, nil
}

// parseTodos 解析一个 TODO.md：`## 标题` 记为当前小节，`- [ ]`/`- [x]` 记为条目
func parseTodos(path string) ([]Todo, error) {
	body, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	items := make([]Todo, 0)
	group := ""
	for _, line := range strings.Split(string(body), "\n") {
		trimmed := strings.TrimSpace(line)
		switch {
		case strings.HasPrefix(trimmed, "## "):
			group = strings.TrimSpace(strings.TrimPrefix(trimmed, "## "))
		case strings.HasPrefix(trimmed, "- [ ]"), strings.HasPrefix(trimmed, "- [x]"), strings.HasPrefix(trimmed, "- [X]"):
			text := strings.TrimSpace(trimmed[len("- [ ]"):])
			if text == "" {
				continue
			}
			items = append(items, Todo{Group: group, Text: text, Done: !strings.HasPrefix(trimmed, "- [ ]")})
		}
	}
	return items, nil
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
