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
	sysApi := GetRestApi()

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
