package lsyslib

import (
	"context"
	"encoding/json"
	"fmt"
	"net/url"
	"rest_client"
	"time"
)

const (
	TokenCreate  = 100
	TokenRefresh = 101
)

func init() {
	OAuthApiClientSetConfig(map[int]rest_client.RestBuild{
		TokenCreate: &OAuthRestBuild{
			Path:    "/oauth/token",
			Timeout: 60 * time.Second,
		},
		TokenRefresh: &OAuthRestBuild{
			Path:    "/oauth/refresh_token",
			Timeout: 60 * time.Second,
		},
	})
}

// TokenData OAUTH数据
type TokenData struct {
	AccessToken  string      `json:"access_token"`
	RefreshToken string      `json:"refresh_token"`
	OpenId       string      `json:"openid"`
	Scope        []string    `json:"scope"`
	ExpiresIn    json.Number `json:"expires_in"`
}

// OAuthAuthorizationUrl Oauth 登录URL生成
func (receiver *LsysApi) OAuthAuthorizationUrl(_ context.Context, callbackUrl string, scope string, state string) string {
	url := fmt.Sprintf(
		"%s?client_id=%s&redirect_uri=%s&response_type=code&scope=%s",
		receiver.config.AppOAuthHost, receiver.config.AppId,
		url.QueryEscape(callbackUrl),
		scope,
	)
	if len(state) > 0 {
		url = fmt.Sprintf("%s&state=%s", url, state)
	}
	return url
}

// OAuthAccessToken Oauth 通过CODE得到TOKEN
// code 用户授权登录后返回
func (receiver *LsysApi) OAuthAccessToken(ctx context.Context, code string) (*TokenData, error) {
	req := <-receiver.oauth.Do(ctx, TokenCreate, map[string]string{
		"code":          code,
		"client_secret": receiver.config.AppOAuthSecret,
		"client_id":     receiver.config.AppId,
	})
	var tokenData TokenData
	err := req.JsonResult().GetStruct("response", &tokenData)
	if err != nil {
		return nil, err
	}
	return &tokenData, nil
}

// TokenRestApi 获取需要TOKEN的rest接口实例
// token 从 OAuthAccessToken 接口获取
func (receiver *LsysApi) TokenRestApi(token string) *TokenRestApi {
	return &TokenRestApi{
		api: receiver,
		rest: receiver.client.NewApi(&RestTokenApiClient{
			OAuthToken: token,
		}),
		token: token,
	}
}

// OAuthRefreshToken Oauth 刷新TOKEN
func (receiver *TokenRestApi) OAuthRefreshToken(ctx context.Context) (*TokenData, error) {
	req := <-receiver.api.oauth.Do(ctx, TokenRefresh, map[string]string{
		"refresh_token": receiver.token,
		"client_secret": receiver.api.config.AppOAuthSecret,
		"client_id":     receiver.api.config.AppId,
	})
	var tokenData TokenData
	err := req.JsonResult().GetStruct("response", &tokenData)
	if err != nil {
		return nil, err
	}
	receiver.rest = receiver.api.client.NewApi(&RestTokenApiClient{
		OAuthToken: tokenData.AccessToken,
	})
	return &tokenData, nil
}
