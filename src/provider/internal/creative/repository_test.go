package creative

import (
	"errors"
	"os"
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

func TestListTodos(t *testing.T) {
	fiction := t.TempDir()
	memory := t.TempDir()

	if err := os.WriteFile(filepath.Join(fiction, "TODO.md"),
		[]byte("# TODO\n\n## 重生言情\n\n- [ ] 重逢篇增加演武大桥场景描写。\n\n## 校园言情\n\n- [x] 已经做完的一条\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	// memory 侧嵌一层也要被找到
	if err := os.MkdirAll(filepath.Join(memory, "default"), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(memory, "default", "TODO.md"), []byte("- [ ] 没有小节的待办\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	// .git 里的 TODO.md 不该被读进来
	if err := os.MkdirAll(filepath.Join(memory, ".git"), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(memory, ".git", "TODO.md"), []byte("- [ ] 不该出现\n"), 0o644); err != nil {
		t.Fatal(err)
	}

	repo := NewRepository(fiction, memory)
	todos, err := repo.ListTodos()
	if err != nil {
		t.Fatalf("ListTodos: %v", err)
	}
	if len(todos) != 3 {
		t.Fatalf("len = %d, want 3（%+v）", len(todos), todos)
	}
	if todos[0].Group != "重生言情" || todos[0].Done {
		t.Errorf("第一条 = %+v, want 重生言情/未完成", todos[0])
	}
	if todos[1].Group != "校园言情" || !todos[1].Done {
		t.Errorf("第二条 = %+v, want 校园言情/已完成", todos[1])
	}
	if todos[2].Group != "" || todos[2].Text != "没有小节的待办" {
		t.Errorf("第三条 = %+v, want 无小节", todos[2])
	}
	for _, todo := range todos {
		if todo.Text == "不该出现" {
			t.Error("读到了 .git 下的 TODO.md")
		}
	}
}

// 空数据源也要返回非 nil 空切片，否则 json 编出 "todos":null，客户端数组假设被击穿
func TestListTodosEmptyReturnsJSONArray(t *testing.T) {
	repo := NewRepository(t.TempDir(), t.TempDir())
	todos, err := repo.ListTodos()
	if err != nil {
		t.Fatalf("ListTodos: %v", err)
	}
	if todos == nil {
		t.Error("todos 为 nil，应是非 nil 空切片")
	}
}

// 数据源路径：默认主仓库本地布局（src/provider/internal/creative → quanttide-founder 根 = 6 级）；
// 可用 QTFOUNDER_TEST_DATA_ROOT 覆盖。目录不存在时跳过（CI 单仓库 checkout 无数据源）
func testDataRepo(t *testing.T) *Repository {
	t.Helper()
	root := os.Getenv("QTFOUNDER_TEST_DATA_ROOT")
	if root == "" {
		root = "../../../.."
	}
	fiction := filepath.Join(root, "assets", "fiction")
	memory := filepath.Join(root, "assets", "memory")
	for _, dir := range []string{fiction, memory} {
		if _, err := os.Stat(dir); err != nil {
			t.Skipf("数据源不可用（跳过）：%v", err)
		}
	}
	return NewRepository(fiction, memory)
}

func TestListChapters(t *testing.T) {
	repo := testDataRepo(t)
	chapters, err := repo.ListChapters()
	if err != nil {
		t.Skipf("数据源不可用（跳过）：%v", err)
	}
	if len(chapters) == 0 {
		t.Error("应读到改稿章节")
	}
	// 章节应按编号升序（首章节编号不大于末章节）
	if len(chapters) > 1 {
		first, last := chapters[0].ID, chapters[len(chapters)-1].ID
		if lessChapterID(last, first) {
			t.Errorf("章节未排序：first=%q last=%q", first, last)
		}
	}
}

func TestListMemoryDocs(t *testing.T) {
	repo := testDataRepo(t)
	docs, err := repo.ListMemoryDocs()
	if err != nil {
		t.Skipf("数据源不可用（跳过）：%v", err)
	}
	if len(docs) == 0 {
		t.Error("应读到 memory 文档")
	}
}

// 改稿目录缺失＝没有改稿章节：空表而非报错（否则 lite 首页整页被一次 500 掀掉）
func TestListChaptersMissingDirReturnsEmpty(t *testing.T) {
	repo := NewRepository(t.TempDir(), t.TempDir())
	chapters, err := repo.ListChapters()
	if err != nil {
		t.Fatalf("目录缺失应返回空表，实际 err = %v", err)
	}
	if chapters == nil || len(chapters) != 0 {
		t.Errorf("chapters = %+v, want 非 nil 空切片", chapters)
	}
}

// 空数据源必须返回非 nil 空切片，否则 json 编出 "docs":null，客户端数组假设被击穿（lite 曾因此白屏）
func TestListEmptySourceReturnsJSONArrays(t *testing.T) {
	repo := NewRepository(t.TempDir(), t.TempDir())
	docs, err := repo.ListMemoryDocs()
	if err != nil {
		t.Fatalf("ListMemoryDocs: %v", err)
	}
	if docs == nil {
		t.Error("docs 为 nil，应是非 nil 空切片")
	}
	chapters, err := repo.ListChapters()
	if err == nil && chapters == nil {
		t.Error("chapters 为 nil，应是非 nil 空切片")
	}
}

func TestGetChapter(t *testing.T) {
	fiction := t.TempDir()
	dir := filepath.Join(fiction, "职场言情", "4_改稿")
	if err := os.MkdirAll(dir, 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, "1_1_咖啡厅重逢.md"), []byte("# 第一段\n正文内容"), 0o644); err != nil {
		t.Fatal(err)
	}
	repo := NewRepository(fiction, t.TempDir())

	detail, err := repo.GetChapter("1_1")
	if err != nil {
		t.Fatalf("GetChapter: %v", err)
	}
	if detail.Title != "咖啡厅重逢" {
		t.Errorf("title = %q, want 咖啡厅重逢", detail.Title)
	}
	if detail.Content != "# 第一段\n正文内容" {
		t.Errorf("content = %q", detail.Content)
	}

	if _, err := repo.GetChapter("9_9"); !errors.Is(err, ErrNotFound) {
		t.Errorf("未知 id err = %v, want ErrNotFound", err)
	}
	// 越界形状的 id 只会「查无此章」——用户输入从不拼进路径
	if _, err := repo.GetChapter("../../etc/passwd"); !errors.Is(err, ErrNotFound) {
		t.Errorf("越界 id err = %v, want ErrNotFound", err)
	}
}
