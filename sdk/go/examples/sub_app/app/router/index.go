package router

import (
	"github.com/gin-gonic/gin"
	"net/http"
	"sub_app/app/service"
)

func Index(c *gin.Context) {
	err, url := service.GetLoginUrl(c, "http://"+c.Request.Host+"/callback")
	if err != nil {
		service.ErrorPage(c, err.Error())
		return
	}
	c.HTML(http.StatusOK, "index.html", gin.H{
		"oauth_url": url,
	})
}
