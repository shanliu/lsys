package main

import (
	"context"
	"encoding/hex"
	"fmt"
	lSysApi "lsysrest/lsysrest"
	"math/rand"
	"testing"
)

// oauth 登录示例

func getApi() *lSysApi.RestApi {
	return lSysApi.NewRestApi(&lSysApi.RestApiConfig{
		//应用在 https://www.lsys.site/app.html#/user/app 申请
		AppId:          "1212f",                            //应用ID
		AppSecret:      "3f95638a1e07b87df2b64e09c2541dac", //应用Secret
		AppHost:        "http://www.lsys.site",               //应用HOST
		AppOAuthSecret: "2a97bf1b4f075b0ca7467e7c6b223f89", //应用OauthSecret
		AppOAuthHost:   "http://www.lsys.site/oauth.html",
	})
}

// 第一步
// 登录地址
func TestGetLoginUrl(t *testing.T) {
	api := getApi()
	b := make([]byte, 6)
	rand.Read(b)
	url := api.OAuthAuthorizationUrl(context.Background(), "http://127.0.0.1:8080/", "user_info,user_mobile", hex.EncodeToString(b))
	fmt.Printf("url :%s \n", url)
}

// 第二步
// 登录后获取TOKEN
func TestGetToken(t *testing.T) {
	api := getApi()
	er, token := api.OAuthAccessToken(context.Background(), "c936ed75412a0b8f")
	if er == nil {
		fmt.Printf("token :%s \n", token.AccessToken)
	} else {
		fmt.Printf("err :%s \n", er.Error())
		return
	}
}

// //通过TOKEN得到用户信息
func TestGetUserData(t *testing.T) {
	api := getApi()
	tokenApi := api.TokenRestApi("58f34acd692b70e1")
	err, data := tokenApi.OAuthUserInfo(context.Background(), true, true, false, false, false, false)
	if err == nil {
		fmt.Printf("sub app output :%s", data)
	} else {
		fmt.Printf("error :%s", err.Error())
	}
}

// //刷新TOKEN
func TestRefreshToken(t *testing.T) {
	api := getApi()
	tokenApi := api.TokenRestApi("58f34acd692b70e1")
	er, token1 := tokenApi.OAuthRefreshToken(context.Background())
	if er == nil {
		fmt.Printf("token :%s \n", token1.AccessToken)
	} else {
		fmt.Printf("err :%s \n", er.Error())
		return
	}
}
