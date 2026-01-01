package service

import (
	"lsysrest/lsyslib"
	"net/http"

	"github.com/gin-gonic/gin"
)

var restApi *lsyslib.LsysApi

func init() {
	if restApi == nil {
<<<<<<< HEAD
		restApi = lSysApi.NewRestApi(&lSysApi.RestApiConfig{
			//应用在 https://www.lsys.cc/app.html#/user/app 申请
			AppId:          "1212f",                            //应用ID
			AppSecret:      "3f95638a1e07b87df2b64e09c2541dac", //应用Secret
			AppHost:        "http://www.lsys.cc",               //应用HOST
			AppOAuthSecret: "2a97bf1b4f075b0ca7467e7c6b223f89", //应用OauthSecret
			AppOAuthHost:   "http://www.lsys.cc/oauth.html",
=======
		restApi = lsyslib.NewRestApi(&lsyslib.LsysApiConfig{
			//应用在 https://www.lsys.cc/app.html#/user/app 申请
			AppId:          "app00122",                         //应用ID
			AppSecret:      "1df933bc3e91571ba1b9e4987a7af701", //应用Secret
			AppHost:        "https://www.lsys.cc",              //应用HOST
			AppOAuthSecret: "344bd74bd0467dd7c7dd7d0822df3de4", //应用OauthSecret
			AppOAuthHost:   "https://www.lsys.cc/oauth.html",
>>>>>>> dev
		})
	}
}

//GetRestApi  建议重构,实现从你的配置里获取,这里仅演示用

func GetRestApi() *lsyslib.LsysApi {
	return restApi
}

func ErrorPage(ctx *gin.Context, msg string) {
	ctx.HTML(http.StatusOK, "err.html", gin.H{
		"msg": msg,
	})
}
