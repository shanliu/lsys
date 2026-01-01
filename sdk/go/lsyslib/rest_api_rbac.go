package lsyslib

import (
	"context"
	"net/http"
	"rest_client"
	"time"
)

const (
	AccessCheck = 400
)

func init() {
	RestApiClientSetConfig(map[int]rest_client.RestBuild{
		AccessCheck: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/base",
			Method:     "access",
			Timeout:    60 * time.Second,
		},
	})
}

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
func (receiver *LsysApi) RbacCheck(ctx context.Context, userId string, relation CheckRelation, checkRes CheckRes) error {
	data1 := (<-receiver.rest.Do(ctx, AccessCheck, map[string]interface{}{
		"user_param": userId,
		"token_data": nil,
		"request_ip": "1.1.0.1",
		"access": map[string]interface{}{
			"role_key":  relation.ToCheckRelation(ctx),
			"check_res": checkRes.ToRbacRes(ctx),
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
