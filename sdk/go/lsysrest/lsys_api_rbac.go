package lsysrest

import (
	"context"
)

// CheckRes 资源接口
type CheckRes interface {
	ToRbacRes(ctx context.Context) [][]map[string]interface{}
}

// CheckRelation 校验权限关系
type CheckRelation interface {
	ToCheckRelation(ctx context.Context) []map[string]interface{}
}
type EmptyCheckRelation struct {
}

func (EmptyCheckRelation) ToCheckRelation(_ context.Context) []map[string]interface{} {
	return make([]map[string]interface{}, 0)
}

// RbacCheck 权限校验
func (receiver *RestApi) RbacCheck(ctx context.Context, userId int, relation CheckRelation, checkRes CheckRes) error {
	data1 := (<-receiver.rest.Do(ctx, AccessCheck, map[string]interface{}{
		"user_id": userId,
		"access": map[string]interface{}{
			"relation_key": relation.ToCheckRelation(ctx),
			"check_res":    checkRes.ToRbacRes(ctx),
		},
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
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
