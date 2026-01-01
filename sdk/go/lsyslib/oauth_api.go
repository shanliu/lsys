package lsyslib

import (
	"context"
	"rest_client"
)

// OAuthApiClient Oauth登录接口
type OAuthApiClient struct{}

// ConfigName 配置名,跟下面相同
func (res *OAuthApiClient) ConfigName(_ context.Context) (string, error) {
	return "lsys-oauth-config", nil
}

var oauthApiClientConfig map[int]rest_client.RestBuild

func OAuthApiClientSetConfig(config map[int]rest_client.RestBuild) {
	if oauthApiClientConfig == nil {
		oauthApiClientConfig = make(map[int]rest_client.RestBuild)
	}
	for k, v := range config {
		oauthApiClientConfig[k] = v
	}
}

// ConfigBuilds 统一配置调用接口
func (res *OAuthApiClient) ConfigBuilds(_ context.Context) (map[int]rest_client.RestBuild, error) {
	return oauthApiClientConfig, nil

}

// RestTokenApiClient 对外请求
type RestTokenApiClient struct {
	OAuthToken string
}

// ConfigName 配置名,跟下面相同
func (res *RestTokenApiClient) ConfigName(_ context.Context) (string, error) {
	return "lsys-rest-config", nil
}

var restTokenApiClientConfig map[int]rest_client.RestBuild

func RestTokenApiClientSetConfig(config map[int]rest_client.RestBuild) {
	if restTokenApiClientConfig == nil {
		restTokenApiClientConfig = make(map[int]rest_client.RestBuild)
	}
	for k, v := range config {
		restTokenApiClientConfig[k] = v
	}
}

// ConfigBuilds 统一配置调用接口
func (res *RestTokenApiClient) ConfigBuilds(_ context.Context) (map[int]rest_client.RestBuild, error) {
	return restTokenApiClientConfig, nil
}

// Token 需要TOKEN的REST接口
func (res *RestTokenApiClient) Token(_ context.Context) (string, error) {
	return res.OAuthToken, nil
}
