package lsyslib

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"net"
	"net/http"
	"rest_client"
	"strings"
	"time"
)

const (
	AppAuthLogin  = 301
	AppAuthLogout = 302
	AppAuthInfo   = 303
)

func init() {
	RestApiClientSetConfig(map[int]rest_client.RestBuild{
		AppAuthLogin: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/auth",
			Method:     "do_login",
			Timeout:    60 * time.Second,
		},
		AppAuthLogout: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/auth",
			Method:     "do_logout",
			Timeout:    60 * time.Second,
		},
		AppAuthInfo: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/auth",
			Method:     "login_info",
			Timeout:    60 * time.Second,
		},
	})
}

func generateRandomString(n int) (string, error) {
	b := make([]byte, n)
	_, err := rand.Read(b)
	if err != nil {
		return "", err // 返回错误，避免panic
	}
	s := base64.URLEncoding.EncodeToString(b)
	return s[:n], nil // 确保返回的字符串长度为n
}
func getUserAgentAndIP(r *http.Request) (userAgent string, clientIP string) {
	// 获取 User-Agent
	userAgent = r.UserAgent()

	// 尝试从 X-Forwarded-For 头部获取 IP 地址
	xForwardedFor := r.Header.Get("X-Forwarded-For")
	if xForwardedFor != "" {
		clientIP = strings.Split(xForwardedFor, ",")[0]
	} else {
		// 直接从 RemoteAddr 获取 IP 地址
		clientIP, _, _ = net.SplitHostPort(r.RemoteAddr)
	}

	return userAgent, clientIP
}

// AppAuthLogin 登录
func (receiver *LsysApi) AppAuthLogin(ctx context.Context, userData, userName string, req *http.Request) (string, error) {
	r, e := generateRandomString(32)
	if e != nil {
		return "", e
	}
	userAgent := ""
	clientIP := "127.0.0.1"
	if req != nil {
		userAgent, clientIP = getUserAgentAndIP(req)
	}
	data1 := (<-receiver.rest.Do(ctx, AppAuthLogin, map[string]interface{}{
		"user_nickname": userName,
		"expire_time":   3600 * 24,
		"token_code":    r,
		"user_data":     userData,
		"user_name":     userName,
		"user_account":  userName,
		"login_ip":      clientIP,
		"device_name":   userAgent,
	})).JsonResult()
	if data1.Err() != nil {
		return "", data1.Err()
	}
	return data1.GetData("response.token_data").String(), nil
}

// AppAuthInfo 退出
func (receiver *LsysApi) AppAuthInfo(ctx context.Context, tokenData string) (*rest_client.JsonData, error) {
	data1 := (<-receiver.rest.Do(ctx, AppAuthInfo, map[string]interface{}{
		"token_data": tokenData,
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	return data1.GetData("response"), nil
}

// AppAuthLogout 退出
func (receiver *LsysApi) AppAuthLogout(ctx context.Context, tokenData string) error {
	data1 := (<-receiver.rest.Do(ctx, AppAuthLogout, map[string]interface{}{
		"token_data": tokenData,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}
