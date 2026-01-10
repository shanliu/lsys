package lsyslib

import (
	"context"
	"net/http"
	"rest_client"
	"time"
)

const (
	SubAppInfo        = 600
	SubAppUser        = 601
	SubAppOAuthScope  = 602
	SubAppOAuthSecret = 603
)

func init() {
	RestApiClientSetConfig(map[int]rest_client.RestBuild{
		SubAppInfo: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/app",
			Method:     "sub_app_info",
			Timeout:    60 * time.Second,
		},
		SubAppUser: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/app",
			Method:     "sub_app_user",
			Timeout:    60 * time.Second,
		},
		SubAppOAuthScope: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/app",
			Method:     "sub_app_oauth_scope",
			Timeout:    60 * time.Second,
		},
		SubAppOAuthSecret: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/app",
			Method:     "sub_app_oauth_secret",
			Timeout:    60 * time.Second,
		},
	})
}

// SubApp 应用信息
func (receiver *LsysApi) SubAppInfo(ctx context.Context, clientId string) (*rest_client.JsonData, error) {
	data1 := (<-receiver.rest.Do(ctx, SubAppInfo, map[string]interface{}{
		"client_id": clientId,
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	return data1.GetData("response"), nil
}

// SubApp 用户信息
func (receiver *LsysApi) SubAppUser(ctx context.Context, clientId string) (*rest_client.JsonData, error) {
	data1 := (<-receiver.rest.Do(ctx, SubAppUser, map[string]interface{}{
		"client_id": clientId,
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	return data1.GetData("response"), nil
}

// SubAppOAuthScope 查询子应用OAuth登录已申请的SCOPE
func (receiver *LsysApi) SubAppOAuthScope(ctx context.Context, clientId string) (*rest_client.JsonData, error) {
	data1 := (<-receiver.rest.Do(ctx, SubAppOAuthScope, map[string]interface{}{
		"client_id": clientId,
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	return data1.GetData("response"), nil
}

// SubAppOAuthSecret 查询子应用OAuth登录信息及秘钥
func (receiver *LsysApi) SubAppOAuthSecret(ctx context.Context, clientId string) (*rest_client.JsonData, error) {
	data1 := (<-receiver.rest.Do(ctx, SubAppOAuthSecret, map[string]interface{}{
		"client_id": clientId,
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	return data1.GetData("response"), nil
}
