package main

import (
	"context"
	"fmt"
	"testing"
)

// 获取子应用数据,用于外部系统校验

func TestGetSubAppInfo(t *testing.T) {
	sysApi := GetRestApi()

	//示例1
	//app信息获取
	err1, data1 := sysApi.SubAppInfo(context.Background(), "afsd")
	if err1 == nil {
		fmt.Printf("token :%s \n", data1)
	} else {
		fmt.Printf("err :%s \n", err1)
	}

}

func TestAppLogin(t *testing.T) {
	sysApi := GetRestApi()

	//app进行登录
	login_code, err2 := sysApi.AppAuthLogin(context.Background(), "user_id_1", "测试用户", nil)
	if err2 == nil {
		fmt.Printf("token :%s \n", login_code)

		//app登录信息
		data3, err3 := sysApi.AppAuthInfo(context.Background(), login_code)
		if err3 == nil {
			fmt.Printf("token :%s \n", data3)
		} else {
			fmt.Printf("err :%s \n", err3)
		}
		//app退出登录
		err4 := sysApi.AppAuthLogout(context.Background(), login_code)
		if err4 != nil {
			fmt.Printf("err :%s \n", err4)
		}

	} else {
		fmt.Printf("err :%s \n", err2)
	}

}
