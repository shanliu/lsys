package main

import (
	"context"
	"fmt"
	"testing"
)

// 邮件发送示例

func TestMail(t *testing.T) {
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
	//邮件发送示例
	for i := 0; i < 1000; i++ {
		var mail []string
		for j := 0; j < 100; j++ {
			mail = append(mail, fmt.Sprintf("shan.liu@msn%d.com", j))
		}
		data, err1 := sysApi.MailSend(
			context.Background(),
			mail,
			"dddd",
			map[string]string{
				"code": "sss",
			},
			"",                 //非必须 例:2023-12-11 11:11:11
			"shan.liu@msn.com", //回复邮箱
			1,
		)
		if err1 == nil {
			println("send OK")
			//取消发送
			res2, err2 := sysApi.MailCancel(context.Background(), []string{data[0].Id})
			if err2 == nil {
				println(res2)
			} else {
				fmt.Printf("err :%s", err2.Error())
			}
		} else {
			fmt.Printf("err :%s \n", err1.Error())
		}
	}

}
