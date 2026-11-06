package creative

import (
	"crypto"
	"crypto/rsa"
	"crypto/sha256"
	"crypto/x509"
	"encoding/base64"
	"encoding/json"
	"encoding/pem"
	"errors"
	"net/http"
	"os"
	"strings"
	"time"
)

// JWTAuth 基于 RS256 JWT 的访问控制中间件。
// 客户端通过 Authorization: Bearer <JWT> 携带令牌（cmd/mint 用本机私钥签发，公钥随部署注入）；
// pub 为 nil（环境变量 QTFOUNDER_JWT_PUBLIC_KEY 未设置）时不启用鉴权，即本地开发模式。
// /healthz 免鉴权：FC 健康检查期望 2xx，应用层拦截会导致实例无法就绪。
func JWTAuth(next http.Handler, pub *rsa.PublicKey) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if pub == nil || r.URL.Path == "/healthz" {
			next.ServeHTTP(w, r)
			return
		}
		if err := verifyBearer(r, pub, time.Now()); err != nil {
			w.Header().Set("WWW-Authenticate", `Bearer realm="qtfounder"`)
			http.Error(w, "unauthorized", http.StatusUnauthorized)
			return
		}
		next.ServeHTTP(w, r)
	})
}

// Preflight 让 CORS 预检在鉴权之前拿到 204：按规范预检不携带任何凭据、
// 且必须以 2xx 返回，否则浏览器直接判网络错误；Access-Control-* 响应头由 FC 网关补齐。
func Preflight(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodOptions {
			w.WriteHeader(http.StatusNoContent)
			return
		}
		next.ServeHTTP(w, r)
	})
}

// PublicKeyFromEnv 读取并解析 PEM 公钥环境变量；未设置时返回 nil（不启用鉴权），
// 设置了但解析不了则报错——宁可启动失败，也不能带着残缺配置开门。
func PublicKeyFromEnv() (*rsa.PublicKey, error) {
	value := os.Getenv("QTFOUNDER_JWT_PUBLIC_KEY")
	if value == "" {
		return nil, nil
	}
	return ParsePublicKey(value)
}

// ParsePublicKey 解析 PKIX（SubjectPublicKeyInfo）PEM 格式的 RSA 公钥。
func ParsePublicKey(pemText string) (*rsa.PublicKey, error) {
	block, _ := pem.Decode([]byte(pemText))
	if block == nil {
		return nil, errors.New("QTFOUNDER_JWT_PUBLIC_KEY 不是合法 PEM")
	}
	key, err := x509.ParsePKIXPublicKey(block.Bytes)
	if err != nil {
		return nil, err
	}
	rsaKey, ok := key.(*rsa.PublicKey)
	if !ok {
		return nil, errors.New("QTFOUNDER_JWT_PUBLIC_KEY 不是 RSA 公钥")
	}
	return rsaKey, nil
}

// verifyBearer 只认 Authorization 头：查询参数会落进 URL，被访问日志、浏览器历史与 Referer 带走。
func verifyBearer(r *http.Request, pub *rsa.PublicKey, now time.Time) error {
	const prefix = "Bearer "
	got := r.Header.Get("Authorization")
	if !strings.HasPrefix(got, prefix) {
		return errors.New("缺少 Bearer 令牌")
	}
	return VerifyJWT(strings.TrimSpace(got[len(prefix):]), pub, now)
}

// VerifyJWT 用标准库校验 RS256 令牌：alg 必须钉死为 RS256（拒绝 none 与对称算法，防算法混淆），
// 签名验过之后 exp 必须存在且未过期——没有过期时间的令牌等于永久通行证，一律拒收。
func VerifyJWT(token string, pub *rsa.PublicKey, now time.Time) error {
	parts := strings.Split(token, ".")
	if len(parts) != 3 {
		return errors.New("令牌不是三段式")
	}
	headerJSON, err := base64.RawURLEncoding.DecodeString(parts[0])
	if err != nil {
		return errors.New("令牌头不可解码")
	}
	var header struct {
		Alg string `json:"alg"`
	}
	if err := json.Unmarshal(headerJSON, &header); err != nil {
		return errors.New("令牌头不是 JSON")
	}
	if header.Alg != "RS256" {
		return errors.New("不接受的签名算法")
	}
	signature, err := base64.RawURLEncoding.DecodeString(parts[2])
	if err != nil {
		return errors.New("签名不可解码")
	}
	digest := sha256.Sum256([]byte(parts[0] + "." + parts[1]))
	if err := rsa.VerifyPKCS1v15(pub, crypto.SHA256, digest[:], signature); err != nil {
		return errors.New("签名不匹配")
	}
	payload, err := base64.RawURLEncoding.DecodeString(parts[1])
	if err != nil {
		return errors.New("令牌体不可解码")
	}
	var claims struct {
		Exp int64 `json:"exp"`
	}
	if err := json.Unmarshal(payload, &claims); err != nil {
		return errors.New("令牌体不是 JSON")
	}
	if claims.Exp <= 0 {
		return errors.New("令牌缺少 exp")
	}
	if now.Unix() >= claims.Exp {
		return errors.New("令牌已过期")
	}
	return nil
}
