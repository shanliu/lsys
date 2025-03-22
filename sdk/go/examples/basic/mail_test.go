package main

import (
	"context"
	"fmt"
	"testing"
)

// 邮件发送示例

func TestMail(t *testing.T) {
	sysApi := GetRestApi()
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
			"", //非必须 例:2023-12-11 11:11:11
			"", //回复邮箱
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
