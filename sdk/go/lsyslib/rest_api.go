package lsyslib

import (
	"context"
	"rest_client"
)

// RestApiClient 对外请求
type RestApiClient struct{}

// ConfigName 配置名,跟下面相同
func (res *RestApiClient) ConfigName(_ context.Context) (string, error) {
	return "lsys-rest-config", nil
}

var restApiClientConfig map[int]rest_client.RestBuild

func RestApiClientSetConfig(config map[int]rest_client.RestBuild) {
	if restApiClientConfig == nil {
		restApiClientConfig = make(map[int]rest_client.RestBuild)
	}
	for k, v := range config {
		restApiClientConfig[k] = v
	}
}

// ConfigBuilds 统一配置调用接口
func (res *RestApiClient) ConfigBuilds(_ context.Context) (map[int]rest_client.RestBuild, error) {
	return restApiClientConfig, nil
}
