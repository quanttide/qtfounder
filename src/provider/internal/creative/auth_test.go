package creative

import (
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"encoding/base64"
	"encoding/pem"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func newKey(t *testing.T) *rsa.PrivateKey {
	t.Helper()
	key, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		t.Fatalf("生成测试密钥失败: %v", err)
	}
	return key
}

func publicPEM(t *testing.T, key *rsa.PrivateKey) string {
	t.Helper()
	der, err := x509.MarshalPKIXPublicKey(&key.PublicKey)
	if err != nil {
		t.Fatalf("导出公钥失败: %v", err)
	}
	return string(pem.EncodeToMemory(&pem.Block{Type: "PUBLIC KEY", Bytes: der}))
}

func newRequest(path, token string) *http.Request {
	req := httptest.NewRequest("GET", path, nil)
	if token != "" {
		req.Header.Set("Authorization", "Bearer "+token)
	}
	return req
}

func okHandler() http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	})
}

// craftToken 用测试私钥手工拼一个指定头与体的令牌，用来构造签名合法但内容非法的边界样本。
func craftToken(t *testing.T, header, payload string, key *rsa.PrivateKey) string {
	t.Helper()
	token, err := signEncoded([]byte(header), []byte(payload), key)
	if err != nil {
		t.Fatalf("构造令牌失败: %v", err)
	}
	return token
}

func TestJWTAuth(t *testing.T) {
	key := newKey(t)
	pub, err := ParsePublicKey(publicPEM(t, key))
	if err != nil {
		t.Fatalf("解析公钥失败: %v", err)
	}
	auth := JWTAuth(okHandler(), pub)

	t.Run("未配置公钥时不启用鉴权", func(t *testing.T) {
		rec := httptest.NewRecorder()
		JWTAuth(okHandler(), nil).ServeHTTP(rec, newRequest("/chapters", ""))
		if rec.Code != http.StatusOK {
			t.Errorf("code = %d, want 200", rec.Code)
		}
	})

	t.Run("/healthz 免鉴权", func(t *testing.T) {
		rec := httptest.NewRecorder()
		auth.ServeHTTP(rec, newRequest("/healthz", ""))
		if rec.Code != http.StatusOK {
			t.Errorf("code = %d, want 200", rec.Code)
		}
	})

	t.Run("缺少令牌返回 401", func(t *testing.T) {
		rec := httptest.NewRecorder()
		auth.ServeHTTP(rec, newRequest("/chapters", ""))
		if rec.Code != http.StatusUnauthorized {
			t.Errorf("code = %d, want 401", rec.Code)
		}
	})

	t.Run("查询参数不再放行", func(t *testing.T) {
		req := httptest.NewRequest("GET", "/chapters?key=anything", nil)
		rec := httptest.NewRecorder()
		auth.ServeHTTP(rec, req)
		if rec.Code != http.StatusUnauthorized {
			t.Errorf("code = %d, want 401", rec.Code)
		}
	})

	t.Run("有效令牌返回 200", func(t *testing.T) {
		token, err := SignJWT(key, time.Hour, time.Now())
		if err != nil {
			t.Fatalf("签发令牌失败: %v", err)
		}
		rec := httptest.NewRecorder()
		auth.ServeHTTP(rec, newRequest("/chapters", token))
		if rec.Code != http.StatusOK {
			t.Errorf("code = %d, want 200", rec.Code)
		}
	})

	t.Run("过期令牌返回 401", func(t *testing.T) {
		token, err := SignJWT(key, time.Hour, time.Now().Add(-24*time.Hour))
		if err != nil {
			t.Fatalf("签发令牌失败: %v", err)
		}
		rec := httptest.NewRecorder()
		auth.ServeHTTP(rec, newRequest("/chapters", token))
		if rec.Code != http.StatusUnauthorized {
			t.Errorf("code = %d, want 401", rec.Code)
		}
	})

	t.Run("别的私钥签的令牌返回 401", func(t *testing.T) {
		token, err := SignJWT(newKey(t), time.Hour, time.Now())
		if err != nil {
			t.Fatalf("签发令牌失败: %v", err)
		}
		rec := httptest.NewRecorder()
		auth.ServeHTTP(rec, newRequest("/chapters", token))
		if rec.Code != http.StatusUnauthorized {
			t.Errorf("code = %d, want 401", rec.Code)
		}
	})

	t.Run("alg 换成 HS256 的令牌返回 401", func(t *testing.T) {
		token := craftToken(t, `{"alg":"HS256","typ":"JWT"}`, `{"exp":4102444800}`, key)
		rec := httptest.NewRecorder()
		auth.ServeHTTP(rec, newRequest("/chapters", token))
		if rec.Code != http.StatusUnauthorized {
			t.Errorf("code = %d, want 401", rec.Code)
		}
	})

	t.Run("alg 为 none 的令牌返回 401", func(t *testing.T) {
		header := base64.RawURLEncoding.EncodeToString([]byte(`{"alg":"none","typ":"JWT"}`))
		payload := base64.RawURLEncoding.EncodeToString([]byte(`{"exp":4102444800}`))
		rec := httptest.NewRecorder()
		auth.ServeHTTP(rec, newRequest("/chapters", header+"."+payload+"."))
		if rec.Code != http.StatusUnauthorized {
			t.Errorf("code = %d, want 401", rec.Code)
		}
	})

	t.Run("缺少 exp 的令牌返回 401", func(t *testing.T) {
		token := craftToken(t, `{"alg":"RS256","typ":"JWT"}`, `{"sub":"qtfounder"}`, key)
		rec := httptest.NewRecorder()
		auth.ServeHTTP(rec, newRequest("/chapters", token))
		if rec.Code != http.StatusUnauthorized {
			t.Errorf("code = %d, want 401", rec.Code)
		}
	})

	t.Run("畸形令牌返回 401", func(t *testing.T) {
		rec := httptest.NewRecorder()
		auth.ServeHTTP(rec, newRequest("/chapters", "not-a-jwt"))
		if rec.Code != http.StatusUnauthorized {
			t.Errorf("code = %d, want 401", rec.Code)
		}
	})
}

func TestParsePublicKeyRejectsGarbage(t *testing.T) {
	if _, err := ParsePublicKey("not a pem"); err == nil {
		t.Error("期望报错，实际通过")
	}
}

func TestPreflight(t *testing.T) {
	t.Run("OPTIONS 返回 204 且不进鉴权", func(t *testing.T) {
		inner := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			t.Error("预检不应落到内层处理器")
		})
		req := httptest.NewRequest("OPTIONS", "/chapters", nil)
		rec := httptest.NewRecorder()
		Preflight(inner).ServeHTTP(rec, req)
		if rec.Code != http.StatusNoContent {
			t.Errorf("code = %d, want 204", rec.Code)
		}
	})

	t.Run("GET 照常进内层", func(t *testing.T) {
		rec := httptest.NewRecorder()
		Preflight(okHandler()).ServeHTTP(rec, newRequest("/chapters", ""))
		if rec.Code != http.StatusOK {
			t.Errorf("code = %d, want 200", rec.Code)
		}
	})
}
