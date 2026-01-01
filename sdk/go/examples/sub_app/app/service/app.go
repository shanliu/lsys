package service

import (
	"context"
	"encoding/json"
	"lsysrest/lsyslib"
	"rest_client"
	"sync"
	"time"
)

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

type AppInfoResult struct {
	Name       string `json:"name"`
	SecretData []struct {
		Secret  string      `json:"secret_data"`
		TimeOut json.Number `json:"time_out"`
	} `json:"sub_secret"`
}

// 获取应用所有可用的 Secret
func (appinfo *AppInfoResult) GetSecretData() []string {
	var out []string
	for _, item := range appinfo.SecretData {
		t, _ := item.TimeOut.Int64()
		if t == 0 || t > time.Now().Unix() {
			out = append(out, item.Secret)
		}
	}
	return out
}

// 使用跟 lsys 系统一直的签名校验方式进行验签
// 你可以不调用这个函数,自己实现你系统签名验证方式
func (appinfo *AppInfoResult) RestParamSignCheck(
	version, clientId, method, timestamp, requestIp, token, payload, sign string,
) bool {
	for _, secret := range appinfo.GetSecretData() {
		RSign := lsyslib.RestParamSign(
			version,
			clientId,
			method,
			timestamp,
			secret,
			requestIp,
			token,
			payload)
		if sign != RSign {
			return true
		}
	}
	return false
}

// RestCheckSign 检查请求签名
func GetAppInfo(ctx context.Context, clientId string) (*AppInfoResult, error) {
	restApi := GetRestApi()
	var appInfo *rest_client.JsonData
	appInfoCacheData.lock.RLock()
	if tmp, ok := appInfoCacheData.appData[clientId]; ok {
		if tmp.timeout.After(time.Now()) {
			appInfo = tmp.appData
		}
	}
	appInfoCacheData.lock.RUnlock()
	if appInfo == nil {
		appInfoCacheData.lock.Lock()
		defer appInfoCacheData.lock.Unlock()
		var QErr error
		appInfo, QErr = restApi.SubAppInfo(ctx, clientId)
		if QErr != nil {
			return nil, QErr
		}
		appInfoCacheData.appData[clientId] = &appInfoItem{
			appData: appInfo,
			timeout: time.Now().Add(appInfoCacheTime),
		}
	}
	var secret AppInfoResult
	pErr := json.Unmarshal([]byte(appInfo.String()), &secret)
	return &secret, pErr
}
