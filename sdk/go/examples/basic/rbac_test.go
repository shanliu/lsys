package main

import (
	"context"
	"fmt"
	"testing"
)

//权限校验示例

type UserPageCheckRes struct {
}

func (res UserPageCheckRes) ToRbacRes(_ context.Context) [][]map[string]interface{} {
	return [][]map[string]interface{}{{
		map[string]interface{}{
			"res_type":     "user-page", //资源key
			"res_data":     "",          //资源key
			"use_app_user": "0",         //是否使用APP所属的用户,为1时 user_param 被忽略
			"user_param":   "ccc",       //资源用户ID
			"ops": []map[string]string{
				{
					"op_key":   "xx", //操作标识
					"req_auth": "1",  //无符合权限配置时,是否通过授权,为true时,无匹配授权将返回授权失败
				},
			}, //必须权限
		},
	}}
}

type UserPageCheckRelation struct {
}

func (res UserPageCheckRelation) ToCheckRelation(_ context.Context) []map[string]interface{} {
	return []map[string]interface{}{
		{
			"role_key":     "friend", //关系角色KEY
			"user_param":   "ccc",    //关系角色属于用户标识
			"use_app_user": "0",      //是否使用APP所属的用户,为1时 user_param 被忽略
		},
	}
}

func TestRbacCheck(t *testing.T) {
	sysApi := GetRestApi()

	//示例1
	//校验权限
	err1 := sysApi.RbacCheck(
		context.Background(),
		"sss",
		&UserPageCheckRelation{},
		&UserPageCheckRes{},
	)
	if err1 == nil {
		fmt.Printf("ok\n")
	} else {
		fmt.Printf("err :%s \n", err1)
	}
}
