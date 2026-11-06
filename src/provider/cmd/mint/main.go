// qtfounder 铸 token：用本机私钥签发访问 provider 的 RS256 JWT。
// 私钥不进仓库，默认 ~/.local/share/qtfounder/jwt/RS256_private.pem，公钥（对应 JWKS）随部署配置。
// 用法：go run ./cmd/mint [-key 私钥路径] [-ttl 24h]，令牌打印到标准输出。
package main

import (
	"crypto/rsa"
	"crypto/x509"
	"encoding/pem"
	"flag"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"time"

	"github.com/quanttide-founder/qtfounder-provider/internal/creative"
)

func main() {
	keyPath := flag.String("key", defaultKeyPath(), "私钥 PEM 路径")
	ttl := flag.Duration("ttl", 24*time.Hour, "令牌有效期")
	flag.Parse()

	priv, err := loadPrivateKey(*keyPath)
	if err != nil {
		log.Fatalf("读取私钥失败: %v", err)
	}
	token, err := creative.SignJWT(priv, *ttl, time.Now())
	if err != nil {
		log.Fatalf("签发令牌失败: %v", err)
	}
	fmt.Println(token)
}

func defaultKeyPath() string {
	if fromEnv := os.Getenv("QTFOUNDER_JWT_PRIVATE_KEY"); fromEnv != "" {
		return fromEnv
	}
	home, err := os.UserHomeDir()
	if err != nil {
		return ""
	}
	return filepath.Join(home, ".local", "share", "qtfounder", "jwt", "RS256_private.pem")
}

func loadPrivateKey(path string) (*rsa.PrivateKey, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	block, _ := pem.Decode(data)
	if block == nil {
		return nil, fmt.Errorf("%s 不是合法 PEM", path)
	}
	if key, err := x509.ParsePKCS8PrivateKey(block.Bytes); err == nil {
		rsaKey, ok := key.(*rsa.PrivateKey)
		if !ok {
			return nil, fmt.Errorf("%s 不是 RSA 私钥", path)
		}
		return rsaKey, nil
	}
	key, err := x509.ParsePKCS1PrivateKey(block.Bytes)
	if err != nil {
		return nil, fmt.Errorf("%s 既不是 PKCS8 也不是 PKCS1 私钥", path)
	}
	return key, nil
}
