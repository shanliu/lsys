package main

import (
	"context"
	"encoding/hex"
	"fmt"
	"math/rand"
	"testing"
)

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
	token, err := sysApi.OAuthAccessToken(context.Background(), "ee54ee6b82b3561140384f32f969ff2b")
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
	tokenApi := sysApi.TokenRestApi("rdubgmhdqctdmhfdiskjrwchmadyezmt")
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
