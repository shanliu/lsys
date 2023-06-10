package lsysrest

import (
	"context"
	"crypto/md5"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/url"
	"rest_client"
	"sort"
	"strings"
	"time"

	"github.com/tidwall/gjson"
)

// RestClientConfig 回收宝内部服务配置
type RestClientConfig struct {
	Name        string
	AppKey      string
	AppSecret   string
	AppUrl      string
	EventCreate func(ctx context.Context) rest_client.RestEvent
}

func (clf *RestClientConfig) GetName() string {
	return clf.Name
}

type RestClientError struct {
	Msg     string
	Code    string
	SubCode string
}

func (err *RestClientError) Error() string {
	return fmt.Sprintf("%s [%s]", err.Msg, err.Code)
}

// NewRestClientError  错误创建
func NewRestClientError(code string, subCode string, msg string) *RestClientError {
	return &RestClientError{
		Code:    code,
		Msg:     msg,
		SubCode: subCode,
	}
}

// RestClientBuild 内部接口配置
type RestClientBuild struct {
	Timeout    time.Duration //指定接口超时时间,默认0,跟全局一致
	Path       string        //接口路径
	HttpMethod string        //http请求类型
	Payload    string        //接口参数传递方式,GET POST
	Method     string        //接口方法

}

// RestRequestId 新增请求header的x-request-id
type RestRequestId interface {
	rest_client.RestApi
	RequestId(ctx context.Context) string
}

// RestRequestIp 客户端请求IP
type RestRequestIp interface {
	rest_client.RestApi
	RequestIp(ctx context.Context) string
}

// RestParamSign 参数签名生成
func RestParamSign(version, appKey, method, timestamp, appSecret, requestIp, token, body string) string {
	reqParam := map[string]string{
		"app_id":    appKey,
		"version":   version,
		"timestamp": timestamp,
	}
	if len(method) > 0 {
		reqParam["method"] = method
	}
	if len(requestIp) > 0 {
		reqParam["request_ip"] = requestIp
	}
	if len(token) > 0 {
		reqParam["token"] = token
	}
	var keys []string
	for k := range reqParam {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	data := url.Values{}
	for _, key := range keys {
		data.Set(key, reqParam[key])
	}
	reqData := data.Encode()
	sigStr := reqData + body + appSecret
	dataSign := md5.Sum([]byte(sigStr))
	return fmt.Sprintf("%x", dataSign)
}

// BuildRequest 执行请求
func (clt *RestClientBuild) BuildRequest(ctx context.Context, client *rest_client.RestClient, _ int, param interface{}, _ *rest_client.RestCallerInfo) *rest_client.RestResult {
	tConfig, err := client.GetConfig(ctx)
	if err != nil {
		return rest_client.NewRestResultFromError(err, &rest_client.RestEventNoop{})
	}
	config, ok := tConfig.(*RestClientConfig)
	if !ok {
		return rest_client.NewRestResultFromError(NewRestClientError("11", "bad", "build config is wrong"), &rest_client.RestEventNoop{})
	}

	var event rest_client.RestEvent
	if config.EventCreate != nil {
		event = config.EventCreate(ctx)
	} else {
		event = &rest_client.RestEventNoop{}
	}

	transport := client.GetTransport()
	headerTime := transport.ResponseHeaderTimeout
	apiUrl := config.AppUrl
	appid := config.AppKey
	keyConfig := config.AppSecret

	var jsonData string
	if param == nil {
		jsonData = ""
	} else {
		jsonParam, err := json.Marshal(param)
		if err != nil {
			return rest_client.NewRestResultFromError(err, event)
		}
		jsonData = string(jsonParam)
	}
	if clt.Payload != http.MethodGet && len(jsonData) == 0 {
		jsonData = "{}"
	}

	var token string
	if token_, find := client.Api.(rest_client.RestTokenApi); find {
		tokenTmp, err := token_.Token(ctx)
		if err != nil {
			return rest_client.NewRestResultFromError(err, event)
		}
		token = tokenTmp
	}
	var reqIp string
	if rid, find := client.Api.(RestRequestId); find {
		reqIp = rid.RequestId(ctx)
	} else {
		address, err := net.InterfaceAddrs()
		if err == nil {
			for _, addr := range address {
				if ipNet, ok := addr.(*net.IPNet); ok && !ipNet.IP.IsLoopback() {
					reqIp = ipNet.IP.String()
					break
				}
			}
		}
	}

	timestamp := time.Now().Format("2006-01-02 15:04:05")
	dataSign := RestParamSign("2.0", appid, clt.Method, timestamp, keyConfig, reqIp, token, jsonData)
	reqParam := map[string]string{
		"app_id":    appid,
		"version":   "2.0",
		"timestamp": timestamp,
		"sign":      dataSign,
	}
	if len(token) > 0 {
		reqParam["token"] = token
	}
	if len(clt.Method) > 0 {
		reqParam["method"] = clt.Method
	}
	if len(reqIp) > 0 {
		reqParam["request_ip"] = reqIp
	}
	if clt.Payload == http.MethodGet {
		if len(jsonData) > 0 {
			reqParam["payload"] = jsonData
		}
	}
	pData := url.Values{}
	for key, val := range reqParam {
		pData.Set(key, val)
	}
	paramStr := pData.Encode()
	apiUrl += clt.Path
	if !strings.Contains(apiUrl, "?") {
		apiUrl += "?" + paramStr
	} else {
		apiUrl += "&" + paramStr
	}
	var ioRead io.Reader
	if clt.Payload != http.MethodGet {
		ioRead = rest_client.NewRestRequestReader(strings.NewReader(jsonData), event)
	}
	event.RequestStart(clt.HttpMethod, apiUrl)
	var req *http.Request
	req, err = http.NewRequest(clt.HttpMethod, apiUrl, ioRead)

	if rid, find := client.Api.(RestRequestId); find {
		tmp := rid.RequestId(ctx)
		req.Header["X-Request-ID"] = []string{tmp}
	}

	if clt.Payload != http.MethodGet {
		req.Header.Set("Content-Type", "application/json")
	}
	if err != nil {
		return rest_client.NewRestResultFromError(err, event)
	}

	if clt.Timeout > 0 {
		transport.ResponseHeaderTimeout = clt.Timeout
	}
	httpClient := &http.Client{
		Transport: transport,
	}
	res, err := httpClient.Do(req)
	if clt.Timeout > 0 {
		transport.ResponseHeaderTimeout = headerTime
	}
	if err != nil {
		return rest_client.NewRestResultFromError(err, event)
	} else {
		return rest_client.NewRestResult(clt, res, event)
	}
}

func (clt *RestClientBuild) CheckJsonResult(body string) error {
	code := gjson.Get(body, "result.code").String()
	state := gjson.Get(body, "result.state").String()
	if code != "200" || state != "ok" {
		msg := gjson.Get(body, "result.message").String()
		if len(msg) == 0 {
			msg = body
		}
		return NewRestClientError(code, gjson.Get(body, "result.state").String(), "server return fail:"+msg)
	}
	return nil
}
