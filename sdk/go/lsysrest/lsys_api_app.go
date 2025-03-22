package lsysrest

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"net"
	"net/http"
	"rest_client"
	"strings"
)

// SubAppInfo 应用信息
func (receiver *RestApi) SubAppInfo(ctx context.Context, appId string) (*rest_client.JsonData, error) {
	data1 := (<-receiver.rest.Do(ctx, SubAppInfo, map[string]interface{}{
		"client_id": appId,
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	return data1.GetData("response"), nil
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
func (receiver *RestApi) AppAuthLogin(ctx context.Context, userData, userName string, req *http.Request) (string, error) {
	r, e := generateRandomString(32)
	if e != nil {
		return "", e
	}
	userAgent := ""
	clientIP := ""
	if req != nil {
		userAgent, clientIP = getUserAgentAndIP(req)
	}
	data1 := (<-receiver.rest.Do(ctx, AppAuthLogin, map[string]interface{}{
		"expire_time":  3600 * 24,
		"token_code":   r,
		"user_data":    userData,
		"user_name":    userName,
		"user_account": userName,
		"login_ip":     clientIP,
		"device_name":  userAgent,
	})).JsonResult()
	if data1.Err() != nil {
		return "", data1.Err()
	}
	return data1.GetData("response.token_data").String(), nil
}

// AppAuthInfo 退出
func (receiver *RestApi) AppAuthInfo(ctx context.Context, loginCode string) (*rest_client.JsonData, error) {
	data1 := (<-receiver.rest.Do(ctx, AppAuthInfo, map[string]interface{}{
		"token_data": loginCode,
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	return data1.GetData("response"), nil
}

// AppAuthLogout 退出
func (receiver *RestApi) AppAuthLogout(ctx context.Context, loginCode string) error {
	data1 := (<-receiver.rest.Do(ctx, AppAuthLogout, map[string]interface{}{
		"token_data": loginCode,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}
