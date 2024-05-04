package main

import (
	"context"
	"fmt"
	lSysApi "lsysrest/lsysrest"
	"testing"
)

//权限校验示例

type UserPageCheckRes struct {
}

func (res UserPageCheckRes) ToRbacRes(_ context.Context) [][]map[string]interface{} {
	return [][]map[string]interface{}{{
		map[string]interface{}{
			"res":        "user-page",              //资源key
			"user_id":    1,                        //资源用户ID
			"ops":        []string{"view", "edit"}, //必须权限
			"option_ops": []string{"del"},          //可选权限
		},
	}}
}

type UserPageCheckRelation struct {
}

func (res UserPageCheckRelation) ToCheckRelation(_ context.Context) []map[string]interface{} {
	return []map[string]interface{}{
		{
			"role_key": "friend", //关系角色KEY
			"user_id":  1,        //关系角色属于用户ID,资源用户ID或为0
		},
	}
}

func TestRbac(t *testing.T) {
	sysApi := lSysApi.NewRestApi(&lSysApi.RestApiConfig{
		//应用在 https://www.lsys.site/app.html#/user/app 申请
		AppId:          "1212f",                            //应用ID
		AppSecret:      "3f95638a1e07b87df2b64e09c2541dac", //应用Secret
		AppHost:        "http://www.lsys.site",               //应用HOST
		AppOAuthSecret: "2a97bf1b4f075b0ca7467e7c6b223f89", //应用OauthSecret
		AppOAuthHost:   "http://www.lsys.site/oauth.html",
	})

	//示例1
	//校验权限
	err1 := sysApi.RbacCheck(
		context.Background(),
		100,
		&UserPageCheckRelation{},
		&UserPageCheckRes{},
	)
	if err1 == nil {
		fmt.Printf("ok\n")
	} else {
		fmt.Printf("err :%s \n", err1)
	}
}
