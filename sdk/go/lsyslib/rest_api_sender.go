package lsyslib

import (
	"context"
	"net/http"
	"rest_client"
	"time"
)

const (
	SmeSend    = 500
	SmeCancel  = 501
	MailSend   = 502
	MailCancel = 503
)

func init() {
	RestApiClientSetConfig(map[int]rest_client.RestBuild{
		SmeSend: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/sms",
			Method:     "send",
			Timeout:    60 * time.Second,
		},
		SmeCancel: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/sms",
			Method:     "cancel",
			Timeout:    60 * time.Second,
		},
		MailSend: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/mail",
			Method:     "send",
			Timeout:    60 * time.Second,
		},
		MailCancel: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/mail",
			Method:     "cancel",
			Timeout:    60 * time.Second,
		},
	})
}

type SmsSendResult struct {
	Mobile string
	Id     string
}

// SmsSend 发送短信
// mobile 接收手机号,批量
// tplId 模板key
// tplData 短信内容变量
// sendTime 发送时间,小于当前时间或空立即发送
// max_try  发送尝试次数
func (receiver *LsysApi) SmsSend(ctx context.Context, mobile []string, tplId string, tplData map[string]string, sendTime string, maxTry uint8) ([]SmsSendResult, error) {
	data1 := (<-receiver.rest.Do(ctx, SmeSend, map[string]interface{}{
		"mobile":    mobile,
		"tpl_key":   tplId,
		"data":      tplData,
		"send_time": sendTime,
		"max_try":   maxTry,
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	var out []SmsSendResult
	for _, tmp := range data1.GetData("response.detail").Array() {
		out = append(out, SmsSendResult{
			Mobile: tmp.Get("mobile").String(),
			Id:     tmp.Get("snid").String(),
		})
	}
	return out, nil
}

type SmsCancelResult struct {
	Msg    string
	Status bool
	Id     string
}

// SmsCancel 取消发送
// cancelKey 取消句柄,发送时设置
func (receiver *LsysApi) SmsCancel(ctx context.Context, cancelKeys []string) ([]SmsCancelResult, error) {
	data1 := (<-receiver.rest.Do(ctx, SmeCancel, map[string]interface{}{
		"snid_data": cancelKeys,
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	var out []SmsCancelResult
	for _, tmp := range data1.GetData("response.detail").Array() {
		out = append(out, SmsCancelResult{
			Status: tmp.Get("status").Bool(),
			Msg:    tmp.Get("msg").String(),
			Id:     tmp.Get("snid").String(),
		})
	}
	return out, nil
}

type MailSendResult struct {
	Mail string
	Id   string
}

// MailSend 发送邮件
// to 接收邮箱,批量
// tplId 模板key
// tplData 短信内容变量
// sendTime 发送时间,小于当前时间或空立即发送
// reply 回复邮件地址.不需要留空
// max_try  发送尝试次数
func (receiver *LsysApi) MailSend(ctx context.Context, to []string, tplId string, tplData map[string]string, sendTime string, reply string, maxTry uint8) ([]MailSendResult, error) {
	data1 := (<-receiver.rest.Do(ctx, MailSend, map[string]interface{}{
		"to":        to,
		"tpl_key":   tplId,
		"data":      tplData,
		"reply":     reply,
		"send_time": sendTime,
		"max_try":   maxTry,
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	var out []MailSendResult
	for _, tmp := range data1.GetData("response.detail").Array() {
		out = append(out, MailSendResult{
			Mail: tmp.Get("mail").String(),
			Id:   tmp.Get("snid").String(),
		})
	}
	return out, nil
}

type MailCancelResult struct {
	Msg    string
	Status bool
	Id     string
}

// MailCancel 取消发送
// cancelKey 取消句柄,发送时设置
func (receiver *LsysApi) MailCancel(ctx context.Context, sendId []string) ([]MailCancelResult, error) {
	data1 := (<-receiver.rest.Do(ctx, MailCancel, map[string]interface{}{
		"snid_data": sendId,
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	var out []MailCancelResult
	for _, tmp := range data1.GetData("response.detail").Array() {
		out = append(out, MailCancelResult{
			Status: tmp.Get("status").Bool(),
			Msg:    tmp.Get("msg").String(),
			Id:     tmp.Get("id").String(),
		})
	}
	return out, nil
}
