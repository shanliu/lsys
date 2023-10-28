package main

import (
	"context"
	"encoding/hex"
	"fmt"
	lSysApi "lsysrest/lsysrest"
	"math/rand"
)

func main() {
	sysApi := lSysApi.NewRestApi(&lSysApi.RestApiConfig{
		//应用在 http://www.lsys.cc/ui/#/user/app 申请
		AppId:          "1212f",                            //应用ID
		AppSecret:      "3f95638a1e07b87df2b64e09c2541dac", //应用Secret
		AppHost:        "http://www.lsys.cc",               //应用HOST
		AppOAuthSecret: "2a97bf1b4f075b0ca7467e7c6b223f89", //应用OauthSecret
		AppOAuthHost:   "http://www.lsys.cc/ui/oauth.html",
	})

	//示例2
	//oauth 登录
	//第一步
	//登录地址
	b := make([]byte, 6)
	rand.Read(b)
	url := sysApi.OAuthAuthorizationUrl(context.Background(), "http://127.0.0.1:8080/", "user_info,user_mobile", hex.EncodeToString(b))
	fmt.Printf("url :%s \n", url)

	//第二步
	//登录后获取TOKEN
	//token, er := sysApi.OAuthAccessToken(context.Background(), "da0c6285c513b7a4e92e7913f86d4b0f")
	//if er == nil {
	//	fmt.Printf("token :%s \n", token)
	//} else {
	//	fmt.Printf("err :%s \n", er)
	//	return
	//}

	////通过TOKEN得到用户信息
	tokenApi := sysApi.TokenRestApi("23b711e8cd05726f83fbd4a69a4c590e")
	data, err := tokenApi.OAuthUserInfo(context.Background(), true, true, false, false, false, false)
	if err == nil {
		fmt.Printf("sub app output :%s", data)
	} else {
		fmt.Printf("error :%s", err)
	}
	//
	////刷新TOKEN
	//tokenApi := sysApi.TokenRestApi("b8b504d63ae1dcaf7e6458092719f23c")
	//token1, er := tokenApi.OAuthRefreshToken(context.Background())
	//if er == nil {
	//	fmt.Printf("token :%s \n", token1)
	//} else {
	//	fmt.Printf("err :%s \n", er)
	//	return
	//}
}
