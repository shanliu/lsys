package router

import (
	"fmt"
	"github.com/gin-gonic/gin"
	"sub_app/app/lib"
	"sub_app/app/service"
)

func RestApi(ctx *gin.Context) {
	req, res := lib.RestCheckSign(ctx, service.GetRestApi())
	defer res.RenderOutput(ctx)
	if req == nil {
		return
	}
	//对外服务业务处理
	switch req.Method {
	case "test1":
		appName := req.AppInfo.Get("name").String()
		res.SetMessage(fmt.Sprintf("request app name is %s method %s", appName, req.Method))
	}
}
