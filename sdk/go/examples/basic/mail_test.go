package main

import (
	"context"
	"fmt"
	lSysApi "lsysrest/lsysrest"
	"testing"
)

func TestMail(t *testing.T) {
	sysApi := lSysApi.NewRestApi(&lSysApi.RestApiConfig{
		//应用在 http://www.lsys.cc/ui/#/user/app 申请
		AppId:          "1212f",                            //应用ID
		AppSecret:      "3f95638a1e07b87df2b64e09c2541dac", //应用Secret
		AppHost:        "http://www.lsys.cc",               //应用HOST
		AppOAuthSecret: "2a97bf1b4f075b0ca7467e7c6b223f89", //应用OauthSecret
		AppOAuthHost:   "http://www.lsys.cc/ui/oauth.html",
	})
	//示例1
	//邮件发送示例
	for i := 0; i < 1000; i++ {

		err1 := sysApi.MailSend(
			context.Background(),
			[]string{
				fmt.Sprintf("shan.liu@msn%d.com", i),
			},
			"dddd",
			map[string]string{
				"code": "sss",
			},
			"", //非必须 例:2023-12-11 11:11:11
			"", //回复邮箱
			"", //取消句柄,取消发送用 例:dddd
		)
		if err1 == nil {
			fmt.Printf("ok\n")
		} else {
			fmt.Printf("err :%s \n", err1)
		}

	}
	//取消发送
	//err1 = sysApi.MailCancel(context.Background(), "dddd")
	//if err1 == nil {
	//	fmt.Printf("ok")
	//} else {
	//	fmt.Printf("err :%s \n", err1)
	//}
}
