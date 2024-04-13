package lsysrest

import (
	"context"
	"rest_client"
)

// SubAppInfo 应用信息
func (receiver *RestApi) SubAppInfo(ctx context.Context, appId string) (error, *rest_client.JsonData) {
	data1 := (<-receiver.rest.Do(ctx, SubAppInfo, map[string]interface{}{
		"client_id": appId,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err(), nil
	}
	return nil, data1.GetData("response")
}
