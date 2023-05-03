package lsysrest

import (
	"context"
	"fmt"
	"net/http"
	url2 "net/url"
	"rest_client"
	"time"
)

const (
	TokenCreate  = iota
	TokenRefresh = iota
)

// OAuthApiClient Oauth登录接口
type OAuthApiClient struct{}

// ConfigName 配置名,跟下面相同
func (res *OAuthApiClient) ConfigName(_ context.Context) (string, error) {
	return "l-sys-oauth-config", nil
}

// ConfigBuilds 统一配置调用接口
func (res *OAuthApiClient) ConfigBuilds(_ context.Context) (map[int]rest_client.RestBuild, error) {
	return map[int]rest_client.RestBuild{
		TokenCreate: &OAuthRestBuild{
			Path:    "/oauth/token",
			Timeout: 60 * time.Second,
		},
		TokenRefresh: &OAuthRestBuild{
			Path:    "/oauth/refresh_token",
			Timeout: 60 * time.Second,
		},
	}, nil
}

const (
	UserInfo = iota
)

// RestTokenApiClient 对外请求
type RestTokenApiClient struct {
	OAuthToken string
}

// ConfigName 配置名,跟下面相同
func (res *RestTokenApiClient) ConfigName(_ context.Context) (string, error) {
	return "l-sys-rest-config", nil
}

// ConfigBuilds 统一配置调用接口
func (res *RestTokenApiClient) ConfigBuilds(_ context.Context) (map[int]rest_client.RestBuild, error) {
	return map[int]rest_client.RestBuild{
		UserInfo: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/oauth/user",
			Method:     "info",
			Timeout:    60 * time.Second,
		},
	}, nil
}

// Token 需要TOKEN的REST接口
func (res *RestTokenApiClient) Token(_ context.Context) (string, error) {
	return res.OAuthToken, nil
}

// TokenRestApi 调用接口
type TokenRestApi struct {
	rest  *rest_client.RestClient
	api   *RestApi
	token string
}

// TokenData OAUTH数据
type TokenData struct {
	AccessToken  string `json:"access_token"`
	RefreshToken string `json:"refresh_token"`
	OpenId       string `json:"openid"`
	Scope        string `json:"scope"`
	ExpiresIn    string `json:"expires_in"`
}

// OAuthAuthorizationUrl Oauth 登录URL生成
func (receiver *RestApi) OAuthAuthorizationUrl(_ context.Context, callbackUrl string, scope string, state string) string {
	url := fmt.Sprintf(
		"%s?client_id=%s&redirect_uri=%s&response_type=code&scope=%s",
		receiver.config.AppOAuthHost, receiver.config.AppId,
		url2.QueryEscape(callbackUrl),
		scope,
	)
	if len(state) > 0 {
		url = fmt.Sprintf("%s&state=%s", url, state)
	}
	return url
}

// OAuthAccessToken Oauth 通过CODE得到TOKEN
// code 用户授权登录后返回
func (receiver *RestApi) OAuthAccessToken(ctx context.Context, code string) (*TokenData, error) {
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
func (receiver *RestApi) TokenRestApi(token string) *TokenRestApi {
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
