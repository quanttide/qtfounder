// Package creative 创作领域：模型、存储、传输
// 数据源：fiction（改稿章节）+ memory（roadmap/context 文档），与 Studio 共用环境变量
package creative

// Item 创作条目（对齐 Studio 的 CreativeItem）
type Item struct {
	Name     string `json:"name"`
	Path     string `json:"path"`
	Category string `json:"category"`
}

// Chapter 改稿章节（fiction/职场言情/4_改稿/*.md）
type Chapter struct {
	ID    string `json:"id"`    // 编号前缀，如 1_1
	Title string `json:"title"` // 标题，如 咖啡厅重逢
	Path  string `json:"path"`
}

// MemoryDoc memory 文档（roadmap/ 或 context/）
type MemoryDoc struct {
	Name     string `json:"name"`
	Category string `json:"category"` // roadmap / context
	Path     string `json:"path"`
}
