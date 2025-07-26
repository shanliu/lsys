package router

import (
	"fmt"
	"net/http"
	"sub_app/app/service"

	"github.com/gin-gonic/gin"
)

//本应用,通过直接用本地用户,登录到lsys系统

func Login(c *gin.Context) {

	account := "test"
	password := "test"
	if c.Request.Method == "POST" {
		//这里替换成:外部系统的用户登录验证
		if c.PostForm("account") == account &&
			c.PostForm("password") == password {

			//外部验证成功,创建lsys登录信息
			//uid1 用户唯一标识
			//test-11 用户名称或账号
			data, err := service.GetRestApi().AppAuthLogin(c.Request.Context(), "uid1", "test-11", c.Request)
			if err != nil {
				service.ErrorPage(c, err.Error())
				return
			}
			loginData, err := service.GetRestApi().AppAuthInfo(c.Request.Context(), data)
			if err != nil {
				service.ErrorPage(c, err.Error())
				return
			} else {
				fmt.Printf("login data:%s", loginData.String()) //登录信息
			}

			//跳转到登录跳转页面
			c.Redirect(301,
				fmt.Sprintf("%s/app.html#/login/app?client_id=%s&code=%s",
					service.GetRestApi().Config().AppHost,
					service.GetRestApi().Config().AppId,
					data))
		} else {
			service.ErrorPage(c, "账号密码错误")
		}

	} else {
		c.HTML(http.StatusOK, "login.html", gin.H{
			"account":  account,
			"password": password,
		})
	}
}
