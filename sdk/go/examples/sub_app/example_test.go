package main

import (
	"context"
	"fmt"
	"lsysrest/lsysrest"
	"net/http"
	"rest_client"
	"testing"
	"time"
)

const (
	TestApi1 = iota
)

type DomeApiClient struct{}

func (res *DomeApiClient) ConfigName(_ context.Context) (string, error) {
	return "dome", nil
}

// ConfigBuilds 统一配置调用接口
func (res *DomeApiClient) ConfigBuilds(_ context.Context) (map[int]rest_client.RestBuild, error) {
	return map[int]rest_client.RestBuild{
		TestApi1: &lsysrest.RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/test",
			Method:     "test1",
			Timeout:    60 * time.Second,
		},
	}, nil
}

func TestExampleServer(t *testing.T) {
	client := rest_client.NewRestClientManager()
	//应用在 http://175.178.90.181/ui/#/user/app 申请
	client.SetRestConfig(&rest_client.AppRestConfig{
		Name:      "dome",
		AppKey:    "llllll",
		AppSecret: "1dbe06c064a5d382eb17c2a54f8f9739",
		AppUrl:    "http://127.0.0.1:8080",
	})
	rest := client.NewApi(&DomeApiClient{})
	data := <-rest.Do(context.Background(), TestApi1, map[string]string{
		"a": "bb",
	})
	fmt.Printf("sub app output :%s", data.JsonResult().GetData("").String())
}
