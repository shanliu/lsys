package main

import (
	"encoding/json"
	"github.com/gin-gonic/gin"
	"rest_client"
	"lsysrest/lsysrest"
	"net/http"
	"sync"
	"time"
)

// 基于GIN的 REST接口实现

// RestRequest 接口请求
type RestRequest struct {
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

// setState 设置返回状态
func (res *JsonResponse) setState(code string) *JsonResponse {
	res.ResultState = code
	return res
}

// setMessage 设置返回消息
func (res *JsonResponse) setMessage(msg string) *JsonResponse {
	res.ResultMessage = msg
	return res
}

// setData 设置返回数据
func (res *JsonResponse) setData(data interface{}) *JsonResponse {
	res.Data = data
	return res
}

// setPageData 设置返回带页码数据
func (res *JsonResponse) setPageData(total int, data interface{}) *JsonResponse {
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
		"app":       "",
		"version":   "",
		"timestamp": "",
		"content":   "",
	}
	var sign string
	var find bool
	if ctx.Request.Method == http.MethodPost && ctx.Request.Header.Get("Content-type") == "application/x-www-form-urlencoded" {
		sign, find = ctx.GetPostForm("sign")
		if !find {
			return nil, RestJsonResponse().setState("miss_param").setMessage("not find sign param")
		}
		for key := range param {
			if value, find := ctx.GetPostForm(key); find {
				param[key] = value
			} else {
				return nil, RestJsonResponse().setState("miss_param").setMessage("request miss param:" + key)
			}
		}
		if token, find := ctx.GetPostForm("token"); find {
			param["token"] = token
		}
		if method, find := ctx.GetPostForm("method"); find {
			param["method"] = method
		}
	} else {
		sign, find = ctx.GetQuery("sign")
		if !find {
			return nil, RestJsonResponse().setState("miss_param").setMessage("not find sign param")
		}
		for key := range param {
			if value, find := ctx.GetQuery(key); find {
				param[key] = value
			} else {
				return nil, RestJsonResponse().setState("miss_param").setMessage("request miss param:" + key)
			}
		}
		if token, find := ctx.GetQuery("token"); find {
			param["token"] = token
		}

		if method, find := ctx.GetPostForm("method"); find {
			param["method"] = method
		}

	}
	timestamp, err := time.Parse("2006-01-02 15:04:05", param["timestamp"])
	if err != nil {
		return nil, RestJsonResponse().setState("miss_param").setMessage("request timestamp format error :" + err.Error())
	}
	var appInfo *rest_client.JsonData
	appInfoCacheData.lock.RLock()
	appId := param["app"]
	if tmp, ok := appInfoCacheData.appData[appId]; ok {
		if tmp.timeout.After(time.Now()) {
			appInfo = tmp.appData
		}
	}
	appInfoCacheData.lock.RUnlock()
	if appInfo == nil {
		appInfoCacheData.lock.Lock()
		defer appInfoCacheData.lock.Unlock()
		err, appInfo = restApi.AppInfo(ctx, param["app"])
		if err != nil {
			return nil, RestJsonResponse().setState("app_error").setMessage(err.Error())
		}
		appInfoCacheData.appData[appId] = &appInfoItem{
			appData: appInfo,
			timeout: time.Now().Add(appInfoCacheTime),
		}
	}
	var tokenPtr *string
	if token, ok := param["token"]; ok {
		tokenPtr = &token
	}

	RSign := rest_client.AppRestParamSign(param["version"], param["app"], param["method"], param["timestamp"], param["content"], appInfo.Get("client_secret").String(), tokenPtr)
	if sign != RSign {
		return nil, RestJsonResponse().setState("app_error").setMessage("your submit param sign wrong")
	}
	return &RestRequest{
		AppInfo:   appInfo,
		Method:    param["method"],
		AppKey:    param["app"],
		Timestamp: timestamp,
		Content:   param["content"],
		Token:     param["token"],
		Version:   param["version"],
	}, RestJsonResponse()
}
