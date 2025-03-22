package lib

import (
	"encoding/json"
	"io/ioutil"
	"lsysrest/lsysrest"
	"rest_client"
	"sync"
	"time"

	"github.com/gin-gonic/gin"
)

// 基于GIN的 REST接口实现

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
	AppInfo   *rest_client.JsonData
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

type appInfoItem struct {
	appData *rest_client.JsonData
	timeout time.Time
}

type appInfoCache struct {
	appData map[string]*appInfoItem
	lock    sync.RWMutex
}

var appInfoCacheData appInfoCache
var appInfoCacheTime time.Duration

func init() {
	//app key 缓存时间
	appInfoCacheTime = time.Second * 60
	//app key 缓存数据
	appInfoCacheData = appInfoCache{
		appData: map[string]*appInfoItem{},
		lock:    sync.RWMutex{},
	}
}

// RestCheckSign 检查请求签名
func RestCheckSign(ctx *gin.Context, restApi *lsysrest.RestApi) (*RestRequest, *JsonResponse) {
	param := map[string]string{
		"app_id":    "",
		"version":   "",
		"timestamp": "",
	}
	sign, find := ctx.GetQuery("sign")
	if !find {
		return nil, RestJsonResponse().SetState("miss_sign").SetMessage("not find sign param")
	}
	for key := range param {
		if value, find := ctx.GetQuery(key); find {
			param[key] = value
		} else {
			return nil, RestJsonResponse().SetState("miss_param").SetMessage("request miss param:" + key)
		}
	}
	timestamp, err := time.Parse("2006-01-02 15:04:05", param["timestamp"])
	if err != nil {
		return nil, RestJsonResponse().SetState("miss_param").SetMessage("request timestamp format error :" + err.Error())
	}

	var appInfo *rest_client.JsonData
	appInfoCacheData.lock.RLock()
	appId := param["app_id"]
	if tmp, ok := appInfoCacheData.appData[appId]; ok {
		if tmp.timeout.After(time.Now()) {
			appInfo = tmp.appData
		}
	}
	appInfoCacheData.lock.RUnlock()
	if appInfo == nil {
		appInfoCacheData.lock.Lock()
		defer appInfoCacheData.lock.Unlock()
		appInfo, err = restApi.SubAppInfo(ctx, param["app_id"])
		if err != nil {
			return nil, RestJsonResponse().SetState("app_error").SetMessage(err.Error())
		}
		appInfoCacheData.appData[appId] = &appInfoItem{
			appData: appInfo,
			timeout: time.Now().Add(appInfoCacheTime),
		}
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
		data, err := ioutil.ReadAll(ctx.Request.Body)
		if err != nil {
			return nil, RestJsonResponse().SetState("app_body_wrong").SetMessage(err.Error())
		} else {
			payload = string(data)
		}
	} else {
		if pl, find := ctx.GetQuery("payload"); find {
			payload = pl
		}
	}
	secret := appInfo.Get("sub_secret").String()
	if len(secret) == 0 {
		return nil, RestJsonResponse().SetState("app_body_wrong").SetMessage("app secret is empty or not config")
	}
	RSign := lsysrest.RestParamSign(
		param["version"],
		param["app_id"],
		param["method"],
		param["timestamp"],
		secret,
		param["request_ip"],
		param["token"],
		payload)
	if sign != RSign {
		return nil, RestJsonResponse().SetState("app_error").SetMessage("your submit param sign wrong")
	}
	return &RestRequest{
		RequestId: requestId,
		RequestIp: param["request_ip"],
		AppInfo:   appInfo,
		Method:    param["method"],
		AppKey:    param["app_id"],
		Timestamp: timestamp,
		Content:   param["content"],
		Token:     param["token"],
		Version:   param["version"],
	}, RestJsonResponse()
}
