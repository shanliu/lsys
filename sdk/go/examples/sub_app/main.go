package main

import (
	"sub_app/app/router"

	"github.com/gin-contrib/sessions"
	"github.com/gin-contrib/sessions/cookie"
	"github.com/gin-gonic/gin"
)

func main() {
	server := gin.Default()
	store := cookie.NewStore([]byte("sss"))
	server.Use(sessions.Sessions("lsys", store))
	server.LoadHTMLGlob("tpls/*")
	r := server.Group("/")
	r.GET("", router.Index)
	r.GET("callback", router.OauthCallback)
	r.GET("info", router.OauthUserInfo)
	r.Any("login", router.Login)
	r.Any("rbac", router.AppRbac)
	r.POST("rest-api", router.RestApi)
	println(server.Run("0.0.0.0:8081"))
}
