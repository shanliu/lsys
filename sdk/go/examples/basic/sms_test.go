package main

import (
	"context"
	"fmt"
	"testing"
)

// 短信发送示例

func TestSms(t *testing.T) {
<<<<<<< HEAD
	sysApi := lSysApi.NewRestApi(&lSysApi.RestApiConfig{
		//应用在 https://www.lsys.cc/app.html#/user/app 申请
		AppId:          "1212f",                            //应用ID
		AppSecret:      "3f95638a1e07b87df2b64e09c2541dac", //应用Secret
		AppHost:        "http://www.lsys.cc",               //应用HOST
		AppOAuthSecret: "2a97bf1b4f075b0ca7467e7c6b223f89", //应用OauthSecret
		AppOAuthHost:   "http://www.lsys.cc/oauth.html",
	})
=======
	sysApi := GetRestApi()
>>>>>>> dev

	//示例1
	//短信发送示例
	data, err1 := sysApi.SmsSend(
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
		res2, err2 := sysApi.SmsCancel(context.Background(), []string{data[0].Id})
		if err2 == nil {
			println(res2)
		} else {
			fmt.Printf("err :%s \n", err2.Error())
		}
		fmt.Printf("ok\n")
	} else {
		fmt.Printf("err :%s \n", err1.Error())
	}

}
