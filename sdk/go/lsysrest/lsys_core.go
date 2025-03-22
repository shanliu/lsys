package lsysrest

import (
	"rest_client"
)

// RestApiConfig 配置
type RestApiConfig struct {
	//应用可在 https://www.lsys.site/app.html#/user/app 申请
	AppId     string //应用ID
	AppHost   string //应用HOST
	AppSecret string //应用Secret
	//不使用OAuth下面可以不要
	AppOAuthHost   string //应用OAuth HOST
	AppOAuthSecret string //应用OAuthSecret
}

// RestApi 调用接口
type RestApi struct {
	client *rest_client.RestClientManager
	rest   *rest_client.RestClient
	oauth  *rest_client.RestClient
	config *RestApiConfig
}

func (me *RestApi) Config() *RestApiConfig {
	return me.config
}

// NewRestApi 新建调用接口实例
func NewRestApi(config *RestApiConfig) *RestApi {
	client := rest_client.NewRestClientManager()
	//配置
	client.SetRestConfig(&RestClientConfig{
		Name:      "l-sys-rest-config",
		AppKey:    config.AppId,
		AppSecret: config.AppSecret,
		AppUrl:    config.AppHost,
	})
	client.SetRestConfig(&OAuthRestConfig{
		Name:      "l-sys-oauth-config",
		AppKey:    config.AppId,
		AppSecret: config.AppOAuthSecret,
		AppUrl:    config.AppHost,
	})
	return &RestApi{
		client: client,
		rest:   client.NewApi(&RestApiClient{}),
		oauth:  client.NewApi(&OAuthApiClient{}),
		config: config,
	}
}
