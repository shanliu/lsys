package lsysrest

import (
	"context"
	"net/http"
	"rest_client"
	"time"
)

const (
	AppInfo     = iota
	AccessCheck = iota
	SmeSend     = iota
	SmeCancel   = iota
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
		AppInfo: &rest_client.AppRestBuild{
			HttpMethod: http.MethodPost,
			Path:       "/rest/app",
			Method:     "view",
			Timeout:    60 * time.Second,
		},
		SmeSend: &rest_client.AppRestBuild{
			HttpMethod: http.MethodPost,
			Path:       "/rest/sms",
			Method:     "send",
			Timeout:    60 * time.Second,
		},
		SmeCancel: &rest_client.AppRestBuild{
			HttpMethod: http.MethodPost,
			Path:       "/rest/sms",
			Method:     "cancel",
			Timeout:    60 * time.Second,
		},
		AccessCheck: &rest_client.AppRestBuild{
			HttpMethod: http.MethodPost,
			Path:       "/rbac/access",
			Method:     "check",
			Timeout:    60 * time.Second,
		},
	}, nil
}
