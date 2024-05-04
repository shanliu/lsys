package main

import (
	"context"
	"fmt"
	lSysApi "lsysrest/lsysrest"
	"testing"
)

// 短信发送示例

func TestSms(t *testing.T) {
	sysApi := lSysApi.NewRestApi(&lSysApi.RestApiConfig{
		//应用在 https://www.lsys.site/app.html#/user/app 申请
		AppId:          "1212f",                            //应用ID
		AppSecret:      "3f95638a1e07b87df2b64e09c2541dac", //应用Secret
		AppHost:        "http://www.lsys.site",               //应用HOST
		AppOAuthSecret: "2a97bf1b4f075b0ca7467e7c6b223f89", //应用OauthSecret
		AppOAuthHost:   "http://www.lsys.site/oauth.html",
	})

	//示例1
	//短信发送示例
	err1, data := sysApi.SmsSend(
		context.Background(),
		[]string{
			"13800138000",
		},
		"dddd",
		map[string]string{
			"code": "sss",
		},
		"", //非必须 例:2023-12-11 11:11:11
		1,
	)
	if err1 == nil {
		println("send ok")
		//取消发送
		err2, res2 := sysApi.SmsCancel(context.Background(), []string{data[0].Id})
		if err2 == nil {
			println(res2)
		} else {
			fmt.Printf("err :%s \n", err1.Error())
		}
		fmt.Printf("ok\n")
	} else {
		fmt.Printf("err :%s \n", err1.Error())
	}

}
