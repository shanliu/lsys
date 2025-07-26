package lib

import (
	"encoding/json"
	"io"
	"sub_app/app/service"
	"time"

	"github.com/gin-gonic/gin"
)

// 基于GIN的 LSYS REST接口实现

// RestRequest 接口请求
type RestRequest struct {
	RequestIp string
	RequestId string
	Method    string
	AppKey    string
	Content   string
	Token     string
	Version   string
	Timestamp time.Time
	AppInfo   *service.AppInfoResult
}

// JsonResponse 接口返回
type JsonResponse struct {
	RequestId     string
	ResultCode    string
	ResultMessage string
	ResultState   string
	Data          interface{}
}

// setCode 设置返回状态码
func (res *JsonResponse) setCode(code string) *JsonResponse {
	res.ResultCode = code
	return res
}

// SetState 设置返回状态
func (res *JsonResponse) SetState(code string) *JsonResponse {
	res.ResultState = code
	return res
}

// SetMessage 设置返回消息
func (res *JsonResponse) SetMessage(msg string) *JsonResponse {
	res.ResultMessage = msg
	return res
}

// SetData 设置返回数据
func (res *JsonResponse) SetData(data interface{}) *JsonResponse {
	res.Data = data
	return res
}

// SetPageData 设置返回带页码数据
func (res *JsonResponse) SetPageData(total int, data interface{}) *JsonResponse {
	res.Data = map[string]interface{}{
		"data":  data,
		"total": total,
	}
	return res
}

// RenderData 渲染返回数据
func (res *JsonResponse) RenderData() []byte {
	data, err := json.Marshal(map[string]interface{}{
		"result": map[string]string{
			"code":    res.ResultCode,
			"message": res.ResultMessage,
			"state":   res.ResultState,
		},
		"response": res.Data,
	})
	if err != nil {
		return []byte("{\"error\":\"" + err.Error() + "\"}")
	}
	return data
}

// RenderOutput 渲染并输出返回数据
func (res *JsonResponse) RenderOutput(ctx *gin.Context) {
	ctx.Data(200, "application/json; charset=utf-8", res.RenderData())
}

// RestJsonResponse 默认 JsonResponse
func RestJsonResponse() *JsonResponse {
	return &JsonResponse{
		ResultCode:    "200",
		ResultMessage: "ok",
		ResultState:   "ok",
	}
}

// 这里实现一个跟 lsys 系统一样的签名验证
// !!!!! 你完全可以根据自己实际需要,实现你的签名验证方式
func RestCheckSign(ctx *gin.Context) (*RestRequest, *JsonResponse) {
	param := map[string]string{
		"client_id": "",
		"version":   "",
		"timestamp": "",
	}
	sign, find := ctx.GetQuery("sign")
	if !find {
		return nil, RestJsonResponse().setCode("400").SetState("miss_sign").SetMessage("not find sign param")
	}
	for key := range param {
		if value, find := ctx.GetQuery(key); find {
			param[key] = value
		} else {
			return nil, RestJsonResponse().setCode("400").SetState("miss_param").SetMessage("request miss param:" + key)
		}
	}
	timestamp, err := time.Parse("2006-01-02 15:04:05", param["timestamp"])
	if err != nil {
		return nil, RestJsonResponse().setCode("400").SetState("miss_param").SetMessage("request timestamp format error :" + err.Error())
	}
	if token, find := ctx.GetQuery("token"); find {
		param["token"] = token
	} else {
		param["token"] = ""
	}
	if method, find := ctx.GetQuery("method"); find {
		param["method"] = method
	} else {
		param["method"] = ""
	}
	if requestIp, find := ctx.GetQuery("request_ip"); find {
		param["request_ip"] = requestIp
	} else {
		param["token"] = ""
	}
	requestId := ctx.Request.Header.Get("X-Request-ID")
	var payload string
	if ctx.Request.Header.Get("Content-type") == "application/json" {
		data, err := io.ReadAll(ctx.Request.Body)
		if err != nil {
			return nil, RestJsonResponse().setCode("400").SetState("app_body_wrong").SetMessage(err.Error())
		} else {
			payload = string(data)
		}
	} else {
		if pl, find := ctx.GetQuery("payload"); find {
			payload = pl
		}
	}
	appData, err := service.GetAppInfo(ctx, param["client_id"])
	if err != nil {
		return nil, RestJsonResponse().setCode("500").SetState("system_error").SetMessage(err.Error())
	}
	// 可以拿APP秘钥,自己实现一个签名方法
	//一个APP可能会同时存在多个可用的 Secret
	// for _, secretKey := range appData.GetSecretData() {
	// 	//分别用 secretKey 参与你的签名验证,任意一个成功为成功
	// }
	//这里实现一个跟LSYS系统一致的签名验证方式,建议跟你的实际需要,自行实现
	if appData.RestParamSignCheck(
		param["version"],
		param["client_id"],
		param["method"],
		param["timestamp"],
		param["request_ip"],
		param["token"],
		payload,
		sign,
	) {
		return nil, RestJsonResponse().setCode("401").SetState("sign_error").SetMessage("your submit param sign wrong")
	}
	return &RestRequest{
		RequestId: requestId,
		RequestIp: param["request_ip"],
		AppInfo:   appData,
		Method:    param["method"],
		AppKey:    param["client_id"],
		Timestamp: timestamp,
		Content:   param["content"],
		Token:     param["token"],
		Version:   param["version"],
	}, RestJsonResponse()
}
