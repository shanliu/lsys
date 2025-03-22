package lsysrest

import (
	"context"
	"net/http"
	"rest_client"
	"time"
)

const (
	SubAppInfo     = iota
	AppAuthLogin     = iota
	AppAuthLogout     = iota
	AppAuthInfo     = iota
	AccessCheck = iota
	SmeSend     = iota
	SmeCancel   = iota
	MailSend    = iota
	MailCancel  = iota
)

// RestApiClient 对外请求
type RestApiClient struct{}

// ConfigName 配置名,跟下面相同
func (res *RestApiClient) ConfigName(_ context.Context) (string, error) {
	return "l-sys-rest-config", nil
}

// ConfigBuilds 统一配置调用接口
func (res *RestApiClient) ConfigBuilds(_ context.Context) (map[int]rest_client.RestBuild, error) {
	return map[int]rest_client.RestBuild{
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
		SubAppInfo: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/subapp",
			Method:     "info",
			Timeout:    60 * time.Second,
		},
		AccessCheck: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/access",
			Method:     "check",
			Timeout:    60 * time.Second,
		},
		SmeSend: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/sms",
			Method:     "send",
			Timeout:    60 * time.Second,
		},
		SmeCancel: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/sms",
			Method:     "cancel",
			Timeout:    60 * time.Second,
		},
		MailSend: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/mail",
			Method:     "send",
			Timeout:    60 * time.Second,
		},
		MailCancel: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/mail",
			Method:     "cancel",
			Timeout:    60 * time.Second,
		},
	}, nil
}
