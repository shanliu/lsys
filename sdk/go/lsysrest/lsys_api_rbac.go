package lsysrest

import (
	"context"
	"rest_client"
)

// CheckRes 资源接口
type CheckRes interface {
	ToRbacRes() [][]map[string]interface{}
}

// CheckRelation 校验权限关系
type CheckRelation struct {
	inner []map[string]interface{}
}

// RbacCheck 权限校验
func (receiver *RestApi) RbacCheck(ctx context.Context, userId int, relation *CheckRelation, checkRes CheckRes) error {
	data1 := (<-receiver.rest.Do(ctx, AccessCheck, map[string]interface{}{
		"user_id": userId,
		"access": map[string]interface{}{
			"relation_key": relation.inner,
			"check_res":    checkRes.ToRbacRes(),
		},
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	} else {
		if data1.GetData("response.pass").Int() != 1 {
			return rest_client.NewAppClientError("400", "not_pass", "access is fail")
		}
	}
	return nil
}

//
//type RbacAddProduct struct {
//	userId int
//}
//
//func (receiver *RbacAddProduct) ToRbacRes() [][]map[string]interface{} {
//	return [][]map[string]interface{}{
//		{
//			{
//				"res":     "hhh",
//				"user_id": receiver.userId,
//				"ops": []map[string]interface{}{
//					{
//						"name":      "xxx",
//						"authorize": true,
//					},
//				},
//			},
//		},
//	}
//}
