package router

import (
	"encoding/json"
	"fmt"
	"lsysrest/lsyslib"
	"net/http"
	"sub_app/app/service"
	"time"

	"github.com/gin-contrib/sessions"
	"github.com/gin-gonic/gin"
)

//本应用,通过OAUTH登录,得到LSYS系统的授权

func OauthCallback(c *gin.Context) {
	code, find1 := c.GetQuery("code")
	state, find2 := c.GetQuery("state")
	if !find1 || !find2 {
		service.ErrorPage(c, "回调缺少参数")
		return
	}
	data, err := service.GetToken(c, state, code)
	if err != nil {
		service.ErrorPage(c, err.Error())
		return
	}
	session := sessions.Default(c)
	dataStr, err := json.Marshal(data)
	if err != nil {
		service.ErrorPage(c, "保存授权数据失败")
		return
	}
	session.Set("oauth-token", string(dataStr))
	session.Save()
	c.Redirect(301, "/info")
}

func OauthUserInfo(c *gin.Context) {
	session := sessions.Default(c)
	tmp, ok := session.Get("oauth-token").(string)
	if !ok || len(tmp) == 0 {
		service.ErrorPage(c, "登录超时或未登录,请重新登录")
		return
	}
	var data *lsyslib.TokenData
	err := json.Unmarshal([]byte(tmp), &data)
	if err != nil {
		service.ErrorPage(c, "授权信息异常，请重新授权:"+err.Error())
		return
	}
	reload, find1 := c.GetQuery("reload")
	if find1 && reload == "1" {
		tmp, err := service.RefreshToken(data.RefreshToken)
		if err != nil {
			service.ErrorPage(c, err.Error())
			return
		}
		dataStr, err := json.Marshal(tmp)
		if err != nil {
			service.ErrorPage(c, "保存授权数据失败")
			return
		}
		session.Set("oauth-token", string(dataStr))
		session.Save()
		data = tmp
	}
	user, err := service.GetUserData(data.AccessToken)
	if err != nil {
		service.ErrorPage(c, fmt.Sprintf("err:[%T] %v", err, err))
		return
	}
	expire, _ := data.ExpiresIn.Int64()
	c.HTML(http.StatusOK, "user.html", gin.H{
		"token":    data.AccessToken,
		"expires":  time.Unix(expire, 0).Format("2006-01-02 15:04:05"),
		"nikename": user.Get("user_data.user.nickname").String(),
		"username": user.Get("user_data.name.username").String(),
	})
}
