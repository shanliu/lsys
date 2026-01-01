package service

import (
	"lsysrest/lsyslib"
	"net/http"

	"github.com/gin-gonic/gin"
)

var restApi *lsyslib.LsysApi

func init() {
	if restApi == nil {
		restApi = lsyslib.NewRestApi(&lsyslib.LsysApiConfig{
			//应用在 https://www.lsys.cc/app.html#/user/app 申请
			AppId:          "app00122",                         //应用ID
			AppSecret:      "1df933bc3e91571ba1b9e4987a7af701", //应用Secret
			AppHost:        "https://www.lsys.cc",              //应用HOST
			AppOAuthSecret: "344bd74bd0467dd7c7dd7d0822df3de4", //应用OauthSecret
			AppOAuthHost:   "https://www.lsys.cc/oauth.html",
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
