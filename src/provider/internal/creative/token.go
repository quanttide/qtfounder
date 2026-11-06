package creative

import (
	"crypto"
	"crypto/rand"
	"crypto/rsa"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"errors"
	"time"
)

// SignJWT 用 RS256 私钥签发带 iat/exp 的令牌，cmd/mint 与测试共用同一实现，
// 保证「签出来的」与「验得过的」永远是同一套编码。
func SignJWT(priv *rsa.PrivateKey, ttl time.Duration, now time.Time) (string, error) {
	if ttl <= 0 {
		return "", errors.New("ttl 必须为正")
	}
	header := []byte(`{"alg":"RS256","typ":"JWT"}`)
	payload, err := json.Marshal(struct {
		Sub string `json:"sub"`
		Iat int64  `json:"iat"`
		Exp int64  `json:"exp"`
	}{
		Sub: "qtfounder",
		Iat: now.Unix(),
		Exp: now.Add(ttl).Unix(),
	})
	if err != nil {
		return "", err
	}
	return signEncoded(header, payload, priv)
}

// signEncoded 拼接并签名前两段，签发与测试手工构造令牌的公共底层。
func signEncoded(header, payload []byte, priv *rsa.PrivateKey) (string, error) {
	signingInput := base64.RawURLEncoding.EncodeToString(header) + "." +
		base64.RawURLEncoding.EncodeToString(payload)
	digest := sha256.Sum256([]byte(signingInput))
	signature, err := rsa.SignPKCS1v15(rand.Reader, priv, crypto.SHA256, digest[:])
	if err != nil {
		return "", err
	}
	return signingInput + "." + base64.RawURLEncoding.EncodeToString(signature), nil
}
