package main

import (
	"context"
	"fmt"
	"lsysrest/lsyslib"
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

func (res *DomeApiClient) ConfigBuilds(_ context.Context) (map[int]rest_client.RestBuild, error) {
	return map[int]rest_client.RestBuild{
		TestApi1: &lsyslib.RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest-api",
			Method:     "test1",
			Timeout:    60 * time.Second,
		},
	}, nil
}

func TestExampleClient(t *testing.T) {
	client := rest_client.NewRestClientManager()
	//先 https://www.lsys.cc/app.html#/user/app 申请本系统的子应用
	// AppKey: 为本系统的子应用的AppID
	// AppSecret: 点击申请应用右边的 `关联应用授权` 图标, 找到需要 `被授权应用` -> 点击 `未设置授权+` 设置自定义 AppSecret
	// 示例参见: https://www.lsys.cc/app.html#/user/app?client_id=Sapp00122
	client.SetRestConfig(
		&lsyslib.RestClientConfig{
			Name:      "dome",
			AppKey:    "Sapp00122",
			AppSecret: "63cb085da700612c957d8b76a8b81730",
			AppUrl:    "http://127.0.0.1:8081",
		})
	rest := client.NewApi(&DomeApiClient{})
	data := <-rest.Do(context.Background(), TestApi1, map[string]string{
		"a": "bb",
	})
	res := data.JsonResult()
	fmt.Printf("sub app output :%s", res.GetData("").String())
}
