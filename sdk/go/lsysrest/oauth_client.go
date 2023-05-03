package lsysrest

import (
	"context"
	"github.com/tidwall/gjson"
	"net/http"
	"net/url"
	"rest_client"
	"time"
)

// OAuthRestConfig Oauth内部服务配置
type OAuthRestConfig struct {
	Name        string
	AppKey      string
	AppSecret   string
	AppUrl      string
	EventCreate func(ctx context.Context) rest_client.RestEvent
}

func (clf *OAuthRestConfig) GetName() string {
	return clf.Name
}

// OAuthRestBuild 内部接口配置
type OAuthRestBuild struct {
	Timeout time.Duration //指定接口超时时间,默认0,跟全局一致
	Path    string        //接口路径
}

// BuildRequest 执行请求
func (clt *OAuthRestBuild) BuildRequest(ctx context.Context, client *rest_client.RestClient, _ int, param interface{}, _ *rest_client.RestCallerInfo) *rest_client.RestResult {
	tConfig, err := client.GetConfig(ctx)
	if err != nil {
		return rest_client.NewRestResultFromError(err, &rest_client.RestEventNoop{})
	}
	config, ok := tConfig.(*OAuthRestConfig)
	if !ok {
		return rest_client.NewRestResultFromError(rest_client.NewRestClientError("11", "build config is wrong"), &rest_client.RestEventNoop{})
	}

	val := url.Values{}
	if p, ok := param.(map[string]string); ok {
		for k, v := range p {
			val.Add(k, v)
		}
	} else if p, ok := param.(*map[string]string); ok {
		for k, v := range *p {
			val.Add(k, v)
		}
	}

	var event rest_client.RestEvent
	if config.EventCreate != nil {
		event = config.EventCreate(ctx)
	} else {
		event = &rest_client.RestEventNoop{}
	}

	uri, err := url.Parse(config.AppUrl + clt.Path)
	if err != nil {
		return rest_client.NewRestResultFromError(err, event)
	}
	uri.RawQuery = val.Encode()

	transport := client.GetTransport()
	headerTime := transport.ResponseHeaderTimeout
	apiUrl := uri.String()
	event.RequestStart(http.MethodGet, apiUrl)
	var req *http.Request
	req, err = http.NewRequest(http.MethodGet, apiUrl, nil)
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

func (clt *OAuthRestBuild) CheckJsonResult(body string) error {
	code := gjson.Get(body, "result.code").String()
	state := gjson.Get(body, "result.state").String()
	if code != "200" || state != "ok" {
		msg := gjson.Get(body, "result.message").String()
		if len(msg) == 0 {
			msg = body
		}
		return rest_client.NewAppClientError(code, gjson.Get(body, "result.state").String(), "server return fail:"+msg)
	}
	return nil
}
