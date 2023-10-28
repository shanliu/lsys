package main

import (
	"fmt"
	lSysApi "lsysrest/lsysrest"

	"github.com/gin-gonic/gin"
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
	server := gin.Default()
	router := server.Group("/")
	router.POST(
		"test",
		func(ctx *gin.Context) {
			req, res := RestCheckSign(ctx, sysApi)
			defer res.RenderOutput(ctx)
			if req == nil {
				return
			}
			//其他处理
			switch req.Method {
			case "test1":
				appName := req.AppInfo.Get("name").String()
				res.setMessage(fmt.Sprintf("request app name is %s method :test1", appName))
			}
		},
	)
	println(server.Run("0.0.0.0:8080"))
}
