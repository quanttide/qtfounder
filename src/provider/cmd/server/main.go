// qtfounder provider：量潮创始人工作台服务端（创作数据 API）
// 数据源：fiction + memory（环境变量 QTFOUNDER_FICTION_PATH / QTFOUNDER_MEMORY_PATH，与 Studio 一致）
package main

import (
	"log"
	"net/http"
	"os"

	"github.com/quanttide/qtfounder-provider/internal/creative"
)

func main() {
	fictionPath := os.Getenv("QTFOUNDER_FICTION_PATH")
	memoryPath := os.Getenv("QTFOUNDER_MEMORY_PATH")
	if fictionPath == "" || memoryPath == "" {
		log.Fatal("请设置 QTFOUNDER_FICTION_PATH 和 QTFOUNDER_MEMORY_PATH（见 README）")
	}

	repo := creative.NewRepository(fictionPath, memoryPath)
	h := creative.NewHandler(repo)

	mux := http.NewServeMux()
	mux.HandleFunc("GET /api/chapters", h.ListChapters)
	mux.HandleFunc("GET /api/memory", h.ListMemoryDocs)
	mux.HandleFunc("GET /health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`{"status":"ok"}`))
	})

	addr := os.Getenv("QTFOUNDER_ADDR")
	if addr == "" {
		addr = ":8080"
	}
	log.Printf("qtfounder provider listening on %s (fiction=%s, memory=%s)", addr, fictionPath, memoryPath)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Fatal(err)
	}
}
