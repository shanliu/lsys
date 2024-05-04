package main

import (
	"context"
	"fmt"
	lSysApi "lsysrest/lsysrest"
	"testing"
)

// 获取子应用数据,用于外部系统校验

func TestGetSubAppInfo(t *testing.T) {
	sysApi := lSysApi.NewRestApi(&lSysApi.RestApiConfig{
		//应用在 https://www.lsys.site/app.html#/user/app 申请
		AppId:          "1212f",                            //应用ID
		AppSecret:      "3f95638a1e07b87df2b64e09c2541dac", //应用Secret
		AppHost:        "http://www.lsys.site",               //应用HOST
		AppOAuthSecret: "2a97bf1b4f075b0ca7467e7c6b223f89", //应用OauthSecret
		AppOAuthHost:   "http://www.lsys.site/oauth.html",
	})

	//示例1
	//app信息获取
	err1, data := sysApi.SubAppInfo(context.Background(), "afsd")
	if err1 == nil {
		fmt.Printf("token :%s \n", data)
	} else {
		fmt.Printf("err :%s \n", err1)
	}
}
