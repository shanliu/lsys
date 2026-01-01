package router

import (
	"fmt"
	"sub_app/app/lib"

	"github.com/gin-gonic/gin"
)

// 此示例演示: 实现站点 `对外应用` 由 lsys 系统托管
// 1. 用户先通过 lsys 平台申请 本系统(系统应用 app00122)的子应用
// 2. 用户通过REST请求时,传入 申请的子应用的 client_id 及其他参数, 本系统查询出 子应用的 Secret ,并以该子应用的 Secret 实现请求签名校验
// 对应客户端示例 参见 ../../rest_api_test.go

func RestApi(ctx *gin.Context) {
	req, res := lib.RestCheckSign(ctx)
	defer res.RenderOutput(ctx)
	if req == nil {
		return
	}
	//对外服务业务处理
	switch req.Method {
	case "test1":
		appName := req.AppInfo.Name
		res.SetMessage(fmt.Sprintf("request app name is %s method %s", appName, req.Method))
	}
}
