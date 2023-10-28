package main

import (
	"context"
	"fmt"
	lSysApi "lsysrest/lsysrest"
	"testing"
)

func TestApp(t *testing.T) {
	sysApi := lSysApi.NewRestApi(&lSysApi.RestApiConfig{
		//应用在 http://www.lsys.cc/ui/#/user/app 申请
		AppId:          "1212f",                            //应用ID
		AppSecret:      "3f95638a1e07b87df2b64e09c2541dac", //应用Secret
		AppHost:        "http://www.lsys.cc",               //应用HOST
		AppOAuthSecret: "2a97bf1b4f075b0ca7467e7c6b223f89", //应用OauthSecret
		AppOAuthHost:   "http://www.lsys.cc/ui/oauth.html",
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
