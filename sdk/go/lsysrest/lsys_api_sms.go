package lsysrest

import (
	"context"
	"encoding/json"
)

// SmsSend 发送短信
// mobile 接受手机号,批量
// tplId 模板key
// tplData 短信内容变量
// sendTime 发送时间,小于当前时间或空立即发送
// cancelKey 取消句柄,不需要取消为空
func (receiver *RestApi) SmsSend(ctx context.Context, mobile []string, tplId string, tplData map[string]string, sendTime string, cancelKey string) error {
	data, err := json.Marshal(tplData)
	if err != nil {
		return err
	}
	data1 := (<-receiver.rest.Do(ctx, SmeSend, map[string]interface{}{
		"mobile":    mobile,
		"tpl":       tplId,
		"data":      string(data),
		"cancel":    cancelKey,
		"send_time": sendTime,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}

// SmsCancel 取消发送
// cancelKey 取消句柄,发送时设置
func (receiver *RestApi) SmsCancel(ctx context.Context, cancelKey string) error {
	data1 := (<-receiver.rest.Do(ctx, SmeCancel, map[string]interface{}{
		"cancel": cancelKey,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}
