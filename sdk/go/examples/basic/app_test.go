package main

import (
	"context"
	"fmt"
	lSysApi "lsysrest/lsysrest"
	"testing"
)

func TestApp(t *testing.T) {
	sysApi := lSysApi.NewRestApi(&lSysApi.RestApiConfig{
		//应用在 http://175.178.90.181/ui/#/user/app 申请
		AppId:          "1212f",                            //应用ID
		AppSecret:      "3f95638a1e07b87df2b64e09c2541dac", //应用Secret
		AppHost:        "http://175.178.90.181",            //应用HOST
		AppOAuthSecret: "2a97bf1b4f075b0ca7467e7c6b223f89", //应用OauthSecret
		AppOAuthHost:   "http://175.178.90.181/ui/oauth.html",
	})

	//示例1
	//app信息获取
	err1, data := sysApi.AppInfo(context.Background(), "1212f")
	if err1 == nil {
		fmt.Printf("token :%s \n", data)
	} else {
		fmt.Printf("err :%s \n", err1)
	}
}
