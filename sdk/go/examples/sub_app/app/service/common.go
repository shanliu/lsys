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
			//应用在 https://lsys.cc/user/app/create 申请
			AppId:          "app00122",                         //应用ID
			AppSecret:      "963753d6fb02c4000a055a530c53fa57", //应用Secret
			AppHost:        "https://www.lsys.cc",              //应用HOST
			AppOAuthSecret: "3f3aa84e3c3066a036ff478df182a645", //应用OauthSecret
			AppOAuthHost:   "https://www.lsys.cc/oauth",
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
