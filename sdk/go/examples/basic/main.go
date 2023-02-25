package main

import (
	"context"
	"fmt"
	lSysApi "lsysrest/lsysrest"
)

func main() {
	sysApi := lSysApi.NewRestApi(&lSysApi.RestApiConfig{
		//应用在 http://175.178.90.181/ui/#/user/app 申请
		AppId:          "1212f",                            //应用ID
		AppSecret:      "3f95638a1e07b87df2b64e09c2541dac", //应用Secret
		AppHost:        "http://175.178.90.181",            //应用HOST
		AppOAuthSecret: "2a97bf1b4f075b0ca7467e7c6b223f89", //应用OauthSecret
		AppOAuthHost:   "http://175.178.90.181/ui/oauth.html",
	})

	//示例2
	//oauth 登录
	//第一步
	//登录地址
	url := sysApi.OAuthAuthorizationUrl(context.Background(), "http://127.0.0.1:8080/", "user_info", "aa")
	fmt.Printf("url :%s \n", url)

	//第二步
	//登录后获取TOKEN
	token, er := sysApi.OAuthAccessToken(context.Background(), "52f5861337c1947beab40df90ade268c")
	if er == nil {
		fmt.Printf("token :%s \n", token)
	} else {
		fmt.Printf("err :%s \n", er)
	}
	//通过TOKEN得到用户信息
	tokenApi := sysApi.TokenRestApi(token.AccessToken)
	data, err := tokenApi.OAuthUserInfo(context.Background(), true, true, false, false, false, false)
	if err == nil {
		fmt.Printf("sub app output :%s", data)
	} else {
		fmt.Printf("error :%s", err)
	}
}
