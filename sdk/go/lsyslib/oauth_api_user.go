package lsyslib

import (
	"context"
	"net/http"
	"rest_client"
	"time"
)

const (
	UserInfo = 200
)

func init() {
	RestTokenApiClientSetConfig(map[int]rest_client.RestBuild{
		UserInfo: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/oauth/user",
			Method:     "info",
			Timeout:    60 * time.Second,
		},
	})
}

// TokenRestApi 调用接口
type TokenRestApi struct {
	rest  *rest_client.RestClient
	api   *LsysApi
	token string
}

// OAuthUserInfo 获取用户资料
func (receiver *TokenRestApi) OAuthUserInfo(ctx context.Context, user bool, name bool, info bool, address bool, email bool, mobile bool) (*rest_client.JsonData, error) {
	req := <-receiver.rest.Do(ctx, UserInfo, map[string]interface{}{
		"user":    user,
		"name":    name,
		"info":    info,
		"address": address,
		"email":   email,
		"mobile":  mobile,
	})
	res := req.JsonResult().GetData("response")
	if res.Err() != nil {
		return nil, res.Err()
	}
	return res, nil
}
