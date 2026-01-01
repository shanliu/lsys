package lsyslib

import (
	"rest_client"
)

// RestApiConfig 配置
type LsysApiConfig struct {
	//应用可在 https://www.lsys.cc/user/app/create 申请
	AppId     string //应用ID
	AppHost   string //应用HOST
	AppSecret string //应用Secret
	//不使用OAuth下面可以不要
	AppOAuthHost   string //应用OAuth HOST
	AppOAuthSecret string //应用OAuthSecret
}

// RestApi 调用接口
type LsysApi struct {
	client *rest_client.RestClientManager
	rest   *rest_client.RestClient
	oauth  *rest_client.RestClient
	config *LsysApiConfig
}

func (me *LsysApi) Config() *LsysApiConfig {
	return me.config
}

// NewRestApi 新建调用接口实例
func NewRestApi(config *LsysApiConfig) *LsysApi {
	client := rest_client.NewRestClientManager()
	//配置
	client.SetRestConfig(&RestClientConfig{
		Name:      "lsys-rest-config",
		AppKey:    config.AppId,
		AppSecret: config.AppSecret,
		AppUrl:    config.AppHost,
	})
	client.SetRestConfig(&OAuthClientConfig{
		Name:      "lsys-oauth-config",
		AppKey:    config.AppId,
		AppSecret: config.AppOAuthSecret,
		AppUrl:    config.AppHost,
	})
	return &LsysApi{
		client: client,
		rest:   client.NewApi(&RestApiClient{}),
		oauth:  client.NewApi(&OAuthApiClient{}),
		config: config,
	}
}
