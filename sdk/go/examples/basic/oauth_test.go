package main

import (
	"context"
	"encoding/hex"
	"fmt"
	"math/rand"
	"testing"
)

<<<<<<< HEAD
// oauth 登录示例

func getApi() *lSysApi.RestApi {
	return lSysApi.NewRestApi(&lSysApi.RestApiConfig{
		//应用在 https://www.lsys.cc/app.html#/user/app 申请
		AppId:          "1212f",                            //应用ID
		AppSecret:      "3f95638a1e07b87df2b64e09c2541dac", //应用Secret
		AppHost:        "http://www.lsys.cc",               //应用HOST
		AppOAuthSecret: "2a97bf1b4f075b0ca7467e7c6b223f89", //应用OauthSecret
		AppOAuthHost:   "http://www.lsys.cc/oauth.html",
	})
}

=======
>>>>>>> dev
// 第一步
// 登录地址
func TestGetLoginUrl(t *testing.T) {
	sysApi := GetRestApi()
	b := make([]byte, 6)
	rand.Read(b)
	url := sysApi.OAuthAuthorizationUrl(context.Background(), "http://127.0.0.1:8080/", "user_info,user_mobile", hex.EncodeToString(b))
	fmt.Printf("url :%s \n", url)
}

// 第二步
// 登录后获取TOKEN
func TestGetToken(t *testing.T) {
	sysApi := GetRestApi()
	token, err := sysApi.OAuthAccessToken(context.Background(), "a7fe813d10846015b1acbe41f515b72a")
	if err == nil {
		fmt.Printf("token :%s \n", token.AccessToken)
	} else {
		fmt.Printf("err :%s \n", err.Error())
		return
	}
}

// //通过TOKEN得到用户信息
func TestGetUserData(t *testing.T) {
	sysApi := GetRestApi()
	tokenApi := sysApi.TokenRestApi("zswdwnrxjccpymuxpapbatxfflyiqzoa")
	data, err := tokenApi.OAuthUserInfo(context.Background(), true, true, false, false, false, false)
	if err == nil {
		fmt.Printf("sub app output :%s", data)
	} else {
		fmt.Printf("error :%s", err.Error())
	}
}

// //刷新TOKEN
func TestRefreshToken(t *testing.T) {
	sysApi := GetRestApi()
	tokenApi := sysApi.TokenRestApi("58f34acd692b70e1")
	token1, err := tokenApi.OAuthRefreshToken(context.Background())
	if err == nil {
		fmt.Printf("token :%s \n", token1.AccessToken)
	} else {
		fmt.Printf("err :%s \n", err.Error())
		return
	}
}
